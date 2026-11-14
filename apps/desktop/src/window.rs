use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, FontWeight, Subscription,
    TitlebarOptions, Window, WindowBounds, WindowOptions,
};

use crate::config::AppWindowConfig;
use crate::theme::{ActiveTheme, GlobalTheme, ResolvedAppearance, Theme, ThemeColor, ThemeMode};

struct RootView {
    _subscriptions: Vec<Subscription>,
}

impl RootView {
    fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        let appearance_subscription = window.observe_window_appearance(|window, cx| {
            Theme::sync_system_appearance(cx, window.appearance().into());
        });
        let theme_subscription = cx.observe_global::<GlobalTheme>(|_, cx| {
            cx.notify();
        });

        Self {
            _subscriptions: vec![appearance_subscription, theme_subscription],
        }
    }
}

fn status_pill(label: &'static str, color: ThemeColor, theme: &Theme) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(theme.spacing.xs_px))
        .px(px(theme.spacing.md_px))
        .py(px(theme.spacing.xs_px))
        .rounded(px(theme.radii.sm_px))
        .bg(theme.colors.muted.to_gpui())
        .text_size(px(theme.typography.xs_px))
        .text_color(theme.colors.text_secondary.to_gpui())
        .child(
            div()
                .size(px(theme.spacing.md_px))
                .rounded_full()
                .bg(color.to_gpui()),
        )
        .child(label)
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let hover_bg = theme.colors.hover.to_gpui();
        let toggle_label = if theme.is_dark() {
            "Switch to Light"
        } else {
            "Switch to Dark"
        };

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .font_family(theme.typography.ui_font_family)
            .text_size(px(theme.typography.base_px))
            .bg(theme.colors.surface.to_gpui())
            .text_color(theme.colors.text_primary.to_gpui())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(theme.spacing.lg_px))
                    .p(px(theme.spacing.section_px))
                    .rounded(px(theme.radii.lg_px))
                    .bg(theme.colors.sidebar.to_gpui())
                    .border_1()
                    .border_color(theme.colors.border.to_gpui())
                    .child(
                        div()
                            .text_size(px(theme.typography.lg_px))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.colors.primary.to_gpui())
                            .child("Table View — Native UI Preview"),
                    )
                    .child(
                        div()
                            .text_size(px(theme.typography.sm_px))
                            .text_color(theme.colors.text_secondary.to_gpui())
                            .child("Experimental GPUI desktop shell"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(theme.spacing.md_px))
                            .child(status_pill("Connected", theme.colors.success, theme))
                            .child(status_pill("Latency", theme.colors.warning, theme))
                            .child(status_pill("Error", theme.colors.danger, theme)),
                    )
                    .child(
                        div()
                            .id("theme-toggle-btn")
                            .cursor_pointer()
                            .px(px(theme.spacing.lg_px))
                            .py(px(theme.spacing.sm_px))
                            .rounded(px(theme.radii.md_px))
                            .border_1()
                            .border_color(theme.colors.border_strong.to_gpui())
                            .bg(theme.colors.surface.to_gpui())
                            .hover(move |style| style.bg(hover_bg))
                            .text_size(px(theme.typography.xs_px))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.colors.text_primary.to_gpui())
                            .on_click(|_, _, cx| Theme::toggle_light_dark(cx))
                            .child(toggle_label),
                    ),
            )
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
}
