//! Pure-Rust theme token definitions and GPUI global theme integration.

/// RGBA color representation using 8-bit channels for pure-Rust token parity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ThemeColor {
    /// Creates an opaque color from a 24-bit `0xRRGGBB` hex literal.
    pub const fn rgb(hex: u32) -> Self {
        let [_, r, g, b] = hex.to_be_bytes();
        Self { r, g, b, a: 255 }
    }

    /// Creates a translucent color from a 24-bit `0xRRGGBB` hex literal and normalized alpha (`0.0..=1.0`).
    pub const fn rgba(hex: u32, alpha: f32) -> Self {
        let [_, r, g, b] = hex.to_be_bytes();
        let clamped = if alpha < 0.0 {
            0.0
        } else if alpha > 1.0 {
            1.0
        } else {
            alpha
        };
        let a = (clamped * 255.0 + 0.5) as u8;
        Self { r, g, b, a }
    }
}

/// User-selected theme mode.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    #[default]
    Dark,
    System,
}

/// Concrete light/dark appearance resolved from `ThemeMode` and OS appearance.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedAppearance {
    Light,
    #[default]
    Dark,
}

/// Semantic, surface, text, and data grid color tokens matching `src/assets/base.css`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemePalette {
    pub primary: ThemeColor,
    pub primary_hover: ThemeColor,
    pub primary_light: ThemeColor,

    pub success: ThemeColor,
    pub success_light: ThemeColor,
    pub danger: ThemeColor,
    pub danger_light: ThemeColor,
    pub warning: ThemeColor,

    pub surface: ThemeColor,
    pub sidebar: ThemeColor,
    pub border: ThemeColor,
    pub border_strong: ThemeColor,
    pub hover: ThemeColor,
    pub active: ThemeColor,
    pub muted: ThemeColor,

    pub text_primary: ThemeColor,
    pub text_secondary: ThemeColor,
    pub text_tertiary: ThemeColor,
    pub text_inverse: ThemeColor,

    pub grid_header: ThemeColor,
    pub grid_row_alt: ThemeColor,
    pub grid_row_hover: ThemeColor,
    pub grid_border: ThemeColor,
}

/// Connection badge color tokens matching `src/assets/base.css`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionColors {
    pub indigo: ThemeColor,
    pub blue: ThemeColor,
    pub teal: ThemeColor,
    pub green: ThemeColor,
    pub amber: ThemeColor,
    pub orange: ThemeColor,
    pub pink: ThemeColor,
    pub gray: ThemeColor,
    pub purple: ThemeColor,
    pub rose: ThemeColor,
    pub emerald: ThemeColor,
    pub cyan: ThemeColor,
    pub violet: ThemeColor,
    pub red: ThemeColor,
}

/// Typography tokens matching `src/assets/base.css`, `src/assets/main.css`, and the design system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeTypography {
    pub ui_font_family: &'static str,
    pub mono_font_family: &'static str,
    pub status_bar_px: f32,
    pub xs_px: f32,
    pub base_px: f32,
    pub sm_px: f32,
    pub lg_px: f32,
    pub xl_px: f32,
    pub line_height: f32,
}

/// 1:3 rhythm spacing tokens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeSpacing {
    pub xs_px: f32,
    pub sm_px: f32,
    pub md_px: f32,
    pub lg_px: f32,
    pub xl_px: f32,
    pub section_px: f32,
    pub group_px: f32,
}

/// Border radius tokens matching `src/assets/base.css`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeRadii {
    pub sm_px: f32,
    pub md_px: f32,
    pub lg_px: f32,
    pub xl_px: f32,
}

/// Structural shell dimension tokens matching `src/assets/base.css`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShellDimensions {
    pub sidebar_width_px: f32,
    pub titlebar_height_px: f32,
    pub tab_height_px: f32,
    pub statusbar_height_px: f32,
}

/// Complete resolved theme state for Table View desktop.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub mode: ThemeMode,
    pub appearance: ResolvedAppearance,
    pub colors: ThemePalette,
    pub connection_colors: ConnectionColors,
    pub typography: ThemeTypography,
    pub spacing: ThemeSpacing,
    pub radii: ThemeRadii,
    pub shell: ShellDimensions,
}

impl ThemePalette {
    /// Light theme color palette matching `src/assets/base.css` `@theme`.
    pub const fn light() -> Self {
        Self {
            primary: ThemeColor::rgb(0x4f46e5),
            primary_hover: ThemeColor::rgb(0x4338ca),
            primary_light: ThemeColor::rgb(0xeef2ff),

            success: ThemeColor::rgb(0x10b981),
            success_light: ThemeColor::rgb(0xecfdf5),
            danger: ThemeColor::rgb(0xf43f5e),
            danger_light: ThemeColor::rgb(0xfff1f2),
            warning: ThemeColor::rgb(0xf59e0b),

            surface: ThemeColor::rgb(0xffffff),
            sidebar: ThemeColor::rgb(0xf9fafb),
            border: ThemeColor::rgb(0xe5e7eb),
            border_strong: ThemeColor::rgb(0xd1d5db),
            hover: ThemeColor::rgb(0xf3f4f6),
            active: ThemeColor::rgb(0xeef2ff),
            muted: ThemeColor::rgb(0xf9fafb),

            text_primary: ThemeColor::rgb(0x1f2937),
            text_secondary: ThemeColor::rgb(0x6b7280),
            text_tertiary: ThemeColor::rgb(0x9ca3af),
            text_inverse: ThemeColor::rgb(0xffffff),

            grid_header: ThemeColor::rgb(0xf8f9fa),
            grid_row_alt: ThemeColor::rgb(0xfafbfc),
            grid_row_hover: ThemeColor::rgb(0xf0f4ff),
            grid_border: ThemeColor::rgb(0xe5e7eb),
        }
    }

    /// Dark theme color palette matching `src/assets/base.css` `.dark`.
    pub const fn dark() -> Self {
        Self {
            primary: ThemeColor::rgb(0x818cf8),
            primary_hover: ThemeColor::rgb(0x6366f1),
            primary_light: ThemeColor::rgba(0x6366f1, 0.15),

            success: ThemeColor::rgb(0x34d399),
            success_light: ThemeColor::rgba(0x10b981, 0.15),
            danger: ThemeColor::rgb(0xfb7185),
            danger_light: ThemeColor::rgba(0xf43f5e, 0.15),
            warning: ThemeColor::rgb(0xfbbf24),

            surface: ThemeColor::rgb(0x1e1e2e),
            sidebar: ThemeColor::rgb(0x181825),
            border: ThemeColor::rgb(0x313244),
            border_strong: ThemeColor::rgb(0x45475a),
            hover: ThemeColor::rgb(0x313244),
            active: ThemeColor::rgba(0x6366f1, 0.15),
            muted: ThemeColor::rgb(0x181825),

            text_primary: ThemeColor::rgb(0xcdd6f4),
            text_secondary: ThemeColor::rgb(0xa6adc8),
            text_tertiary: ThemeColor::rgb(0x6c7086),
            text_inverse: ThemeColor::rgb(0x1e1e2e),

            grid_header: ThemeColor::rgb(0x1e1e2e),
            grid_row_alt: ThemeColor::rgb(0x181825),
            grid_row_hover: ThemeColor::rgba(0x6366f1, 0.08),
            grid_border: ThemeColor::rgb(0x313244),
        }
    }
}

impl ConnectionColors {
    /// Standard 14 connection badge colors from `src/assets/base.css`.
    pub const fn standard() -> Self {
        Self {
            indigo: ThemeColor::rgb(0x4f46e5),
            blue: ThemeColor::rgb(0x3b82f6),
            teal: ThemeColor::rgb(0x14b8a6),
            green: ThemeColor::rgb(0x22c55e),
            amber: ThemeColor::rgb(0xf59e0b),
            orange: ThemeColor::rgb(0xf97316),
            pink: ThemeColor::rgb(0xec4899),
            gray: ThemeColor::rgb(0x9ca3af),
            purple: ThemeColor::rgb(0xa855f7),
            rose: ThemeColor::rgb(0xf43f5e),
            emerald: ThemeColor::rgb(0x10b981),
            cyan: ThemeColor::rgb(0x06b6d4),
            violet: ThemeColor::rgb(0x7c3aed),
            red: ThemeColor::rgb(0xef4444),
        }
    }
}

impl ThemeTypography {
    /// Standard desktop typography scale.
    pub const fn standard() -> Self {
        Self {
            ui_font_family: "Inter",
            mono_font_family: "JetBrains Mono",
            status_bar_px: 11.0,
            xs_px: 12.0,
            base_px: 13.0,
            sm_px: 14.0,
            lg_px: 18.0,
            xl_px: 20.0,
            line_height: 1.5,
        }
    }
}

impl ThemeSpacing {
    /// Standard 1:3 rhythm spacing tokens.
    pub const fn standard() -> Self {
        Self {
            xs_px: 4.0,
            sm_px: 6.0,
            md_px: 8.0,
            lg_px: 12.0,
            xl_px: 16.0,
            section_px: 24.0,
            group_px: 32.0,
        }
    }
}

impl ThemeRadii {
    /// Standard border radii matching `src/assets/base.css`.
    pub const fn standard() -> Self {
        Self {
            sm_px: 4.0,
            md_px: 6.0,
            lg_px: 8.0,
            xl_px: 12.0,
        }
    }
}

impl ShellDimensions {
    /// Standard structural shell dimensions matching `src/assets/base.css`.
    pub const fn standard() -> Self {
        Self {
            sidebar_width_px: 260.0,
            titlebar_height_px: 48.0,
            tab_height_px: 38.0,
            statusbar_height_px: 32.0,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Returns the default dark theme.
    pub const fn dark() -> Self {
        Self::from_mode_and_appearance(ThemeMode::Dark, ResolvedAppearance::Dark)
    }

    /// Returns the light theme.
    #[allow(dead_code)]
    pub const fn light() -> Self {
        Self::from_mode_and_appearance(ThemeMode::Light, ResolvedAppearance::Light)
    }

    /// Resolves a concrete `Theme` from a `ThemeMode` and current OS `ResolvedAppearance`.
    pub const fn resolve(mode: ThemeMode, system_appearance: ResolvedAppearance) -> Self {
        let appearance = match mode {
            ThemeMode::Light => ResolvedAppearance::Light,
            ThemeMode::Dark => ResolvedAppearance::Dark,
            ThemeMode::System => system_appearance,
        };
        Self::from_mode_and_appearance(mode, appearance)
    }

    const fn from_mode_and_appearance(mode: ThemeMode, appearance: ResolvedAppearance) -> Self {
        let colors = match appearance {
            ResolvedAppearance::Light => ThemePalette::light(),
            ResolvedAppearance::Dark => ThemePalette::dark(),
        };
        Self {
            mode,
            appearance,
            colors,
            connection_colors: ConnectionColors::standard(),
            typography: ThemeTypography::standard(),
            spacing: ThemeSpacing::standard(),
            radii: ThemeRadii::standard(),
            shell: ShellDimensions::standard(),
        }
    }

    /// Returns `true` when the resolved appearance is dark.
    pub const fn is_dark(&self) -> bool {
        matches!(self.appearance, ResolvedAppearance::Dark)
    }
}

#[cfg(feature = "native-ui")]
mod gpui_integration {
    use super::*;

    impl ThemeColor {
        /// Converts this color to GPUI's `Rgba`.
        pub fn to_gpui(self) -> gpui::Rgba {
            gpui::Rgba {
                r: f32::from(self.r) / 255.0,
                g: f32::from(self.g) / 255.0,
                b: f32::from(self.b) / 255.0,
                a: f32::from(self.a) / 255.0,
            }
        }

        /// Converts this color to GPUI's `Hsla`.
        pub fn to_hsla(self) -> gpui::Hsla {
            gpui::Hsla::from(self.to_gpui())
        }
    }

    impl From<ThemeColor> for gpui::Rgba {
        fn from(color: ThemeColor) -> Self {
            color.to_gpui()
        }
    }

    impl From<ThemeColor> for gpui::Hsla {
        fn from(color: ThemeColor) -> Self {
            color.to_hsla()
        }
    }

    impl From<gpui::WindowAppearance> for ResolvedAppearance {
        fn from(appearance: gpui::WindowAppearance) -> Self {
            match appearance {
                gpui::WindowAppearance::Light | gpui::WindowAppearance::VibrantLight => Self::Light,
                gpui::WindowAppearance::Dark | gpui::WindowAppearance::VibrantDark => Self::Dark,
            }
        }
    }

    /// GPUI global wrapper holding the active [`Theme`].
    #[derive(Debug, Clone)]
    pub struct GlobalTheme(pub Theme);

    impl gpui::Global for GlobalTheme {}

    /// Extension trait for reading the active [`Theme`] from GPUI's `App` context.
    pub trait ActiveTheme {
        /// Returns a reference to the active [`Theme`].
        fn theme(&self) -> &Theme;
    }

    impl ActiveTheme for gpui::App {
        fn theme(&self) -> &Theme {
            &self.global::<GlobalTheme>().0
        }
    }

    impl Theme {
        /// Returns the active global [`Theme`].
        #[allow(dead_code)]
        pub fn global(cx: &gpui::App) -> &Theme {
            cx.theme()
        }

        /// Initializes [`GlobalTheme`] in GPUI's `App` context.
        pub fn init(
            cx: &mut gpui::App,
            mode: ThemeMode,
            system_appearance: ResolvedAppearance,
        ) {
            cx.set_global(GlobalTheme(Self::resolve(mode, system_appearance)));
        }

        /// Updates the active [`ThemeMode`] and re-resolves [`GlobalTheme`].
        #[allow(dead_code)]
        pub fn set_mode(
            cx: &mut gpui::App,
            mode: ThemeMode,
            system_appearance: ResolvedAppearance,
        ) {
            cx.set_global(GlobalTheme(Self::resolve(mode, system_appearance)));
        }

        /// Re-resolves [`GlobalTheme`] on OS appearance change when `mode == ThemeMode::System`.
        pub fn sync_system_appearance(
            cx: &mut gpui::App,
            system_appearance: ResolvedAppearance,
        ) {
            if cx.theme().mode == ThemeMode::System {
                cx.set_global(GlobalTheme(Self::resolve(ThemeMode::System, system_appearance)));
            }
        }

        /// Toggles between `ThemeMode::Light` and `ThemeMode::Dark` based on current `is_dark()`.
        pub fn toggle_light_dark(cx: &mut gpui::App) {
            let next_mode = if cx.theme().is_dark() {
                ThemeMode::Light
            } else {
                ThemeMode::Dark
            };
            let current_appearance = cx.theme().appearance;
            cx.set_global(GlobalTheme(Self::resolve(next_mode, current_appearance)));
        }
    }
}

#[cfg(feature = "native-ui")]
#[allow(unused_imports)]
pub use gpui_integration::{ActiveTheme, GlobalTheme};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme_is_dark_with_base_css_tokens() {
        let theme = Theme::default();
        assert_eq!(theme.mode, ThemeMode::Dark);
        assert_eq!(theme.appearance, ResolvedAppearance::Dark);
        assert!(theme.is_dark());
        assert_eq!(theme.colors.surface, ThemeColor::rgb(0x1e1e2e));
        assert_eq!(theme.colors.sidebar, ThemeColor::rgb(0x181825));
        assert_eq!(theme.colors.primary, ThemeColor::rgb(0x818cf8));
        assert_eq!(theme.colors.text_primary, ThemeColor::rgb(0xcdd6f4));
        assert_eq!(theme.colors.primary_light, ThemeColor::rgba(0x6366f1, 0.15));
        assert_eq!(
            theme.colors.grid_row_hover,
            ThemeColor::rgba(0x6366f1, 0.08)
        );
    }

    #[test]
    fn test_light_theme_tokens_match_base_css() {
        let theme = Theme::light();
        assert_eq!(theme.mode, ThemeMode::Light);
        assert_eq!(theme.appearance, ResolvedAppearance::Light);
        assert!(!theme.is_dark());
        assert_eq!(theme.colors.surface, ThemeColor::rgb(0xffffff));
        assert_eq!(theme.colors.sidebar, ThemeColor::rgb(0xf9fafb));
        assert_eq!(theme.colors.primary, ThemeColor::rgb(0x4f46e5));
        assert_eq!(theme.colors.text_primary, ThemeColor::rgb(0x1f2937));
        assert_eq!(theme.colors.grid_header, ThemeColor::rgb(0xf8f9fa));
    }

    #[test]
    fn test_system_mode_resolves_from_system_appearance() {
        let sys_light = Theme::resolve(ThemeMode::System, ResolvedAppearance::Light);
        assert_eq!(sys_light.mode, ThemeMode::System);
        assert_eq!(sys_light.appearance, ResolvedAppearance::Light);
        assert_eq!(sys_light.colors, Theme::light().colors);

        let sys_dark = Theme::resolve(ThemeMode::System, ResolvedAppearance::Dark);
        assert_eq!(sys_dark.mode, ThemeMode::System);
        assert_eq!(sys_dark.appearance, ResolvedAppearance::Dark);
        assert_eq!(sys_dark.colors, Theme::dark().colors);

        let forced_light = Theme::resolve(ThemeMode::Light, ResolvedAppearance::Dark);
        assert_eq!(forced_light.appearance, ResolvedAppearance::Light);
        assert_eq!(forced_light.colors, Theme::light().colors);

        let forced_dark = Theme::resolve(ThemeMode::Dark, ResolvedAppearance::Light);
        assert_eq!(forced_dark.appearance, ResolvedAppearance::Dark);
        assert_eq!(forced_dark.colors, Theme::dark().colors);
    }

    #[test]
    fn test_typography_spacing_radii_shell_and_connection_tokens() {
        let theme = Theme::default();

        assert_eq!(theme.connection_colors.indigo, ThemeColor::rgb(0x4f46e5));
        assert_eq!(theme.connection_colors.blue, ThemeColor::rgb(0x3b82f6));
        assert_eq!(theme.connection_colors.teal, ThemeColor::rgb(0x14b8a6));
        assert_eq!(theme.connection_colors.green, ThemeColor::rgb(0x22c55e));
        assert_eq!(theme.connection_colors.amber, ThemeColor::rgb(0xf59e0b));
        assert_eq!(theme.connection_colors.orange, ThemeColor::rgb(0xf97316));
        assert_eq!(theme.connection_colors.pink, ThemeColor::rgb(0xec4899));
        assert_eq!(theme.connection_colors.gray, ThemeColor::rgb(0x9ca3af));
        assert_eq!(theme.connection_colors.purple, ThemeColor::rgb(0xa855f7));
        assert_eq!(theme.connection_colors.rose, ThemeColor::rgb(0xf43f5e));
        assert_eq!(theme.connection_colors.emerald, ThemeColor::rgb(0x10b981));
        assert_eq!(theme.connection_colors.cyan, ThemeColor::rgb(0x06b6d4));
        assert_eq!(theme.connection_colors.violet, ThemeColor::rgb(0x7c3aed));
        assert_eq!(theme.connection_colors.red, ThemeColor::rgb(0xef4444));

        assert_eq!(theme.typography.ui_font_family, "Inter");
        assert_eq!(theme.typography.mono_font_family, "JetBrains Mono");
        assert_eq!(theme.typography.status_bar_px, 11.0);
        assert_eq!(theme.typography.xs_px, 12.0);
        assert_eq!(theme.typography.base_px, 13.0);
        assert_eq!(theme.typography.sm_px, 14.0);
        assert_eq!(theme.typography.lg_px, 18.0);
        assert_eq!(theme.typography.xl_px, 20.0);
        assert_eq!(theme.typography.line_height, 1.5);

        assert_eq!(theme.spacing.xs_px, 4.0);
        assert_eq!(theme.spacing.sm_px, 6.0);
        assert_eq!(theme.spacing.md_px, 8.0);
        assert_eq!(theme.spacing.lg_px, 12.0);
        assert_eq!(theme.spacing.xl_px, 16.0);
        assert_eq!(theme.spacing.section_px, 24.0);
        assert_eq!(theme.spacing.group_px, 32.0);

        assert_eq!(theme.radii.sm_px, 4.0);
        assert_eq!(theme.radii.md_px, 6.0);
        assert_eq!(theme.radii.lg_px, 8.0);
        assert_eq!(theme.radii.xl_px, 12.0);

        assert_eq!(theme.shell.sidebar_width_px, 260.0);
        assert_eq!(theme.shell.titlebar_height_px, 48.0);
        assert_eq!(theme.shell.tab_height_px, 38.0);
        assert_eq!(theme.shell.statusbar_height_px, 32.0);
    }
}

#[cfg(all(test, feature = "native-ui"))]
mod gpui_tests {
    use super::*;

    #[test]
    fn test_theme_color_to_gpui_rgba_preserves_channels_and_alpha() {
        let opaque = ThemeColor::rgb(0x4f46e5).to_gpui();
        assert!((opaque.r - (0x4f as f32 / 255.0)).abs() < 1e-6);
        assert!((opaque.g - (0x46 as f32 / 255.0)).abs() < 1e-6);
        assert!((opaque.b - (0xe5 as f32 / 255.0)).abs() < 1e-6);
        assert!((opaque.a - 1.0).abs() < 1e-6);

        let translucent = ThemeColor::rgba(0x6366f1, 0.15).to_gpui();
        assert!((translucent.r - (0x63 as f32 / 255.0)).abs() < 1e-6);
        assert!((translucent.g - (0x66 as f32 / 255.0)).abs() < 1e-6);
        assert!((translucent.b - (0xf1 as f32 / 255.0)).abs() < 1e-6);
        assert!((translucent.a - (38.0 / 255.0)).abs() < 1e-6);
    }

    #[test]
    fn test_gpui_global_theme_init_toggle_and_system_sync() {
        gpui::Application::headless().run(|cx| {
            Theme::init(cx, ThemeMode::Dark, ResolvedAppearance::Light);
            assert!(cx.theme().is_dark());
            assert_eq!(cx.theme().mode, ThemeMode::Dark);

            Theme::toggle_light_dark(cx);
            assert!(!cx.theme().is_dark());
            assert_eq!(cx.theme().mode, ThemeMode::Light);

            Theme::set_mode(cx, ThemeMode::System, ResolvedAppearance::Light);
            assert!(!cx.theme().is_dark());

            Theme::sync_system_appearance(cx, ResolvedAppearance::Dark);
            assert!(cx.theme().is_dark());
            assert_eq!(cx.theme().mode, ThemeMode::System);

            cx.quit();
        });
    }
}
