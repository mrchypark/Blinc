//! Fuchsia input handling
//!
//! Converts fuchsia.ui.pointer events to Blinc input events.

use blinc_platform::{InputEvent, TouchEvent};

/// Touch phase from fuchsia.ui.pointer
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchPhase {
    /// Touch began (ADD in Fuchsia)
    Began,
    /// Touch moved (CHANGE in Fuchsia)
    Moved,
    /// Touch ended (REMOVE in Fuchsia)
    Ended,
    /// Touch cancelled (CANCEL in Fuchsia)
    Cancelled,
}

/// A single touch point
#[derive(Clone, Debug)]
pub struct Touch {
    /// Unique identifier for this touch (pointer_id in Fuchsia)
    pub id: u64,
    /// X position in logical pixels
    pub x: f32,
    /// Y position in logical pixels
    pub y: f32,
    /// Touch phase
    pub phase: TouchPhase,
}

impl Touch {
    /// Create a new touch event
    pub fn new(id: u64, x: f32, y: f32, phase: TouchPhase) -> Self {
        Self { id, x, y, phase }
    }
}

/// Convert a Fuchsia touch to a Blinc input event
pub fn convert_touch(touch: &Touch) -> InputEvent {
    match touch.phase {
        TouchPhase::Began => InputEvent::Touch(TouchEvent::Started {
            id: touch.id,
            x: touch.x,
            y: touch.y,
            pressure: 1.0,
        }),
        TouchPhase::Moved => InputEvent::Touch(TouchEvent::Moved {
            id: touch.id,
            x: touch.x,
            y: touch.y,
            pressure: 1.0,
        }),
        TouchPhase::Ended => InputEvent::Touch(TouchEvent::Ended {
            id: touch.id,
            x: touch.x,
            y: touch.y,
        }),
        TouchPhase::Cancelled => InputEvent::Touch(TouchEvent::Cancelled { id: touch.id }),
    }
}

/// Convert fuchsia.ui.pointer.TouchEvent to Blinc Touch
///
/// Note: This function would be implemented when building with Fuchsia SDK
#[cfg(target_os = "fuchsia")]
pub fn from_fuchsia_touch_event(
    _event: (), // fidl_fuchsia_ui_pointer::TouchEvent
    _scale_factor: f64,
) -> Vec<Touch> {
    // TODO: Implement conversion from fidl_fuchsia_ui_pointer::TouchEvent
    // - Extract pointer_id, position, phase
    // - Convert coordinates using scale_factor
    vec![]
}
