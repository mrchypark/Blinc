//! Base widget trait and types

use slotmap::new_key_type;

new_key_type! {
    pub struct WidgetId;
}

/// Base trait for all widgets
pub trait Widget {
    /// Get the widget's unique ID
    fn id(&self) -> WidgetId;
}

/// Trait for widgets that can build their UI representation
///
/// Widgets that implement this trait can convert themselves into
/// a layout element (typically a Div) for rendering.
pub trait WidgetBuilder {
    /// The element type this widget builds into
    type Element;

    /// Build the widget's UI representation
    ///
    /// This is called during rendering to convert the widget's
    /// current state into layout elements.
    fn build(&self, ctx: &crate::context::WidgetContext) -> Self::Element;
}
