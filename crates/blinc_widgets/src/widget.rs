//! Base widget trait and types

use slotmap::new_key_type;

new_key_type! {
    pub struct WidgetId;
}

/// Base trait for all widgets
pub trait Widget {
    /// Get the widget's unique ID
    fn id(&self) -> WidgetId;

    /// Render the widget (called when reactive state changes)
    fn render(&self);

    /// Handle an event
    fn handle_event(&mut self, event: &blinc_core::events::Event);
}
