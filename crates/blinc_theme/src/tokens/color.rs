//! Color tokens for theming

use blinc_core::Color;

/// Semantic color token keys for dynamic access
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum ColorToken {
    // Brand colors
    Primary,
    PrimaryHover,
    PrimaryActive,
    Secondary,
    SecondaryHover,
    SecondaryActive,

    // Semantic colors
    Success,
    SuccessBg,
    Warning,
    WarningBg,
    Error,
    ErrorBg,
    Info,
    InfoBg,

    // Surface colors
    Background,
    Surface,
    SurfaceElevated,
    SurfaceOverlay,

    // Text colors
    TextPrimary,
    TextSecondary,
    TextTertiary,
    TextInverse,
    TextLink,

    // Border colors
    Border,
    BorderSecondary,
    BorderHover,
    BorderFocus,
    BorderError,

    // Input element colors
    InputBg,
    InputBgHover,
    InputBgFocus,
    InputBgDisabled,

    // Selection colors
    Selection,
    SelectionText,

    // Accent
    Accent,
    AccentSubtle,

    // Tooltip colors (inverted colors)
    TooltipBackground,
    TooltipText,
}

/// Complete set of semantic color tokens
#[derive(Clone, Debug)]
pub struct ColorTokens {
    // Brand colors
    pub primary: Color,
    pub primary_hover: Color,
    pub primary_active: Color,
    pub secondary: Color,
    pub secondary_hover: Color,
    pub secondary_active: Color,

    // Semantic colors
    pub success: Color,
    pub success_bg: Color,
    pub warning: Color,
    pub warning_bg: Color,
    pub error: Color,
    pub error_bg: Color,
    pub info: Color,
    pub info_bg: Color,

    // Surface colors
    pub background: Color,
    pub surface: Color,
    pub surface_elevated: Color,
    pub surface_overlay: Color,

    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,
    pub text_inverse: Color,
    pub text_link: Color,

    // Border colors
    pub border: Color,
    pub border_secondary: Color,
    pub border_hover: Color,
    pub border_focus: Color,
    pub border_error: Color,

    // Input element colors
    pub input_bg: Color,
    pub input_bg_hover: Color,
    pub input_bg_focus: Color,
    pub input_bg_disabled: Color,

    // Selection colors
    pub selection: Color,
    pub selection_text: Color,

    // Accent
    pub accent: Color,
    pub accent_subtle: Color,

    // Tooltip colors (inverted colors)
    pub tooltip_bg: Color,
    pub tooltip_text: Color,
}

impl ColorTokens {
    /// Get a color by token key
    pub fn get(&self, token: ColorToken) -> Color {
        match token {
            ColorToken::Primary => self.primary,
            ColorToken::PrimaryHover => self.primary_hover,
            ColorToken::PrimaryActive => self.primary_active,
            ColorToken::Secondary => self.secondary,
            ColorToken::SecondaryHover => self.secondary_hover,
            ColorToken::SecondaryActive => self.secondary_active,
            ColorToken::Success => self.success,
            ColorToken::SuccessBg => self.success_bg,
            ColorToken::Warning => self.warning,
            ColorToken::WarningBg => self.warning_bg,
            ColorToken::Error => self.error,
            ColorToken::ErrorBg => self.error_bg,
            ColorToken::Info => self.info,
            ColorToken::InfoBg => self.info_bg,
            ColorToken::Background => self.background,
            ColorToken::Surface => self.surface,
            ColorToken::SurfaceElevated => self.surface_elevated,
            ColorToken::SurfaceOverlay => self.surface_overlay,
            ColorToken::TextPrimary => self.text_primary,
            ColorToken::TextSecondary => self.text_secondary,
            ColorToken::TextTertiary => self.text_tertiary,
            ColorToken::TextInverse => self.text_inverse,
            ColorToken::TextLink => self.text_link,
            ColorToken::Border => self.border,
            ColorToken::BorderSecondary => self.border_secondary,
            ColorToken::BorderHover => self.border_hover,
            ColorToken::BorderFocus => self.border_focus,
            ColorToken::BorderError => self.border_error,
            ColorToken::InputBg => self.input_bg,
            ColorToken::InputBgHover => self.input_bg_hover,
            ColorToken::InputBgFocus => self.input_bg_focus,
            ColorToken::InputBgDisabled => self.input_bg_disabled,
            ColorToken::Selection => self.selection,
            ColorToken::SelectionText => self.selection_text,
            ColorToken::Accent => self.accent,
            ColorToken::AccentSubtle => self.accent_subtle,
            ColorToken::TooltipBackground => self.tooltip_bg,
            ColorToken::TooltipText => self.tooltip_text,
        }
    }

    /// Linear interpolation between two color token sets
    pub fn lerp(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            primary: Color::lerp(&from.primary, &to.primary, t),
            primary_hover: Color::lerp(&from.primary_hover, &to.primary_hover, t),
            primary_active: Color::lerp(&from.primary_active, &to.primary_active, t),
            secondary: Color::lerp(&from.secondary, &to.secondary, t),
            secondary_hover: Color::lerp(&from.secondary_hover, &to.secondary_hover, t),
            secondary_active: Color::lerp(&from.secondary_active, &to.secondary_active, t),
            success: Color::lerp(&from.success, &to.success, t),
            success_bg: Color::lerp(&from.success_bg, &to.success_bg, t),
            warning: Color::lerp(&from.warning, &to.warning, t),
            warning_bg: Color::lerp(&from.warning_bg, &to.warning_bg, t),
            error: Color::lerp(&from.error, &to.error, t),
            error_bg: Color::lerp(&from.error_bg, &to.error_bg, t),
            info: Color::lerp(&from.info, &to.info, t),
            info_bg: Color::lerp(&from.info_bg, &to.info_bg, t),
            background: Color::lerp(&from.background, &to.background, t),
            surface: Color::lerp(&from.surface, &to.surface, t),
            surface_elevated: Color::lerp(&from.surface_elevated, &to.surface_elevated, t),
            surface_overlay: Color::lerp(&from.surface_overlay, &to.surface_overlay, t),
            text_primary: Color::lerp(&from.text_primary, &to.text_primary, t),
            text_secondary: Color::lerp(&from.text_secondary, &to.text_secondary, t),
            text_tertiary: Color::lerp(&from.text_tertiary, &to.text_tertiary, t),
            text_inverse: Color::lerp(&from.text_inverse, &to.text_inverse, t),
            text_link: Color::lerp(&from.text_link, &to.text_link, t),
            border: Color::lerp(&from.border, &to.border, t),
            border_secondary: Color::lerp(&from.border_secondary, &to.border_secondary, t),
            border_hover: Color::lerp(&from.border_hover, &to.border_hover, t),
            border_focus: Color::lerp(&from.border_focus, &to.border_focus, t),
            border_error: Color::lerp(&from.border_error, &to.border_error, t),
            input_bg: Color::lerp(&from.input_bg, &to.input_bg, t),
            input_bg_hover: Color::lerp(&from.input_bg_hover, &to.input_bg_hover, t),
            input_bg_focus: Color::lerp(&from.input_bg_focus, &to.input_bg_focus, t),
            input_bg_disabled: Color::lerp(&from.input_bg_disabled, &to.input_bg_disabled, t),
            selection: Color::lerp(&from.selection, &to.selection, t),
            selection_text: Color::lerp(&from.selection_text, &to.selection_text, t),
            accent: Color::lerp(&from.accent, &to.accent, t),
            accent_subtle: Color::lerp(&from.accent_subtle, &to.accent_subtle, t),
            tooltip_bg: Color::lerp(&from.tooltip_bg, &to.tooltip_bg, t),
            tooltip_text: Color::lerp(&from.tooltip_text, &to.tooltip_text, t),
        }
    }
}

impl Default for ColorTokens {
    fn default() -> Self {
        // Default to a basic light theme
        Self {
            primary: Color::from_hex(0x1E66F5),
            primary_hover: Color::from_hex(0x1758D1),
            primary_active: Color::from_hex(0x114AB3),
            secondary: Color::from_hex(0x8839EF),
            secondary_hover: Color::from_hex(0x7530D4),
            secondary_active: Color::from_hex(0x6228B9),
            success: Color::from_hex(0x40A02B),
            success_bg: Color::from_hex(0x40A02B).with_alpha(0.1),
            warning: Color::from_hex(0xDF8E1D),
            warning_bg: Color::from_hex(0xDF8E1D).with_alpha(0.1),
            error: Color::from_hex(0xD20F39),
            error_bg: Color::from_hex(0xD20F39).with_alpha(0.1),
            info: Color::from_hex(0x04A5E5),
            info_bg: Color::from_hex(0x04A5E5).with_alpha(0.1),
            background: Color::from_hex(0xEFF1F5),
            surface: Color::WHITE,
            surface_elevated: Color::WHITE,
            surface_overlay: Color::from_hex(0xE6E9EF),
            text_primary: Color::from_hex(0x4C4F69),
            text_secondary: Color::from_hex(0x6C6F85),
            text_tertiary: Color::from_hex(0x9CA0B0),
            text_inverse: Color::WHITE,
            text_link: Color::from_hex(0x1E66F5),
            border: Color::from_hex(0xCCD0DA),
            border_secondary: Color::from_hex(0xBCC0CC),
            border_hover: Color::from_hex(0xBCC0CC),
            border_focus: Color::from_hex(0x1E66F5),
            border_error: Color::from_hex(0xD20F39),
            input_bg: Color::WHITE,
            input_bg_hover: Color::from_hex(0xF9FAFB),
            input_bg_focus: Color::WHITE,
            input_bg_disabled: Color::from_hex(0xE6E9EF),
            selection: Color::from_hex(0x1E66F5).with_alpha(0.3),
            selection_text: Color::from_hex(0x4C4F69),
            accent: Color::from_hex(0x1E66F5),
            accent_subtle: Color::from_hex(0x1E66F5).with_alpha(0.1),
            tooltip_bg: Color::from_hex(0x1C1C1E), // Dark bg for light theme
            tooltip_text: Color::from_hex(0xF5F5F5), // Light text for dark bg
        }
    }
}
