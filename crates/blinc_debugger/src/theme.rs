//! Debugger theme using blinc_theme system
//!
//! Uses the global ThemeState for colors and spacing,
//! with debugger-specific extensions for event colors and panel dimensions.

use blinc_core::Color;
use blinc_theme::{ColorToken, ThemeState};

/// Debugger-specific colors (extensions not in base theme)
pub struct DebuggerColors;

impl DebuggerColors {
    pub fn bg_base() -> Color {
        ThemeState::get().color(ColorToken::Background)
    }
}

/// Design tokens for the debugger UI
pub struct DebuggerTokens;

impl DebuggerTokens {
    // Panel dimensions used by debugger layout
    pub const TREE_PANEL_WIDTH: f32 = 280.0;
    pub const INSPECTOR_WIDTH: f32 = 300.0;
    pub const TIMELINE_HEIGHT: f32 = 150.0;
}
