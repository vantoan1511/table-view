use db_bridge::{Config, QueryResult, SchemaResult, TableDataResult};
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, FocusHandle, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Subscription, Task, TitlebarOptions, Window,
    WindowBounds, WindowOptions,
};

use crate::config::AppWindowConfig;
use crate::db::{
    ConnectRequest, ConnectionMetadata, DatabaseEngine, DbFuture, QueryRequest, SchemaRequest,
    TableDataRequest,
};
use crate::layout::{
    bind_workspace_keys, render_main_content, render_sidebar, render_sidebar_resize_handle,
    render_status_bar, render_tab_strip, CloseActiveTab, NewQueryTab, ToggleSidebar,
};
use crate::theme::{ActiveTheme, GlobalTheme, ResolvedAppearance, Theme, ThemeMode};
use crate::workspace::{QueryStatus, WorkspaceState};

#[derive(Clone)]
struct GlobalDbEngine(DatabaseEngine);

impl gpui::Global for GlobalDbEngine {}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SidebarDragState {
    start_x_px: f32,
    start_width_px: f32,
}

struct RootView {
    workspace: WorkspaceState,
    db_engine: Option<DatabaseEngine>,
    #[allow(dead_code)]
    last_schema: Option<SchemaResult>,
    #[allow(dead_code)]
    last_query_result: Option<QueryResult>,
    #[allow(dead_code)]
    last_table_data: Option<TableDataResult>,
    focus_handle: FocusHandle,
    sidebar_drag: Option<SidebarDragState>,
    _subscriptions: Vec<Subscription>,
}

impl RootView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle);

        let appearance_subscription = window.observe_window_appearance(|window, cx| {
            Theme::sync_system_appearance(cx, window.appearance().into());
        });
        let theme_subscription = cx.observe_global::<GlobalTheme>(|_, cx| {
            cx.notify();
        });

        let mut workspace = WorkspaceState::new();
        let db_engine = if let Some(existing) = cx.try_global::<GlobalDbEngine>() {
            Some(existing.0.clone())
        } else {
            match DatabaseEngine::new() {
                Ok(engine) => {
                    cx.set_global(GlobalDbEngine(engine.clone()));
                    Some(engine)
                }
                Err(err) => {
                    log::error!("Failed to initialize in-process database engine: {err}");
                    workspace.fail_query(err);
                    None
                }
            }
        };

        Self {
            workspace,
            db_engine,
            last_schema: None,
            last_query_result: None,
            last_table_data: None,
            focus_handle,
            sidebar_drag: None,
            _subscriptions: vec![appearance_subscription, theme_subscription],
        }
    }

    fn spawn_db_task<T, F, U>(
        &mut self,
        cx: &mut Context<Self>,
        call: F,
        on_complete: U,
    ) -> Task<Result<T, String>>
    where
        T: Clone + Send + 'static,
        F: FnOnce(&DatabaseEngine) -> DbFuture<T>,
        U: FnOnce(&mut Self, &Result<T, String>) + Send + 'static,
    {
        let Some(engine) = self.db_engine.as_ref() else {
            let outcome: Result<T, String> =
                Err("Database engine is not initialized".to_string());
            on_complete(self, &outcome);
            cx.notify();
            return Task::ready(outcome);
        };

        let future = call(engine);
        cx.spawn(async move |this, cx| {
            let outcome = future.await;
            if let Err(err) = this.update(cx, |view, cx| {
                on_complete(view, &outcome);
                cx.notify();
            }) {
                log::warn!("RootView entity unavailable after database task: {err}");
            }
            outcome
        })
    }

    /// Tests a database connection configuration asynchronously without modifying active state.
    #[allow(dead_code)]
    pub fn test_database_connection(
        &self,
        config: Config,
        cx: &mut Context<Self>,
    ) -> Task<Result<(), String>> {
        let Some(engine) = self.db_engine.clone() else {
            return Task::ready(Err("Database engine is not initialized".to_string()));
        };

        cx.spawn(async move |_this, _cx| engine.test_connection(config).await)
    }

    /// Connects to a database asynchronously and updates workspace connection status.
    #[allow(dead_code)]
    pub fn connect_database(
        &mut self,
        request: ConnectRequest,
        cx: &mut Context<Self>,
    ) -> Task<Result<ConnectionMetadata, String>> {
        self.workspace
            .begin_connecting(request.connection_name.clone());
        cx.notify();

        self.spawn_db_task(
            cx,
            |engine| engine.connect(request),
            |view, outcome| match outcome {
                Ok(metadata) => view.workspace.finish_connecting(metadata),
                Err(err) => view.workspace.fail_connecting(err.clone()),
            },
        )
    }

    /// Disconnects a pooled database connection and clears active workspace connection state.
    #[allow(dead_code)]
    pub fn disconnect_database(
        &mut self,
        connection_id: String,
        cx: &mut Context<Self>,
    ) -> Task<Result<(), String>> {
        let disconnected_id = connection_id.clone();
        self.spawn_db_task(
            cx,
            |engine| engine.disconnect(connection_id),
            move |view, outcome| match outcome {
                Ok(()) => {
                    if view.workspace.active_connection_id.as_deref()
                        == Some(disconnected_id.as_str())
                    {
                        view.workspace.set_disconnected();
                        view.last_schema = None;
                        view.last_query_result = None;
                        view.last_table_data = None;
                    }
                }
                Err(err) => view.workspace.fail_query(err.clone()),
            },
        )
    }

    /// Introspects schema metadata asynchronously and stores the latest result on `RootView`.
    #[allow(dead_code)]
    pub fn load_schema(
        &mut self,
        request: SchemaRequest,
        cx: &mut Context<Self>,
    ) -> Task<Result<SchemaResult, String>> {
        self.spawn_db_task(
            cx,
            |engine| engine.get_schema(request),
            |view, outcome| match outcome {
                Ok(schema) => view.last_schema = Some(schema.clone()),
                Err(err) => view.workspace.fail_query(err.clone()),
            },
        )
    }

    /// Executes a SQL query asynchronously off the GPUI render thread.
    #[allow(dead_code)]
    pub fn run_query(
        &mut self,
        request: QueryRequest,
        cx: &mut Context<Self>,
    ) -> Task<Result<QueryResult, String>> {
        self.workspace.begin_query();
        cx.notify();

        self.spawn_db_task(
            cx,
            |engine| engine.execute_query(request),
            |view, outcome| match outcome {
                Ok(result) => {
                    view.last_query_result = Some(result.clone());
                    view.workspace.finish_query(result);
                }
                Err(err) => view.workspace.fail_query(err.clone()),
            },
        )
    }

    /// Fetches paginated table rows asynchronously off the GPUI render thread.
    #[allow(dead_code)]
    pub fn load_table_data(
        &mut self,
        request: TableDataRequest,
        cx: &mut Context<Self>,
    ) -> Task<Result<TableDataResult, String>> {
        self.workspace.begin_query();
        cx.notify();

        self.spawn_db_task(
            cx,
            |engine| engine.fetch_table_data(request),
            |view, outcome| match outcome {
                Ok(data) => {
                    view.last_table_data = Some(data.clone());
                    view.workspace.set_query_status(QueryStatus::Completed {
                        elapsed_ms: data.execution_time,
                        row_count: data.rows.len(),
                    });
                }
                Err(err) => view.workspace.fail_query(err.clone()),
            },
        )
    }

    fn on_toggle_sidebar(
        &mut self,
        _: &ToggleSidebar,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.workspace.toggle_sidebar();
        cx.notify();
    }

    fn on_new_query_tab(&mut self, _: &NewQueryTab, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.open_new_sql_tab();
        cx.notify();
    }

    fn on_close_active_tab(
        &mut self,
        _: &CloseActiveTab,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.workspace.close_active_tab() {
            cx.notify();
        }
    }

    fn start_sidebar_drag(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.sidebar_drag = Some(SidebarDragState {
            start_x_px: f32::from(event.position.x),
            start_width_px: self.workspace.sidebar.width_px,
        });
        cx.notify();
    }

    fn update_sidebar_drag(&mut self, current_x_px: f32) {
        if let Some(drag) = self.sidebar_drag {
            self.workspace
                .sidebar
                .apply_drag_delta(drag.start_width_px, current_x_px - drag.start_x_px);
        }
    }

    fn end_sidebar_drag(&mut self) {
        self.sidebar_drag = None;
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let select_entity = cx.entity();
        let close_entity = cx.entity();

        let mut top_row = div().flex().flex_1().min_h_0().w_full();
        if self.workspace.sidebar.is_visible {
            top_row = top_row
                .child(render_sidebar(
                    &self.workspace,
                    theme,
                    cx.listener(|this, _, _, cx| {
                        this.workspace.open_new_sql_tab();
                        cx.notify();
                    }),
                ))
                .child(render_sidebar_resize_handle(
                    self.sidebar_drag.is_some(),
                    theme,
                    cx.listener(Self::start_sidebar_drag),
                ));
        }

        let right_workspace = div()
            .flex_1()
            .flex()
            .flex_col()
            .min_w_0()
            .min_h_0()
            .child(render_tab_strip(
                &self.workspace,
                theme,
                move |tab_id, _, _, cx| {
                    select_entity.update(cx, |this, cx| {
                        if this.workspace.activate_tab(tab_id) {
                            cx.notify();
                        }
                    });
                },
                move |tab_id, _, _, cx| {
                    close_entity.update(cx, |this, cx| {
                        if this.workspace.close_tab(tab_id) {
                            cx.notify();
                        }
                    });
                },
                cx.listener(|this, _, _, cx| {
                    this.workspace.open_new_sql_tab();
                    cx.notify();
                }),
            ))
            .child(render_main_content(&self.workspace, theme));

        top_row = top_row.child(right_workspace);

        let mut root = div()
            .id("workspace-root")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_toggle_sidebar))
            .on_action(cx.listener(Self::on_new_query_tab))
            .on_action(cx.listener(Self::on_close_active_tab))
            .flex()
            .flex_col()
            .size_full()
            .font_family(theme.typography.ui_font_family)
            .text_size(px(theme.typography.base_px))
            .bg(theme.colors.surface.to_gpui())
            .text_color(theme.colors.text_primary.to_gpui())
            .child(top_row)
            .child(render_status_bar(
                &self.workspace,
                theme,
                |_, _, cx| Theme::toggle_light_dark(cx),
                cx.listener(|this, _, _, cx| {
                    this.workspace.toggle_sidebar();
                    cx.notify();
                }),
            ));

        if self.sidebar_drag.is_some() {
            root = root
                .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                    this.update_sidebar_drag(f32::from(event.position.x));
                    cx.notify();
                }))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                        this.end_sidebar_drag();
                        cx.notify();
                    }),
                )
                .on_mouse_up_out(
                    MouseButton::Left,
                    cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                        this.end_sidebar_drag();
                        cx.notify();
                    }),
                );
        }

        root
    }
}

fn validate_window_dimensions(config: &AppWindowConfig) -> Option<(f32, f32)> {
    if config.width_px.is_finite()
        && config.width_px > 0.0
        && config.height_px.is_finite()
        && config.height_px > 0.0
    {
        Some((config.width_px, config.height_px))
    } else {
        None
    }
}

/// Launches the GPUI native desktop window with the given configuration.
pub fn run_native_window(config: AppWindowConfig) {
    let Some((width_px, height_px)) = validate_window_dimensions(&config) else {
        eprintln!("Invalid native window dimensions: {:?}", config);
        return;
    };

    Application::new().run(move |cx: &mut App| {
        let system_appearance = ResolvedAppearance::from(cx.window_appearance());
        Theme::init(cx, ThemeMode::default(), system_appearance);
        bind_workspace_keys(cx);

        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                if let Some(engine) = cx.try_global::<GlobalDbEngine>().map(|g| g.0.clone()) {
                    cx.spawn(async move |cx| {
                        if let Err(err) = engine.close_all().await {
                            log::warn!("Failed to close pooled database connections on exit: {err}");
                        }
                        let _ = cx.update(|cx| cx.quit());
                    })
                    .detach();
                } else {
                    cx.quit();
                }
            }
        })
        .detach();

        let bounds = Bounds::centered(None, size(px(width_px), px(height_px)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some(config.title.into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        if let Err(err) = cx.open_window(options, |window, cx| {
            cx.new(|cx| RootView::new(window, cx))
        }) {
            eprintln!("Failed to open native GPUI window: {err}");
            cx.quit();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::SidebarState;

    #[test]
    fn test_window_bounds_dimensions() {
        let config = AppWindowConfig::default();
        let dims = validate_window_dimensions(&config);
        assert_eq!(dims, Some((1024.0, 700.0)));
    }

    #[test]
    fn test_root_view_toggles_global_theme_mode() {
        Application::headless().run(|cx| {
            Theme::init(cx, ThemeMode::default(), ResolvedAppearance::Dark);
            assert!(cx.theme().is_dark());

            Theme::toggle_light_dark(cx);
            assert!(!cx.theme().is_dark());

            Theme::toggle_light_dark(cx);
            assert!(cx.theme().is_dark());

            cx.quit();
        });
    }

    #[test]
    fn test_root_view_workspace_sidebar_and_tab_actions() {
        Application::headless().run(|cx| {
            Theme::init(cx, ThemeMode::default(), ResolvedAppearance::Dark);
            bind_workspace_keys(cx);

            let window = cx
                .open_window(WindowOptions::default(), |window, cx| {
                    cx.new(|cx| RootView::new(window, cx))
                })
                .expect("headless window should open");

            window
                .update(cx, |view, _window, cx| {
                    assert!(view.workspace.sidebar.is_visible);
                    assert_eq!(view.workspace.sidebar.width_px, 260.0);
                    assert!(view.workspace.tabs.is_empty());
                    assert_eq!(view.workspace.active_tab_id, None);

                    view.workspace.toggle_sidebar();
                    assert!(!view.workspace.sidebar.is_visible);

                    let tab_id = view.workspace.open_new_sql_tab();
                    assert_eq!(view.workspace.tabs.len(), 1);
                    assert_eq!(view.workspace.active_tab_id, Some(tab_id));

                    view.workspace.set_sidebar_width(320.0);
                    assert_eq!(view.workspace.sidebar.width_px, 320.0);

                    assert!(view.workspace.close_active_tab());
                    assert!(view.workspace.tabs.is_empty());
                    assert_eq!(view.workspace.active_tab_id, None);
                    cx.notify();
                })
                .expect("window update should succeed");

            cx.quit();
        });
    }

    #[test]
    fn test_sidebar_drag_resize_clamps_width() {
        let mut sidebar = SidebarState::default();
        sidebar.apply_drag_delta(260.0, -500.0);
        assert_eq!(sidebar.width_px, 150.0);

        sidebar.apply_drag_delta(260.0, 1000.0);
        assert_eq!(sidebar.width_px, 500.0);

        sidebar.apply_drag_delta(260.0, 40.0);
        assert_eq!(sidebar.width_px, 300.0);

        sidebar.apply_drag_delta(260.0, f32::NAN);
        assert_eq!(sidebar.width_px, 300.0);
    }

    #[test]
    fn test_root_view_bridges_tokio_db_connect_schema_query_and_disconnect() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        let db_path = std::env::temp_dir().join(format!(
            "table_view_gpui_{}_{nanos}.db",
            std::process::id()
        ));
        std::fs::File::create(&db_path).expect("temp sqlite file should be created");
        let db_path_str = db_path.to_string_lossy().to_string();

        Application::headless().run(move |cx| {
            Theme::init(cx, ThemeMode::default(), ResolvedAppearance::Dark);
            bind_workspace_keys(cx);

            let window = cx
                .open_window(WindowOptions::default(), |window, cx| {
                    cx.new(|cx| RootView::new(window, cx))
                })
                .expect("headless window should open");

            let db_path_for_cleanup = db_path.clone();
            let db_path_for_config = db_path_str.clone();

            cx.spawn(async move |cx| {
                let test_result: Result<(), String> = async {
                    let config = db_bridge::Config {
                        db_type: "sqlite".to_string(),
                        database: db_path_for_config,
                        ..Default::default()
                    };

                    let test_conn_task = window
                        .update(cx, |view, _window, cx| {
                            view.test_database_connection(config.clone(), cx)
                        })
                        .map_err(|err| err.to_string())?;
                    test_conn_task.await?;

                    let connect_task = window
                        .update(cx, |view, _window, cx| {
                            let task = view.connect_database(
                                crate::db::ConnectRequest {
                                    connection_id: "gpui-conn".to_string(),
                                    connection_name: "gpui-sqlite".to_string(),
                                    color: crate::workspace::ConnectionColorTag::Emerald,
                                    config,
                                },
                                cx,
                            );
                            assert!(matches!(
                                view.workspace.connection_status,
                                crate::workspace::ConnectionStatus::Connecting { .. }
                            ));
                            task
                        })
                        .map_err(|err| err.to_string())?;

                    let metadata = connect_task.await?;
                    if metadata.connection_id != "gpui-conn" {
                        return Err(format!("Unexpected connection_id: {}", metadata.connection_id));
                    }

                    window
                        .update(cx, |view, _window, _cx| {
                            assert_eq!(
                                view.workspace.active_connection_id.as_deref(),
                                Some("gpui-conn")
                            );
                            assert!(matches!(
                                view.workspace.connection_status,
                                crate::workspace::ConnectionStatus::Connected { .. }
                            ));
                        })
                        .map_err(|err| err.to_string())?;

                    let create_task = window
                        .update(cx, |view, _window, cx| {
                            view.run_query(
                                crate::db::QueryRequest {
                                    connection_id: "gpui-conn".to_string(),
                                    sql: "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)"
                                        .to_string(),
                                    target_database: None,
                                },
                                cx,
                            )
                        })
                        .map_err(|err| err.to_string())?;
                    create_task.await?;

                    let insert_task = window
                        .update(cx, |view, _window, cx| {
                            view.run_query(
                                crate::db::QueryRequest {
                                    connection_id: "gpui-conn".to_string(),
                                    sql: "INSERT INTO users VALUES (1, 'Ada')".to_string(),
                                    target_database: None,
                                },
                                cx,
                            )
                        })
                        .map_err(|err| err.to_string())?;
                    insert_task.await?;

                    let schema_task = window
                        .update(cx, |view, _window, cx| {
                            view.load_schema(
                                crate::db::SchemaRequest {
                                    connection_id: "gpui-conn".to_string(),
                                    all_databases: false,
                                    schema_name: None,
                                    target_database: None,
                                },
                                cx,
                            )
                        })
                        .map_err(|err| err.to_string())?;
                    let schema = schema_task.await?;
                    if !schema.tables.iter().any(|t| t.name == "users") {
                        return Err("Expected users table in schema".to_string());
                    }

                    let select_task = window
                        .update(cx, |view, _window, cx| {
                            view.run_query(
                                crate::db::QueryRequest {
                                    connection_id: "gpui-conn".to_string(),
                                    sql: "SELECT id, name FROM users".to_string(),
                                    target_database: None,
                                },
                                cx,
                            )
                        })
                        .map_err(|err| err.to_string())?;
                    let query_result = select_task.await?;
                    if query_result.row_count != 1 {
                        return Err(format!("Expected 1 row, got {}", query_result.row_count));
                    }

                    window
                        .update(cx, |view, _window, _cx| {
                            assert!(view
                                .last_schema
                                .as_ref()
                                .is_some_and(|s| s.tables.iter().any(|t| t.name == "users")));
                            assert_eq!(
                                view.last_query_result.as_ref().map(|r| r.row_count),
                                Some(1)
                            );
                            assert!(matches!(
                                view.workspace.query_status,
                                crate::workspace::QueryStatus::Completed { row_count: 1, .. }
                            ));
                        })
                        .map_err(|err| err.to_string())?;

                    let table_data_task = window
                        .update(cx, |view, _window, cx| {
                            view.load_table_data(
                                crate::db::TableDataRequest {
                                    connection_id: "gpui-conn".to_string(),
                                    table_name: "users".to_string(),
                                    limit: 50,
                                    offset: 0,
                                    sort_column: String::new(),
                                    sort_direction: String::new(),
                                    filter: String::new(),
                                    target_database: None,
                                },
                                cx,
                            )
                        })
                        .map_err(|err| err.to_string())?;
                    let table_data = table_data_task.await?;
                    if table_data.total_count != 1 {
                        return Err(format!(
                            "Expected total_count 1, got {}",
                            table_data.total_count
                        ));
                    }

                    window
                        .update(cx, |view, _window, _cx| {
                            assert_eq!(
                                view.last_table_data.as_ref().map(|d| d.total_count),
                                Some(1)
                            );
                        })
                        .map_err(|err| err.to_string())?;

                    let disconnect_other_task = window
                        .update(cx, |view, _window, cx| {
                            view.disconnect_database("non-active-conn".to_string(), cx)
                        })
                        .map_err(|err| err.to_string())?;
                    disconnect_other_task.await?;

                    window
                        .update(cx, |view, _window, _cx| {
                            assert_eq!(
                                view.workspace.active_connection_id.as_deref(),
                                Some("gpui-conn")
                            );
                            assert!(matches!(
                                view.workspace.connection_status,
                                crate::workspace::ConnectionStatus::Connected { .. }
                            ));
                        })
                        .map_err(|err| err.to_string())?;

                    let disconnect_task = window
                        .update(cx, |view, _window, cx| {
                            view.disconnect_database("gpui-conn".to_string(), cx)
                        })
                        .map_err(|err| err.to_string())?;
                    disconnect_task.await?;

                    window
                        .update(cx, |view, _window, _cx| {
                            assert_eq!(
                                view.workspace.connection_status,
                                crate::workspace::ConnectionStatus::Disconnected
                            );
                            assert_eq!(view.workspace.active_connection_id, None);
                        })
                        .map_err(|err| err.to_string())?;

                    Ok(())
                }
                .await;

                let _ = std::fs::remove_file(&db_path_for_cleanup);
                let _ = cx.update(|cx| cx.quit());
                test_result.expect("GPUI DB bridge lifecycle should succeed");
            })
            .detach();
        });
    }

    #[test]
    fn test_root_view_surfaces_query_error_without_panicking() {
        Application::headless().run(|cx| {
            Theme::init(cx, ThemeMode::default(), ResolvedAppearance::Dark);
            bind_workspace_keys(cx);

            let window = cx
                .open_window(WindowOptions::default(), |window, cx| {
                    cx.new(|cx| RootView::new(window, cx))
                })
                .expect("headless window should open");

            cx.spawn(async move |cx| {
                let test_result: Result<(), String> = async {
                    let connect_task = window
                        .update(cx, |view, _window, cx| {
                            view.connect_database(
                                crate::db::ConnectRequest {
                                    connection_id: "mem-err".to_string(),
                                    connection_name: "mem-err".to_string(),
                                    color: crate::workspace::ConnectionColorTag::Red,
                                    config: db_bridge::Config {
                                        db_type: "sqlite".to_string(),
                                        database: ":memory:".to_string(),
                                        ..Default::default()
                                    },
                                },
                                cx,
                            )
                        })
                        .map_err(|err| err.to_string())?;
                    connect_task.await?;

                    let bad_query_task = window
                        .update(cx, |view, _window, cx| {
                            view.run_query(
                                crate::db::QueryRequest {
                                    connection_id: "mem-err".to_string(),
                                    sql: "SELECT * FROM missing_table".to_string(),
                                    target_database: None,
                                },
                                cx,
                            )
                        })
                        .map_err(|err| err.to_string())?;
                    let err_outcome = bad_query_task.await;
                    assert!(err_outcome.is_err());

                    window
                        .update(cx, |view, _window, _cx| {
                            assert!(matches!(
                                view.workspace.query_status,
                                crate::workspace::QueryStatus::Error { .. }
                            ));
                        })
                        .map_err(|err| err.to_string())?;

                    let disconnect_task = window
                        .update(cx, |view, _window, cx| {
                            view.disconnect_database("mem-err".to_string(), cx)
                        })
                        .map_err(|err| err.to_string())?;
                    disconnect_task.await?;

                    Ok(())
                }
                .await;

                let _ = cx.update(|cx| cx.quit());
                test_result.expect("GPUI error handling test should complete cleanly");
            })
            .detach();
        });
    }

    #[test]
    fn test_root_view_reuses_global_db_engine_across_windows() {
        Application::headless().run(|cx| {
            Theme::init(cx, ThemeMode::default(), ResolvedAppearance::Dark);
            bind_workspace_keys(cx);

            let first_window = cx
                .open_window(WindowOptions::default(), |window, cx| {
                    cx.new(|cx| RootView::new(window, cx))
                })
                .expect("first headless window should open");
            let second_window = cx
                .open_window(WindowOptions::default(), |window, cx| {
                    cx.new(|cx| RootView::new(window, cx))
                })
                .expect("second headless window should open");

            cx.spawn(async move |cx| {
                let test_result: Result<(), String> = async {
                    let connect_task = first_window
                        .update(cx, |view, _window, cx| {
                            view.connect_database(
                                crate::db::ConnectRequest {
                                    connection_id: "shared-conn".to_string(),
                                    connection_name: "shared-mem".to_string(),
                                    color: crate::workspace::ConnectionColorTag::Indigo,
                                    config: db_bridge::Config {
                                        db_type: "sqlite".to_string(),
                                        database: ":memory:".to_string(),
                                        ..Default::default()
                                    },
                                },
                                cx,
                            )
                        })
                        .map_err(|err| err.to_string())?;
                    connect_task.await?;

                    // Second window reuses the same GlobalDbEngine pool, so it can query "shared-conn".
                    let query_task = second_window
                        .update(cx, |view, _window, cx| {
                            view.run_query(
                                crate::db::QueryRequest {
                                    connection_id: "shared-conn".to_string(),
                                    sql: "SELECT 42 AS answer".to_string(),
                                    target_database: None,
                                },
                                cx,
                            )
                        })
                        .map_err(|err| err.to_string())?;
                    let result = query_task.await?;
                    if result.row_count != 1 {
                        return Err(format!("Expected 1 row, got {}", result.row_count));
                    }

                    let disconnect_task = first_window
                        .update(cx, |view, _window, cx| {
                            view.disconnect_database("shared-conn".to_string(), cx)
                        })
                        .map_err(|err| err.to_string())?;
                    disconnect_task.await?;

                    Ok(())
                }
                .await;

                let _ = cx.update(|cx| cx.quit());
                test_result.expect("Multi-window RootView should share GlobalDbEngine");
            })
            .detach();
        });
    }
}
