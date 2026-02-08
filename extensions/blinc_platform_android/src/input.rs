//! Android touch input handling
//!
//! Converts Android touch events to Blinc events.

/// Touch pointer state
#[derive(Debug, Clone, Copy)]
pub struct TouchPointer {
    /// Pointer ID (for multi-touch)
    pub id: i32,
    /// X coordinate in window space
    pub x: f32,
    /// Y coordinate in window space
    pub y: f32,
    /// Pressure (0.0 - 1.0)
    pub pressure: f32,
    /// Touch size
    pub size: f32,
}

/// Touch event types
#[derive(Debug, Clone)]
pub enum TouchEvent {
    /// A new touch started
    Down {
        pointer: TouchPointer,
        pointers: Vec<TouchPointer>,
    },
    /// Touch position changed
    Move { pointers: Vec<TouchPointer> },
    /// Touch ended
    Up {
        pointer: TouchPointer,
        pointers: Vec<TouchPointer>,
    },
    /// Touch cancelled (e.g., system gesture)
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinchPhase {
    Started,
    Moved,
    Ended,
}

#[derive(Debug, Clone, Copy)]
pub struct PinchGesture {
    pub scale: f32,
    pub center: (f32, f32),
    pub phase: PinchPhase,
}

#[derive(Debug, Default)]
pub struct PinchState {
    prev_span: Option<f32>,
    last_center: Option<(f32, f32)>,
    active: bool,
}

impl PinchState {
    pub fn reset(&mut self) {
        self.prev_span = None;
        self.last_center = None;
        self.active = false;
    }
}

pub fn detect_pinch(pointers: &[TouchPointer], state: &mut PinchState) -> Option<PinchGesture> {
    if pointers.len() < 2 {
        if state.active {
            let center = state.last_center.unwrap_or((0.0, 0.0));
            state.reset();
            return Some(PinchGesture {
                scale: 1.0,
                center,
                phase: PinchPhase::Ended,
            });
        }
        state.reset();
        return None;
    }

    let p0 = pointers[0];
    let p1 = pointers[1];
    let center = ((p0.x + p1.x) * 0.5, (p0.y + p1.y) * 0.5);
    let dx = p1.x - p0.x;
    let dy = p1.y - p0.y;
    let span = (dx * dx + dy * dy).sqrt();

    let gesture = match state.prev_span {
        None => PinchGesture {
            scale: 1.0,
            center,
            phase: PinchPhase::Started,
        },
        Some(prev_span) if prev_span > 0.0 => {
            let mut scale = span / prev_span;
            scale = scale.clamp(0.90, 1.10);
            PinchGesture {
                scale,
                center,
                phase: PinchPhase::Moved,
            }
        }
        Some(_) => PinchGesture {
            scale: 1.0,
            center,
            phase: PinchPhase::Moved,
        },
    };

    state.prev_span = Some(span);
    state.last_center = Some(center);
    state.active = true;
    Some(gesture)
}

/// Converts Android MotionEvent to Blinc TouchEvent
#[cfg(target_os = "android")]
pub fn convert_motion_event(event: &ndk::event::MotionEvent) -> Option<TouchEvent> {
    use ndk::event::MotionAction;

    let action = event.action();
    let pointer_count = event.pointer_count();

    // Collect all current pointers
    let pointers: Vec<TouchPointer> = (0..pointer_count)
        .map(|i| {
            let p = event.pointer_at_index(i);
            TouchPointer {
                id: p.pointer_id(),
                x: p.x(),
                y: p.y(),
                pressure: p.pressure(),
                size: p.size(),
            }
        })
        .collect();

    // Get the action pointer index (for UP/DOWN events)
    let action_pointer_index = event.pointer_index();

    match action {
        MotionAction::Down | MotionAction::PointerDown => {
            let pointer = pointers.get(action_pointer_index).copied()?;
            Some(TouchEvent::Down { pointer, pointers })
        }
        MotionAction::Move => Some(TouchEvent::Move { pointers }),
        MotionAction::Up | MotionAction::PointerUp => {
            let pointer = pointers.get(action_pointer_index).copied()?;
            Some(TouchEvent::Up { pointer, pointers })
        }
        MotionAction::Cancel => Some(TouchEvent::Cancel),
        _ => None,
    }
}

/// Maps touch events to FSM events for widgets
pub mod fsm_events {
    /// FSM event IDs for touch interactions
    pub const POINTER_DOWN: u32 = 1;
    pub const POINTER_UP: u32 = 2;
    pub const POINTER_MOVE: u32 = 3;
    pub const POINTER_CANCEL: u32 = 4;
    pub const LONG_PRESS: u32 = 5;
    pub const SWIPE_LEFT: u32 = 6;
    pub const SWIPE_RIGHT: u32 = 7;
    pub const SWIPE_UP: u32 = 8;
    pub const SWIPE_DOWN: u32 = 9;
    pub const PINCH_START: u32 = 10;
    pub const PINCH_END: u32 = 11;
}

/// Gesture detector for recognizing common touch gestures
pub struct GestureDetector {
    /// Initial touch position for gesture detection
    start_position: Option<(f32, f32)>,
    /// Time of initial touch
    start_time: Option<std::time::Instant>,
    /// Threshold for swipe detection (in pixels)
    swipe_threshold: f32,
    /// Duration for long press detection
    long_press_duration: std::time::Duration,
}

impl GestureDetector {
    pub fn new() -> Self {
        Self {
            start_position: None,
            start_time: None,
            swipe_threshold: 50.0,
            long_press_duration: std::time::Duration::from_millis(500),
        }
    }

    /// Process a touch event and return detected gesture FSM event ID
    pub fn process(&mut self, event: &TouchEvent) -> Option<u32> {
        match event {
            TouchEvent::Down { pointer, .. } => {
                self.start_position = Some((pointer.x, pointer.y));
                self.start_time = Some(std::time::Instant::now());
                Some(fsm_events::POINTER_DOWN)
            }
            TouchEvent::Move { pointers } => {
                if let (Some((start_x, start_y)), Some(pointer)) =
                    (self.start_position, pointers.first())
                {
                    let dx = pointer.x - start_x;
                    let dy = pointer.y - start_y;

                    // Check for swipe
                    if dx.abs() > self.swipe_threshold || dy.abs() > self.swipe_threshold {
                        if dx.abs() > dy.abs() {
                            // Horizontal swipe
                            self.start_position = None;
                            return Some(if dx > 0.0 {
                                fsm_events::SWIPE_RIGHT
                            } else {
                                fsm_events::SWIPE_LEFT
                            });
                        } else {
                            // Vertical swipe
                            self.start_position = None;
                            return Some(if dy > 0.0 {
                                fsm_events::SWIPE_DOWN
                            } else {
                                fsm_events::SWIPE_UP
                            });
                        }
                    }
                }
                Some(fsm_events::POINTER_MOVE)
            }
            TouchEvent::Up { .. } => {
                // Check for long press
                if let Some(start) = self.start_time {
                    if start.elapsed() >= self.long_press_duration {
                        self.start_position = None;
                        self.start_time = None;
                        return Some(fsm_events::LONG_PRESS);
                    }
                }
                self.start_position = None;
                self.start_time = None;
                Some(fsm_events::POINTER_UP)
            }
            TouchEvent::Cancel => {
                self.start_position = None;
                self.start_time = None;
                Some(fsm_events::POINTER_CANCEL)
            }
        }
    }
}

impl Default for GestureDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gesture_detector_tap() {
        let mut detector = GestureDetector::new();

        let down = TouchEvent::Down {
            pointer: TouchPointer {
                id: 0,
                x: 100.0,
                y: 100.0,
                pressure: 1.0,
                size: 0.0,
            },
            pointers: vec![],
        };

        let up = TouchEvent::Up {
            pointer: TouchPointer {
                id: 0,
                x: 100.0,
                y: 100.0,
                pressure: 1.0,
                size: 0.0,
            },
            pointers: vec![],
        };

        assert_eq!(detector.process(&down), Some(fsm_events::POINTER_DOWN));
        assert_eq!(detector.process(&up), Some(fsm_events::POINTER_UP));
    }
}
