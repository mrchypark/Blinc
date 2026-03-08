//! Element handle for programmatic element manipulation

use std::sync::{Arc, Mutex};

use blinc_core::context_state::{
    MotionAnimationState, ProgrammaticElementEvent, ScrollIntoViewOptions,
};
use blinc_core::BlincContextState;

use crate::element::{ElementBounds, RenderProps};
use crate::event_router::{EventRouter, MouseButton};
use crate::renderer::RenderTree;
use crate::tree::LayoutNodeId;

use super::registry::{ElementRegistry, OnReadyCallback};
use super::ScrollOptions;

fn router_bounds_or_absolute(
    tree: &RenderTree,
    router: &EventRouter,
    node_id: LayoutNodeId,
) -> Option<(f32, f32, f32, f32)> {
    router.get_node_bounds(node_id).or_else(|| {
        tree.get_absolute_bounds(node_id)
            .map(|bounds| (bounds.x, bounds.y, bounds.width, bounds.height))
    })
}

fn dispatch_pointer_events(
    tree: &mut RenderTree,
    router: &EventRouter,
    events: Vec<(LayoutNodeId, u32)>,
    mouse_x: f32,
    mouse_y: f32,
) -> bool {
    let mut dispatched = false;
    for (node_id, event_type) in events {
        let Some((bounds_x, bounds_y, bounds_width, bounds_height)) =
            router_bounds_or_absolute(tree, router, node_id)
        else {
            continue;
        };
        tree.dispatch_event_full(
            node_id,
            event_type,
            mouse_x,
            mouse_y,
            mouse_x - bounds_x,
            mouse_y - bounds_y,
            bounds_x,
            bounds_y,
            bounds_width,
            bounds_height,
            0.0,
            0.0,
            1.0,
        );
        dispatched = true;
    }
    dispatched
}

fn dispatch_router_events(
    tree: &mut RenderTree,
    router: &EventRouter,
    events: Vec<(LayoutNodeId, u32)>,
) -> bool {
    let mut dispatched = false;
    for (node_id, event_type) in events {
        let Some((bounds_x, bounds_y, bounds_width, bounds_height)) =
            router_bounds_or_absolute(tree, router, node_id)
        else {
            continue;
        };
        let mouse_x = bounds_x + bounds_width * 0.5;
        let mouse_y = bounds_y + bounds_height * 0.5;
        tree.dispatch_event_full(
            node_id,
            event_type,
            mouse_x,
            mouse_y,
            mouse_x - bounds_x,
            mouse_y - bounds_y,
            bounds_x,
            bounds_y,
            bounds_width,
            bounds_height,
            0.0,
            0.0,
            1.0,
        );
        dispatched = true;
    }
    dispatched
}

fn ancestor_chain_for(tree: &RenderTree, node_id: LayoutNodeId) -> Vec<LayoutNodeId> {
    let mut ancestors = tree.element_registry().ancestors(node_id);
    ancestors.reverse();
    ancestors.push(node_id);
    ancestors
}

fn target_point(bounds: ElementBounds, local_x: f32, local_y: f32) -> (f32, f32) {
    let max_x = (bounds.width - f32::EPSILON).max(0.0);
    let max_y = (bounds.height - f32::EPSILON).max(0.0);
    let clamped_x = local_x.clamp(0.0, max_x);
    let clamped_y = local_y.clamp(0.0, max_y);
    (bounds.x + clamped_x, bounds.y + clamped_y)
}

fn bounds_center(bounds: ElementBounds) -> (f32, f32) {
    (
        bounds.x + bounds.width * 0.5,
        bounds.y + bounds.height * 0.5,
    )
}

fn modifier_flags(modifiers: u8) -> (bool, bool, bool, bool) {
    (
        modifiers & 0b0001 != 0,
        modifiers & 0b0010 != 0,
        modifiers & 0b0100 != 0,
        modifiers & 0b1000 != 0,
    )
}

fn is_text_entry_target(tree: &RenderTree, node_id: LayoutNodeId) -> bool {
    matches!(
        tree.element_registry().get_element_type(node_id).as_deref(),
        Some("input") | Some("textarea")
    )
}

/// Route a programmatic interaction for a resolved node through the existing runtime path.
pub fn dispatch_programmatic_event_to_node(
    tree: &mut RenderTree,
    router: &mut EventRouter,
    node_id: LayoutNodeId,
    event: ProgrammaticElementEvent,
) -> bool {
    match event {
        ProgrammaticElementEvent::Click { x, y } => {
            let Some(bounds) = tree.get_absolute_bounds(node_id) else {
                return false;
            };
            let (mouse_x, mouse_y) = target_point(bounds, x, y);
            let hover_events = router.on_mouse_move(tree, mouse_x, mouse_y);
            let mut dispatched =
                dispatch_pointer_events(tree, router, hover_events, mouse_x, mouse_y);
            let down_events = router.on_mouse_down(tree, mouse_x, mouse_y, MouseButton::Left);
            dispatched |= dispatch_pointer_events(tree, router, down_events, mouse_x, mouse_y);
            let up_events = router.on_mouse_up(tree, mouse_x, mouse_y, MouseButton::Left);
            dispatched |= dispatch_pointer_events(tree, router, up_events, mouse_x, mouse_y);
            dispatched
        }
        ProgrammaticElementEvent::MouseEnter => {
            let Some(bounds) = tree.get_absolute_bounds(node_id) else {
                return false;
            };
            let (mouse_x, mouse_y) = bounds_center(bounds);
            let hover_events = router.on_mouse_move(tree, mouse_x, mouse_y);
            dispatch_pointer_events(tree, router, hover_events, mouse_x, mouse_y)
        }
        ProgrammaticElementEvent::MouseLeave => {
            let mouse = tree
                .get_absolute_bounds(node_id)
                .map(bounds_center)
                .unwrap_or((0.0, 0.0));
            let leave_events = router.on_mouse_leave();
            dispatch_pointer_events(tree, router, leave_events, mouse.0, mouse.1)
        }
        ProgrammaticElementEvent::KeyDown { key, modifiers } => {
            let ancestors = ancestor_chain_for(tree, node_id);
            router.set_focus_with_ancestors(Some(node_id), ancestors);
            let (shift, ctrl, alt, meta) = modifier_flags(modifiers);
            if let Some((_focused, event_type)) =
                router.on_key_down_with_modifiers(key, shift, ctrl, alt, meta)
            {
                tree.broadcast_key_event(event_type, key, shift, ctrl, alt, meta);
                true
            } else {
                false
            }
        }
        ProgrammaticElementEvent::KeyUp { key, modifiers } => {
            let ancestors = ancestor_chain_for(tree, node_id);
            router.set_focus_with_ancestors(Some(node_id), ancestors);
            let (shift, ctrl, alt, meta) = modifier_flags(modifiers);
            if let Some((_focused, event_type)) =
                router.on_key_up_with_modifiers(key, shift, ctrl, alt, meta)
            {
                tree.broadcast_key_event(event_type, key, shift, ctrl, alt, meta);
                true
            } else {
                false
            }
        }
        ProgrammaticElementEvent::TextInput { text, modifiers } => {
            let ancestors = ancestor_chain_for(tree, node_id);
            router.set_focus_with_ancestors(Some(node_id), ancestors);
            let (shift, ctrl, alt, meta) = modifier_flags(modifiers);
            if router.on_text_input(text).is_some() {
                tree.broadcast_text_input_event(text, shift, ctrl, alt, meta);
                true
            } else {
                false
            }
        }
        ProgrammaticElementEvent::Scroll { dx, dy } => {
            let Some(bounds) = tree.get_absolute_bounds(node_id) else {
                return false;
            };
            let (mouse_x, mouse_y) = bounds_center(bounds);
            let hover_events = router.on_mouse_move(tree, mouse_x, mouse_y);
            let _ = dispatch_pointer_events(tree, router, hover_events, mouse_x, mouse_y);
            let ancestors = ancestor_chain_for(tree, node_id);
            let mut chain = vec![node_id];
            chain.extend(ancestors.iter().rev().copied().filter(|id| *id != node_id));
            if chain
                .iter()
                .copied()
                .any(|candidate| tree.scroll_node_by(candidate, dx, dy))
            {
                return true;
            }
            let (remaining_x, remaining_y) =
                tree.dispatch_scroll_chain(node_id, &ancestors, mouse_x, mouse_y, dx, dy);
            (remaining_x - dx).abs() > f32::EPSILON || (remaining_y - dy).abs() > f32::EPSILON
        }
        ProgrammaticElementEvent::Custom(event_type) => {
            let Some(bounds) = tree.get_absolute_bounds(node_id) else {
                return false;
            };
            let (mouse_x, mouse_y) = bounds_center(bounds);
            tree.dispatch_event_full(
                node_id,
                event_type,
                mouse_x,
                mouse_y,
                mouse_x - bounds.x,
                mouse_y - bounds.y,
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
                0.0,
                0.0,
                1.0,
            );
            true
        }
    }
}

/// Route a programmatic interaction through the existing runtime event path.
pub fn dispatch_programmatic_event_to_runtime(
    tree: &mut RenderTree,
    router: &mut EventRouter,
    target_id: &str,
    event: ProgrammaticElementEvent,
) -> bool {
    let Some(node_id) = tree.query_by_id(target_id) else {
        return false;
    };
    dispatch_programmatic_event_to_node(tree, router, node_id, event)
}

/// Synchronize a resolved focus target into the existing runtime router.
pub fn sync_focus_node_to_runtime(
    tree: &mut RenderTree,
    router: &mut EventRouter,
    target: Option<LayoutNodeId>,
) -> bool {
    match target {
        Some(node_id) => {
            if is_text_entry_target(tree, node_id) {
                return dispatch_programmatic_event_to_node(
                    tree,
                    router,
                    node_id,
                    ProgrammaticElementEvent::Click { x: 0.0, y: 0.0 },
                );
            }
            crate::widgets::blur_all_text_inputs();
            let ancestors = ancestor_chain_for(tree, node_id);
            let (events, _) = router
                .collect_events(|router| router.set_focus_with_ancestors(Some(node_id), ancestors));
            dispatch_router_events(tree, router, events)
        }
        None => {
            crate::widgets::blur_all_text_inputs();
            let (events, _) = router.collect_events(|router| router.set_focus(None));
            dispatch_router_events(tree, router, events) || router.focused().is_none()
        }
    }
}

/// Synchronize programmatic focus changes into the existing runtime router.
pub fn sync_focus_to_runtime(
    tree: &mut RenderTree,
    router: &mut EventRouter,
    target_id: Option<&str>,
) -> bool {
    match target_id {
        Some(id) => {
            let Some(node_id) = tree.query_by_id(id) else {
                return false;
            };
            sync_focus_node_to_runtime(tree, router, Some(node_id))
        }
        None => sync_focus_node_to_runtime(tree, router, None),
    }
}

/// Mirror the runtime router focus back into `BlincContextState`.
pub fn sync_context_focus_from_runtime(tree: &RenderTree, router: &EventRouter) {
    let focused_id = router
        .focused()
        .and_then(|node_id| tree.element_registry().get_id(node_id));
    if let Some(ctx) = BlincContextState::try_get() {
        ctx.sync_focus_state(focused_id.as_deref());
    }
}

/// Drain queued focus/programmatic requests through the existing runtime path.
pub fn drain_programmatic_runtime_requests(
    tree: &mut RenderTree,
    router: &mut EventRouter,
    pending_focus_changes: &Arc<Mutex<Vec<Option<String>>>>,
    pending_scroll_requests: &Arc<Mutex<Vec<(String, ScrollIntoViewOptions)>>>,
    programmatic_events: &Arc<Mutex<Vec<(String, ProgrammaticElementEvent)>>>,
) -> bool {
    let mut handled = false;

    if let Ok(mut pending) = pending_focus_changes.lock() {
        for target_id in pending.drain(..) {
            handled |= sync_focus_to_runtime(tree, router, target_id.as_deref());
        }
    }

    if let Ok(mut pending) = pending_scroll_requests.lock() {
        for (target_id, options) in pending.drain(..) {
            handled |= tree.scroll_element_into_view(&target_id, options.into());
        }
    }

    if let Ok(mut pending) = programmatic_events.lock() {
        for (target_id, event) in pending.drain(..) {
            handled |= dispatch_programmatic_event_to_runtime(tree, router, &target_id, event);
        }
    }

    handled
}

/// Handle to a queried element for programmatic manipulation
///
/// Returned by `ctx.query("element-id")` for element manipulation.
/// The handle can be created even before the element exists in the tree,
/// allowing operations like `on_ready` to be registered early.
#[derive(Clone)]
pub struct ElementHandle<T = ()> {
    /// The string ID used to query this element
    string_id: String,
    /// Cached node_id (may be default if element doesn't exist yet)
    node_id: LayoutNodeId,
    registry: Arc<ElementRegistry>,
    /// Typed element data (if available)
    _marker: std::marker::PhantomData<T>,
}

impl<T> ElementHandle<T> {
    /// Create a new element handle from a string ID
    ///
    /// The handle is valid even if the element doesn't exist yet.
    /// Operations like `on_ready` will work and fire when the element is laid out.
    pub fn new(string_id: impl Into<String>, registry: Arc<ElementRegistry>) -> Self {
        let string_id = string_id.into();
        let node_id = registry.get(&string_id).unwrap_or_default();
        Self {
            string_id,
            node_id,
            registry,
            _marker: std::marker::PhantomData,
        }
    }

    /// Get the underlying layout node ID
    ///
    /// Returns a default ID if the element doesn't exist yet.
    pub fn node_id(&self) -> LayoutNodeId {
        // Refresh from registry in case element was created after handle
        self.registry.get(&self.string_id).unwrap_or(self.node_id)
    }

    /// Get the string ID of this element
    pub fn id(&self) -> &str {
        &self.string_id
    }

    /// Check if the element currently exists in the tree
    pub fn exists(&self) -> bool {
        self.registry.get(&self.string_id).is_some()
    }

    // =========================================================================
    // Layout & Visibility
    // =========================================================================

    /// Get the computed bounds of this element
    ///
    /// Returns None if layout hasn't been computed yet or the element doesn't exist.
    pub fn bounds(&self) -> Option<ElementBounds> {
        // Get bounds from the registry cache (populated by RenderTree after layout)
        let bounds = self.registry.get_bounds(&self.string_id)?;
        Some(ElementBounds::new(
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
        ))
    }

    /// Check if this element is visible in the viewport
    ///
    /// An element is visible if its bounds intersect with the window viewport.
    /// This is a simple viewport check - does not account for scroll container clipping.
    pub fn is_visible(&self) -> bool {
        let Some(bounds) = self.registry.get_bounds(&self.string_id) else {
            return false;
        };

        // Get viewport size from BlincContextState
        if let Some(ctx) = BlincContextState::try_get() {
            let (vw, vh) = ctx.viewport_size();
            // Check if element bounds intersect viewport
            bounds.x < vw
                && bounds.x + bounds.width > 0.0
                && bounds.y < vh
                && bounds.y + bounds.height > 0.0
        } else {
            // No context state, assume visible
            true
        }
    }

    // =========================================================================
    // Tree Traversal
    // =========================================================================

    /// Get the parent element handle
    pub fn parent(&self) -> Option<ElementHandle<()>> {
        let current_node_id = self.node_id();
        let parent_node_id = self.registry.get_parent(current_node_id)?;
        let parent_string_id = self.registry.get_id(parent_node_id)?;
        Some(ElementHandle::new(parent_string_id, self.registry.clone()))
    }

    /// Get all ancestors (immediate parent to root)
    pub fn ancestors(&self) -> impl Iterator<Item = ElementHandle<()>> {
        let current_node_id = self.node_id();
        let ancestors = self.registry.ancestors(current_node_id);
        let registry = self.registry.clone();
        ancestors.into_iter().filter_map(move |id| {
            let string_id = registry.get_id(id)?;
            Some(ElementHandle::new(string_id, registry.clone()))
        })
    }

    // =========================================================================
    // Navigation (scroll, focus)
    // =========================================================================

    /// Scroll this element into view using default options
    pub fn scroll_into_view(&self) {
        self.scroll_into_view_with(ScrollOptions::default());
    }

    /// Scroll this element into view with custom options
    pub fn scroll_into_view_with(&self, options: ScrollOptions) {
        // Use BlincContextState callback to scroll the element
        if let Some(ctx) = BlincContextState::try_get() {
            ctx.scroll_element_into_view_with_options(&self.string_id, options.into());
        }
    }

    /// Focus this element
    ///
    /// For focusable elements like TextInput, this sets keyboard focus.
    /// For other elements, this updates the EventRouter's focus state.
    pub fn focus(&self) {
        if let Some(ctx) = BlincContextState::try_get() {
            ctx.set_focus(Some(&self.string_id));
        }
    }

    /// Remove focus from this element
    pub fn blur(&self) {
        if let Some(ctx) = BlincContextState::try_get() {
            // Only blur if this element is currently focused
            if ctx.is_focused(&self.string_id) {
                ctx.set_focus(None);
            }
        }
    }

    /// Check if this element is currently focused
    pub fn is_focused(&self) -> bool {
        BlincContextState::try_get()
            .map(|ctx| ctx.is_focused(&self.string_id))
            .unwrap_or(false)
    }

    // =========================================================================
    // Signal Operations
    // =========================================================================

    /// Emit a signal to trigger reactive updates
    ///
    /// This notifies stateful elements that depend on this signal, triggering
    /// only the affected subtree rebuilds through the reactive system.
    ///
    /// Note: For typed signal updates, use `State::set()` directly which
    /// automatically triggers dependent updates.
    pub fn emit_signal(&self, signal_id: blinc_core::SignalId) {
        if let Some(ctx) = BlincContextState::try_get() {
            // Notify stateful elements via the callback - this triggers
            // targeted subtree rebuilds, not a full UI rebuild
            ctx.notify_stateful_deps(&[signal_id]);
        }
    }

    /// Mark this element as dirty, forcing a rebuild
    ///
    /// This triggers a UI rebuild. The hash-based diffing system will
    /// determine what actually needs to be updated.
    pub fn mark_dirty(&self) {
        if let Some(ctx) = BlincContextState::try_get() {
            ctx.request_rebuild();
        }
    }

    /// Mark this element's subtree as dirty with new children
    ///
    /// This queues an explicit subtree rebuild with the provided new children.
    /// Use this for more efficient updates when you know exactly what the
    /// new children should be.
    pub fn mark_dirty_subtree(&self, new_children: crate::div::Div) {
        if let Some(node_id) = self.registry.get(&self.string_id) {
            crate::stateful::queue_subtree_rebuild(node_id, new_children);
        }
    }

    /// Mark this element as visually dirty with new render props
    ///
    /// This queues a visual-only update that **skips layout recomputation**.
    /// Use this for changes to background, opacity, shadows, transforms, etc.
    ///
    /// This is the most efficient update method when you only need to change
    /// visual properties without affecting layout.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Change background color without triggering layout
    /// ctx.query("my-button").mark_visual_dirty(
    ///     RenderProps::default().with_background(Color::RED.into())
    /// );
    /// ```
    pub fn mark_visual_dirty(&self, props: RenderProps) {
        if let Some(node_id) = self.registry.get(&self.string_id) {
            crate::stateful::queue_prop_update(node_id, props);
        }
    }

    // =========================================================================
    // Event Simulation
    // =========================================================================

    /// Simulate a click event on this element
    pub fn click(&self) {
        if let Some(bounds) = self.bounds() {
            self.click_at(bounds.width * 0.5, bounds.height * 0.5);
        } else {
            self.dispatch_event(ElementEvent::Click { x: 0.0, y: 0.0 });
        }
    }

    /// Simulate a click at specific coordinates within the element
    pub fn click_at(&self, x: f32, y: f32) {
        self.dispatch_event(ElementEvent::Click { x, y });
    }

    /// Simulate hover enter or leave
    pub fn hover(&self, enter: bool) {
        if enter {
            self.dispatch_event(ElementEvent::MouseEnter);
        } else {
            self.dispatch_event(ElementEvent::MouseLeave);
        }
    }

    /// Dispatch a custom event to this element
    pub fn dispatch_event(&self, event: ElementEvent) {
        let Some(ctx) = BlincContextState::try_get() else {
            return;
        };

        match event {
            ElementEvent::Click { x, y } => {
                ctx.dispatch_programmatic_event(
                    &self.string_id,
                    ProgrammaticElementEvent::Click { x, y },
                );
            }
            ElementEvent::MouseEnter => {
                ctx.dispatch_programmatic_event(
                    &self.string_id,
                    ProgrammaticElementEvent::MouseEnter,
                );
            }
            ElementEvent::MouseLeave => {
                ctx.dispatch_programmatic_event(
                    &self.string_id,
                    ProgrammaticElementEvent::MouseLeave,
                );
            }
            ElementEvent::Focus => self.focus(),
            ElementEvent::Blur => self.blur(),
            ElementEvent::KeyDown { key, modifiers } => {
                ctx.dispatch_programmatic_event(
                    &self.string_id,
                    ProgrammaticElementEvent::KeyDown { key, modifiers },
                );
            }
            ElementEvent::Custom(event_type) => {
                ctx.dispatch_programmatic_event(
                    &self.string_id,
                    ProgrammaticElementEvent::Custom(event_type),
                );
            }
        }
    }

    // =========================================================================
    // On-Ready Callback
    // =========================================================================

    /// Register an on_ready callback for this element
    ///
    /// The callback will be invoked once after the element's first successful
    /// layout computation. The callback receives the element's computed bounds.
    ///
    /// This works even if the element doesn't exist yet - the callback will
    /// fire when the element is first laid out. If the element already exists
    /// and has been laid out, the callback fires on the next layout pass.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Query element and register callback
    /// ctx.query("progress-bar").on_ready(|bounds| {
    ///     progress_anim.lock().unwrap().set_target(bounds.width * 0.75);
    /// });
    /// ```
    pub fn on_ready<F>(&self, callback: F)
    where
        F: Fn(ElementBounds) + Send + Sync + 'static,
    {
        self.registry
            .register_on_ready_for_id(&self.string_id, Arc::new(callback));
    }

    /// Register an on_ready callback (Arc version for shared callbacks)
    pub fn on_ready_arc(&self, callback: OnReadyCallback) {
        self.registry
            .register_on_ready_for_id(&self.string_id, callback);
    }
}

/// Events that can be programmatically dispatched to elements
#[derive(Debug, Clone)]
pub enum ElementEvent {
    /// Mouse click at local coordinates
    Click { x: f32, y: f32 },
    /// Mouse entered element bounds
    MouseEnter,
    /// Mouse left element bounds
    MouseLeave,
    /// Element received focus
    Focus,
    /// Element lost focus
    Blur,
    /// Key pressed while focused
    KeyDown {
        key: u32,      // Key code
        modifiers: u8, // Modifier flags
    },
    /// Custom user-defined event
    Custom(u32),
}

/// Trait for elements that can be queried by type
///
/// Implement this for your element types to enable typed queries:
/// ```rust,ignore
/// ctx.query::<Image>("my-image")
/// ```
pub trait Queryable: Sized {
    /// Try to extract this type from an element handle
    fn from_handle(handle: &ElementHandle<()>) -> Option<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Mutex, Once};

    use blinc_core::context_state::{HookState, SharedHookState};
    use blinc_core::reactive::ReactiveGraph;

    use crate::prelude::*;

    #[test]
    fn test_handle_creation() {
        let registry = Arc::new(ElementRegistry::new());
        let node_id = LayoutNodeId::default();

        registry.register("test", node_id);

        let handle: ElementHandle<()> = ElementHandle::new("test", registry);
        assert_eq!(handle.node_id(), node_id);
        assert_eq!(handle.id(), "test");
        assert!(handle.exists());
    }

    #[test]
    fn test_handle_for_nonexistent_element() {
        let registry = Arc::new(ElementRegistry::new());

        // Handle can be created for element that doesn't exist yet
        let handle: ElementHandle<()> = ElementHandle::new("future-element", registry);
        assert_eq!(handle.id(), "future-element");
        assert!(!handle.exists());
    }

    #[test]
    fn test_parent_traversal() {
        let registry = Arc::new(ElementRegistry::new());
        let parent_id = LayoutNodeId::default();
        let child_id = LayoutNodeId::default();

        registry.register("parent", parent_id);
        registry.register("child", child_id);
        registry.register_parent(child_id, parent_id);

        let child_handle: ElementHandle<()> = ElementHandle::new("child", registry);
        let parent = child_handle.parent();

        assert!(parent.is_some());
        // Note: In real usage with distinct IDs this would work properly
    }

    // =========================================================================
    // On-Ready Callback Tests
    // =========================================================================

    #[test]
    fn test_handle_on_ready_registers_callback() {
        let registry = Arc::new(ElementRegistry::new());

        // Create handle for element that doesn't exist yet
        let handle: ElementHandle<()> = ElementHandle::new("my-element", registry.clone());

        // Register on_ready callback
        handle.on_ready(|_bounds| {
            // Callback logic here
        });

        // Should have pending callback
        assert!(registry.has_pending_on_ready());

        let pending = registry.take_pending_on_ready();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].0, "my-element");
    }

    #[test]
    fn test_handle_on_ready_uses_string_id() {
        let registry = Arc::new(ElementRegistry::new());
        let node_id = LayoutNodeId::default();

        // Register element
        registry.register("progress-bar", node_id);

        // Create handle and register callback
        let handle: ElementHandle<()> = ElementHandle::new("progress-bar", registry.clone());
        handle.on_ready(|_| {});

        // Callback should be registered with string ID
        let pending = registry.take_pending_on_ready();
        assert_eq!(pending[0].0, "progress-bar");
    }

    #[test]
    fn test_handle_on_ready_skips_if_already_triggered() {
        let registry = Arc::new(ElementRegistry::new());

        // Mark as already triggered
        registry.mark_on_ready_triggered("my-element");

        // Create handle and try to register callback
        let handle: ElementHandle<()> = ElementHandle::new("my-element", registry.clone());
        handle.on_ready(|_| {});

        // Should NOT have pending callback
        assert!(!registry.has_pending_on_ready());
    }

    #[test]
    fn test_handle_on_ready_works_before_element_exists() {
        let registry = Arc::new(ElementRegistry::new());

        // Create handle for nonexistent element
        let handle: ElementHandle<()> = ElementHandle::new("future-element", registry.clone());
        assert!(!handle.exists());

        // Register callback anyway
        handle.on_ready(|_| {});

        // Callback should be pending
        assert!(registry.has_pending_on_ready());

        let pending = registry.take_pending_on_ready();
        assert_eq!(pending[0].0, "future-element");
    }

    #[test]
    fn test_handle_on_ready_arc() {
        let registry = Arc::new(ElementRegistry::new());
        let handle: ElementHandle<()> = ElementHandle::new("my-element", registry.clone());

        // Use Arc version for shared callback
        let callback: super::OnReadyCallback = Arc::new(|_| {});
        handle.on_ready_arc(callback);

        assert!(registry.has_pending_on_ready());
    }

    fn selector_test_guard() -> std::sync::MutexGuard<'static, ()> {
        static TEST_LOCK: Mutex<()> = Mutex::new(());
        static INIT: Once = Once::new();

        let guard = TEST_LOCK.lock().unwrap();
        INIT.call_once(|| {
            blinc_theme::ThemeState::init_default();
            if !BlincContextState::is_initialized() {
                let reactive = Arc::new(Mutex::new(ReactiveGraph::new()));
                let hooks: SharedHookState = Arc::new(Mutex::new(HookState::new()));
                let dirty = Arc::new(std::sync::atomic::AtomicBool::new(false));
                BlincContextState::init_with_callback(reactive, hooks, dirty, Arc::new(|_| {}));
            }
        });
        guard
    }

    #[test]
    fn element_handle_click_dispatches_into_runtime_event_path() {
        let _guard = selector_test_guard();
        let click_count = Arc::new(AtomicUsize::new(0));
        let click_count_for_handler = Arc::clone(&click_count);

        let ui = div()
            .id("submit")
            .w(120.0)
            .h(44.0)
            .on_click(move |_| {
                click_count_for_handler.fetch_add(1, Ordering::SeqCst);
            })
            .child(text("Submit"));

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(200.0, 120.0);
        let mut router = EventRouter::new();
        let pending_events = Arc::new(Mutex::new(Vec::<(String, ProgrammaticElementEvent)>::new()));

        let pending_events_for_callback = Arc::clone(&pending_events);
        BlincContextState::get().set_programmatic_event_callback(Arc::new(move |id, event| {
            pending_events_for_callback
                .lock()
                .unwrap()
                .push((id.to_string(), event));
        }));

        let registry = Arc::clone(tree.element_registry());
        let handle: ElementHandle<()> = ElementHandle::new("submit", registry);
        handle.click();

        for (target_id, event) in pending_events.lock().unwrap().drain(..) {
            dispatch_programmatic_event_to_runtime(&mut tree, &mut router, &target_id, event);
        }

        assert_eq!(click_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn element_handle_focus_updates_context_focus_state() {
        let _guard = selector_test_guard();

        let ui = div().id("field").w(120.0).h(32.0).child(text("Field"));
        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(200.0, 120.0);
        let mut router = EventRouter::new();
        let pending_focus = Arc::new(Mutex::new(Vec::<Option<String>>::new()));

        let pending_focus_for_callback = Arc::clone(&pending_focus);
        BlincContextState::get().set_focus_callback(Arc::new(move |id| {
            pending_focus_for_callback
                .lock()
                .unwrap()
                .push(id.map(str::to_string));
        }));

        let registry = Arc::clone(tree.element_registry());
        let handle: ElementHandle<()> = ElementHandle::new("field", registry);
        handle.focus();

        assert_eq!(
            BlincContextState::get().focused_element().as_deref(),
            Some("field")
        );
        for target_id in pending_focus.lock().unwrap().drain(..) {
            sync_focus_to_runtime(&mut tree, &mut router, target_id.as_deref());
        }
        let focused_node = tree.query_by_id("field").expect("field should exist");
        assert_eq!(router.focused(), Some(focused_node));

        handle.blur();

        assert_eq!(BlincContextState::get().focused_element(), None);
        for target_id in pending_focus.lock().unwrap().drain(..) {
            sync_focus_to_runtime(&mut tree, &mut router, target_id.as_deref());
        }
        assert_eq!(router.focused(), None);
    }

    #[test]
    fn sync_focus_runtime_dispatches_focus_and_blur_handlers() {
        let _guard = selector_test_guard();

        let focus_hits = Arc::new(AtomicUsize::new(0));
        let blur_hits = Arc::new(AtomicUsize::new(0));
        let focus_hits_for_handler = Arc::clone(&focus_hits);
        let blur_hits_for_handler = Arc::clone(&blur_hits);

        let ui = div()
            .child(
                div()
                    .id("field")
                    .w(120.0)
                    .h(32.0)
                    .on_focus(move |_| {
                        focus_hits_for_handler.fetch_add(1, Ordering::SeqCst);
                    })
                    .on_blur(move |_| {
                        blur_hits_for_handler.fetch_add(1, Ordering::SeqCst);
                    })
                    .child(text("Field")),
            )
            .child(div().id("other").w(120.0).h(32.0).child(text("Other")));
        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(240.0, 160.0);
        let mut router = EventRouter::new();
        let pending_focus = Arc::new(Mutex::new(Vec::<Option<String>>::new()));

        let pending_focus_for_callback = Arc::clone(&pending_focus);
        BlincContextState::get().set_focus_callback(Arc::new(move |id| {
            pending_focus_for_callback
                .lock()
                .unwrap()
                .push(id.map(str::to_string));
        }));

        BlincContextState::get().set_focus(Some("field"));
        for target_id in pending_focus.lock().unwrap().drain(..) {
            sync_focus_to_runtime(&mut tree, &mut router, target_id.as_deref());
        }
        assert_eq!(focus_hits.load(Ordering::SeqCst), 1);
        assert_eq!(blur_hits.load(Ordering::SeqCst), 0);

        BlincContextState::get().set_focus(Some("other"));
        for target_id in pending_focus.lock().unwrap().drain(..) {
            sync_focus_to_runtime(&mut tree, &mut router, target_id.as_deref());
        }
        assert_eq!(focus_hits.load(Ordering::SeqCst), 1);
        assert_eq!(blur_hits.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn sync_context_focus_from_runtime_tracks_router_focus() {
        let _guard = selector_test_guard();

        let ui = div().id("field").w(120.0).h(32.0).child(text("Field"));
        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(200.0, 120.0);
        let mut router = EventRouter::new();

        let field = tree.query_by_id("field").expect("field should exist");
        router.set_focus(Some(field));
        sync_context_focus_from_runtime(&tree, &router);
        assert_eq!(
            BlincContextState::get().focused_element().as_deref(),
            Some("field")
        );

        router.set_focus(None);
        sync_context_focus_from_runtime(&tree, &router);
        assert_eq!(BlincContextState::get().focused_element(), None);
    }

    #[test]
    fn scroll_into_view_with_preserves_requested_options() {
        let _guard = selector_test_guard();
        let captured = Arc::new(Mutex::new(None::<(String, ScrollIntoViewOptions)>));
        let captured_for_callback = Arc::clone(&captured);
        BlincContextState::get().set_scroll_callback(Arc::new(move |id, options| {
            *captured_for_callback
                .lock()
                .expect("scroll capture should lock") = Some((id.to_string(), options));
        }));

        let registry = Arc::new(ElementRegistry::new());
        let handle: ElementHandle<()> = ElementHandle::new("item-24", registry);
        handle.scroll_into_view_with(ScrollOptions {
            behavior: ScrollBehavior::Smooth,
            block: ScrollBlock::Center,
            inline: ScrollInline::End,
        });

        let captured = captured.lock().expect("scroll capture should lock");
        let (id, options) = captured
            .as_ref()
            .expect("scroll request should be captured");
        assert_eq!(id, "item-24");
        assert_eq!(
            *options,
            ScrollIntoViewOptions {
                behavior: blinc_core::ScrollBehaviorHint::Smooth,
                block: blinc_core::ScrollBlockHint::Center,
                inline: blinc_core::ScrollInlineHint::End,
            }
        );
    }

    #[test]
    fn element_handle_focus_can_activate_text_input() {
        let _guard = selector_test_guard();

        let email = text_input_state_with_placeholder("Email");
        let ui = div().child(text_input(&email).id("login.email").w(240.0));
        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(320.0, 120.0);
        let mut router = EventRouter::new();
        let pending_focus = Arc::new(Mutex::new(Vec::<Option<String>>::new()));

        let pending_focus_for_callback = Arc::clone(&pending_focus);
        BlincContextState::get().set_focus_callback(Arc::new(move |id| {
            pending_focus_for_callback
                .lock()
                .unwrap()
                .push(id.map(str::to_string));
        }));

        let registry = Arc::clone(tree.element_registry());
        let handle: ElementHandle<()> = ElementHandle::new("login.email", registry);
        handle.focus();

        for target_id in pending_focus.lock().unwrap().drain(..) {
            sync_focus_to_runtime(&mut tree, &mut router, target_id.as_deref());
        }

        tree.broadcast_text_input_event('a', false, false, false, false);
        assert_eq!(email.lock().unwrap().value, "a");
    }
}

// =============================================================================
// MotionHandle - Handle for querying motion animation state
// =============================================================================

/// Handle to a motion animation for querying its state
///
/// Returned by `query_motion("motion-key")` for animation state queries.
/// Use this to check if a parent motion animation has settled before
/// rendering child content with hover effects, etc.
///
/// # Example
///
/// ```ignore
/// use blinc_layout::selector::query_motion;
///
/// // Inside a Stateful on_state callback:
/// let motion = query_motion("dialog-content");
/// if motion.is_settled() {
///     // Safe to render hover effects
///     container.merge(button_with_hover());
/// } else {
///     // Render without hover effects during animation
///     container.merge(button_static());
/// }
/// ```
#[derive(Clone, Debug)]
pub struct MotionHandle {
    /// The stable key used to query this motion
    key: String,
    /// Current animation state
    state: MotionAnimationState,
}

impl MotionHandle {
    /// Create a new motion handle from a stable key
    ///
    /// Queries the current animation state via `BlincContextState`.
    pub fn new(key: impl Into<String>) -> Self {
        let key = key.into();
        let state = BlincContextState::try_get()
            .map(|ctx| ctx.query_motion(&key))
            .unwrap_or(MotionAnimationState::NotFound);
        Self { key, state }
    }

    /// Get the stable key for this motion
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the current animation state
    pub fn state(&self) -> MotionAnimationState {
        self.state
    }

    /// Check if the animation is still playing (not settled)
    ///
    /// Returns true if the motion is in `Suspended`, `Waiting`, `Entering`, or `Exiting` state.
    pub fn is_animating(&self) -> bool {
        self.state.is_animating()
    }

    /// Check if the animation has settled (fully visible)
    ///
    /// Returns true if the motion is in `Visible` state.
    /// This is when it's safe to render child content with hover effects.
    pub fn is_settled(&self) -> bool {
        self.state.is_settled()
    }

    /// Check if the motion is suspended (waiting for explicit start)
    ///
    /// A suspended motion is mounted with opacity 0 and waits for `start()` to be called.
    pub fn is_suspended(&self) -> bool {
        self.state.is_suspended()
    }

    /// Check if the element is entering
    pub fn is_entering(&self) -> bool {
        self.state.is_entering()
    }

    /// Check if the element is exiting
    pub fn is_exiting(&self) -> bool {
        self.state.is_exiting()
    }

    /// Get the animation progress (0.0 to 1.0)
    ///
    /// Returns 0.0 for Suspended/Waiting, 1.0 for Visible/Removed, and the actual
    /// progress for Entering/Exiting states.
    pub fn progress(&self) -> f32 {
        self.state.progress()
    }

    /// Check if a motion with this key exists
    pub fn exists(&self) -> bool {
        !matches!(self.state, MotionAnimationState::NotFound)
    }

    /// Start the enter animation for a suspended motion
    ///
    /// Use this to explicitly trigger the enter animation for a motion that was
    /// created with `.suspended()`. The motion transitions from `Suspended` →
    /// `Waiting` or `Entering` state.
    ///
    /// This is useful for tab transitions and other cases where you want to:
    /// 1. Mount the content invisibly (opacity 0)
    /// 2. Perform any setup/measurement
    /// 3. Then trigger the animation manually
    ///
    /// No-op if the motion is not in `Suspended` state.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // In tabs.rs on_state callback:
    /// let motion_key = format!("tabs_motion:{}", active_tab);
    ///
    /// // Create suspended motion first
    /// let m = motion_derived(&motion_key)
    ///     .suspended()
    ///     .enter_animation(enter)
    ///     .child(content);
    ///
    /// // Then trigger the animation after mounting
    /// query_motion(&motion_key).start();
    /// ```
    pub fn start(&self) {
        // Queue the start to be processed during the next render frame
        crate::queue_global_motion_start(self.key.clone());
    }

    /// Cancel the exit animation and return the motion to Visible state
    ///
    /// Used when an overlay's close is cancelled (e.g., mouse re-enters hover card).
    /// This interrupts the exit animation and immediately sets the motion to fully visible.
    ///
    /// No-op if the motion is not in Exiting state.
    pub fn cancel_exit(&self) {
        // Queue the cancellation to be processed during the next render frame
        crate::queue_global_motion_exit_cancel(self.key.clone());
    }

    /// Trigger the exit animation for this motion
    ///
    /// Used to explicitly trigger the exit animation (e.g., when a hover card
    /// close countdown completes). This transitions the motion from Visible → Exiting.
    ///
    /// No-op if the motion is not in Visible state.
    pub fn exit(&self) {
        // Queue the exit to be processed during the next render frame
        crate::queue_global_motion_exit_start(self.key.clone());
    }
}
