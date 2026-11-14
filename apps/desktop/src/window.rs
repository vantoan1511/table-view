use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, FocusHandle, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Subscription, TitlebarOptions, Window,
    WindowBounds, WindowOptions,
};

use crate::config::AppWindowConfig;
use crate::layout::{
    bind_workspace_keys, render_main_content, render_sidebar, render_sidebar_resize_handle,
    render_status_bar, render_tab_strip, CloseActiveTab, NewQueryTab, ToggleSidebar,
};
use crate::theme::{ActiveTheme, GlobalTheme, ResolvedAppearance, Theme, ThemeMode};
use crate::workspace::WorkspaceState;

#[derive(Debug, Clone, Copy, PartialEq)]
struct SidebarDragState {
    start_x_px: f32,
    start_width_px: f32,
}

struct RootView {
    workspace: WorkspaceState,
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

        Self {
            workspace: WorkspaceState::new(),
            focus_handle,
            sidebar_drag: None,
            _subscriptions: vec![appearance_subscription, theme_subscription],
        }
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
                cx.quit();
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
}
