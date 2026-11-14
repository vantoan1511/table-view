//! GPUI workspace shell renderers, actions, and keybindings.

use std::rc::Rc;

use gpui::{
    div, prelude::*, px, Action, App, ClickEvent, DummyKeyboardMapper, FontWeight, KeyBinding,
    MouseButton, MouseDownEvent, SharedString, Window,
};

use crate::theme::Theme;
use crate::workspace::{ConnectionStatus, QueryStatus, TabItem, WorkspaceState};

gpui::actions!(workspace, [ToggleSidebar, NewQueryTab, CloseActiveTab]);

/// Registers global keyboard shortcuts for workspace shell actions.
pub fn bind_workspace_keys(cx: &mut App) {
    let bindings = [
        ("ctrl-b", ToggleSidebar.boxed_clone()),
        ("ctrl-t", NewQueryTab.boxed_clone()),
        ("ctrl-w", CloseActiveTab.boxed_clone()),
        #[cfg(target_os = "macos")]
        ("cmd-b", ToggleSidebar.boxed_clone()),
        #[cfg(target_os = "macos")]
        ("cmd-t", NewQueryTab.boxed_clone()),
        #[cfg(target_os = "macos")]
        ("cmd-w", CloseActiveTab.boxed_clone()),
    ];

    let mut loaded = Vec::with_capacity(bindings.len());
    for (keystroke, action) in bindings {
        match KeyBinding::load(keystroke, action, None, false, None, &DummyKeyboardMapper) {
            Ok(binding) => loaded.push(binding),
            Err(err) => {
                eprintln!("Failed to bind workspace keystroke '{keystroke}': {err}");
            }
        }
    }

    if !loaded.is_empty() {
        cx.bind_keys(loaded);
    }
}

/// Renders the left explorer sidebar region.
pub fn render_sidebar(
    workspace: &WorkspaceState,
    theme: &Theme,
    on_new_query: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let hover_bg = theme.colors.hover.to_gpui();

    div()
        .id("workspace-sidebar")
        .flex()
        .flex_col()
        .flex_shrink_0()
        .w(px(workspace.sidebar.width_px))
        .h_full()
        .bg(theme.colors.sidebar.to_gpui())
        .border_r_1()
        .border_color(theme.colors.border.to_gpui())
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .h(px(theme.shell.titlebar_height_px))
                .px(px(theme.spacing.lg_px))
                .border_b_1()
                .border_color(theme.colors.border.to_gpui())
                .child(
                    div()
                        .text_size(px(theme.typography.status_bar_px))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.colors.text_tertiary.to_gpui())
                        .child("EXPLORER"),
                )
                .child(
                    div()
                        .id("sidebar-new-query-btn")
                        .cursor_pointer()
                        .px(px(theme.spacing.md_px))
                        .py(px(theme.spacing.xs_px))
                        .rounded(px(theme.radii.sm_px))
                        .bg(theme.colors.muted.to_gpui())
                        .border_1()
                        .border_color(theme.colors.border.to_gpui())
                        .hover(move |s| s.bg(hover_bg))
                        .text_size(px(theme.typography.xs_px))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.colors.text_primary.to_gpui())
                        .on_click(on_new_query)
                        .child("+ New Query"),
                ),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .justify_center()
                .items_center()
                .p(px(theme.spacing.xl_px))
                .gap(px(theme.spacing.sm_px))
                .child(
                    div()
                        .text_size(px(theme.typography.xs_px))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.colors.text_secondary.to_gpui())
                        .child("No connection selected"),
                )
                .child(
                    div()
                        .text_size(px(theme.typography.status_bar_px))
                        .text_color(theme.colors.text_tertiary.to_gpui())
                        .child("Schema explorer will appear here (#96)."),
                ),
        )
}

/// Renders the draggable 4px vertical resize handle between sidebar and main workspace.
pub fn render_sidebar_resize_handle(
    is_dragging: bool,
    theme: &Theme,
    on_drag_start: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let primary_bg = theme.colors.primary.to_gpui();
    let base_bg = if is_dragging {
        primary_bg
    } else {
        theme.colors.surface.to_gpui()
    };

    div()
        .id("sidebar-resize-handle")
        .flex_shrink_0()
        .w(px(4.0))
        .h_full()
        .cursor_col_resize()
        .bg(base_bg)
        .hover(move |s| s.bg(primary_bg))
        .on_mouse_down(MouseButton::Left, on_drag_start)
}

/// Renders the top tab strip with horizontally scrollable tabs and a `+` new query button.
pub fn render_tab_strip(
    workspace: &WorkspaceState,
    theme: &Theme,
    on_select_tab: impl Fn(u64, &ClickEvent, &mut Window, &mut App) + 'static,
    on_close_tab: impl Fn(u64, &ClickEvent, &mut Window, &mut App) + 'static,
    on_new_query: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let on_select = Rc::new(on_select_tab);
    let on_close = Rc::new(on_close_tab);
    let hover_bg = theme.colors.hover.to_gpui();

    let mut tabs_row = div()
        .id("workspace-tab-scroll")
        .flex_1()
        .flex()
        .items_end()
        .h_full()
        .min_w_0()
        .overflow_x_scroll()
        .gap(px(theme.spacing.xs_px))
        .px(px(theme.spacing.md_px));

    for tab in &workspace.tabs {
        let is_active = workspace.active_tab_id == Some(tab.id);
        let select_cb = Rc::clone(&on_select);
        let close_cb = Rc::clone(&on_close);
        tabs_row = tabs_row.child(render_tab_pill(tab, is_active, theme, select_cb, close_cb));
    }

    div()
        .id("workspace-tab-strip")
        .flex()
        .items_center()
        .flex_shrink_0()
        .w_full()
        .h(px(theme.shell.titlebar_height_px))
        .bg(theme.colors.muted.to_gpui())
        .border_b_1()
        .border_color(theme.colors.border.to_gpui())
        .child(tabs_row)
        .child(
            div()
                .flex()
                .items_center()
                .flex_shrink_0()
                .px(px(theme.spacing.md_px))
                .child(
                    div()
                        .id("tab-strip-new-btn")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(28.0))
                        .rounded(px(theme.radii.sm_px))
                        .border_1()
                        .border_color(theme.colors.border.to_gpui())
                        .bg(theme.colors.surface.to_gpui())
                        .hover(move |s| s.bg(hover_bg))
                        .text_size(px(theme.typography.sm_px))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.colors.text_primary.to_gpui())
                        .on_click(on_new_query)
                        .child("+"),
                ),
        )
}

fn render_tab_pill(
    tab: &TabItem,
    is_active: bool,
    theme: &Theme,
    on_select: Rc<impl Fn(u64, &ClickEvent, &mut Window, &mut App) + 'static>,
    on_close: Rc<impl Fn(u64, &ClickEvent, &mut Window, &mut App) + 'static>,
) -> impl IntoElement {
    let tab_id = tab.id;
    let hover_bg = theme.colors.hover.to_gpui();
    let close_hover_bg = theme.colors.border_strong.to_gpui();

    let (bg, border, text_color) = if is_active {
        (
            theme.colors.surface.to_gpui(),
            theme.colors.border.to_gpui(),
            theme.colors.text_primary.to_gpui(),
        )
    } else {
        (
            theme.colors.muted.to_gpui(),
            theme.colors.muted.to_gpui(),
            theme.colors.text_secondary.to_gpui(),
        )
    };

    let top_accent = tab
        .connection_color
        .map(|tag| tag.resolve(&theme.connection_colors).to_gpui())
        .unwrap_or_else(|| {
            if is_active {
                theme.colors.primary.to_gpui()
            } else {
                bg
            }
        });

    let mut pill = div()
        .id(("workspace-tab", tab_id))
        .relative()
        .cursor_pointer()
        .flex()
        .items_center()
        .flex_shrink_0()
        .h(px(theme.shell.tab_height_px))
        .max_w(px(220.0))
        .px(px(theme.spacing.md_px))
        .gap(px(theme.spacing.sm_px))
        .rounded_t(px(theme.radii.md_px))
        .bg(bg)
        .border_t_1()
        .border_l_1()
        .border_r_1()
        .border_color(border)
        .hover(move |s| if is_active { s } else { s.bg(hover_bg) })
        .on_click(move |event, window, cx| {
            on_select(tab_id, event, window, cx);
        })
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .h(px(2.0))
                .bg(top_accent),
        )
        .child(
            div()
                .px(px(theme.spacing.xs_px))
                .py(px(1.0))
                .rounded(px(theme.radii.sm_px))
                .bg(theme.colors.active.to_gpui())
                .text_size(px(10.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.colors.primary.to_gpui())
                .child(tab.kind.badge_label()),
        )
        .child(
            div()
                .min_w_0()
                .truncate()
                .text_size(px(theme.typography.xs_px))
                .font_weight(if is_active {
                    FontWeight::MEDIUM
                } else {
                    FontWeight::NORMAL
                })
                .text_color(text_color)
                .child(SharedString::from(tab.title.clone())),
        );

    if tab.is_dirty {
        pill = pill.child(
            div()
                .size(px(6.0))
                .rounded_full()
                .bg(theme.colors.warning.to_gpui()),
        );
    }

    pill.child(
        div()
            .id(("workspace-tab-close", tab_id))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .size(px(18.0))
            .rounded(px(theme.radii.sm_px))
            .hover(move |s| s.bg(close_hover_bg))
            .text_size(px(theme.typography.xs_px))
            .text_color(theme.colors.text_tertiary.to_gpui())
            .on_click(move |event, window, cx| {
                cx.stop_propagation();
                on_close(tab_id, event, window, cx);
            })
            .child("×"),
    )
}

/// Renders the main content area for the active tab or the empty state.
pub fn render_main_content(workspace: &WorkspaceState, theme: &Theme) -> impl IntoElement {
    let content = match workspace.active_tab() {
        Some(tab) => {
            let context_label = match (&tab.connection_name, &tab.schema) {
                (Some(conn), Some(schema)) => format!("{conn} • {schema}"),
                (Some(conn), None) => conn.clone(),
                _ => "Disconnected workspace".to_string(),
            };

            div()
                .flex_1()
                .flex()
                .flex_col()
                .p(px(theme.spacing.section_px))
                .gap(px(theme.spacing.lg_px))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(theme.spacing.md_px))
                        .child(
                            div()
                                .px(px(theme.spacing.sm_px))
                                .py(px(2.0))
                                .rounded(px(theme.radii.sm_px))
                                .bg(theme.colors.active.to_gpui())
                                .text_size(px(theme.typography.status_bar_px))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.colors.primary.to_gpui())
                                .child(tab.kind.badge_label()),
                        )
                        .child(
                            div()
                                .text_size(px(theme.typography.lg_px))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.colors.text_primary.to_gpui())
                                .child(SharedString::from(tab.title.clone())),
                        ),
                )
                .child(
                    div()
                        .p(px(theme.spacing.xl_px))
                        .rounded(px(theme.radii.lg_px))
                        .bg(theme.colors.sidebar.to_gpui())
                        .border_1()
                        .border_color(theme.colors.border.to_gpui())
                        .flex()
                        .flex_col()
                        .gap(px(theme.spacing.sm_px))
                        .child(
                            div()
                                .text_size(px(theme.typography.xs_px))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.colors.text_secondary.to_gpui())
                                .child(SharedString::from(context_label)),
                        )
                        .child(
                            div()
                                .text_size(px(theme.typography.base_px))
                                .text_color(theme.colors.text_tertiary.to_gpui())
                                .child("Active tab shell ready for SQL editor and data grid views."),
                        ),
                )
        }
        None => div()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(theme.spacing.md_px))
            .p(px(theme.spacing.section_px))
            .child(
                div()
                    .text_size(px(theme.typography.lg_px))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.colors.text_primary.to_gpui())
                    .child("No tab open"),
            )
            .child(
                div()
                    .text_size(px(theme.typography.base_px))
                    .text_color(theme.colors.text_secondary.to_gpui())
                    .child("Open a new query tab (Ctrl+T) or select a table once Explorer is connected."),
            ),
    };

    div()
        .id("workspace-main-content")
        .flex_1()
        .flex()
        .flex_col()
        .min_w_0()
        .min_h_0()
        .bg(theme.colors.surface.to_gpui())
        .child(content)
}

/// Renders the bottom 32px status bar with theme toggle, connection summary, query metrics, and sidebar toggle.
pub fn render_status_bar(
    workspace: &WorkspaceState,
    theme: &Theme,
    on_toggle_theme: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_toggle_sidebar: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let hover_bg = theme.colors.hover.to_gpui();
    let theme_label = if theme.is_dark() { "Dark" } else { "Light" };
    let sidebar_label = if workspace.sidebar.is_visible {
        "Hide Sidebar"
    } else {
        "Show Sidebar"
    };

    let (conn_dot_color, conn_text) = match &workspace.connection_status {
        ConnectionStatus::Disconnected => (
            theme.colors.text_tertiary.to_gpui(),
            "Not connected".to_string(),
        ),
        ConnectionStatus::Connecting { connection_name } => (
            theme.colors.warning.to_gpui(),
            format!("Connecting to {connection_name}..."),
        ),
        ConnectionStatus::Connected {
            engine,
            host_port,
            schema,
            ..
        } => (
            theme.colors.success.to_gpui(),
            format!("Connected | {engine} | {host_port} | {schema}"),
        ),
    };

    let (query_text, query_color) = match &workspace.query_status {
        QueryStatus::Idle => (String::new(), theme.colors.text_tertiary.to_gpui()),
        QueryStatus::Running => (
            "Running...".to_string(),
            theme.colors.warning.to_gpui(),
        ),
        QueryStatus::Completed {
            elapsed_ms,
            row_count,
        } => (
            format!("{elapsed_ms} ms | {row_count} rows"),
            theme.colors.text_secondary.to_gpui(),
        ),
        QueryStatus::Error { message } => (
            format!("Error: {message}"),
            theme.colors.danger.to_gpui(),
        ),
    };

    div()
        .id("workspace-status-bar")
        .flex()
        .items_center()
        .justify_between()
        .flex_shrink_0()
        .w_full()
        .h(px(theme.shell.statusbar_height_px))
        .px(px(theme.spacing.lg_px))
        .bg(theme.colors.surface.to_gpui())
        .border_t_1()
        .border_color(theme.colors.border.to_gpui())
        .text_size(px(theme.typography.status_bar_px))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(theme.spacing.md_px))
                .child(
                    div()
                        .id("status-theme-toggle-btn")
                        .cursor_pointer()
                        .px(px(theme.spacing.sm_px))
                        .py(px(2.0))
                        .rounded(px(theme.radii.sm_px))
                        .hover(move |s| s.bg(hover_bg))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.colors.text_secondary.to_gpui())
                        .on_click(on_toggle_theme)
                        .child(format!("Theme: {theme_label}")),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(theme.spacing.sm_px))
                .child(
                    div()
                        .size(px(7.0))
                        .rounded_full()
                        .bg(conn_dot_color),
                )
                .child(
                    div()
                        .text_color(theme.colors.text_secondary.to_gpui())
                        .child(SharedString::from(conn_text)),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(theme.spacing.md_px))
                .child(
                    div()
                        .text_color(query_color)
                        .child(SharedString::from(query_text)),
                )
                .child(
                    div()
                        .id("status-sidebar-toggle-btn")
                        .cursor_pointer()
                        .px(px(theme.spacing.sm_px))
                        .py(px(2.0))
                        .rounded(px(theme.radii.sm_px))
                        .hover(move |s| s.bg(hover_bg))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.colors.text_secondary.to_gpui())
                        .on_click(on_toggle_sidebar)
                        .child(sidebar_label),
                ),
        )
}
