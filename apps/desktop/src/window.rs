use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, FontWeight,
    TitlebarOptions, Window, WindowBounds, WindowOptions,
};

use crate::config::AppWindowConfig;

struct RootView;

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .bg(rgb(0x1e1e2e))
            .text_color(rgb(0xcdd6f4))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_3()
                    .px_8()
                    .py_6()
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(0x313244))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(0x818cf8))
                            .child("Table View — Native UI Preview"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xcdd6f4))
                            .child("Experimental GPUI desktop shell"),
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

        if let Err(err) = cx.open_window(options, |_, cx| cx.new(|_| RootView)) {
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
}
