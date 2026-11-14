/// Configuration for the Table View native desktop window.
#[derive(Debug, Clone, PartialEq)]
pub struct AppWindowConfig {
    pub title: &'static str,
    pub width_px: f32,
    pub height_px: f32,
}

impl Default for AppWindowConfig {
    fn default() -> Self {
        Self {
            title: "Table View",
            width_px: 1024.0,
            height_px: 700.0,
        }
    }
}

/// Returns whether the crate was compiled with the `native-ui` Cargo feature.
#[allow(dead_code)]
pub fn is_native_ui_enabled() -> bool {
    cfg!(feature = "native-ui")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_window_config() {
        let config = AppWindowConfig::default();
        assert_eq!(config.title, "Table View");
        assert_eq!(config.width_px, 1024.0);
        assert_eq!(config.height_px, 700.0);
    }

    #[test]
    fn test_feature_flag_matches_compile_cfg() {
        assert_eq!(is_native_ui_enabled(), cfg!(feature = "native-ui"));
    }
}
