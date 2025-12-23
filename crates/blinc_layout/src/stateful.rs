//! Stateful elements with user-defined state types
//!
//! Provides `Stateful<S>` - a generic stateful element where users define
//! their own state enum/type and use pattern matching in callbacks:
//!
//! ```ignore
//! use blinc_layout::prelude::*;
//! use blinc_core::Color;
//!
//! // Define your own state type
//! #[derive(Clone, Copy, PartialEq, Eq, Hash)]
//! enum ButtonState {
//!     Idle,
//!     Hovered,
//!     Pressed,
//!     Disabled,
//! }
//!
//! // Map events to state transitions
//! impl StateTransitions for ButtonState {
//!     fn on_event(&self, event: u32) -> Option<Self> {
//!         use blinc_core::events::event_types::*;
//!         match (self, event) {
//!             (ButtonState::Idle, POINTER_ENTER) => Some(ButtonState::Hovered),
//!             (ButtonState::Hovered, POINTER_LEAVE) => Some(ButtonState::Idle),
//!             (ButtonState::Hovered, POINTER_DOWN) => Some(ButtonState::Pressed),
//!             (ButtonState::Pressed, POINTER_UP) => Some(ButtonState::Hovered),
//!             (ButtonState::Pressed, POINTER_LEAVE) => Some(ButtonState::Idle),
//!             _ => None,
//!         }
//!     }
//! }
//!
//! let button = Stateful::new(ButtonState::Idle)
//!     .w(100.0)
//!     .h(40.0)
//!     .on_state(|state, div| {
//!         match state {
//!             ButtonState::Idle => {
//!                 *div = div.swap().bg(Color::BLUE).rounded(4.0);
//!             }
//!             ButtonState::Hovered => {
//!                 *div = div.swap().bg(Color::CYAN).rounded(8.0);
//!             }
//!             ButtonState::Pressed => {
//!                 *div = div.swap().bg(Color::BLUE).scale(0.97);
//!             }
//!             ButtonState::Disabled => {
//!                 *div = div.swap().bg(Color::GRAY).opacity(0.5);
//!             }
//!         }
//!     })
//!     .child(text("Click me"));
//! ```
//!
//! State callbacks receive the current state for pattern matching and a
//! mutable reference to the inner `Div` for full mutation capability.

use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

use blinc_core::fsm::StateMachine;

use crate::div::{Div, ElementBuilder, ElementRef, ElementTypeId};
use crate::element::RenderProps;
use crate::interactive::InteractiveContext;
use crate::tree::{LayoutNodeId, LayoutTree};

// =========================================================================
// State Traits
// =========================================================================

/// Trait for user-defined state types that can handle event transitions
///
/// Implement this trait on your state enum to define how events cause
/// state transitions.
///
/// # Example
///
/// ```ignore
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// enum MyButtonState {
///     #[default]
///     Idle,
///     Hovered,
///     Pressed,
/// }
///
/// impl StateTransitions for MyButtonState {
///     fn on_event(&self, event: u32) -> Option<Self> {
///         use blinc_core::events::event_types::*;
///         match (self, event) {
///             (MyButtonState::Idle, POINTER_ENTER) => Some(MyButtonState::Hovered),
///             (MyButtonState::Hovered, POINTER_LEAVE) => Some(MyButtonState::Idle),
///             (MyButtonState::Hovered, POINTER_DOWN) => Some(MyButtonState::Pressed),
///             (MyButtonState::Pressed, POINTER_UP) => Some(MyButtonState::Hovered),
///             _ => None,
///         }
///     }
/// }
/// ```
pub trait StateTransitions:
    Clone + Copy + PartialEq + Eq + Hash + Send + Sync + std::fmt::Debug + 'static
{
    /// Handle an event and return the new state, or None if no transition
    fn on_event(&self, event: u32) -> Option<Self>;
}

/// Trait for converting user state to/from internal u32 representation
///
/// This is auto-implemented for types that implement `Into<u32>` and `TryFrom<u32>`.
pub trait StateId: Clone + Copy + PartialEq + Eq + Hash + Send + Sync + 'static {
    /// Convert to internal u32 state ID
    fn to_id(&self) -> u32;

    /// Convert from internal u32 state ID
    fn from_id(id: u32) -> Option<Self>;
}

// =========================================================================
// State Callback Types
// =========================================================================

/// Callback type for state changes with user state type
pub type StateCallback<S> = Box<dyn Fn(&S, &mut Div) + Send + Sync>;

// =========================================================================
// Built-in State Types
// =========================================================================

/// Common button interaction states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ButtonState {
    #[default]
    Idle,
    Hovered,
    Pressed,
    Disabled,
}

impl StateTransitions for ButtonState {
    fn on_event(&self, event: u32) -> Option<Self> {
        use blinc_core::events::event_types::*;
        match (self, event) {
            (ButtonState::Idle, POINTER_ENTER) => Some(ButtonState::Hovered),
            (ButtonState::Hovered, POINTER_LEAVE) => Some(ButtonState::Idle),
            (ButtonState::Hovered, POINTER_DOWN) => Some(ButtonState::Pressed),
            (ButtonState::Pressed, POINTER_UP) => Some(ButtonState::Hovered),
            (ButtonState::Pressed, POINTER_LEAVE) => Some(ButtonState::Idle),
            (ButtonState::Disabled, _) => None, // Disabled ignores all events
            _ => None,
        }
    }
}

/// Toggle states (on/off)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToggleState {
    #[default]
    Off,
    On,
}

impl StateTransitions for ToggleState {
    fn on_event(&self, event: u32) -> Option<Self> {
        use blinc_core::events::event_types::*;
        match (self, event) {
            (ToggleState::Off, POINTER_UP) => Some(ToggleState::On),
            (ToggleState::On, POINTER_UP) => Some(ToggleState::Off),
            _ => None,
        }
    }
}

/// Checkbox states combining checked status and hover
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CheckboxState {
    #[default]
    UncheckedIdle,
    UncheckedHovered,
    CheckedIdle,
    CheckedHovered,
}

impl CheckboxState {
    /// Returns true if the checkbox is checked
    pub fn is_checked(&self) -> bool {
        matches!(self, CheckboxState::CheckedIdle | CheckboxState::CheckedHovered)
    }

    /// Returns true if the checkbox is hovered
    pub fn is_hovered(&self) -> bool {
        matches!(
            self,
            CheckboxState::UncheckedHovered | CheckboxState::CheckedHovered
        )
    }
}

impl StateTransitions for CheckboxState {
    fn on_event(&self, event: u32) -> Option<Self> {
        use blinc_core::events::event_types::*;
        match (self, event) {
            // Unchecked transitions
            (CheckboxState::UncheckedIdle, POINTER_ENTER) => Some(CheckboxState::UncheckedHovered),
            (CheckboxState::UncheckedHovered, POINTER_LEAVE) => Some(CheckboxState::UncheckedIdle),
            (CheckboxState::UncheckedHovered, POINTER_UP) => Some(CheckboxState::CheckedHovered),
            // Checked transitions
            (CheckboxState::CheckedIdle, POINTER_ENTER) => Some(CheckboxState::CheckedHovered),
            (CheckboxState::CheckedHovered, POINTER_LEAVE) => Some(CheckboxState::CheckedIdle),
            (CheckboxState::CheckedHovered, POINTER_UP) => Some(CheckboxState::UncheckedHovered),
            _ => None,
        }
    }
}

/// Text field focus states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextFieldState {
    #[default]
    Idle,
    Hovered,
    Focused,
    FocusedHovered,
    Disabled,
}

impl TextFieldState {
    /// Returns true if the text field is focused
    pub fn is_focused(&self) -> bool {
        matches!(
            self,
            TextFieldState::Focused | TextFieldState::FocusedHovered
        )
    }

    /// Returns true if the text field is hovered
    pub fn is_hovered(&self) -> bool {
        matches!(
            self,
            TextFieldState::Hovered | TextFieldState::FocusedHovered
        )
    }
}

impl StateTransitions for TextFieldState {
    fn on_event(&self, event: u32) -> Option<Self> {
        use blinc_core::events::event_types::*;
        match (self, event) {
            // Idle transitions
            (TextFieldState::Idle, POINTER_ENTER) => Some(TextFieldState::Hovered),
            (TextFieldState::Idle, FOCUS) => Some(TextFieldState::Focused),
            // Hovered transitions
            (TextFieldState::Hovered, POINTER_LEAVE) => Some(TextFieldState::Idle),
            (TextFieldState::Hovered, POINTER_DOWN) => Some(TextFieldState::Focused),
            (TextFieldState::Hovered, FOCUS) => Some(TextFieldState::FocusedHovered),
            // Focused transitions
            (TextFieldState::Focused, BLUR) => Some(TextFieldState::Idle),
            (TextFieldState::Focused, POINTER_ENTER) => Some(TextFieldState::FocusedHovered),
            // FocusedHovered transitions
            (TextFieldState::FocusedHovered, POINTER_LEAVE) => Some(TextFieldState::Focused),
            (TextFieldState::FocusedHovered, BLUR) => Some(TextFieldState::Hovered),
            // Disabled ignores all events
            (TextFieldState::Disabled, _) => None,
            _ => None,
        }
    }
}

// =========================================================================
// Stateful<S> - Generic Stateful Element
// =========================================================================

/// A stateful element with user-defined state type
///
/// The state type `S` must implement `StateTransitions` to define how
/// events cause state changes. Use the `on_state` callback to apply
/// visual changes based on state using pattern matching.
///
/// # Example
///
/// ```ignore
/// use blinc_layout::prelude::*;
///
/// let button = Stateful::new(ButtonState::Idle)
///     .w(100.0).h(40.0)
///     .on_state(|state, div| match state {
///         ButtonState::Idle => { *div = div.swap().bg(Color::BLUE); }
///         ButtonState::Hovered => { *div = div.swap().bg(Color::CYAN); }
///         ButtonState::Pressed => { *div = div.swap().bg(Color::BLUE).scale(0.97); }
///         ButtonState::Disabled => { *div = div.swap().bg(Color::GRAY); }
///     });
/// ```
pub struct Stateful<S: StateTransitions> {
    /// Inner div with all layout/visual properties
    inner: Div,

    /// Current state
    state: S,

    /// State change callback (receives state for pattern matching)
    state_callback: Option<StateCallback<S>>,

    /// Node ID for event dispatch
    layout_node_id: Option<LayoutNodeId>,

    /// Phantom for FSM (kept for future integration)
    _fsm: PhantomData<StateMachine>,
}

impl<S: StateTransitions + Default> Default for Stateful<S> {
    fn default() -> Self {
        Self::new(S::default())
    }
}

// Deref to Div so all Div methods are available
impl<S: StateTransitions> Deref for Stateful<S> {
    type Target = Div;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S: StateTransitions> DerefMut for Stateful<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<S: StateTransitions> Stateful<S> {
    /// Create a new stateful element with initial state
    pub fn new(initial_state: S) -> Self {
        Self {
            inner: Div::new(),
            state: initial_state,
            state_callback: None,
            layout_node_id: None,
            _fsm: PhantomData,
        }
    }

    /// Set the initial/default state
    ///
    /// This is useful when using the generic `stateful()` constructor
    /// with a custom state type.
    ///
    /// # Example
    ///
    /// ```ignore
    /// stateful()
    ///     .default_state(MyState::Ready)
    ///     .on_state(|state, div| { ... })
    /// ```
    pub fn default_state(mut self, state: S) -> Self {
        self.state = state;
        self
    }

    /// Get the layout node ID (set after registration)
    pub fn layout_node_id(&self) -> Option<LayoutNodeId> {
        self.layout_node_id
    }

    /// Get the current state
    pub fn state(&self) -> &S {
        &self.state
    }

    // =========================================================================
    // State Callback
    // =========================================================================

    /// Set the state change callback
    ///
    /// The callback receives the current state for pattern matching and
    /// a mutable reference to the inner Div for applying visual changes.
    /// The callback is immediately applied to set the initial visual state.
    ///
    /// # Example
    ///
    /// ```ignore
    /// .on_state(|state, div| match state {
    ///     ButtonState::Idle => { *div = div.swap().bg(Color::BLUE); }
    ///     ButtonState::Hovered => { *div = div.swap().bg(Color::CYAN); }
    ///     // ...
    /// })
    /// ```
    pub fn on_state<F>(mut self, callback: F) -> Self
    where
        F: Fn(&S, &mut Div) + Send + Sync + 'static,
    {
        self.state_callback = Some(Box::new(callback));
        // Immediately apply to set initial visual state
        self.apply_state_callback();
        self
    }

    /// Dispatch a new state
    ///
    /// Updates the current state and applies the callback if the state changed.
    /// Returns true if the state changed.
    pub fn dispatch_state(&mut self, new_state: S) -> bool {
        if self.state != new_state {
            self.state = new_state;
            self.apply_state_callback();
            true
        } else {
            false
        }
    }

    /// Handle an event and potentially transition state
    ///
    /// Returns true if the state changed.
    pub fn handle_event(&mut self, event: u32) -> bool {
        if let Some(new_state) = self.state.on_event(event) {
            self.dispatch_state(new_state)
        } else {
            false
        }
    }

    /// Apply the callback for the current state (if any)
    fn apply_state_callback(&mut self) {
        if let Some(ref callback) = self.state_callback {
            callback(&self.state, &mut self.inner);
        }
    }

    /// Register with an interactive context for event handling
    pub fn register(mut self, ctx: &mut InteractiveContext) -> Self {
        use slotmap::KeyData;

        // Generate unique ID
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let layout_id = LayoutNodeId::from(KeyData::from_ffi((1u64 << 32) | id));

        ctx.register(layout_id, None);
        self.layout_node_id = Some(layout_id);
        self
    }

    // =========================================================================
    // Builder pattern methods that return Self (not Div)
    // =========================================================================

    /// Set width (builder pattern)
    pub fn w(mut self, px: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).w(px);
        self
    }

    /// Set height (builder pattern)
    pub fn h(mut self, px: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).h(px);
        self
    }

    /// Set width to 100% (builder pattern)
    pub fn w_full(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).w_full();
        self
    }

    /// Set height to 100% (builder pattern)
    pub fn h_full(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).h_full();
        self
    }

    /// Set both width and height (builder pattern)
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).size(w, h);
        self
    }

    /// Set square size (builder pattern)
    pub fn square(mut self, size: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).square(size);
        self
    }

    /// Set flex direction to row (builder pattern)
    pub fn flex_row(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).flex_row();
        self
    }

    /// Set flex direction to column (builder pattern)
    pub fn flex_col(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).flex_col();
        self
    }

    /// Set flex grow (builder pattern)
    pub fn flex_grow(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).flex_grow();
        self
    }

    /// Set padding all sides (builder pattern)
    pub fn p(mut self, units: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).p(units);
        self
    }

    /// Set horizontal padding (builder pattern)
    pub fn px(mut self, units: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).px(units);
        self
    }

    /// Set vertical padding (builder pattern)
    pub fn py(mut self, units: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).py(units);
        self
    }

    /// Set gap (builder pattern)
    pub fn gap(mut self, units: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).gap(units);
        self
    }

    /// Center items (builder pattern)
    pub fn items_center(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).items_center();
        self
    }

    /// Center justify (builder pattern)
    pub fn justify_center(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).justify_center();
        self
    }

    /// Space between (builder pattern)
    pub fn justify_between(mut self) -> Self {
        self.inner = std::mem::take(&mut self.inner).justify_between();
        self
    }

    /// Set background (builder pattern)
    pub fn bg(mut self, color: impl Into<blinc_core::Brush>) -> Self {
        self.inner = std::mem::take(&mut self.inner).background(color);
        self
    }

    /// Set corner radius (builder pattern)
    pub fn rounded(mut self, radius: f32) -> Self {
        self.inner = std::mem::take(&mut self.inner).rounded(radius);
        self
    }

    /// Set shadow (builder pattern)
    pub fn shadow(mut self, shadow: blinc_core::Shadow) -> Self {
        self.inner = std::mem::take(&mut self.inner).shadow(shadow);
        self
    }

    /// Set transform (builder pattern)
    pub fn transform(mut self, transform: blinc_core::Transform) -> Self {
        self.inner = std::mem::take(&mut self.inner).transform(transform);
        self
    }

    /// Add child (builder pattern)
    pub fn child(mut self, child: impl ElementBuilder + 'static) -> Self {
        self.inner = std::mem::take(&mut self.inner).child(child);
        self
    }

    /// Add children (builder pattern)
    pub fn children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator,
        I::Item: ElementBuilder + 'static,
    {
        self.inner = std::mem::take(&mut self.inner).children(children);
        self
    }

    /// Bind this element to an ElementRef for external access
    ///
    /// Returns a `BoundStateful` that continues the fluent API chain while
    /// also making the element accessible via the ref.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let button_ref = ElementRef::<StatefulButton>::new();
    ///
    /// let ui = div()
    ///     .child(
    ///         stateful_button()
    ///             .bind(&button_ref)  // Binds and continues chain
    ///             .on_state(|state, div| { ... })
    ///     );
    ///
    /// // Later, access via the ref
    /// button_ref.with_mut(|btn| {
    ///     btn.dispatch_state(ButtonState::Pressed);
    /// });
    /// ```
    pub fn bind(self, element_ref: &ElementRef<Self>) -> BoundStateful<S> {
        // Store self in the ElementRef's shared storage
        element_ref.set(self);
        // Return a wrapper that shares the same storage
        BoundStateful {
            storage: element_ref.storage(),
        }
    }
}

// =========================================================================
// BoundStateful - Wrapper for bound stateful elements
// =========================================================================

/// A bound stateful element that maintains shared storage with an ElementRef
///
/// This wrapper is returned by `Stateful::bind()` and provides the same
/// fluent API as `Stateful`, but all modifications go through shared storage
/// accessible via the original `ElementRef`.
pub struct BoundStateful<S: StateTransitions> {
    storage: Arc<Mutex<Option<Stateful<S>>>>,
}

impl<S: StateTransitions> BoundStateful<S> {
    /// Apply a transformation to the stored element
    fn transform_inner<F>(self, f: F) -> Self
    where
        F: FnOnce(Stateful<S>) -> Stateful<S>,
    {
        let mut guard = self.storage.lock().unwrap();
        if let Some(elem) = guard.take() {
            *guard = Some(f(elem));
        }
        drop(guard);
        self
    }

    // =========================================================================
    // Delegated builder methods
    // =========================================================================

    /// Set the state callback (builder pattern)
    pub fn on_state<F>(self, callback: F) -> Self
    where
        F: Fn(&S, &mut Div) + Send + Sync + 'static,
    {
        self.transform_inner(|s| s.on_state(callback))
    }

    /// Set width (builder pattern)
    pub fn w(self, px: f32) -> Self {
        self.transform_inner(|s| s.w(px))
    }

    /// Set height (builder pattern)
    pub fn h(self, px: f32) -> Self {
        self.transform_inner(|s| s.h(px))
    }

    /// Set width to 100% (builder pattern)
    pub fn w_full(self) -> Self {
        self.transform_inner(|s| s.w_full())
    }

    /// Set height to 100% (builder pattern)
    pub fn h_full(self) -> Self {
        self.transform_inner(|s| s.h_full())
    }

    /// Set both width and height (builder pattern)
    pub fn size(self, w: f32, h: f32) -> Self {
        self.transform_inner(|s| s.size(w, h))
    }

    /// Set square size (builder pattern)
    pub fn square(self, size: f32) -> Self {
        self.transform_inner(|s| s.square(size))
    }

    /// Set flex direction to row (builder pattern)
    pub fn flex_row(self) -> Self {
        self.transform_inner(|s| s.flex_row())
    }

    /// Set flex direction to column (builder pattern)
    pub fn flex_col(self) -> Self {
        self.transform_inner(|s| s.flex_col())
    }

    /// Set flex grow (builder pattern)
    pub fn flex_grow(self) -> Self {
        self.transform_inner(|s| s.flex_grow())
    }

    /// Set padding all sides (builder pattern)
    pub fn p(self, units: f32) -> Self {
        self.transform_inner(|s| s.p(units))
    }

    /// Set horizontal padding (builder pattern)
    pub fn px(self, units: f32) -> Self {
        self.transform_inner(|s| s.px(units))
    }

    /// Set vertical padding (builder pattern)
    pub fn py(self, units: f32) -> Self {
        self.transform_inner(|s| s.py(units))
    }

    /// Set gap (builder pattern)
    pub fn gap(self, units: f32) -> Self {
        self.transform_inner(|s| s.gap(units))
    }

    /// Center items (builder pattern)
    pub fn items_center(self) -> Self {
        self.transform_inner(|s| s.items_center())
    }

    /// Center justify (builder pattern)
    pub fn justify_center(self) -> Self {
        self.transform_inner(|s| s.justify_center())
    }

    /// Space between (builder pattern)
    pub fn justify_between(self) -> Self {
        self.transform_inner(|s| s.justify_between())
    }

    /// Set background (builder pattern)
    pub fn bg(self, color: impl Into<blinc_core::Brush>) -> Self {
        let brush = color.into();
        self.transform_inner(|s| s.bg(brush))
    }

    /// Set corner radius (builder pattern)
    pub fn rounded(self, radius: f32) -> Self {
        self.transform_inner(|s| s.rounded(radius))
    }

    /// Set shadow (builder pattern)
    pub fn shadow(self, shadow: blinc_core::Shadow) -> Self {
        self.transform_inner(|s| s.shadow(shadow))
    }

    /// Set transform (builder pattern)
    pub fn transform_style(self, xform: blinc_core::Transform) -> Self {
        self.transform_inner(|s| s.transform(xform))
    }

    /// Add child (builder pattern)
    pub fn child(self, child: impl ElementBuilder + 'static) -> Self {
        self.transform_inner(|s| s.child(child))
    }
}

impl<S: StateTransitions + Default> ElementBuilder for BoundStateful<S> {
    fn build(&self, tree: &mut LayoutTree) -> LayoutNodeId {
        self.storage
            .lock()
            .unwrap()
            .as_ref()
            .map(|s| s.build(tree))
            .expect("BoundStateful: element not bound")
    }

    fn render_props(&self) -> RenderProps {
        self.storage
            .lock()
            .unwrap()
            .as_ref()
            .map(|s| s.render_props())
            .unwrap_or_default()
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        // Can't return reference through mutex, children handled via build()
        &[]
    }

    fn element_type_id(&self) -> ElementTypeId {
        ElementTypeId::Div
    }
}

impl<S: StateTransitions> ElementBuilder for Stateful<S> {
    fn build(&self, tree: &mut LayoutTree) -> LayoutNodeId {
        self.inner.build(tree)
    }

    fn render_props(&self) -> RenderProps {
        let mut props = self.inner.render_props();
        props.node_id = self.layout_node_id;
        props
    }

    fn children_builders(&self) -> &[Box<dyn ElementBuilder>] {
        self.inner.children_builders()
    }

    fn element_type_id(&self) -> ElementTypeId {
        ElementTypeId::Div
    }
}

// =========================================================================
// Convenience Type Aliases
// =========================================================================

/// A stateful button element
pub type StatefulButton = Stateful<ButtonState>;

/// A stateful toggle element
pub type StatefulToggle = Stateful<ToggleState>;

/// A stateful checkbox element
pub type StatefulCheckbox = Stateful<CheckboxState>;

/// A stateful text field element
pub type StatefulTextField = Stateful<TextFieldState>;

// =========================================================================
// Convenience Constructors
// =========================================================================

/// Create a new stateful element with default state
pub fn stateful<S: StateTransitions + Default>() -> Stateful<S> {
    Stateful::new(S::default())
}

/// Create a stateful button (idle state)
pub fn stateful_button() -> StatefulButton {
    Stateful::new(ButtonState::Idle)
}

/// Create a stateful toggle
pub fn stateful_toggle(initially_on: bool) -> StatefulToggle {
    Stateful::new(if initially_on {
        ToggleState::On
    } else {
        ToggleState::Off
    })
}

/// Create a stateful checkbox
pub fn stateful_checkbox(initially_checked: bool) -> StatefulCheckbox {
    Stateful::new(if initially_checked {
        CheckboxState::CheckedIdle
    } else {
        CheckboxState::UncheckedIdle
    })
}

/// Create a stateful text field
pub fn stateful_text_field() -> StatefulTextField {
    Stateful::new(TextFieldState::Idle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::text;
    use blinc_core::events::event_types;
    use blinc_core::{Brush, Color, CornerRadius, Shadow, Transform};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_stateful_basic() {
        let elem: Stateful<ButtonState> =
            Stateful::new(ButtonState::Idle).w(100.0).h(40.0).bg(Color::BLUE).rounded(8.0);

        let mut tree = LayoutTree::new();
        let _node = elem.build(&mut tree);
    }

    #[test]
    fn test_state_callback_with_pattern_matching() {
        let elem = stateful_button()
            .w(100.0)
            .h(40.0)
            .on_state(|state, div| match state {
                ButtonState::Idle => {
                    *div = div.swap().bg(Color::BLUE).rounded(4.0);
                }
                ButtonState::Hovered => {
                    *div = div.swap().bg(Color::GREEN).rounded(8.0);
                }
                ButtonState::Pressed => {
                    *div = div.swap().bg(Color::RED);
                }
                ButtonState::Disabled => {
                    *div = div.swap().bg(Color::GRAY);
                }
            });

        let props = elem.render_props();
        // Should have blue background from idle state
        assert!(matches!(props.background, Some(Brush::Solid(c)) if c == Color::BLUE));
        assert_eq!(props.border_radius, CornerRadius::uniform(4.0));
    }

    #[test]
    fn test_state_transition_with_enum() {
        let mut elem = stateful_button()
            .w(100.0)
            .h(40.0)
            .on_state(|state, div| match state {
                ButtonState::Idle => {
                    *div = div.swap().bg(Color::BLUE);
                }
                ButtonState::Hovered => {
                    *div = div.swap().bg(Color::GREEN);
                }
                _ => {}
            });

        // Initial state is idle (blue)
        let props = elem.render_props();
        assert!(matches!(props.background, Some(Brush::Solid(c)) if c == Color::BLUE));

        // Transition to hovered
        let changed = elem.dispatch_state(ButtonState::Hovered);
        assert!(changed);
        assert_eq!(*elem.state(), ButtonState::Hovered);

        let props = elem.render_props();
        assert!(matches!(props.background, Some(Brush::Solid(c)) if c == Color::GREEN));

        // Transition to same state should return false
        let changed = elem.dispatch_state(ButtonState::Hovered);
        assert!(!changed);
    }

    #[test]
    fn test_handle_event() {
        let mut elem = stateful_button()
            .w(100.0)
            .on_state(|state, div| match state {
                ButtonState::Idle => {
                    *div = div.swap().bg(Color::BLUE);
                }
                ButtonState::Hovered => {
                    *div = div.swap().bg(Color::GREEN);
                }
                ButtonState::Pressed => {
                    *div = div.swap().bg(Color::RED);
                }
                _ => {}
            });

        assert_eq!(*elem.state(), ButtonState::Idle);

        // Pointer enter -> Hovered
        let changed = elem.handle_event(event_types::POINTER_ENTER);
        assert!(changed);
        assert_eq!(*elem.state(), ButtonState::Hovered);

        // Pointer down -> Pressed
        let changed = elem.handle_event(event_types::POINTER_DOWN);
        assert!(changed);
        assert_eq!(*elem.state(), ButtonState::Pressed);

        // Pointer up -> Back to Hovered
        let changed = elem.handle_event(event_types::POINTER_UP);
        assert!(changed);
        assert_eq!(*elem.state(), ButtonState::Hovered);
    }

    #[test]
    fn test_callback_is_called() {
        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let _elem = stateful_button()
            .w(100.0)
            .on_state(move |_state, _div| {
                call_count_clone.fetch_add(1, Ordering::SeqCst);
            });

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_button_with_state_callbacks() {
        let btn = stateful_button()
            .w(120.0)
            .h(40.0)
            .px(16.0)
            .py(8.0)
            .on_state(|state, div| match state {
                ButtonState::Idle => {
                    *div = div.swap()
                        .bg(Color::rgba(0.2, 0.5, 0.9, 1.0))
                        .rounded(8.0);
                }
                ButtonState::Hovered => {
                    *div = div.swap()
                        .bg(Color::rgba(0.3, 0.6, 1.0, 1.0))
                        .rounded(8.0)
                        .shadow(Shadow::new(0.0, 2.0, 4.0, Color::rgba(0.0, 0.0, 0.0, 0.1)));
                }
                ButtonState::Pressed => {
                    *div = div.swap()
                        .bg(Color::rgba(0.15, 0.4, 0.8, 1.0))
                        .rounded(8.0)
                        .transform(Transform::scale(0.97, 0.97));
                }
                ButtonState::Disabled => {
                    *div = div.swap().bg(Color::GRAY);
                }
            })
            .child(text("Click me"));

        let mut tree = LayoutTree::new();
        let _node = btn.build(&mut tree);

        // Verify idle state
        let props = btn.render_props();
        assert!(props.background.is_some());
        assert_eq!(props.border_radius, CornerRadius::uniform(8.0));
    }

    #[test]
    fn test_toggle_states() {
        let mut toggle = stateful_toggle(false)
            .w(50.0)
            .h(30.0)
            .on_state(|state, div| match state {
                ToggleState::Off => {
                    *div = div.swap().bg(Color::GRAY);
                }
                ToggleState::On => {
                    *div = div.swap().bg(Color::GREEN);
                }
            });

        // Initially off
        assert_eq!(*toggle.state(), ToggleState::Off);
        let props = toggle.render_props();
        assert!(matches!(props.background, Some(Brush::Solid(c)) if c == Color::GRAY));

        // Click to toggle on
        toggle.handle_event(event_types::POINTER_UP);
        assert_eq!(*toggle.state(), ToggleState::On);
        let props = toggle.render_props();
        assert!(matches!(props.background, Some(Brush::Solid(c)) if c == Color::GREEN));

        // Click to toggle off
        toggle.handle_event(event_types::POINTER_UP);
        assert_eq!(*toggle.state(), ToggleState::Off);
    }

    #[test]
    fn test_checkbox_states() {
        let mut checkbox = stateful_checkbox(false)
            .square(24.0)
            .on_state(|state, div| match state {
                CheckboxState::UncheckedIdle => {
                    *div = div.swap().bg(Color::WHITE).rounded(4.0);
                }
                CheckboxState::UncheckedHovered => {
                    *div = div.swap().bg(Color::GRAY).rounded(4.0);
                }
                CheckboxState::CheckedIdle => {
                    *div = div.swap().bg(Color::BLUE).rounded(4.0);
                }
                CheckboxState::CheckedHovered => {
                    *div = div.swap().bg(Color::CYAN).rounded(4.0);
                }
            });

        // Start unchecked
        assert!(!checkbox.state().is_checked());

        // Hover
        checkbox.handle_event(event_types::POINTER_ENTER);
        assert_eq!(*checkbox.state(), CheckboxState::UncheckedHovered);
        assert!(checkbox.state().is_hovered());

        // Click to check
        checkbox.handle_event(event_types::POINTER_UP);
        assert_eq!(*checkbox.state(), CheckboxState::CheckedHovered);
        assert!(checkbox.state().is_checked());

        // Leave hover while checked
        checkbox.handle_event(event_types::POINTER_LEAVE);
        assert_eq!(*checkbox.state(), CheckboxState::CheckedIdle);
        assert!(checkbox.state().is_checked());
        assert!(!checkbox.state().is_hovered());
    }

    #[test]
    fn test_text_field_states() {
        let mut field = stateful_text_field()
            .w(200.0)
            .h(40.0)
            .on_state(|state, div| match state {
                TextFieldState::Idle => {
                    *div = div.swap().bg(Color::WHITE).rounded(4.0);
                }
                TextFieldState::Hovered => {
                    *div = div.swap().bg(Color::WHITE).rounded(4.0);
                }
                TextFieldState::Focused => {
                    *div = div.swap().bg(Color::WHITE).rounded(4.0);
                }
                TextFieldState::FocusedHovered => {
                    *div = div.swap().bg(Color::WHITE).rounded(4.0);
                }
                TextFieldState::Disabled => {
                    *div = div.swap().bg(Color::GRAY);
                }
            });

        assert_eq!(*field.state(), TextFieldState::Idle);
        assert!(!field.state().is_focused());

        // Click to focus
        field.handle_event(event_types::POINTER_ENTER);
        field.handle_event(event_types::POINTER_DOWN);
        assert!(field.state().is_focused());

        // Blur
        field.handle_event(event_types::BLUR);
        assert!(!field.state().is_focused());
    }

    #[test]
    fn test_custom_state_type() {
        // User can define their own state type
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        enum MyState {
            #[default]
            Ready,
            Loading,
            Success,
            Error,
        }

        impl StateTransitions for MyState {
            fn on_event(&self, event: u32) -> Option<Self> {
                // Custom event mapping
                const START_LOADING: u32 = 1000;
                const LOAD_SUCCESS: u32 = 1001;
                const LOAD_ERROR: u32 = 1002;
                const RESET: u32 = 1003;

                match (self, event) {
                    (MyState::Ready, START_LOADING) => Some(MyState::Loading),
                    (MyState::Loading, LOAD_SUCCESS) => Some(MyState::Success),
                    (MyState::Loading, LOAD_ERROR) => Some(MyState::Error),
                    (MyState::Success | MyState::Error, RESET) => Some(MyState::Ready),
                    _ => None,
                }
            }
        }

        let mut elem: Stateful<MyState> = stateful()
            .default_state(MyState::Ready)
            .w(100.0)
            .on_state(|state, div| match state {
                MyState::Ready => {
                    *div = div.swap().bg(Color::BLUE);
                }
                MyState::Loading => {
                    *div = div.swap().bg(Color::YELLOW);
                }
                MyState::Success => {
                    *div = div.swap().bg(Color::GREEN);
                }
                MyState::Error => {
                    *div = div.swap().bg(Color::RED);
                }
            });

        assert_eq!(*elem.state(), MyState::Ready);

        // Transition via event
        elem.handle_event(1000); // START_LOADING
        assert_eq!(*elem.state(), MyState::Loading);

        elem.handle_event(1001); // LOAD_SUCCESS
        assert_eq!(*elem.state(), MyState::Success);

        elem.handle_event(1003); // RESET
        assert_eq!(*elem.state(), MyState::Ready);
    }

    #[test]
    fn test_deref_to_div() {
        let mut elem: Stateful<ButtonState> = Stateful::new(ButtonState::Idle).w(100.0);

        // Can access Div methods via DerefMut
        elem.style_mut().flex_grow = 2.0;

        let mut tree = LayoutTree::new();
        let _node = elem.build(&mut tree);
    }

    #[test]
    fn test_full_div_api_access() {
        let elem = stateful_button()
            .w(200.0)
            .h(100.0)
            .flex_row()
            .gap(4.0)
            .p(2.0)
            .items_center()
            .justify_between()
            .bg(Color::BLUE)
            .rounded(8.0)
            .shadow(Shadow::new(0.0, 4.0, 8.0, Color::BLACK))
            .child(text("Hello"))
            .child(text("World"));

        let mut tree = LayoutTree::new();
        let _node = elem.build(&mut tree);
    }

    #[test]
    fn test_disabled_button_ignores_events() {
        let mut btn = Stateful::new(ButtonState::Disabled)
            .w(100.0)
            .on_state(|_state, _div| {});

        assert_eq!(*btn.state(), ButtonState::Disabled);

        // All events should be ignored
        assert!(!btn.handle_event(event_types::POINTER_ENTER));
        assert!(!btn.handle_event(event_types::POINTER_DOWN));
        assert!(!btn.handle_event(event_types::POINTER_UP));

        assert_eq!(*btn.state(), ButtonState::Disabled);
    }

    #[test]
    fn test_bind_fluent_api() {
        use crate::div::{div, ElementRef};

        // Create a ref for external access
        let button_ref = ElementRef::<StatefulButton>::new();

        // Use .bind() in the fluent chain - continues naturally
        let _ui = div()
            .flex_col()
            .child(
                stateful_button()
                    .bind(&button_ref)  // Binds here, chain continues
                    .w(100.0)
                    .h(40.0)
                    .on_state(|state, div| match state {
                        ButtonState::Idle => {
                            *div = div.swap().bg(Color::BLUE);
                        }
                        ButtonState::Hovered => {
                            *div = div.swap().bg(Color::CYAN);
                        }
                        _ => {}
                    }),
            );

        // Ref is now bound
        assert!(button_ref.is_bound());

        // Direct access via borrow() - cleaner API
        let state = *button_ref.borrow().state();
        assert_eq!(state, ButtonState::Idle);

        // Direct mutation via borrow_mut() - like the user's desired pattern:
        // button_ref.borrow_mut().dispatch_state(...)
        button_ref.borrow_mut().dispatch_state(ButtonState::Hovered);

        // Verify state changed
        let new_state = *button_ref.borrow().state();
        assert_eq!(new_state, ButtonState::Hovered);

        // Closure-based API still available for fallible access
        let via_closure = button_ref.with(|btn| *btn.state());
        assert_eq!(via_closure, Some(ButtonState::Hovered));
    }
}
