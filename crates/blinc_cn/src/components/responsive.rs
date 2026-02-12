//! Responsive helpers for Tailwind-style breakpoints in `blinc_cn`.

use blinc_core::context_state::BlincContextState;

/// Tailwind-compatible breakpoint widths in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TailwindBreakpoints {
    /// Small breakpoint (`sm`) - 640px
    pub sm: f32,
    /// Medium breakpoint (`md`) - 768px
    pub md: f32,
    /// Large breakpoint (`lg`) - 1024px
    pub lg: f32,
    /// Extra large breakpoint (`xl`) - 1280px
    pub xl: f32,
    /// 2x large breakpoint (`2xl`) - 1536px
    pub xxl: f32,
}

impl TailwindBreakpoints {
    /// Default Tailwind breakpoints (`sm`/`md`/`lg`/`xl`/`2xl`).
    pub const DEFAULT: Self = Self {
        sm: 640.0,
        md: 768.0,
        lg: 1024.0,
        xl: 1280.0,
        xxl: 1536.0,
    };
}

impl Default for TailwindBreakpoints {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Device-class abstraction derived from Tailwind breakpoints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceClass {
    /// Width < `md` (768px)
    Mobile,
    /// `md` <= width < `lg` (1024px)
    Tablet,
    /// width >= `lg` (1024px)
    Desktop,
}

/// Classify device width into mobile/tablet/desktop using Tailwind defaults.
pub fn device_class_for_width(width: f32) -> DeviceClass {
    let bp = TailwindBreakpoints::DEFAULT;
    if width < bp.md {
        DeviceClass::Mobile
    } else if width < bp.lg {
        DeviceClass::Tablet
    } else {
        DeviceClass::Desktop
    }
}

/// Return current device class from global viewport when available.
///
/// If viewport is not initialized yet, defaults to `Desktop`.
pub fn current_device_class() -> DeviceClass {
    let width = BlincContextState::try_get()
        .map(|ctx| ctx.viewport_size().0)
        .filter(|w| *w > 0.0)
        .unwrap_or(TailwindBreakpoints::DEFAULT.lg);
    device_class_for_width(width)
}
