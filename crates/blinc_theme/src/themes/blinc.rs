//! Default Blinc theme derived from Catppuccin design system
//!
//! Catppuccin is a community-driven pastel theme with four flavors:
//! - Latte (light)
//! - Frappé (light-dark)
//! - Macchiato (dark-light)
//! - Mocha (dark)
//!
//! We use Latte for light mode and Mocha for dark mode.

use crate::theme::{ColorScheme, Theme, ThemeBundle};
use crate::tokens::*;
use blinc_core::Color;

/// Catppuccin Latte palette (light theme)
pub mod latte {
    use blinc_core::Color;

    // Base colors
    pub const ROSEWATER: Color = Color::rgb(220.0 / 255.0, 138.0 / 255.0, 120.0 / 255.0);
    pub const FLAMINGO: Color = Color::rgb(221.0 / 255.0, 120.0 / 255.0, 120.0 / 255.0);
    pub const PINK: Color = Color::rgb(234.0 / 255.0, 118.0 / 255.0, 203.0 / 255.0);
    pub const MAUVE: Color = Color::rgb(136.0 / 255.0, 57.0 / 255.0, 239.0 / 255.0);
    pub const RED: Color = Color::rgb(210.0 / 255.0, 15.0 / 255.0, 57.0 / 255.0);
    pub const MAROON: Color = Color::rgb(230.0 / 255.0, 69.0 / 255.0, 83.0 / 255.0);
    pub const PEACH: Color = Color::rgb(254.0 / 255.0, 100.0 / 255.0, 11.0 / 255.0);
    pub const YELLOW: Color = Color::rgb(223.0 / 255.0, 142.0 / 255.0, 29.0 / 255.0);
    pub const GREEN: Color = Color::rgb(64.0 / 255.0, 160.0 / 255.0, 43.0 / 255.0);
    pub const TEAL: Color = Color::rgb(23.0 / 255.0, 146.0 / 255.0, 153.0 / 255.0);
    pub const SKY: Color = Color::rgb(4.0 / 255.0, 165.0 / 255.0, 229.0 / 255.0);
    pub const SAPPHIRE: Color = Color::rgb(32.0 / 255.0, 159.0 / 255.0, 181.0 / 255.0);
    pub const BLUE: Color = Color::rgb(30.0 / 255.0, 102.0 / 255.0, 245.0 / 255.0);
    pub const LAVENDER: Color = Color::rgb(114.0 / 255.0, 135.0 / 255.0, 253.0 / 255.0);

    // Surface colors
    pub const TEXT: Color = Color::rgb(76.0 / 255.0, 79.0 / 255.0, 105.0 / 255.0);
    pub const SUBTEXT1: Color = Color::rgb(92.0 / 255.0, 95.0 / 255.0, 119.0 / 255.0);
    pub const SUBTEXT0: Color = Color::rgb(108.0 / 255.0, 111.0 / 255.0, 133.0 / 255.0);
    pub const OVERLAY2: Color = Color::rgb(124.0 / 255.0, 127.0 / 255.0, 147.0 / 255.0);
    pub const OVERLAY1: Color = Color::rgb(140.0 / 255.0, 143.0 / 255.0, 161.0 / 255.0);
    pub const OVERLAY0: Color = Color::rgb(156.0 / 255.0, 160.0 / 255.0, 176.0 / 255.0);
    pub const SURFACE2: Color = Color::rgb(172.0 / 255.0, 176.0 / 255.0, 190.0 / 255.0);
    pub const SURFACE1: Color = Color::rgb(188.0 / 255.0, 192.0 / 255.0, 204.0 / 255.0);
    pub const SURFACE0: Color = Color::rgb(204.0 / 255.0, 208.0 / 255.0, 218.0 / 255.0);
    pub const BASE: Color = Color::rgb(239.0 / 255.0, 241.0 / 255.0, 245.0 / 255.0);
    pub const MANTLE: Color = Color::rgb(230.0 / 255.0, 233.0 / 255.0, 239.0 / 255.0);
    pub const CRUST: Color = Color::rgb(220.0 / 255.0, 224.0 / 255.0, 232.0 / 255.0);
}

/// Catppuccin Mocha palette (dark theme)
pub mod mocha {
    use blinc_core::Color;

    // Base colors
    pub const ROSEWATER: Color = Color::rgb(245.0 / 255.0, 224.0 / 255.0, 220.0 / 255.0);
    pub const FLAMINGO: Color = Color::rgb(242.0 / 255.0, 205.0 / 255.0, 205.0 / 255.0);
    pub const PINK: Color = Color::rgb(245.0 / 255.0, 194.0 / 255.0, 231.0 / 255.0);
    pub const MAUVE: Color = Color::rgb(203.0 / 255.0, 166.0 / 255.0, 247.0 / 255.0);
    pub const RED: Color = Color::rgb(243.0 / 255.0, 139.0 / 255.0, 168.0 / 255.0);
    pub const MAROON: Color = Color::rgb(235.0 / 255.0, 160.0 / 255.0, 172.0 / 255.0);
    pub const PEACH: Color = Color::rgb(250.0 / 255.0, 179.0 / 255.0, 135.0 / 255.0);
    pub const YELLOW: Color = Color::rgb(249.0 / 255.0, 226.0 / 255.0, 175.0 / 255.0);
    pub const GREEN: Color = Color::rgb(166.0 / 255.0, 227.0 / 255.0, 161.0 / 255.0);
    pub const TEAL: Color = Color::rgb(148.0 / 255.0, 226.0 / 255.0, 213.0 / 255.0);
    pub const SKY: Color = Color::rgb(137.0 / 255.0, 220.0 / 255.0, 235.0 / 255.0);
    pub const SAPPHIRE: Color = Color::rgb(116.0 / 255.0, 199.0 / 255.0, 236.0 / 255.0);
    pub const BLUE: Color = Color::rgb(137.0 / 255.0, 180.0 / 255.0, 250.0 / 255.0);
    pub const LAVENDER: Color = Color::rgb(180.0 / 255.0, 190.0 / 255.0, 254.0 / 255.0);

    // Surface colors
    pub const TEXT: Color = Color::rgb(205.0 / 255.0, 214.0 / 255.0, 244.0 / 255.0);
    pub const SUBTEXT1: Color = Color::rgb(186.0 / 255.0, 194.0 / 255.0, 222.0 / 255.0);
    pub const SUBTEXT0: Color = Color::rgb(166.0 / 255.0, 173.0 / 255.0, 200.0 / 255.0);
    pub const OVERLAY2: Color = Color::rgb(147.0 / 255.0, 153.0 / 255.0, 178.0 / 255.0);
    pub const OVERLAY1: Color = Color::rgb(127.0 / 255.0, 132.0 / 255.0, 156.0 / 255.0);
    pub const OVERLAY0: Color = Color::rgb(108.0 / 255.0, 112.0 / 255.0, 134.0 / 255.0);
    pub const SURFACE2: Color = Color::rgb(88.0 / 255.0, 91.0 / 255.0, 112.0 / 255.0);
    pub const SURFACE1: Color = Color::rgb(69.0 / 255.0, 71.0 / 255.0, 90.0 / 255.0);
    pub const SURFACE0: Color = Color::rgb(49.0 / 255.0, 50.0 / 255.0, 68.0 / 255.0);
    pub const BASE: Color = Color::rgb(30.0 / 255.0, 30.0 / 255.0, 46.0 / 255.0);
    pub const MANTLE: Color = Color::rgb(24.0 / 255.0, 24.0 / 255.0, 37.0 / 255.0);
    pub const CRUST: Color = Color::rgb(17.0 / 255.0, 17.0 / 255.0, 27.0 / 255.0);
}

/// Default Blinc theme derived from Catppuccin
#[derive(Clone, Debug)]
pub struct BlincTheme {
    scheme: ColorScheme,
    colors: ColorTokens,
    typography: TypographyTokens,
    spacing: SpacingTokens,
    radii: RadiusTokens,
    shadows: ShadowTokens,
    animations: AnimationTokens,
}

impl BlincTheme {
    /// Create the light variant (Catppuccin Latte)
    pub fn light() -> Self {
        Self {
            scheme: ColorScheme::Light,
            colors: ColorTokens {
                primary: latte::BLUE,
                primary_hover: Color::from_hex(0x1758D1),
                primary_active: Color::from_hex(0x114AB3),
                secondary: latte::MAUVE,
                secondary_hover: Color::from_hex(0x7530D4),
                secondary_active: Color::from_hex(0x6228B9),
                success: latte::GREEN,
                success_bg: latte::GREEN.with_alpha(0.1),
                warning: latte::YELLOW,
                warning_bg: latte::YELLOW.with_alpha(0.1),
                error: latte::RED,
                error_bg: latte::RED.with_alpha(0.1),
                info: latte::SKY,
                info_bg: latte::SKY.with_alpha(0.1),
                background: latte::BASE,
                surface: Color::WHITE,
                surface_elevated: Color::WHITE,
                surface_overlay: latte::MANTLE,
                text_primary: latte::TEXT,
                text_secondary: latte::SUBTEXT1,
                text_tertiary: latte::OVERLAY0,
                text_inverse: Color::WHITE,
                text_link: latte::BLUE,
                border: latte::SURFACE0,
                border_secondary: latte::OVERLAY0, // Visible solid gray for form controls
                border_hover: latte::SURFACE1,
                border_focus: latte::BLUE,
                border_error: latte::RED,
                input_bg: Color::WHITE,
                input_bg_hover: latte::BASE,
                input_bg_focus: Color::WHITE,
                input_bg_disabled: latte::MANTLE,
                selection: latte::BLUE.with_alpha(0.3),
                selection_text: latte::TEXT,
                accent: latte::BLUE,
                accent_subtle: latte::BLUE.with_alpha(0.1),
                // Tooltip (inverted for light theme)
                tooltip_bg: mocha::BASE,
                tooltip_text: mocha::TEXT,
            },
            typography: TypographyTokens::default(),
            spacing: SpacingTokens::default(),
            radii: RadiusTokens::default(),
            shadows: ShadowTokens::light(),
            animations: AnimationTokens::default(),
        }
    }

    /// Create the dark variant (Catppuccin Mocha)
    pub fn dark() -> Self {
        Self {
            scheme: ColorScheme::Dark,
            colors: ColorTokens {
                primary: mocha::BLUE,
                primary_hover: Color::from_hex(0x9ECBFC),
                primary_active: Color::from_hex(0xB5D7FD),
                secondary: mocha::MAUVE,
                secondary_hover: Color::from_hex(0xD5C4F7),
                secondary_active: Color::from_hex(0xE2D8FA),
                success: mocha::GREEN,
                success_bg: mocha::GREEN.with_alpha(0.15),
                warning: mocha::YELLOW,
                warning_bg: mocha::YELLOW.with_alpha(0.15),
                error: mocha::RED,
                error_bg: mocha::RED.with_alpha(0.15),
                info: mocha::SKY,
                info_bg: mocha::SKY.with_alpha(0.15),
                background: mocha::BASE,
                surface: mocha::SURFACE0,
                surface_elevated: mocha::SURFACE1,
                surface_overlay: mocha::MANTLE,
                text_primary: mocha::TEXT,
                text_secondary: mocha::SUBTEXT1,
                text_tertiary: mocha::OVERLAY0,
                text_inverse: mocha::CRUST,
                text_link: mocha::BLUE,
                border: mocha::SURFACE1,
                border_secondary: mocha::OVERLAY0, // Visible solid gray for form controls
                border_hover: mocha::SURFACE2,
                border_focus: mocha::BLUE,
                border_error: mocha::RED,
                input_bg: mocha::SURFACE0,
                input_bg_hover: mocha::SURFACE1,
                input_bg_focus: mocha::SURFACE0,
                input_bg_disabled: mocha::MANTLE,
                selection: mocha::BLUE.with_alpha(0.3),
                selection_text: mocha::TEXT,
                accent: mocha::BLUE,
                accent_subtle: mocha::BLUE.with_alpha(0.15),
                // Tooltip (inverted for dark theme)
                tooltip_bg: latte::BASE,
                tooltip_text: latte::TEXT,
            },
            typography: TypographyTokens::default(),
            spacing: SpacingTokens::default(),
            radii: RadiusTokens::default(),
            shadows: ShadowTokens::dark(),
            animations: AnimationTokens::default(),
        }
    }

    /// Create a theme bundle with light and dark variants
    pub fn bundle() -> ThemeBundle {
        ThemeBundle::new("Blinc", Self::light(), Self::dark())
    }
}

impl Theme for BlincTheme {
    fn name(&self) -> &str {
        "Blinc"
    }

    fn color_scheme(&self) -> ColorScheme {
        self.scheme
    }

    fn colors(&self) -> &ColorTokens {
        &self.colors
    }

    fn typography(&self) -> &TypographyTokens {
        &self.typography
    }

    fn spacing(&self) -> &SpacingTokens {
        &self.spacing
    }

    fn radii(&self) -> &RadiusTokens {
        &self.radii
    }

    fn shadows(&self) -> &ShadowTokens {
        &self.shadows
    }

    fn animations(&self) -> &AnimationTokens {
        &self.animations
    }
}
