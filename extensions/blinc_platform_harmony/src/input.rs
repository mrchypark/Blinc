//! HarmonyOS input handling
//!
//! Converts XComponent touch events to Blinc input events.

use blinc_platform::{InputEvent, TouchEvent};

/// Touch phase from OH_NativeXComponent_TouchEvent
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchPhase {
    /// Touch began (OH_NATIVEXCOMPONENT_DOWN)
    Began,
    /// Touch moved (OH_NATIVEXCOMPONENT_MOVE)
    Moved,
    /// Touch ended (OH_NATIVEXCOMPONENT_UP)
    Ended,
    /// Touch cancelled (OH_NATIVEXCOMPONENT_CANCEL)
    Cancelled,
}

/// A single touch point
#[derive(Clone, Debug)]
pub struct Touch {
    /// Unique identifier for this touch
    pub id: u64,
    /// X position in logical pixels
    pub x: f32,
    /// Y position in logical pixels
    pub y: f32,
    /// Touch phase
    pub phase: TouchPhase,
    /// Touch pressure (0.0 - 1.0)
    pub pressure: f32,
}

impl Touch {
    /// Create a new touch event
    pub fn new(id: u64, x: f32, y: f32, phase: TouchPhase) -> Self {
        Self {
            id,
            x,
            y,
            phase,
            pressure: 1.0,
        }
    }

    /// Create a touch with pressure
    pub fn with_pressure(id: u64, x: f32, y: f32, phase: TouchPhase, pressure: f32) -> Self {
        Self {
            id,
            x,
            y,
            phase,
            pressure,
        }
    }
}

/// Convert a HarmonyOS touch to a Blinc input event
pub fn convert_touch(touch: &Touch) -> InputEvent {
    match touch.phase {
        TouchPhase::Began => InputEvent::Touch(TouchEvent::Started {
            id: touch.id,
            x: touch.x,
            y: touch.y,
            pressure: touch.pressure,
        }),
        TouchPhase::Moved => InputEvent::Touch(TouchEvent::Moved {
            id: touch.id,
            x: touch.x,
            y: touch.y,
            pressure: touch.pressure,
        }),
        TouchPhase::Ended => InputEvent::Touch(TouchEvent::Ended {
            id: touch.id,
            x: touch.x,
            y: touch.y,
        }),
        TouchPhase::Cancelled => InputEvent::Touch(TouchEvent::Cancelled { id: touch.id }),
    }
}

/// Convert OH_NativeXComponent_TouchEvent to Blinc Touch
///
/// Note: This would be implemented with actual XComponent types
pub fn from_xcomponent_touch(
    action: i32,
    id: u64,
    x: f32,
    y: f32,
    scale_factor: f64,
) -> Option<Touch> {
    // OH_NativeXComponent_TouchEventType values
    const DOWN: i32 = 0;
    const UP: i32 = 1;
    const MOVE: i32 = 2;
    const CANCEL: i32 = 3;

    let phase = match action {
        DOWN => TouchPhase::Began,
        UP => TouchPhase::Ended,
        MOVE => TouchPhase::Moved,
        CANCEL => TouchPhase::Cancelled,
        _ => return None,
    };

    // Convert physical to logical coordinates
    let logical_x = x / scale_factor as f32;
    let logical_y = y / scale_factor as f32;

    Some(Touch::new(id, logical_x, logical_y, phase))
}
