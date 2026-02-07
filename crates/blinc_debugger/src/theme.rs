//! Debugger theme using blinc_theme system
//!
//! Uses the global ThemeState for colors and spacing,
//! with debugger-specific extensions for event colors and panel dimensions.

use blinc_core::Color;
use blinc_layout::div::FontWeight;
use blinc_theme::{ColorToken, RadiusToken, SpacingToken, ThemeState};

/// Debugger-specific colors (extensions not in base theme)
pub struct DebuggerColors;

#[allow(dead_code)]
impl DebuggerColors {
    // Access theme colors via ThemeState
    pub fn bg_base() -> Color {
        ThemeState::get().color(ColorToken::Background)
    }

    pub fn bg_elevated() -> Color {
        ThemeState::get().color(ColorToken::SurfaceElevated)
    }

    pub fn bg_surface() -> Color {
        ThemeState::get().color(ColorToken::Surface)
    }

    pub fn bg_hover() -> Color {
        ThemeState::get().color(ColorToken::InputBgHover)
    }

    pub fn border_default() -> Color {
        ThemeState::get().color(ColorToken::Border)
    }

    pub fn border_subtle() -> Color {
        ThemeState::get().color(ColorToken::Border).with_alpha(0.5)
    }

    pub fn text_primary() -> Color {
        ThemeState::get().color(ColorToken::TextPrimary)
    }

    pub fn text_secondary() -> Color {
        ThemeState::get().color(ColorToken::TextSecondary)
    }

    pub fn text_muted() -> Color {
        ThemeState::get().color(ColorToken::TextTertiary)
    }

    pub fn primary() -> Color {
        ThemeState::get().color(ColorToken::Primary)
    }

    pub fn primary_hover() -> Color {
        ThemeState::get().color(ColorToken::PrimaryHover)
    }

    pub fn secondary() -> Color {
        ThemeState::get().color(ColorToken::Secondary)
    }

    pub fn success() -> Color {
        ThemeState::get().color(ColorToken::Success)
    }

    pub fn warning() -> Color {
        ThemeState::get().color(ColorToken::Warning)
    }

    pub fn error() -> Color {
        ThemeState::get().color(ColorToken::Error)
    }

    pub fn info() -> Color {
        ThemeState::get().color(ColorToken::Info)
    }

    // Diff colors (for tree diff visualization) - use semantic colors
    pub fn diff_added() -> Color {
        ThemeState::get().color(ColorToken::Success)
    }

    pub fn diff_removed() -> Color {
        ThemeState::get().color(ColorToken::Error)
    }

    pub fn diff_modified() -> Color {
        ThemeState::get().color(ColorToken::Warning)
    }

    pub fn diff_unchanged() -> Color {
        ThemeState::get().color(ColorToken::TextTertiary)
    }

    // Event type colors (debugger-specific) - derived from theme
    pub fn event_mouse() -> Color {
        ThemeState::get().color(ColorToken::Primary)
    }

    pub fn event_keyboard() -> Color {
        ThemeState::get().color(ColorToken::Info)
    }

    pub fn event_scroll() -> Color {
        ThemeState::get().color(ColorToken::Secondary)
    }

    pub fn event_focus() -> Color {
        ThemeState::get().color(ColorToken::Accent)
    }

    pub fn event_hover() -> Color {
        ThemeState::get().color(ColorToken::Warning)
    }
}

/// Design tokens for the debugger UI
pub struct DebuggerTokens;

#[allow(dead_code)]
impl DebuggerTokens {
    // Border radius - use theme tokens
    pub fn radius_sm() -> f32 {
        ThemeState::get().radius(RadiusToken::Sm)
    }

    pub fn radius_md() -> f32 {
        ThemeState::get().radius(RadiusToken::Md)
    }

    pub fn radius_lg() -> f32 {
        ThemeState::get().radius(RadiusToken::Lg)
    }

    pub fn radius_full() -> f32 {
        ThemeState::get().radius(RadiusToken::Full)
    }

    // Spacing - use theme tokens
    pub fn space_1() -> f32 {
        ThemeState::get().spacing_value(SpacingToken::Space1)
    }

    pub fn space_2() -> f32 {
        ThemeState::get().spacing_value(SpacingToken::Space2)
    }

    pub fn space_3() -> f32 {
        ThemeState::get().spacing_value(SpacingToken::Space3)
    }

    pub fn space_4() -> f32 {
        ThemeState::get().spacing_value(SpacingToken::Space4)
    }

    pub fn space_5() -> f32 {
        ThemeState::get().spacing_value(SpacingToken::Space5)
    }

    pub fn space_6() -> f32 {
        ThemeState::get().spacing_value(SpacingToken::Space6)
    }

    pub fn space_8() -> f32 {
        ThemeState::get().spacing_value(SpacingToken::Space8)
    }

    pub fn space_10() -> f32 {
        ThemeState::get().spacing_value(SpacingToken::Space10)
    }

    // Typography - use theme tokens
    pub fn font_size_xs() -> f32 {
        ThemeState::get().typography().text_xs
    }

    pub fn font_size_sm() -> f32 {
        ThemeState::get().typography().text_sm
    }

    pub fn font_size_base() -> f32 {
        ThemeState::get().typography().text_base
    }

    pub fn font_size_lg() -> f32 {
        ThemeState::get().typography().text_lg
    }

    pub fn font_size_xl() -> f32 {
        ThemeState::get().typography().text_xl
    }

    // Font weights - these stay as constants since they're enums
    pub const FONT_WEIGHT_NORMAL: FontWeight = FontWeight::Normal;
    pub const FONT_WEIGHT_MEDIUM: FontWeight = FontWeight::Medium;
    pub const FONT_WEIGHT_SEMIBOLD: FontWeight = FontWeight::SemiBold;
    pub const FONT_WEIGHT_BOLD: FontWeight = FontWeight::Bold;

    // Panel dimensions (debugger-specific layout constants)
    pub const TREE_PANEL_WIDTH: f32 = 280.0;
    pub const INSPECTOR_WIDTH: f32 = 300.0;
    pub const TIMELINE_HEIGHT: f32 = 150.0;
    pub const HEADER_HEIGHT: f32 = 48.0;
    pub const PANEL_GAP: f32 = 8.0;
    pub const CARD_PADDING: f32 = 16.0;
    pub const CARD_GAP: f32 = 12.0;
}
