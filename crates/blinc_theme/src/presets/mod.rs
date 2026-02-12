//! Built-in theme presets inspired by shadcn base color presets.

use crate::theme::{ColorScheme, Theme, ThemeBundle};
use crate::themes::BlincTheme;
use crate::tokens::*;
use blinc_core::Color;
use std::fmt::{Display, Formatter};

/// Built-in theme preset catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ThemePreset {
    /// Existing Blinc (Catppuccin-based) theme.
    Blinc,
    /// shadcn-inspired neutral preset.
    Neutral,
    /// shadcn-inspired slate preset.
    Slate,
    /// shadcn-inspired zinc preset.
    Zinc,
}

impl ThemePreset {
    /// Stable preset id for config/serialization.
    pub fn id(self) -> &'static str {
        match self {
            Self::Blinc => "blinc",
            Self::Neutral => "neutral",
            Self::Slate => "slate",
            Self::Zinc => "zinc",
        }
    }

    /// User-facing display name.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Blinc => "Blinc",
            Self::Neutral => "Neutral",
            Self::Slate => "Slate",
            Self::Zinc => "Zinc",
        }
    }

    /// Full preset list.
    pub fn all() -> &'static [ThemePreset] {
        const PRESETS: [ThemePreset; 4] = [
            ThemePreset::Blinc,
            ThemePreset::Neutral,
            ThemePreset::Slate,
            ThemePreset::Zinc,
        ];
        &PRESETS
    }

    /// Build a light/dark theme bundle for this preset.
    pub fn bundle(self) -> ThemeBundle {
        match self {
            Self::Blinc => BlincTheme::bundle(),
            Self::Neutral => shadcn_bundle("Neutral", neutral_light(), neutral_dark()),
            Self::Slate => shadcn_bundle("Slate", slate_light(), slate_dark()),
            Self::Zinc => shadcn_bundle("Zinc", zinc_light(), zinc_dark()),
        }
    }
}

impl Display for ThemePreset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}

/// Convenience free function for ergonomic imports.
pub fn preset_bundle(preset: ThemePreset) -> ThemeBundle {
    preset.bundle()
}

#[derive(Clone, Copy)]
struct BasePalette {
    background: Color,
    foreground: Color,
    card: Color,
    primary: Color,
    primary_foreground: Color,
    secondary: Color,
    muted: Color,
    muted_foreground: Color,
    accent: Color,
    destructive: Color,
    border: Color,
    ring: Color,
}

#[derive(Clone, Debug)]
struct PresetTheme {
    name: &'static str,
    scheme: ColorScheme,
    colors: ColorTokens,
    typography: TypographyTokens,
    spacing: SpacingTokens,
    radii: RadiusTokens,
    shadows: ShadowTokens,
    animations: AnimationTokens,
}

impl Theme for PresetTheme {
    fn name(&self) -> &str {
        self.name
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

fn shadcn_bundle(name: &'static str, light: BasePalette, dark: BasePalette) -> ThemeBundle {
    ThemeBundle::new(
        name,
        PresetTheme {
            name,
            scheme: ColorScheme::Light,
            colors: build_colors(light, ColorScheme::Light),
            typography: TypographyTokens::default(),
            spacing: SpacingTokens::default(),
            radii: shadcn_radii(),
            shadows: ShadowTokens::light(),
            animations: AnimationTokens::default(),
        },
        PresetTheme {
            name,
            scheme: ColorScheme::Dark,
            colors: build_colors(dark, ColorScheme::Dark),
            typography: TypographyTokens::default(),
            spacing: SpacingTokens::default(),
            radii: shadcn_radii(),
            shadows: ShadowTokens::dark(),
            animations: AnimationTokens::default(),
        },
    )
}

fn build_colors(base: BasePalette, scheme: ColorScheme) -> ColorTokens {
    let (primary_hover_mix, primary_active_mix, secondary_hover_mix, secondary_active_mix) =
        match scheme {
            ColorScheme::Light => (0.10, 0.20, 0.08, 0.16),
            ColorScheme::Dark => (0.06, 0.12, 0.06, 0.10),
        };
    let state_target = match scheme {
        ColorScheme::Light => Color::BLACK,
        ColorScheme::Dark => Color::WHITE,
    };

    let text_tertiary = blend(base.muted_foreground, base.background, 0.25);
    let border_hover = blend(base.border, state_target, 0.16);
    let selection_alpha = match scheme {
        ColorScheme::Light => 0.22,
        ColorScheme::Dark => 0.28,
    };
    let subtle_alpha = match scheme {
        ColorScheme::Light => 0.14,
        ColorScheme::Dark => 0.24,
    };
    let success = match scheme {
        ColorScheme::Light => Color::from_hex(0x16A34A),
        ColorScheme::Dark => Color::from_hex(0x22C55E),
    };
    let warning = match scheme {
        ColorScheme::Light => Color::from_hex(0xD97706),
        ColorScheme::Dark => Color::from_hex(0xF59E0B),
    };
    let info = match scheme {
        ColorScheme::Light => Color::from_hex(0x0EA5E9),
        ColorScheme::Dark => Color::from_hex(0x38BDF8),
    };

    ColorTokens {
        primary: base.primary,
        primary_hover: blend(base.primary, state_target, primary_hover_mix),
        primary_active: blend(base.primary, state_target, primary_active_mix),
        secondary: base.secondary,
        secondary_hover: blend(base.secondary, state_target, secondary_hover_mix),
        secondary_active: blend(base.secondary, state_target, secondary_active_mix),
        success,
        success_bg: success.with_alpha(subtle_alpha),
        warning,
        warning_bg: warning.with_alpha(subtle_alpha),
        error: base.destructive,
        error_bg: base.destructive.with_alpha(subtle_alpha),
        info,
        info_bg: info.with_alpha(subtle_alpha),
        background: base.background,
        surface: base.card,
        surface_elevated: blend(base.card, state_target, 0.04),
        surface_overlay: base.muted,
        text_primary: base.foreground,
        text_secondary: base.muted_foreground,
        text_tertiary,
        text_inverse: base.primary_foreground,
        text_link: base.primary,
        border: base.border,
        border_hover,
        border_focus: base.ring,
        border_error: base.destructive,
        input_bg: base.background,
        input_bg_hover: base.card,
        input_bg_focus: base.background,
        input_bg_disabled: base.muted,
        selection: base.primary.with_alpha(selection_alpha),
        selection_text: base.foreground,
        accent: base.accent,
        accent_subtle: base.accent.with_alpha(subtle_alpha),
        tooltip_bg: base.foreground,
        tooltip_text: base.background,
    }
}

fn shadcn_radii() -> RadiusTokens {
    RadiusTokens {
        radius_none: 0.0,
        radius_sm: 6.0,
        radius_default: 8.0,
        radius_md: 10.0,
        radius_lg: 14.0,
        radius_xl: 18.0,
        radius_2xl: 22.0,
        radius_3xl: 26.0,
        radius_full: 9999.0,
    }
}

fn blend(a: Color, b: Color, t: f32) -> Color {
    Color::lerp(&a, &b, t)
}

fn neutral_light() -> BasePalette {
    BasePalette {
        background: Color::from_hex(0xFFFFFF),
        foreground: Color::from_hex(0x0A0A0A),
        card: Color::from_hex(0xFFFFFF),
        primary: Color::from_hex(0x171717),
        primary_foreground: Color::from_hex(0xFAFAFA),
        secondary: Color::from_hex(0xF5F5F5),
        muted: Color::from_hex(0xF5F5F5),
        muted_foreground: Color::from_hex(0x737373),
        accent: Color::from_hex(0xF5F5F5),
        destructive: Color::from_hex(0xEF4444),
        border: Color::from_hex(0xE5E5E5),
        ring: Color::from_hex(0x0A0A0A),
    }
}

fn neutral_dark() -> BasePalette {
    BasePalette {
        background: Color::from_hex(0x0A0A0A),
        foreground: Color::from_hex(0xFAFAFA),
        card: Color::from_hex(0x0A0A0A),
        primary: Color::from_hex(0xFAFAFA),
        primary_foreground: Color::from_hex(0x171717),
        secondary: Color::from_hex(0x262626),
        muted: Color::from_hex(0x262626),
        muted_foreground: Color::from_hex(0xA3A3A3),
        accent: Color::from_hex(0x262626),
        destructive: Color::from_hex(0x7F1D1D),
        border: Color::from_hex(0x262626),
        ring: Color::from_hex(0xD4D4D4),
    }
}

fn slate_light() -> BasePalette {
    BasePalette {
        background: Color::from_hex(0xFFFFFF),
        foreground: Color::from_hex(0x020817),
        card: Color::from_hex(0xFFFFFF),
        primary: Color::from_hex(0x0F172A),
        primary_foreground: Color::from_hex(0xF8FAFC),
        secondary: Color::from_hex(0xF1F5F9),
        muted: Color::from_hex(0xF1F5F9),
        muted_foreground: Color::from_hex(0x64748B),
        accent: Color::from_hex(0xF1F5F9),
        destructive: Color::from_hex(0xEF4444),
        border: Color::from_hex(0xE2E8F0),
        ring: Color::from_hex(0x020817),
    }
}

fn slate_dark() -> BasePalette {
    BasePalette {
        background: Color::from_hex(0x020817),
        foreground: Color::from_hex(0xF8FAFC),
        card: Color::from_hex(0x020817),
        primary: Color::from_hex(0xF8FAFC),
        primary_foreground: Color::from_hex(0x0F172A),
        secondary: Color::from_hex(0x1E293B),
        muted: Color::from_hex(0x1E293B),
        muted_foreground: Color::from_hex(0x94A3B8),
        accent: Color::from_hex(0x1E293B),
        destructive: Color::from_hex(0x7F1D1D),
        border: Color::from_hex(0x1E293B),
        ring: Color::from_hex(0xCBD5E1),
    }
}

fn zinc_light() -> BasePalette {
    BasePalette {
        background: Color::from_hex(0xFFFFFF),
        foreground: Color::from_hex(0x09090B),
        card: Color::from_hex(0xFFFFFF),
        primary: Color::from_hex(0x18181B),
        primary_foreground: Color::from_hex(0xFAFAFA),
        secondary: Color::from_hex(0xF4F4F5),
        muted: Color::from_hex(0xF4F4F5),
        muted_foreground: Color::from_hex(0x71717A),
        accent: Color::from_hex(0xF4F4F5),
        destructive: Color::from_hex(0xEF4444),
        border: Color::from_hex(0xE4E4E7),
        ring: Color::from_hex(0x09090B),
    }
}

fn zinc_dark() -> BasePalette {
    BasePalette {
        background: Color::from_hex(0x09090B),
        foreground: Color::from_hex(0xFAFAFA),
        card: Color::from_hex(0x09090B),
        primary: Color::from_hex(0xFAFAFA),
        primary_foreground: Color::from_hex(0x18181B),
        secondary: Color::from_hex(0x27272A),
        muted: Color::from_hex(0x27272A),
        muted_foreground: Color::from_hex(0xA1A1AA),
        accent: Color::from_hex(0x27272A),
        destructive: Color::from_hex(0x7F1D1D),
        border: Color::from_hex(0x27272A),
        ring: Color::from_hex(0xD4D4D8),
    }
}
