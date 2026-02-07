//! Event simulator for replaying recorded events.
//!
//! Provides methods to simulate user input events by converting
//! recorded events back into UI-consumable form.

use crate::capture::{Key, Modifiers, MouseButton, Point, RecordedEvent, TimestampedEvent};

/// Simulates recorded events by converting them to input actions.
pub struct EventSimulator {
    /// Currently pressed mouse buttons.
    pressed_buttons: Vec<MouseButton>,
    /// Currently pressed modifier keys.
    modifiers: Modifiers,
    /// Current mouse position.
    mouse_position: Point,
}

impl EventSimulator {
    /// Create a new event simulator.
    pub fn new() -> Self {
        Self {
            pressed_buttons: Vec::new(),
            modifiers: Modifiers::none(),
            mouse_position: Point::new(0.0, 0.0),
        }
    }

    /// Reset the simulator state.
    pub fn reset(&mut self) {
        self.pressed_buttons.clear();
        self.modifiers = Modifiers::none();
        self.mouse_position = Point::new(0.0, 0.0);
    }

    /// Get the current mouse position.
    pub fn mouse_position(&self) -> Point {
        self.mouse_position
    }

    /// Get the current modifier state.
    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    /// Process a timestamped event and return the simulated input.
    ///
    /// Updates internal state and returns a SimulatedInput that can be
    /// dispatched to the event router.
    pub fn process(&mut self, event: &TimestampedEvent) -> SimulatedInput {
        self.process_event(&event.event)
    }

    /// Process a recorded event and return the simulated input.
    pub fn process_event(&mut self, event: &RecordedEvent) -> SimulatedInput {
        match event {
            RecordedEvent::Click(e) => {
                self.mouse_position = e.position;
                self.modifiers = e.modifiers;
                SimulatedInput::Click {
                    position: e.position,
                    button: e.button,
                    modifiers: e.modifiers,
                }
            }
            RecordedEvent::DoubleClick(e) => {
                self.mouse_position = e.position;
                self.modifiers = e.modifiers;
                SimulatedInput::DoubleClick {
                    position: e.position,
                    button: e.button,
                    modifiers: e.modifiers,
                }
            }
            RecordedEvent::MouseDown(e) => {
                self.mouse_position = e.position;
                self.modifiers = e.modifiers;
                if !self.pressed_buttons.contains(&e.button) {
                    self.pressed_buttons.push(e.button);
                }
                SimulatedInput::MouseDown {
                    position: e.position,
                    button: e.button,
                    modifiers: e.modifiers,
                }
            }
            RecordedEvent::MouseUp(e) => {
                self.mouse_position = e.position;
                self.modifiers = e.modifiers;
                self.pressed_buttons.retain(|&b| b != e.button);
                SimulatedInput::MouseUp {
                    position: e.position,
                    button: e.button,
                    modifiers: e.modifiers,
                }
            }
            RecordedEvent::MouseMove(e) => {
                self.mouse_position = e.position;
                SimulatedInput::MouseMove {
                    position: e.position,
                    hover_element: e.hover_element.clone(),
                }
            }
            RecordedEvent::Scroll(e) => {
                self.mouse_position = e.position;
                SimulatedInput::Scroll {
                    position: e.position,
                    delta_x: e.delta_x,
                    delta_y: e.delta_y,
                    target_element: e.target_element.clone(),
                }
            }
            RecordedEvent::KeyDown(e) => {
                self.update_modifiers_from_key(&e.key, true);
                SimulatedInput::KeyDown {
                    key: e.key.clone(),
                    modifiers: e.modifiers,
                    is_repeat: e.is_repeat,
                }
            }
            RecordedEvent::KeyUp(e) => {
                self.update_modifiers_from_key(&e.key, false);
                SimulatedInput::KeyUp {
                    key: e.key.clone(),
                    modifiers: e.modifiers,
                }
            }
            RecordedEvent::TextInput(e) => SimulatedInput::TextInput {
                text: e.text.clone(),
            },
            RecordedEvent::FocusChange(e) => SimulatedInput::FocusChange {
                from: e.from.clone(),
                to: e.to.clone(),
            },
            RecordedEvent::HoverEnter(e) => {
                self.mouse_position = e.position;
                SimulatedInput::HoverEnter {
                    element_id: e.element_id.clone(),
                    position: e.position,
                }
            }
            RecordedEvent::HoverLeave(e) => {
                self.mouse_position = e.position;
                SimulatedInput::HoverLeave {
                    element_id: e.element_id.clone(),
                    position: e.position,
                }
            }
            RecordedEvent::WindowResize(e) => SimulatedInput::WindowResize {
                width: e.width,
                height: e.height,
                scale_factor: e.scale_factor,
            },
            RecordedEvent::WindowFocus(focused) => {
                SimulatedInput::WindowFocus { focused: *focused }
            }
            RecordedEvent::Custom(e) => SimulatedInput::Custom {
                name: e.name.clone(),
                payload: e.payload.clone(),
            },
        }
    }

    /// Update modifier state based on key press/release.
    fn update_modifiers_from_key(&mut self, key: &Key, pressed: bool) {
        match key {
            Key::Shift => {
                self.modifiers.shift = pressed;
            }
            Key::Control => {
                self.modifiers.ctrl = pressed;
            }
            Key::Alt => {
                self.modifiers.alt = pressed;
            }
            Key::Meta => {
                self.modifiers.meta = pressed;
            }
            _ => {}
        }
    }
}

impl Default for EventSimulator {
    fn default() -> Self {
        Self::new()
    }
}

/// A simulated input event that can be dispatched to the UI.
#[derive(Clone, Debug)]
pub enum SimulatedInput {
    /// Mouse click (down + up).
    Click {
        position: Point,
        button: MouseButton,
        modifiers: Modifiers,
    },
    /// Double click.
    DoubleClick {
        position: Point,
        button: MouseButton,
        modifiers: Modifiers,
    },
    /// Mouse button pressed.
    MouseDown {
        position: Point,
        button: MouseButton,
        modifiers: Modifiers,
    },
    /// Mouse button released.
    MouseUp {
        position: Point,
        button: MouseButton,
        modifiers: Modifiers,
    },
    /// Mouse moved.
    MouseMove {
        position: Point,
        hover_element: Option<String>,
    },
    /// Scroll wheel.
    Scroll {
        position: Point,
        delta_x: f32,
        delta_y: f32,
        target_element: Option<String>,
    },
    /// Key pressed.
    KeyDown {
        key: Key,
        modifiers: Modifiers,
        is_repeat: bool,
    },
    /// Key released.
    KeyUp { key: Key, modifiers: Modifiers },
    /// Text input.
    TextInput { text: String },
    /// Focus changed.
    FocusChange {
        from: Option<String>,
        to: Option<String>,
    },
    /// Mouse entered element.
    HoverEnter { element_id: String, position: Point },
    /// Mouse left element.
    HoverLeave { element_id: String, position: Point },
    /// Window resized.
    WindowResize {
        width: u32,
        height: u32,
        scale_factor: Option<f64>,
    },
    /// Window focus changed.
    WindowFocus { focused: bool },
    /// Custom event.
    Custom {
        name: String,
        payload: Option<String>,
    },
}

impl SimulatedInput {
    /// Check if this is a mouse event.
    pub fn is_mouse_event(&self) -> bool {
        matches!(
            self,
            Self::Click { .. }
                | Self::DoubleClick { .. }
                | Self::MouseDown { .. }
                | Self::MouseUp { .. }
                | Self::MouseMove { .. }
                | Self::Scroll { .. }
        )
    }

    /// Check if this is a keyboard event.
    pub fn is_keyboard_event(&self) -> bool {
        matches!(
            self,
            Self::KeyDown { .. } | Self::KeyUp { .. } | Self::TextInput { .. }
        )
    }

    /// Get the position if this is a mouse event.
    pub fn position(&self) -> Option<Point> {
        match self {
            Self::Click { position, .. }
            | Self::DoubleClick { position, .. }
            | Self::MouseDown { position, .. }
            | Self::MouseUp { position, .. }
            | Self::MouseMove { position, .. }
            | Self::Scroll { position, .. }
            | Self::HoverEnter { position, .. }
            | Self::HoverLeave { position, .. } => Some(*position),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::{KeyEvent, MouseMoveEvent};

    #[test]
    fn test_simulator_creation() {
        let sim = EventSimulator::new();
        assert_eq!(sim.mouse_position().x, 0.0);
        assert_eq!(sim.mouse_position().y, 0.0);
        assert!(!sim.modifiers().shift);
    }

    #[test]
    fn test_mouse_move_tracking() {
        let mut sim = EventSimulator::new();

        let event = RecordedEvent::MouseMove(MouseMoveEvent {
            position: Point::new(100.0, 200.0),
            hover_element: None,
        });

        sim.process_event(&event);
        assert_eq!(sim.mouse_position().x, 100.0);
        assert_eq!(sim.mouse_position().y, 200.0);
    }

    #[test]
    fn test_modifier_tracking() {
        let mut sim = EventSimulator::new();

        // Press shift
        let shift_down = RecordedEvent::KeyDown(KeyEvent {
            key: Key::Shift,
            modifiers: Modifiers::none(),
            is_repeat: false,
            focused_element: None,
        });
        sim.process_event(&shift_down);
        assert!(sim.modifiers().shift);

        // Release shift
        let shift_up = RecordedEvent::KeyUp(KeyEvent {
            key: Key::Shift,
            modifiers: Modifiers {
                shift: true,
                ctrl: false,
                alt: false,
                meta: false,
            },
            is_repeat: false,
            focused_element: None,
        });
        sim.process_event(&shift_up);
        assert!(!sim.modifiers().shift);
    }

    #[test]
    fn test_simulated_input_types() {
        let click = SimulatedInput::Click {
            position: Point::new(0.0, 0.0),
            button: MouseButton::Left,
            modifiers: Modifiers::none(),
        };
        assert!(click.is_mouse_event());
        assert!(!click.is_keyboard_event());

        let key = SimulatedInput::KeyDown {
            key: Key::A,
            modifiers: Modifiers::none(),
            is_repeat: false,
        };
        assert!(!key.is_mouse_event());
        assert!(key.is_keyboard_event());
    }
}
