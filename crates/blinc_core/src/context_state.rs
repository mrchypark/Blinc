//! Global context state singleton
//!
//! BlincContextState provides a global singleton for accessing reactive state management
//! and other context-level resources without requiring explicit context parameters.
//!
//! This enables components to create internal state without leaking implementation details:
//!
//! ```ignore
//! // Before: user must manage internal component state
//! let fruit_open = ctx.use_state_keyed("fruit_open", || false);
//! cn::select(&fruit, &fruit_open)
//!
//! // After: component manages internal state via singleton
//! cn::select(&fruit)  // open_state is internal to the component
//! ```
//!
//! # Initialization
//!
//! The singleton must be initialized by the app layer before use:
//!
//! ```ignore
//! // In WindowedApp::run()
//! BlincContextState::init(reactive, hooks, dirty_flag);
//! ```
//!
//! # Usage
//!
//! Components can access state management via free functions:
//!
//! ```ignore
//! use blinc_core::context_state::{use_state_keyed, use_signal_keyed};
//!
//! // In a component:
//! let open_state = use_state_keyed("my_component_open", || false);
//! ```

use crate::reactive::{ReactiveGraph, Signal, SignalId, State};
use std::any::{type_name, Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

/// Global context state instance
static CONTEXT_STATE: OnceLock<BlincContextState> = OnceLock::new();
thread_local! {
    static CONTEXT_RESOURCE_OVERRIDE: RefCell<Option<ContextResourceOverride>> = const { RefCell::new(None) };
    static CONTEXT_BINDING_OVERRIDE: RefCell<Option<ContextBindingOverride>> = const { RefCell::new(None) };
}

/// Shared reactive graph for thread-safe access
pub type SharedReactiveGraph = Arc<Mutex<ReactiveGraph>>;

/// Shared dirty flag for triggering UI rebuilds
pub type DirtyFlag = Arc<AtomicBool>;

/// Key for identifying a signal in the keyed state system
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StateKey {
    /// Hash of the user-provided key
    key_hash: u64,
    /// Type ID of the signal value
    type_id: TypeId,
}

impl StateKey {
    /// Create a new StateKey from a hashable key and type
    pub fn new<T: 'static, K: Hash>(key: &K) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        Self {
            key_hash: hasher.finish(),
            type_id: TypeId::of::<T>(),
        }
    }

    /// Create a StateKey from a string key and type
    pub fn from_string<T: 'static>(key: &str) -> Self {
        Self::new::<T, _>(&key)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HookDebugRegistration {
    signal_id: u64,
    key: String,
    type_name: &'static str,
}

/// Debug-facing keyed state inventory entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedStateDebugEntry {
    pub key: String,
    pub type_name: String,
    pub value_summary: String,
}

/// Stores keyed state across rebuilds
///
/// This enables component-level state management where each signal
/// is identified by a unique string key rather than call order.
pub struct HookState {
    /// Keyed signals: key -> raw signal ID
    signals: HashMap<StateKey, HookDebugRegistration>,
}

impl Default for HookState {
    fn default() -> Self {
        Self::new()
    }
}

impl HookState {
    /// Create a new HookState
    pub fn new() -> Self {
        Self {
            signals: HashMap::new(),
        }
    }

    /// Get an existing signal by key
    pub fn get(&self, key: &StateKey) -> Option<u64> {
        self.signals.get(key).map(|entry| entry.signal_id)
    }

    /// Store a signal with the given key
    pub fn insert<T: 'static>(
        &mut self,
        key: StateKey,
        debug_key: impl Into<String>,
        signal_id: u64,
    ) {
        self.signals.insert(
            key,
            HookDebugRegistration {
                signal_id,
                key: debug_key.into(),
                type_name: type_name::<T>(),
            },
        );
    }

    /// Store a signal whose originating key is not directly printable.
    pub fn insert_opaque<T: 'static>(&mut self, key: StateKey, signal_id: u64) {
        let debug_key = format!("#{:016x}", key.key_hash);
        self.insert::<T>(key, debug_key, signal_id);
    }

    fn debug_registrations(&self) -> Vec<HookDebugRegistration> {
        let mut entries = self.signals.values().cloned().collect::<Vec<_>>();
        entries.sort_by(|left, right| left.key.cmp(&right.key));
        entries
    }

    pub fn clear(&mut self) {
        self.signals.clear();
    }
}

/// Shared hook state for the application
pub type SharedHookState = Arc<Mutex<HookState>>;

/// Callback for notifying stateful elements of signal changes
pub type StatefulCallback = Arc<dyn Fn(&[SignalId]) + Send + Sync>;

/// Callback for querying elements by ID
/// Returns the raw node ID (u64) if found, None otherwise
pub type QueryCallback = Arc<dyn Fn(&str) -> Option<u64> + Send + Sync>;

/// Simple bounds representation for element queries
/// Used by BlincContextState to avoid circular dependencies with blinc_layout
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bounds {
    /// X position (absolute, after layout)
    pub x: f32,
    /// Y position (absolute, after layout)
    pub y: f32,
    /// Computed width
    pub width: f32,
    /// Computed height
    pub height: f32,
}

impl Bounds {
    /// Create new bounds
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Check if a point is inside the bounds
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }

    /// Check if bounds intersect with another bounds
    pub fn intersects(&self, other: &Bounds) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

/// Callback for getting element bounds by string ID
pub type BoundsCallback = Arc<dyn Fn(&str) -> Option<Bounds> + Send + Sync>;

/// Callback for focus management
/// Called with Some(id) to focus an element, None to clear focus
pub type FocusCallback = Arc<dyn Fn(Option<&str>) + Send + Sync>;

/// Core-owned scroll behavior hint for query-driven scroll-into-view requests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollBehaviorHint {
    /// Instant scroll (no animation).
    #[default]
    Auto,
    /// Smooth animated scroll.
    Smooth,
}

/// Core-owned vertical alignment hint for scroll-into-view requests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollBlockHint {
    /// Align target to the top of the viewport.
    Start,
    /// Align target to the center of the viewport.
    Center,
    /// Align target to the bottom of the viewport.
    End,
    /// Scroll the minimum amount needed to make the target visible.
    #[default]
    Nearest,
}

/// Core-owned horizontal alignment hint for scroll-into-view requests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollInlineHint {
    /// Align target to the left edge of the viewport.
    Start,
    /// Align target to the center of the viewport.
    Center,
    /// Align target to the right edge of the viewport.
    End,
    /// Scroll the minimum amount needed to make the target visible.
    #[default]
    Nearest,
}

/// Scroll-into-view request options carried through `BlincContextState`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollIntoViewOptions {
    pub behavior: ScrollBehaviorHint,
    pub block: ScrollBlockHint,
    pub inline: ScrollInlineHint,
}

/// Callback for scrolling an element into view
pub type ScrollCallback = Arc<dyn Fn(&str, ScrollIntoViewOptions) + Send + Sync>;

/// Programmatic element event dispatched through the existing runtime path.
#[derive(Clone, Debug, PartialEq)]
pub enum ProgrammaticElementEvent {
    /// Mouse click at local coordinates within the target element.
    Click { x: f32, y: f32 },
    /// Pointer enters the target element.
    MouseEnter,
    /// Pointer leaves the current hovered chain.
    MouseLeave,
    /// Key pressed while the target element is focused.
    KeyDown { key: u32, modifiers: u8 },
    /// Key released while the target element is focused.
    KeyUp { key: u32, modifiers: u8 },
    /// Text input routed through the focused element.
    TextInput { text: char, modifiers: u8 },
    /// Scroll delta routed through the target hit-test chain.
    Scroll { dx: f32, dy: f32 },
    /// Custom user-defined event type.
    Custom(u32),
}

/// Callback for programmatic element events triggered via ElementHandle APIs.
pub type ProgrammaticEventCallback = Arc<dyn Fn(&str, ProgrammaticElementEvent) + Send + Sync>;

/// Motion animation state for query API
///
/// Represents the current state of a motion animation.
/// Used by MotionHandle to query animation progress.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MotionAnimationState {
    /// Animation is suspended (waiting for explicit start)
    /// The motion is mounted with opacity 0, waiting for `MotionHandle.start()` to trigger
    Suspended,
    /// Animation hasn't started yet (waiting for delay)
    Waiting,
    /// Element is entering (fade-in, scale-in, etc.)
    Entering {
        /// Animation progress from 0.0 to 1.0
        progress: f32,
    },
    /// Element is fully visible (animation complete)
    Visible,
    /// Element is exiting (fade-out, scale-out, etc.)
    Exiting {
        /// Animation progress from 0.0 to 1.0
        progress: f32,
    },
    /// Element has been removed (exit animation complete)
    Removed,
    /// No motion animation found for this key
    NotFound,
}

impl MotionAnimationState {
    /// Check if the animation is still playing (not settled)
    ///
    /// Returns true for `Waiting`, `Entering`, `Exiting`, or `Suspended` states.
    /// Suspended is considered "animating" because the motion is waiting to start.
    pub fn is_animating(&self) -> bool {
        matches!(
            self,
            MotionAnimationState::Suspended
                | MotionAnimationState::Waiting
                | MotionAnimationState::Entering { .. }
                | MotionAnimationState::Exiting { .. }
        )
    }

    /// Check if the animation has settled (fully visible)
    pub fn is_settled(&self) -> bool {
        matches!(self, MotionAnimationState::Visible)
    }

    /// Check if the motion is suspended (waiting for explicit start)
    pub fn is_suspended(&self) -> bool {
        matches!(self, MotionAnimationState::Suspended)
    }

    /// Check if the element is entering
    pub fn is_entering(&self) -> bool {
        matches!(self, MotionAnimationState::Entering { .. })
    }

    /// Check if the element is exiting
    pub fn is_exiting(&self) -> bool {
        matches!(self, MotionAnimationState::Exiting { .. })
    }

    /// Get the animation progress (0.0 to 1.0)
    ///
    /// Returns 0.0 for Suspended/Waiting, 1.0 for Visible/Removed, and the actual
    /// progress for Entering/Exiting states.
    pub fn progress(&self) -> f32 {
        match self {
            MotionAnimationState::Suspended => 0.0,
            MotionAnimationState::Waiting => 0.0,
            MotionAnimationState::Entering { progress } => *progress,
            MotionAnimationState::Visible => 1.0,
            MotionAnimationState::Exiting { progress } => *progress,
            MotionAnimationState::Removed => 1.0,
            MotionAnimationState::NotFound => 1.0, // Treat as settled if not found
        }
    }
}

/// Callback for querying motion animation state by stable key
pub type MotionStateCallback = Arc<dyn Fn(&str) -> MotionAnimationState + Send + Sync>;

/// Callback for canceling a motion's exit animation
pub type MotionCancelExitCallback = Arc<dyn Fn(&str) + Send + Sync>;

#[derive(Clone, Default)]
pub struct ContextBindingOverride {
    query_callback: Option<QueryCallback>,
    bounds_callback: Option<BoundsCallback>,
    focus_callback: Option<FocusCallback>,
    scroll_callback: Option<ScrollCallback>,
    programmatic_event_callback: Option<ProgrammaticEventCallback>,
    viewport_size: Option<(f32, f32)>,
    focused_element: Option<String>,
    element_registry: Option<AnyElementRegistry>,
    motion_state_callback: Option<MotionStateCallback>,
    motion_cancel_exit_callback: Option<MotionCancelExitCallback>,
}

/// Temporary reactive resources that can override the default process-wide
/// state for scoped runtimes such as automation sessions.
#[derive(Clone)]
pub struct ContextResourceOverride {
    reactive: SharedReactiveGraph,
    hooks: SharedHookState,
    dirty_flag: DirtyFlag,
}

impl ContextResourceOverride {
    pub fn new(
        reactive: SharedReactiveGraph,
        hooks: SharedHookState,
        dirty_flag: DirtyFlag,
    ) -> Self {
        Self {
            reactive,
            hooks,
            dirty_flag,
        }
    }

    pub fn reactive(&self) -> SharedReactiveGraph {
        Arc::clone(&self.reactive)
    }

    pub fn hooks(&self) -> SharedHookState {
        Arc::clone(&self.hooks)
    }

    pub fn dirty_flag(&self) -> DirtyFlag {
        Arc::clone(&self.dirty_flag)
    }
}

// =========================================================================
// Recorder Callbacks (for blinc_recorder integration)
// =========================================================================

/// Type-erased recorded event for recorder callbacks
/// This avoids circular dependencies by using a boxed Any type
pub type RecordedEventAny = Box<dyn Any + Send>;

/// Callback for recording events (mouse, keyboard, scroll, etc.)
/// Events are passed as type-erased Any to avoid circular dependencies
pub type RecorderEventCallback = Arc<dyn Fn(RecordedEventAny) + Send + Sync>;

/// Type-erased tree snapshot for recorder callbacks
pub type TreeSnapshotAny = Box<dyn Any + Send>;

/// Callback for capturing tree snapshots after each frame
pub type RecorderSnapshotCallback = Arc<dyn Fn(TreeSnapshotAny) + Send + Sync>;

/// Update category for element change tracking
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateCategory {
    /// Visual-only change (color, opacity, etc.)
    Visual,
    /// Layout change (position, size)
    Layout,
    /// Structural change (children added/removed)
    Structural,
}

/// Callback for tracking element updates with category
pub type RecorderUpdateCallback = Arc<dyn Fn(&str, UpdateCategory) + Send + Sync>;

/// Type-erased element registry storage
/// This allows blinc_core to store the registry without depending on blinc_layout
pub type AnyElementRegistry = Arc<dyn Any + Send + Sync>;

/// Global context state singleton
///
/// Provides access to reactive state management and other context-level
/// resources without requiring explicit context parameters.
///
/// This follows the same OnceLock pattern as ThemeState.
pub struct BlincContextState {
    /// Reactive graph for signal-based state management
    reactive: SharedReactiveGraph,
    /// Hook state for keyed signal persistence
    hooks: SharedHookState,
    /// Dirty flag for triggering UI rebuilds
    dirty_flag: DirtyFlag,
    /// Optional callback for notifying stateful elements of signal changes
    stateful_callback: Option<StatefulCallback>,
    /// Optional callback for querying elements by ID
    query_callback: RwLock<Option<QueryCallback>>,

    // =========================================================================
    // ElementHandle Callbacks (set by WindowedApp)
    // =========================================================================
    /// Callback for getting element bounds by string ID
    bounds_callback: RwLock<Option<BoundsCallback>>,
    /// Callback for focus management
    focus_callback: RwLock<Option<FocusCallback>>,
    /// Callback for scrolling elements into view
    scroll_callback: RwLock<Option<ScrollCallback>>,
    /// Callback for programmatic element interactions
    programmatic_event_callback: RwLock<Option<ProgrammaticEventCallback>>,
    /// Current viewport size (width, height)
    viewport_size: RwLock<(f32, f32)>,
    /// Currently focused element ID
    focused_element: RwLock<Option<String>>,
    /// Type-erased element registry for query API
    /// Stored as `AnyElementRegistry` to avoid circular dependency with blinc_layout
    element_registry: RwLock<Option<AnyElementRegistry>>,
    /// Callback for querying motion animation state by stable key
    motion_state_callback: RwLock<Option<MotionStateCallback>>,
    /// Callback for canceling a motion's exit animation
    motion_cancel_exit_callback: RwLock<Option<MotionCancelExitCallback>>,

    // =========================================================================
    // Recorder Callbacks (for blinc_recorder integration)
    // =========================================================================
    /// Callback for recording events
    recorder_event_callback: RwLock<Option<RecorderEventCallback>>,
    /// Callback for capturing tree snapshots
    recorder_snapshot_callback: RwLock<Option<RecorderSnapshotCallback>>,
    /// Callback for tracking element updates with category
    recorder_update_callback: RwLock<Option<RecorderUpdateCallback>>,
}

impl BlincContextState {
    /// Initialize the global context state (call once at app startup)
    ///
    /// # Panics
    ///
    /// Panics if called more than once.
    pub fn init(reactive: SharedReactiveGraph, hooks: SharedHookState, dirty_flag: DirtyFlag) {
        let state = BlincContextState {
            reactive,
            hooks,
            dirty_flag,
            stateful_callback: None,
            query_callback: RwLock::new(None),
            bounds_callback: RwLock::new(None),
            focus_callback: RwLock::new(None),
            scroll_callback: RwLock::new(None),
            programmatic_event_callback: RwLock::new(None),
            viewport_size: RwLock::new((0.0, 0.0)),
            focused_element: RwLock::new(None),
            element_registry: RwLock::new(None),
            motion_state_callback: RwLock::new(None),
            motion_cancel_exit_callback: RwLock::new(None),
            recorder_event_callback: RwLock::new(None),
            recorder_snapshot_callback: RwLock::new(None),
            recorder_update_callback: RwLock::new(None),
        };

        if CONTEXT_STATE.set(state).is_err() {
            panic!("BlincContextState::init() called more than once");
        }
    }

    /// Initialize with a stateful callback for notifying elements of signal changes
    pub fn init_with_callback(
        reactive: SharedReactiveGraph,
        hooks: SharedHookState,
        dirty_flag: DirtyFlag,
        callback: StatefulCallback,
    ) {
        let state = BlincContextState {
            reactive,
            hooks,
            dirty_flag,
            stateful_callback: Some(callback),
            query_callback: RwLock::new(None),
            bounds_callback: RwLock::new(None),
            focus_callback: RwLock::new(None),
            scroll_callback: RwLock::new(None),
            programmatic_event_callback: RwLock::new(None),
            viewport_size: RwLock::new((0.0, 0.0)),
            focused_element: RwLock::new(None),
            element_registry: RwLock::new(None),
            motion_state_callback: RwLock::new(None),
            motion_cancel_exit_callback: RwLock::new(None),
            recorder_event_callback: RwLock::new(None),
            recorder_snapshot_callback: RwLock::new(None),
            recorder_update_callback: RwLock::new(None),
        };

        if CONTEXT_STATE.set(state).is_err() {
            panic!("BlincContextState::init() called more than once");
        }
    }

    /// Get the global context state instance
    ///
    /// # Panics
    ///
    /// Panics if `init()` has not been called.
    pub fn get() -> &'static BlincContextState {
        CONTEXT_STATE.get().expect(
            "BlincContextState not initialized. Call BlincContextState::init() at app startup.",
        )
    }

    /// Try to get the global context state (returns None if not initialized)
    pub fn try_get() -> Option<&'static BlincContextState> {
        CONTEXT_STATE.get()
    }

    /// Check if the context state has been initialized
    pub fn is_initialized() -> bool {
        CONTEXT_STATE.get().is_some()
    }

    /// Reset mutable singleton state and scoped overrides for test isolation.
    ///
    /// This preserves the OnceLock instance and any installed stateful callback,
    /// but clears reactive state, keyed hooks, runtime callbacks, recorder hooks,
    /// and thread-local overrides so tests start from a clean baseline.
    #[doc(hidden)]
    pub fn reseed_for_tests(&self) {
        self.reactive.lock().unwrap().clear();
        self.hooks.lock().unwrap().clear();
        self.dirty_flag.store(false, Ordering::SeqCst);

        *self.query_callback.write().unwrap() = None;
        *self.bounds_callback.write().unwrap() = None;
        *self.focus_callback.write().unwrap() = None;
        *self.scroll_callback.write().unwrap() = None;
        *self.programmatic_event_callback.write().unwrap() = None;
        *self.viewport_size.write().unwrap() = (0.0, 0.0);
        *self.focused_element.write().unwrap() = None;
        *self.element_registry.write().unwrap() = None;
        *self.motion_state_callback.write().unwrap() = None;
        *self.motion_cancel_exit_callback.write().unwrap() = None;
        *self.recorder_event_callback.write().unwrap() = None;
        *self.recorder_snapshot_callback.write().unwrap() = None;
        *self.recorder_update_callback.write().unwrap() = None;

        CONTEXT_RESOURCE_OVERRIDE.with(|override_slot| {
            if let Some(resources) = override_slot.borrow_mut().take() {
                resources.reactive().lock().unwrap().clear();
                resources.hooks().lock().unwrap().clear();
                resources.dirty_flag().store(false, Ordering::SeqCst);
            }
        });
        CONTEXT_BINDING_OVERRIDE.with(|override_slot| {
            override_slot.borrow_mut().take();
        });
    }

    // =========================================================================
    // Reactive State Management
    // =========================================================================

    fn active_resources(&self) -> ContextResourceOverride {
        CONTEXT_RESOURCE_OVERRIDE
            .with(|override_slot| override_slot.borrow().clone())
            .unwrap_or_else(|| {
                ContextResourceOverride::new(
                    Arc::clone(&self.reactive),
                    Arc::clone(&self.hooks),
                    Arc::clone(&self.dirty_flag),
                )
            })
    }

    fn with_binding_override<R>(&self, f: impl FnOnce(&ContextBindingOverride) -> R) -> Option<R> {
        let _ = self;
        CONTEXT_BINDING_OVERRIDE.with(|override_slot| override_slot.borrow().as_ref().map(f))
    }

    fn with_binding_override_mut<R>(
        &self,
        f: impl FnOnce(&mut ContextBindingOverride) -> R,
    ) -> Option<R> {
        let _ = self;
        CONTEXT_BINDING_OVERRIDE.with(|override_slot| override_slot.borrow_mut().as_mut().map(f))
    }

    /// Override the active resources used by the keyed-state APIs.
    pub fn set_resource_override(
        &self,
        resources: ContextResourceOverride,
    ) -> Option<ContextResourceOverride> {
        let _ = self;
        CONTEXT_RESOURCE_OVERRIDE
            .with(|override_slot| override_slot.borrow_mut().replace(resources))
    }

    /// Restore the previous scoped resource override.
    pub fn restore_resource_override(&self, resources: Option<ContextResourceOverride>) {
        let _ = self;
        CONTEXT_RESOURCE_OVERRIDE.with(|override_slot| {
            *override_slot.borrow_mut() = resources;
        });
    }

    /// Override the active query/focus/programmatic bindings for the current thread.
    pub fn set_binding_override(
        &self,
        bindings: ContextBindingOverride,
    ) -> Option<ContextBindingOverride> {
        let _ = self;
        CONTEXT_BINDING_OVERRIDE.with(|override_slot| override_slot.borrow_mut().replace(bindings))
    }

    /// Restore the previous scoped query/focus/programmatic bindings.
    pub fn restore_binding_override(&self, bindings: Option<ContextBindingOverride>) {
        let _ = self;
        CONTEXT_BINDING_OVERRIDE.with(|override_slot| {
            *override_slot.borrow_mut() = bindings;
        });
    }

    /// Return the currently active reactive graph, including any scoped
    /// override installed by automation.
    pub fn active_reactive(&self) -> SharedReactiveGraph {
        self.active_resources().reactive()
    }

    /// Return the currently active hook state, including any scoped override.
    pub fn active_hooks(&self) -> SharedHookState {
        self.active_resources().hooks()
    }

    /// Return the currently active dirty flag, including any scoped override.
    pub fn active_dirty_flag(&self) -> DirtyFlag {
        self.active_resources().dirty_flag()
    }

    /// Return a best-effort debug inventory of keyed state currently active
    /// for this thread's resources.
    pub fn debug_keyed_state_entries(&self) -> Vec<KeyedStateDebugEntry> {
        let registrations = {
            let hooks = self.active_hooks();
            let registrations = hooks.lock().unwrap().debug_registrations();
            registrations
        };

        let reactive = self.active_reactive();
        let graph = reactive.lock().unwrap();
        registrations
            .into_iter()
            .map(|entry| {
                let signal_id = SignalId::from_raw(entry.signal_id);
                let value_summary = match graph.debug_signal_summary(signal_id) {
                    Some(summary) => summary,
                    None if graph.has_signal(signal_id) => "<opaque>".to_string(),
                    None => "<stale>".to_string(),
                };

                KeyedStateDebugEntry {
                    key: entry.key,
                    type_name: entry.type_name.to_string(),
                    value_summary,
                }
            })
            .collect()
    }

    /// Create a persistent state value that survives across UI rebuilds (keyed)
    ///
    /// This creates component-level state identified by a unique string key.
    /// Returns a `State<T>` with direct `.get()` and `.set()` methods.
    pub fn use_state_keyed<T, F>(&self, key: &str, init: F) -> State<T>
    where
        T: Clone + Send + 'static,
        F: FnOnce() -> T,
    {
        let resources = self.active_resources();
        let state_key = StateKey::from_string::<T>(key);
        // IMPORTANT: Do not execute `init()` while holding internal locks.
        // Otherwise, `init()` may call back into keyed state APIs and deadlock.
        let existing_raw_id = { resources.hooks.lock().unwrap().get(&state_key) };

        let signal = if let Some(raw_id) = existing_raw_id {
            let signal_id = SignalId::from_raw(raw_id);
            Signal::from_id(signal_id)
        } else {
            let initial = init();
            let signal = resources.reactive.lock().unwrap().create_signal(initial);
            let raw_id = signal.id().to_raw();
            resources
                .hooks
                .lock()
                .unwrap()
                .insert::<T>(state_key, key, raw_id);
            signal
        };

        // Create State with or without stateful callback
        if let Some(ref callback) = self.stateful_callback {
            State::with_stateful_callback(
                signal,
                Arc::clone(&resources.reactive),
                Arc::clone(&resources.dirty_flag),
                Arc::clone(callback),
            )
        } else {
            State::new(
                signal,
                Arc::clone(&resources.reactive),
                Arc::clone(&resources.dirty_flag),
            )
        }
    }

    /// Create a persistent signal that survives across UI rebuilds (keyed)
    ///
    /// Unlike `use_signal()` which creates a new signal each call, this method
    /// persists the signal using a unique string key.
    pub fn use_signal_keyed<T, F>(&self, key: &str, init: F) -> Signal<T>
    where
        T: Clone + Send + 'static,
        F: FnOnce() -> T,
    {
        let resources = self.active_resources();
        let state_key = StateKey::from_string::<T>(key);
        // Same locking rule as `use_state_keyed`: run `init()` lock-free.
        let existing_raw_id = { resources.hooks.lock().unwrap().get(&state_key) };

        if let Some(raw_id) = existing_raw_id {
            let signal_id = SignalId::from_raw(raw_id);
            Signal::from_id(signal_id)
        } else {
            let initial = init();
            let signal = resources.reactive.lock().unwrap().create_signal(initial);
            let raw_id = signal.id().to_raw();
            resources
                .hooks
                .lock()
                .unwrap()
                .insert::<T>(state_key, key, raw_id);
            signal
        }
    }

    /// Create a new reactive signal with an initial value (low-level API)
    ///
    /// **Note**: Prefer `use_state_keyed` in most cases, as it automatically
    /// persists signals across rebuilds.
    pub fn use_signal<T: Send + 'static>(&self, initial: T) -> Signal<T> {
        self.active_reactive()
            .lock()
            .unwrap()
            .create_signal(initial)
    }

    /// Get the current value of a signal
    pub fn get_signal<T: Clone + 'static>(&self, signal: Signal<T>) -> Option<T> {
        self.active_reactive().lock().unwrap().get(signal)
    }

    /// Set the value of a signal, triggering reactive updates
    pub fn set_signal<T: Send + 'static>(&self, signal: Signal<T>, value: T) {
        self.active_reactive().lock().unwrap().set(signal, value);
    }

    /// Update a signal using a function
    pub fn update<T: Clone + Send + 'static, F: FnOnce(T) -> T>(&self, signal: Signal<T>, f: F) {
        let reactive = self.active_reactive();
        let mut graph = reactive.lock().unwrap();
        if let Some(current) = graph.get(signal) {
            graph.set(signal, f(current));
        }
    }

    // =========================================================================
    // Access to Internal Resources
    // =========================================================================

    /// Get the shared reactive graph
    pub fn reactive(&self) -> &SharedReactiveGraph {
        &self.reactive
    }

    /// Get the shared hook state
    pub fn hooks(&self) -> &SharedHookState {
        &self.hooks
    }

    /// Get the dirty flag
    pub fn dirty_flag(&self) -> &DirtyFlag {
        &self.dirty_flag
    }

    /// Request a UI rebuild by setting the dirty flag
    pub fn request_rebuild(&self) {
        self.active_dirty_flag().store(true, Ordering::SeqCst);
    }

    /// Notify stateful elements of signal changes
    ///
    /// This triggers only the stateful elements that depend on the given signals,
    /// causing targeted subtree rebuilds rather than a full UI rebuild.
    pub fn notify_stateful_deps(&self, signal_ids: &[SignalId]) {
        if let Some(ref callback) = self.stateful_callback {
            callback(signal_ids);
        }
    }

    // =========================================================================
    // Element Query System
    // =========================================================================

    /// Set the query callback for element lookup
    ///
    /// This is called by `WindowedApp` to enable element querying by ID.
    /// The callback receives an element ID and returns the raw node ID if found.
    pub fn set_query_callback(&self, callback: QueryCallback) {
        if self
            .with_binding_override_mut(|bindings| {
                bindings.query_callback = Some(Arc::clone(&callback));
            })
            .is_some()
        {
            return;
        }
        *self.query_callback.write().unwrap() = Some(callback);
    }

    /// Get the current query callback.
    pub fn query_callback(&self) -> Option<QueryCallback> {
        if let Some(callback) =
            self.with_binding_override(|bindings| bindings.query_callback.clone())
        {
            return callback;
        }
        self.query_callback.read().unwrap().clone()
    }

    /// Clear the query callback.
    pub fn clear_query_callback(&self) {
        if self
            .with_binding_override_mut(|bindings| {
                bindings.query_callback = None;
            })
            .is_some()
        {
            return;
        }
        *self.query_callback.write().unwrap() = None;
    }

    /// Query an element by ID
    ///
    /// Returns the raw node ID (u64) if an element with the given ID exists.
    /// This enables components to look up elements without needing a context reference.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use blinc_core::context_state::query;
    ///
    /// if let Some(node_id) = query("my-element") {
    ///     // Element exists
    /// }
    /// ```
    pub fn query(&self, id: &str) -> Option<u64> {
        self.query_callback().as_ref().and_then(|cb| cb(id))
    }

    // =========================================================================
    // Element Bounds & Visibility
    // =========================================================================

    /// Set the bounds callback for element bounds lookup
    ///
    /// Called by `WindowedApp` to enable bounds queries by element ID.
    pub fn set_bounds_callback(&self, callback: BoundsCallback) {
        if self
            .with_binding_override_mut(|bindings| {
                bindings.bounds_callback = Some(Arc::clone(&callback));
            })
            .is_some()
        {
            return;
        }
        *self.bounds_callback.write().unwrap() = Some(callback);
    }

    /// Get the current bounds callback.
    pub fn bounds_callback(&self) -> Option<BoundsCallback> {
        if let Some(callback) =
            self.with_binding_override(|bindings| bindings.bounds_callback.clone())
        {
            return callback;
        }
        self.bounds_callback.read().unwrap().clone()
    }

    /// Clear the bounds callback.
    pub fn clear_bounds_callback(&self) {
        if self
            .with_binding_override_mut(|bindings| {
                bindings.bounds_callback = None;
            })
            .is_some()
        {
            return;
        }
        *self.bounds_callback.write().unwrap() = None;
    }

    /// Get element bounds by string ID
    ///
    /// Returns the computed bounds after layout, or None if the element
    /// doesn't exist or hasn't been laid out yet.
    pub fn get_bounds(&self, id: &str) -> Option<Bounds> {
        self.bounds_callback().as_ref().and_then(|cb| cb(id))
    }

    /// Set the current viewport size
    ///
    /// Called by `WindowedApp` when the window is resized.
    pub fn set_viewport_size(&self, width: f32, height: f32) {
        if self
            .with_binding_override_mut(|bindings| {
                bindings.viewport_size = Some((width, height));
            })
            .is_some()
        {
            return;
        }
        *self.viewport_size.write().unwrap() = (width, height);
    }

    /// Get the current viewport size (width, height)
    pub fn viewport_size(&self) -> (f32, f32) {
        if let Some(size) = self.with_binding_override(|bindings| bindings.viewport_size) {
            return size.unwrap_or_else(|| *self.viewport_size.read().unwrap());
        }
        *self.viewport_size.read().unwrap()
    }

    // =========================================================================
    // Focus Management
    // =========================================================================

    /// Set the focus callback
    ///
    /// Called by `WindowedApp` to wire focus changes to the EventRouter.
    pub fn set_focus_callback(&self, callback: FocusCallback) {
        if self
            .with_binding_override_mut(|bindings| {
                bindings.focus_callback = Some(Arc::clone(&callback));
            })
            .is_some()
        {
            return;
        }
        *self.focus_callback.write().unwrap() = Some(callback);
    }

    /// Get the current focus callback.
    pub fn focus_callback(&self) -> Option<FocusCallback> {
        if let Some(callback) =
            self.with_binding_override(|bindings| bindings.focus_callback.clone())
        {
            return callback;
        }
        self.focus_callback.read().unwrap().clone()
    }

    /// Clear the focus callback.
    pub fn clear_focus_callback(&self) {
        if self
            .with_binding_override_mut(|bindings| {
                bindings.focus_callback = None;
            })
            .is_some()
        {
            return;
        }
        *self.focus_callback.write().unwrap() = None;
    }

    /// Set focus to an element by string ID
    ///
    /// Pass `None` to clear focus.
    pub fn set_focus(&self, id: Option<&str>) {
        let callback = self.focus_callback();
        if self
            .with_binding_override_mut(|bindings| {
                bindings.focused_element = id.map(str::to_string);
            })
            .is_none()
        {
            *self.focused_element.write().unwrap() = id.map(str::to_string);
        }

        if let Some(cb) = callback.as_ref() {
            cb(id);
        }
    }

    /// Synchronize focus state from the runtime without invoking focus callbacks.
    ///
    /// This keeps query APIs in sync with the active runtime focus when the
    /// router changes focus as part of normal event handling.
    pub fn sync_focus_state(&self, id: Option<&str>) {
        if self
            .with_binding_override_mut(|bindings| {
                bindings.focused_element = id.map(str::to_string);
            })
            .is_none()
        {
            *self.focused_element.write().unwrap() = id.map(str::to_string);
        }
    }

    /// Get the currently focused element ID
    pub fn focused_element(&self) -> Option<String> {
        if let Some(focused) =
            self.with_binding_override(|bindings| bindings.focused_element.clone())
        {
            return focused;
        }
        self.focused_element.read().unwrap().clone()
    }

    /// Check if an element is currently focused
    pub fn is_focused(&self, id: &str) -> bool {
        self.focused_element().as_deref() == Some(id)
    }

    // =========================================================================
    // Scroll Into View
    // =========================================================================

    /// Set the scroll callback
    ///
    /// Called by `WindowedApp` to wire scroll requests to the RenderTree.
    pub fn set_scroll_callback(&self, callback: ScrollCallback) {
        if self
            .with_binding_override_mut(|bindings| {
                bindings.scroll_callback = Some(Arc::clone(&callback));
            })
            .is_some()
        {
            return;
        }
        *self.scroll_callback.write().unwrap() = Some(callback);
    }

    /// Get the current scroll callback.
    pub fn scroll_callback(&self) -> Option<ScrollCallback> {
        if let Some(callback) =
            self.with_binding_override(|bindings| bindings.scroll_callback.clone())
        {
            return callback;
        }
        self.scroll_callback.read().unwrap().clone()
    }

    /// Scroll an element into view
    pub fn scroll_element_into_view_with_options(&self, id: &str, options: ScrollIntoViewOptions) {
        let callback = self.scroll_callback();
        if let Some(cb) = callback.as_ref() {
            cb(id, options);
        }
    }

    /// Scroll an element into view with default alignment behavior.
    pub fn scroll_element_into_view(&self, id: &str) {
        self.scroll_element_into_view_with_options(id, ScrollIntoViewOptions::default());
    }

    /// Set the programmatic element event callback.
    ///
    /// Called by the app runtime to route ElementHandle interactions through
    /// the existing event router and render tree.
    pub fn set_programmatic_event_callback(&self, callback: ProgrammaticEventCallback) {
        if self
            .with_binding_override_mut(|bindings| {
                bindings.programmatic_event_callback = Some(Arc::clone(&callback));
            })
            .is_some()
        {
            return;
        }
        *self.programmatic_event_callback.write().unwrap() = Some(callback);
    }

    /// Get the current programmatic event callback.
    pub fn programmatic_event_callback(&self) -> Option<ProgrammaticEventCallback> {
        if let Some(callback) =
            self.with_binding_override(|bindings| bindings.programmatic_event_callback.clone())
        {
            return callback;
        }
        self.programmatic_event_callback.read().unwrap().clone()
    }

    /// Clear the programmatic event callback.
    pub fn clear_programmatic_event_callback(&self) {
        if self
            .with_binding_override_mut(|bindings| {
                bindings.programmatic_event_callback = None;
            })
            .is_some()
        {
            return;
        }
        *self.programmatic_event_callback.write().unwrap() = None;
    }

    /// Dispatch a programmatic event to a target element by string ID.
    pub fn dispatch_programmatic_event(&self, id: &str, event: ProgrammaticElementEvent) {
        if let Some(cb) = self.programmatic_event_callback().as_ref() {
            cb(id, event);
        }
    }

    // =========================================================================
    // Element Registry (for query API)
    // =========================================================================

    /// Set the element registry
    ///
    /// Called by `WindowedApp` to store the registry for the query API.
    /// The registry is stored as type-erased `AnyElementRegistry` to avoid
    /// circular dependencies with blinc_layout.
    pub fn set_element_registry(&self, registry: AnyElementRegistry) {
        if self
            .with_binding_override_mut(|bindings| {
                bindings.element_registry = Some(Arc::clone(&registry));
            })
            .is_some()
        {
            return;
        }
        *self.element_registry.write().unwrap() = Some(registry);
    }

    /// Clear the element registry.
    pub fn clear_element_registry(&self) {
        if self
            .with_binding_override_mut(|bindings| {
                bindings.element_registry = None;
            })
            .is_some()
        {
            return;
        }
        *self.element_registry.write().unwrap() = None;
    }

    /// Get the element registry as type-erased Any
    ///
    /// Returns the raw `Arc` which can be downcast to the concrete
    /// `ElementRegistry` type in blinc_layout.
    pub fn element_registry_any(&self) -> Option<AnyElementRegistry> {
        if let Some(registry) =
            self.with_binding_override(|bindings| bindings.element_registry.clone())
        {
            return registry;
        }
        self.element_registry.read().unwrap().clone()
    }

    /// Get the element registry, downcasting to the expected type
    ///
    /// This is a convenience method for use by blinc_layout's query function.
    pub fn element_registry<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.element_registry_any()
            .and_then(|registry| registry.downcast::<T>().ok())
    }

    // =========================================================================
    // Motion Animation State Query
    // =========================================================================

    /// Set the motion state callback
    ///
    /// Called by `WindowedApp` to enable motion animation state queries.
    /// The callback receives a stable motion key and returns its animation state.
    pub fn set_motion_state_callback(&self, callback: MotionStateCallback) {
        if self
            .with_binding_override_mut(|bindings| {
                bindings.motion_state_callback = Some(Arc::clone(&callback));
            })
            .is_some()
        {
            return;
        }
        *self.motion_state_callback.write().unwrap() = Some(callback);
    }

    /// Query motion animation state by stable key
    ///
    /// Returns the current state of a motion animation identified by its stable key.
    /// This enables components to check if a parent motion is still animating
    /// before rendering their own content.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use blinc_core::context_state::query_motion;
    ///
    /// let state = query_motion("dialog-content");
    /// if state.is_settled() {
    ///     // Safe to render hover effects, etc.
    /// }
    /// ```
    pub fn query_motion(&self, key: &str) -> MotionAnimationState {
        self.with_binding_override(|bindings| bindings.motion_state_callback.clone())
            .unwrap_or_else(|| self.motion_state_callback.read().unwrap().clone())
            .as_ref()
            .map(|cb| cb(key))
            .unwrap_or(MotionAnimationState::NotFound)
    }

    /// Set the motion cancel exit callback
    ///
    /// Called by `WindowedApp` to enable motion exit cancellation.
    /// The callback receives a stable motion key and cancels its exit animation.
    pub fn set_motion_cancel_exit_callback(&self, callback: MotionCancelExitCallback) {
        if self
            .with_binding_override_mut(|bindings| {
                bindings.motion_cancel_exit_callback = Some(Arc::clone(&callback));
            })
            .is_some()
        {
            return;
        }
        *self.motion_cancel_exit_callback.write().unwrap() = Some(callback);
    }

    /// Cancel a motion's exit animation by stable key
    ///
    /// Used when an overlay's close is cancelled (e.g., mouse re-enters hover card).
    /// This interrupts the exit animation and immediately sets the motion to fully visible.
    ///
    /// No-op if the motion is not in Exiting state or callback is not set.
    pub fn cancel_motion_exit(&self, key: &str) {
        let callback = self
            .with_binding_override(|bindings| bindings.motion_cancel_exit_callback.clone())
            .unwrap_or_else(|| self.motion_cancel_exit_callback.read().unwrap().clone());
        if let Some(ref cb) = callback {
            cb(key);
        }
    }

    // =========================================================================
    // Recorder Integration (for blinc_recorder)
    // =========================================================================

    /// Set the recorder event callback
    ///
    /// Called by `blinc_recorder` to capture user interaction events.
    /// Events are passed as type-erased `RecordedEventAny` to avoid circular dependencies.
    pub fn set_recorder_event_callback(&self, callback: RecorderEventCallback) {
        *self.recorder_event_callback.write().unwrap() = Some(callback);
    }

    /// Clear the recorder event callback
    pub fn clear_recorder_event_callback(&self) {
        *self.recorder_event_callback.write().unwrap() = None;
    }

    /// Record an event if a recorder callback is set
    ///
    /// This is called by EventRouter and other event sources to record user interactions.
    pub fn record_event(&self, event: RecordedEventAny) {
        if let Some(ref cb) = *self.recorder_event_callback.read().unwrap() {
            cb(event);
        }
    }

    /// Check if event recording is enabled
    pub fn is_recording_events(&self) -> bool {
        self.recorder_event_callback.read().unwrap().is_some()
    }

    /// Set the recorder snapshot callback
    ///
    /// Called by `blinc_recorder` to capture tree snapshots after each frame.
    /// Snapshots are passed as type-erased `TreeSnapshotAny` to avoid circular dependencies.
    pub fn set_recorder_snapshot_callback(&self, callback: RecorderSnapshotCallback) {
        *self.recorder_snapshot_callback.write().unwrap() = Some(callback);
    }

    /// Clear the recorder snapshot callback
    pub fn clear_recorder_snapshot_callback(&self) {
        *self.recorder_snapshot_callback.write().unwrap() = None;
    }

    /// Record a tree snapshot if a recorder callback is set
    ///
    /// This is called by RenderTree after each frame to capture the element tree state.
    pub fn record_snapshot(&self, snapshot: TreeSnapshotAny) {
        if let Some(ref cb) = *self.recorder_snapshot_callback.read().unwrap() {
            cb(snapshot);
        }
    }

    /// Check if snapshot recording is enabled
    pub fn is_recording_snapshots(&self) -> bool {
        self.recorder_snapshot_callback.read().unwrap().is_some()
    }

    /// Set the recorder update callback
    ///
    /// Called by `blinc_recorder` to track element update categories.
    pub fn set_recorder_update_callback(&self, callback: RecorderUpdateCallback) {
        *self.recorder_update_callback.write().unwrap() = Some(callback);
    }

    /// Clear the recorder update callback
    pub fn clear_recorder_update_callback(&self) {
        *self.recorder_update_callback.write().unwrap() = None;
    }

    /// Record an element update if a recorder callback is set
    ///
    /// This is called by diff/stateful when element updates are detected.
    pub fn record_update(&self, element_id: &str, category: UpdateCategory) {
        if let Some(ref cb) = *self.recorder_update_callback.read().unwrap() {
            cb(element_id, category);
        }
    }

    /// Check if update recording is enabled
    pub fn is_recording_updates(&self) -> bool {
        self.recorder_update_callback.read().unwrap().is_some()
    }

    // =========================================================================
    // Scroll Ref Support (for blinc_layout integration)
    // =========================================================================

    /// Get or create a persisted value for scroll ref inner state
    ///
    /// This is a low-level method used by blinc_layout's `use_scroll_ref` function
    /// to persist ScrollRefInner across rebuilds without circular dependencies.
    ///
    /// Returns (signal_id, value) tuple where the value is retrieved from or stored
    /// in the reactive graph.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The type to store (typically `Arc<Mutex<ScrollRefInner>>`)
    /// - `F`: Factory function to create initial value
    pub fn get_or_create_persisted<T, F>(&self, key: &str, create: F) -> (SignalId, T)
    where
        T: Clone + Send + 'static,
        F: FnOnce() -> T,
    {
        let resources = self.active_resources();
        let state_key = StateKey::from_string::<T>(key);
        let mut hooks = resources.hooks.lock().unwrap();

        if let Some(raw_id) = hooks.get(&state_key) {
            // Reconstruct the signal ID and get the value from the reactive graph
            let signal_id = SignalId::from_raw(raw_id);
            let value = resources
                .reactive
                .lock()
                .unwrap()
                .get_untracked(Signal::<T>::from_id(signal_id))
                .unwrap_or_else(create);
            (signal_id, value)
        } else {
            // First time - create a new value and store it in the reactive graph
            let new_value = create();
            let signal = resources
                .reactive
                .lock()
                .unwrap()
                .create_signal(new_value.clone());
            let raw_id = signal.id().to_raw();
            hooks.insert::<T>(state_key, key, raw_id);
            (signal.id(), new_value)
        }
    }
}

// =========================================================================
// Convenience Free Functions
// =========================================================================

/// Create a persistent state value that survives across UI rebuilds (keyed)
///
/// This is a convenience wrapper around `BlincContextState::get().use_state_keyed()`.
///
/// # Panics
///
/// Panics if `BlincContextState::init()` has not been called.
///
/// # Example
///
/// ```ignore
/// use blinc_core::context_state::use_state_keyed;
///
/// // In a component:
/// let open_state = use_state_keyed("my_component_open", || false);
/// ```
pub fn use_state_keyed<T, F>(key: &str, init: F) -> State<T>
where
    T: Clone + Send + 'static,
    F: FnOnce() -> T,
{
    BlincContextState::get().use_state_keyed(key, init)
}

/// Create a persistent signal that survives across UI rebuilds (keyed)
///
/// This is a convenience wrapper around `BlincContextState::get().use_signal_keyed()`.
///
/// # Panics
///
/// Panics if `BlincContextState::init()` has not been called.
pub fn use_signal_keyed<T, F>(key: &str, init: F) -> Signal<T>
where
    T: Clone + Send + 'static,
    F: FnOnce() -> T,
{
    BlincContextState::get().use_signal_keyed(key, init)
}

/// Request a UI rebuild
///
/// This is a convenience wrapper around `BlincContextState::get().request_rebuild()`.
///
/// # Panics
///
/// Panics if `BlincContextState::init()` has not been called.
pub fn request_rebuild() {
    BlincContextState::get().request_rebuild();
}

/// Query an element by ID
///
/// Returns the raw node ID (u64) if an element with the given ID exists.
/// This is a convenience wrapper around `BlincContextState::get().query()`.
///
/// # Panics
///
/// Panics if `BlincContextState::init()` has not been called.
///
/// # Example
///
/// ```ignore
/// use blinc_core::context_state::query;
///
/// if let Some(node_id) = query("my-element") {
///     // Element with ID "my-element" exists
/// }
/// ```
pub fn query(id: &str) -> Option<u64> {
    BlincContextState::get().query(id)
}

/// Query motion animation state by stable key
///
/// Returns the current state of a motion animation identified by its stable key.
/// This enables components to check if a parent motion is still animating
/// before rendering their own content.
///
/// # Panics
///
/// Panics if `BlincContextState::init()` has not been called.
///
/// # Example
///
/// ```ignore
/// use blinc_core::context_state::query_motion;
///
/// let state = query_motion("dialog-content");
/// if state.is_settled() {
///     // Safe to render hover effects, etc.
/// }
/// ```
pub fn query_motion(key: &str) -> MotionAnimationState {
    BlincContextState::get().query_motion(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    fn make_test_state() -> BlincContextState {
        let state = BlincContextState {
            reactive: Arc::new(Mutex::new(ReactiveGraph::new())),
            hooks: Arc::new(Mutex::new(HookState::new())),
            dirty_flag: Arc::new(AtomicBool::new(false)),
            stateful_callback: None,
            query_callback: RwLock::new(None),
            bounds_callback: RwLock::new(None),
            focus_callback: RwLock::new(None),
            scroll_callback: RwLock::new(None),
            programmatic_event_callback: RwLock::new(None),
            viewport_size: RwLock::new((0.0, 0.0)),
            focused_element: RwLock::new(None),
            element_registry: RwLock::new(None),
            motion_state_callback: RwLock::new(None),
            motion_cancel_exit_callback: RwLock::new(None),
            recorder_event_callback: RwLock::new(None),
            recorder_snapshot_callback: RwLock::new(None),
            recorder_update_callback: RwLock::new(None),
        };
        state.restore_resource_override(None);
        state.restore_binding_override(None);
        state
    }

    #[test]
    fn test_state_key() {
        let key1 = StateKey::from_string::<i32>("counter");
        let key2 = StateKey::from_string::<i32>("counter");
        let key3 = StateKey::from_string::<String>("counter");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3); // Different types
    }

    #[test]
    fn test_hook_state() {
        let mut hooks = HookState::new();
        let key = StateKey::from_string::<i32>("test");

        assert!(hooks.get(&key).is_none());

        hooks.insert::<i32>(key.clone(), "test", 42);
        assert_eq!(hooks.get(&key), Some(42));
        assert_eq!(
            hooks.debug_registrations(),
            vec![HookDebugRegistration {
                signal_id: 42,
                key: "test".to_string(),
                type_name: "i32",
            }]
        );
    }

    #[test]
    fn test_use_state_keyed_init_can_call_use_state_keyed_without_deadlock() {
        let state = make_test_state();

        let outer: State<i32> = state.use_state_keyed("outer", || {
            let inner: State<i32> = state.use_state_keyed("inner", || 123);
            // This read used to deadlock because `outer` init ran while holding the reactive lock.
            inner.get() + 1
        });

        assert_eq!(outer.get(), 124);
    }

    #[test]
    fn test_use_signal_keyed_init_can_call_use_state_keyed_without_deadlock() {
        let state = make_test_state();

        let sig: Signal<i32> = state.use_signal_keyed("sig", || {
            let inner: State<i32> = state.use_state_keyed("inner2", || 7);
            inner.get() * 3
        });

        assert_eq!(state.active_reactive().lock().unwrap().get(sig), Some(21));
    }

    #[test]
    fn resource_override_scopes_keyed_state_without_touching_base_resources() {
        let state = make_test_state();
        let override_reactive = Arc::new(Mutex::new(ReactiveGraph::new()));
        let override_hooks: SharedHookState = Arc::new(Mutex::new(HookState::new()));
        let override_dirty = Arc::new(AtomicBool::new(false));

        state.set_resource_override(ContextResourceOverride::new(
            Arc::clone(&override_reactive),
            Arc::clone(&override_hooks),
            Arc::clone(&override_dirty),
        ));

        let signal: Signal<i32> = state.use_signal_keyed("override-counter", || 5);
        state.set_signal(signal, 8);
        state.request_rebuild();

        assert_eq!(override_reactive.lock().unwrap().get(signal), Some(8));
        assert!(override_dirty.load(Ordering::SeqCst));
        assert_eq!(
            override_hooks
                .lock()
                .unwrap()
                .get(&StateKey::from_string::<i32>("override-counter")),
            Some(signal.id().to_raw())
        );
        assert!(state
            .hooks
            .lock()
            .unwrap()
            .get(&StateKey::from_string::<i32>("override-counter"))
            .is_none());

        state.restore_resource_override(None);
        assert!(!Arc::ptr_eq(&state.active_reactive(), &override_reactive));
    }

    #[test]
    fn binding_override_scopes_query_and_focus_state_to_current_thread() {
        let state = make_test_state();
        state.set_query_callback(Arc::new(|_| Some(7)));
        state.set_focus(Some("base"));
        state.set_viewport_size(10.0, 20.0);

        let _previous = state.set_binding_override(ContextBindingOverride::default());
        state.set_query_callback(Arc::new(|_| Some(99)));
        state.set_focus(Some("override"));
        state.set_viewport_size(30.0, 40.0);

        assert_eq!(state.query("node"), Some(99));
        assert_eq!(state.focused_element().as_deref(), Some("override"));
        assert_eq!(state.viewport_size(), (30.0, 40.0));

        state.restore_binding_override(None);

        assert_eq!(state.query("node"), Some(7));
        assert_eq!(state.focused_element().as_deref(), Some("base"));
        assert_eq!(state.viewport_size(), (10.0, 20.0));
    }

    #[test]
    fn reseed_for_tests_clears_base_state_and_thread_overrides() {
        let state = make_test_state();
        state.set_query_callback(Arc::new(|_| Some(7)));
        state.set_focus(Some("base"));
        state.set_viewport_size(10.0, 20.0);
        state.set_programmatic_event_callback(Arc::new(|_, _| {}));
        state.set_element_registry(Arc::new(123usize));
        state.set_recorder_event_callback(Arc::new(|_| {}));
        state.set_recorder_snapshot_callback(Arc::new(|_| {}));
        state.set_recorder_update_callback(Arc::new(|_, _| {}));
        let _: State<i32> = state.use_state_keyed("base-counter", || 1);

        let override_reactive = Arc::new(Mutex::new(ReactiveGraph::new()));
        let override_hooks: SharedHookState = Arc::new(Mutex::new(HookState::new()));
        let override_dirty = Arc::new(AtomicBool::new(true));
        state.set_resource_override(ContextResourceOverride::new(
            Arc::clone(&override_reactive),
            Arc::clone(&override_hooks),
            Arc::clone(&override_dirty),
        ));
        state.set_binding_override(ContextBindingOverride::default());
        state.set_query_callback(Arc::new(|_| Some(99)));
        state.set_focus(Some("override"));
        state.set_viewport_size(30.0, 40.0);
        let _: State<i32> = state.use_state_keyed("override-counter", || 2);

        state.reseed_for_tests();

        assert_eq!(state.query("node"), None);
        assert_eq!(state.focused_element(), None);
        assert_eq!(state.viewport_size(), (0.0, 0.0));
        assert!(state.programmatic_event_callback().is_none());
        assert!(state.element_registry_any().is_none());
        assert!(!state.is_recording_events());
        assert!(!state.is_recording_snapshots());
        assert!(!state.is_recording_updates());
        assert!(state.debug_keyed_state_entries().is_empty());
        assert!(state.with_binding_override(|_| ()).is_none());
        assert!(state
            .active_hooks()
            .lock()
            .unwrap()
            .debug_registrations()
            .is_empty());
        assert!(!Arc::ptr_eq(&state.active_reactive(), &override_reactive));
        assert!(override_hooks
            .lock()
            .unwrap()
            .debug_registrations()
            .is_empty());
        assert!(!override_dirty.load(Ordering::SeqCst));
    }
}
