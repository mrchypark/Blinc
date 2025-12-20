//! Event dispatch system
//!
//! Unified event handling across all platforms.

use rustc_hash::FxHashMap;

/// Event type identifier
pub type EventType = u32;

/// Common event types
pub mod event_types {
    use super::EventType;

    pub const POINTER_DOWN: EventType = 1;
    pub const POINTER_UP: EventType = 2;
    pub const POINTER_MOVE: EventType = 3;
    pub const POINTER_ENTER: EventType = 4;
    pub const POINTER_LEAVE: EventType = 5;
    pub const FOCUS: EventType = 10;
    pub const BLUR: EventType = 11;
    pub const KEY_DOWN: EventType = 20;
    pub const KEY_UP: EventType = 21;
    pub const SCROLL: EventType = 30;
    pub const RESIZE: EventType = 40;
}

/// A UI event with associated data
#[derive(Clone, Debug)]
pub struct Event {
    pub event_type: EventType,
    pub target: u64, // Widget ID
    pub data: EventData,
    pub timestamp: u64,
    pub propagation_stopped: bool,
}

/// Event-specific data
#[derive(Clone, Debug)]
pub enum EventData {
    Pointer {
        x: f32,
        y: f32,
        button: u8,
        pressure: f32,
    },
    Key {
        code: u32,
        modifiers: u32,
    },
    Scroll {
        delta_x: f32,
        delta_y: f32,
    },
    Resize {
        width: u32,
        height: u32,
    },
    None,
}

impl Event {
    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }
}

/// Event handler function type
pub type EventHandler = Box<dyn Fn(&Event) + Send + Sync>;

/// Dispatches events to registered handlers
pub struct EventDispatcher {
    handlers: FxHashMap<(u64, EventType), Vec<EventHandler>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            handlers: FxHashMap::default(),
        }
    }

    /// Register an event handler for a widget and event type
    pub fn register<F>(&mut self, widget_id: u64, event_type: EventType, handler: F)
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        self.handlers
            .entry((widget_id, event_type))
            .or_default()
            .push(Box::new(handler));
    }

    /// Dispatch an event to all registered handlers
    pub fn dispatch(&self, event: &mut Event) {
        if let Some(handlers) = self.handlers.get(&(event.target, event.event_type)) {
            for handler in handlers {
                if event.propagation_stopped {
                    break;
                }
                handler(event);
            }
        }
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
