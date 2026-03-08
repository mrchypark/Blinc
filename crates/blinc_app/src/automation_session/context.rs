use std::sync::{atomic::AtomicBool, Arc, Mutex};

use blinc_core::context_state::{
    ContextResourceOverride, FocusCallback, HookState, ProgrammaticEventCallback, ScrollCallback,
    SharedHookState, SharedReactiveGraph,
};
use blinc_core::reactive::{ReactiveGraph, SignalId};
use blinc_core::{BlincContextState, DirtyFlag};
use blinc_layout::selector::SharedElementRegistry;

use super::{
    AutomationRuntimeMode, PendingFocusChanges, PendingProgrammaticEvents, PendingScrollRequests,
};

#[derive(Clone)]
pub(super) struct ContextBindingsSnapshot {
    pub(super) focus_callback: Option<FocusCallback>,
    pub(super) scroll_callback: Option<ScrollCallback>,
    pub(super) programmatic_event_callback: Option<ProgrammaticEventCallback>,
}

pub(super) fn shared_state_for_automation(
    _runtime_mode: AutomationRuntimeMode,
) -> (
    SharedReactiveGraph,
    SharedHookState,
    DirtyFlag,
    Option<ContextResourceOverride>,
    bool,
) {
    let reactive: SharedReactiveGraph = Arc::new(Mutex::new(ReactiveGraph::new()));
    let hooks: SharedHookState = Arc::new(Mutex::new(HookState::new()));
    let dirty: DirtyFlag = Arc::new(AtomicBool::new(false));

    let ctx = if let Some(ctx) = BlincContextState::try_get() {
        ctx
    } else {
        let base_reactive: SharedReactiveGraph = Arc::new(Mutex::new(ReactiveGraph::new()));
        let base_hooks: SharedHookState = Arc::new(Mutex::new(HookState::new()));
        let base_dirty: DirtyFlag = Arc::new(AtomicBool::new(false));
        let stateful_callback: Arc<dyn Fn(&[SignalId]) + Send + Sync> = Arc::new(|signal_ids| {
            blinc_layout::check_stateful_deps(signal_ids);
        });
        BlincContextState::init_with_callback(
            base_reactive,
            base_hooks,
            base_dirty,
            stateful_callback,
        );
        BlincContextState::get()
    };

    let previous_override = ctx.set_resource_override(ContextResourceOverride::new(
        Arc::clone(&reactive),
        Arc::clone(&hooks),
        Arc::clone(&dirty),
    ));
    (reactive, hooks, dirty, previous_override, true)
}

pub(super) fn snapshot_context_bindings(ctx: &BlincContextState) -> ContextBindingsSnapshot {
    ContextBindingsSnapshot {
        focus_callback: ctx.focus_callback(),
        scroll_callback: ctx.scroll_callback(),
        programmatic_event_callback: ctx.programmatic_event_callback(),
    }
}

pub(super) fn configure_context_callbacks(
    runtime_mode: AutomationRuntimeMode,
    element_registry: &SharedElementRegistry,
    pending_focus_changes: &PendingFocusChanges,
    pending_scroll_requests: &PendingScrollRequests,
    pending_programmatic_events: &PendingProgrammaticEvents,
    previous_bindings: Option<&ContextBindingsSnapshot>,
) {
    let query_registry = Arc::clone(element_registry);
    let query_callback: blinc_core::QueryCallback =
        Arc::new(move |id| query_registry.get(id).map(|node_id| node_id.to_raw()));
    BlincContextState::get().set_query_callback(query_callback);

    let bounds_registry = Arc::clone(element_registry);
    let bounds_callback: blinc_core::BoundsCallback =
        Arc::new(move |id| bounds_registry.get_bounds(id));
    BlincContextState::get().set_bounds_callback(bounds_callback);

    let focus_queue = Arc::clone(pending_focus_changes);
    let chained_focus = previous_bindings.and_then(|bindings| {
        matches!(runtime_mode, AutomationRuntimeMode::DesktopHarness)
            .then(|| bindings.focus_callback.clone())
            .flatten()
    });
    BlincContextState::get().set_focus_callback(Arc::new(move |id| {
        if let Ok(mut pending) = focus_queue.lock() {
            pending.push(id.map(str::to_string));
        }
        if let Some(callback) = chained_focus.as_ref() {
            callback(id);
        }
    }));

    let scroll_queue = Arc::clone(pending_scroll_requests);
    let chained_scroll = previous_bindings.and_then(|bindings| {
        matches!(runtime_mode, AutomationRuntimeMode::DesktopHarness)
            .then(|| bindings.scroll_callback.clone())
            .flatten()
    });
    BlincContextState::get().set_scroll_callback(Arc::new(move |id, options| {
        if let Ok(mut pending) = scroll_queue.lock() {
            pending.push((id.to_string(), options));
        }
        if let Some(callback) = chained_scroll.as_ref() {
            callback(id, options);
        }
    }));

    let event_queue = Arc::clone(pending_programmatic_events);
    let chained_programmatic = previous_bindings.and_then(|bindings| {
        matches!(runtime_mode, AutomationRuntimeMode::DesktopHarness)
            .then(|| bindings.programmatic_event_callback.clone())
            .flatten()
    });
    BlincContextState::get().set_programmatic_event_callback(Arc::new(move |id, event| {
        if let Ok(mut pending) = event_queue.lock() {
            pending.push((id.to_string(), event.clone()));
        }
        if let Some(callback) = chained_programmatic.as_ref() {
            callback(id, event);
        }
    }));

    BlincContextState::get()
        .set_element_registry(Arc::clone(element_registry) as blinc_core::AnyElementRegistry);
}
