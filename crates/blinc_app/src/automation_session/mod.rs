use std::sync::{atomic::Ordering, Arc, Mutex};

use anyhow::{bail, Result};
use blinc_animation::AnimationScheduler;
use blinc_core::context_state::{ContextBindingOverride, ContextResourceOverride};
use blinc_core::{BlincContextState, MotionAnimationState};
use blinc_core::{ProgrammaticElementEvent, ScrollIntoViewOptions};
use blinc_layout::recorder_bridge::{capture_tree_snapshot, to_tree_snapshot};
use blinc_layout::selector::SharedElementRegistry;
use blinc_layout::selector::{
    dispatch_programmatic_event_to_node, drain_programmatic_runtime_requests,
    resolve_semantic_locator, sync_context_focus_from_runtime, sync_focus_node_to_runtime,
    sync_focus_to_runtime,
};
use blinc_layout::widgets::overlay::OverlayManagerExt;
use blinc_layout::{CssAnimationStore, RenderState, SharedMotionStates, UpdateResult};
use blinc_recorder::{
    get_recorder, install_recorder, uninstall_recorder, RecordingConfig, RecordingExport,
    SharedRecordingSession, TraceArtifactRecord, TraceAssertionRecord, TraceCommandRecord,
    TraceEntryKind, TraceLocatorResolution, TreeSnapshot,
};

use crate::headless_runtime::HeadlessRunConfig;
use crate::windowed::{
    RefDirtyFlag, SharedAnimationScheduler, SharedReadyCallbacks, WindowedContext,
};

mod context;
mod interactions;
mod runner;
mod runtime;
mod types;

use context::{
    configure_context_callbacks, shared_state_for_automation, snapshot_context_bindings,
};
pub use runner::{run_desktop_harness_scenario, run_headless_scenario};
use types::{parse_key, select_all_modifiers};
pub use types::{AutomationFailure, AutomationLocator, AutomationRun, AutomationRuntimeMode};

type AutomationResult<T> = std::result::Result<T, AutomationFailure>;
type PendingProgrammaticEvents = Arc<Mutex<Vec<(String, ProgrammaticElementEvent)>>>;
type PendingFocusChanges = Arc<Mutex<Vec<Option<String>>>>;
type PendingScrollRequests = Arc<Mutex<Vec<(String, ScrollIntoViewOptions)>>>;
type AutomationSessionGuard = std::sync::MutexGuard<'static, ()>;

#[derive(Clone, Debug)]
struct ResolvedTarget {
    node_id: blinc_layout::LayoutNodeId,
    target: Option<String>,
}

pub struct AutomationSession<F, E>
where
    F: FnMut(&mut WindowedContext) -> E,
    E: blinc_layout::ElementBuilder + 'static,
{
    session_guard: Option<AutomationSessionGuard>,
    runtime_mode: AutomationRuntimeMode,
    runtime_cfg: HeadlessRunConfig,
    ui_builder: F,
    ctx: WindowedContext,
    tree: blinc_layout::RenderTree,
    element_registry: SharedElementRegistry,
    ref_dirty_flag: RefDirtyFlag,
    pending_programmatic_events: PendingProgrammaticEvents,
    pending_focus_changes: PendingFocusChanges,
    pending_scroll_requests: PendingScrollRequests,
    recording: Arc<SharedRecordingSession>,
    render_state: RenderState,
    css_anim_store: Arc<Mutex<CssAnimationStore>>,
    last_frame_time_ms: u64,
    latest_snapshot: Option<TreeSnapshot>,
    previous_binding_override: Option<ContextBindingOverride>,
    previous_resource_override: Option<ContextResourceOverride>,
    restore_resource_override: bool,
    previous_recorder: Option<Arc<SharedRecordingSession>>,
}

impl<F, E> AutomationSession<F, E>
where
    F: FnMut(&mut WindowedContext) -> E,
    E: blinc_layout::ElementBuilder + 'static,
{
    pub fn new_headless(runtime_cfg: HeadlessRunConfig, ui_builder: F) -> Self {
        Self::new_with_mode(AutomationRuntimeMode::Headless, runtime_cfg, ui_builder)
    }

    pub fn new_desktop_harness(runtime_cfg: HeadlessRunConfig, ui_builder: F) -> Self {
        Self::new_with_mode(
            AutomationRuntimeMode::DesktopHarness,
            runtime_cfg,
            ui_builder,
        )
    }

    pub fn runtime_mode(&self) -> AutomationRuntimeMode {
        self.runtime_mode
    }

    fn new_with_mode(
        runtime_mode: AutomationRuntimeMode,
        runtime_cfg: HeadlessRunConfig,
        mut ui_builder: F,
    ) -> Self {
        static AUTOMATION_SESSION_LOCK: Mutex<()> = Mutex::new(());

        let session_guard = AUTOMATION_SESSION_LOCK.lock().unwrap_or_else(|err| {
            tracing::warn!("automation session lock was poisoned; recovering exclusive access");
            err.into_inner()
        });
        blinc_layout::widgets::blur_all_text_inputs();
        let previous_context_bindings = BlincContextState::try_get().map(snapshot_context_bindings);
        let previous_recorder = get_recorder();
        let (
            reactive,
            hooks,
            ref_dirty_flag,
            previous_resource_override,
            restore_resource_override,
        ) = shared_state_for_automation(runtime_mode);
        let previous_binding_override =
            BlincContextState::get().set_binding_override(ContextBindingOverride::default());
        let animations: SharedAnimationScheduler = Arc::new(Mutex::new(AnimationScheduler::new()));
        let shared_motion_states: SharedMotionStates = blinc_layout::create_shared_motion_states();
        {
            let motion_states_for_callback = Arc::clone(&shared_motion_states);
            let motion_callback: blinc_core::MotionStateCallback = Arc::new(move |key: &str| {
                motion_states_for_callback
                    .read()
                    .ok()
                    .and_then(|states| states.get(key).copied())
                    .unwrap_or(MotionAnimationState::NotFound)
            });
            BlincContextState::get().set_motion_state_callback(motion_callback);
        }
        let css_anim_store = Arc::new(Mutex::new(CssAnimationStore::new()));
        let mut render_state = RenderState::new(Arc::clone(&animations));
        render_state.set_shared_motion_states(shared_motion_states);
        let element_registry: SharedElementRegistry =
            Arc::new(blinc_layout::selector::ElementRegistry::new());
        let ready_callbacks: SharedReadyCallbacks = Arc::new(Mutex::new(Vec::new()));
        let pending_programmatic_events: PendingProgrammaticEvents =
            Arc::new(Mutex::new(Vec::new()));
        let pending_focus_changes: PendingFocusChanges = Arc::new(Mutex::new(Vec::new()));
        let pending_scroll_requests: PendingScrollRequests = Arc::new(Mutex::new(Vec::new()));
        let mut ctx = WindowedContext::new_headless(
            runtime_cfg.width as f32,
            runtime_cfg.height as f32,
            animations,
            Arc::clone(&ref_dirty_flag),
            reactive,
            hooks,
            Arc::clone(&element_registry),
            ready_callbacks,
        );

        configure_context_callbacks(
            runtime_mode,
            &element_registry,
            &pending_focus_changes,
            &pending_scroll_requests,
            &pending_programmatic_events,
            previous_context_bindings.as_ref(),
        );
        BlincContextState::get()
            .set_viewport_size(runtime_cfg.width as f32, runtime_cfg.height as f32);

        let current_time = 0u64;
        ctx.prepare_windowless_frame(current_time);
        let user_ui = ui_builder(&mut ctx);
        let ui = ctx.compose_runtime_ui(user_ui);
        let mut tree = blinc_layout::RenderTree::from_element_with_registry(
            &ui,
            Arc::clone(&element_registry),
        );
        tree.set_animations(&ctx.animations);
        tree.set_css_anim_store(Arc::clone(&css_anim_store));
        tree.set_scale_factor(ctx.scale_factor as f32);
        if let Some(ref stylesheet) = ctx.stylesheet {
            tree.set_stylesheet_arc(stylesheet.clone());
            tree.apply_all_stylesheet_styles();
        }
        tree.compute_layout(runtime_cfg.width as f32, runtime_cfg.height as f32);
        render_state.begin_stable_motion_frame();
        tree.initialize_motion_animations(&mut render_state);
        render_state.end_stable_motion_frame();
        render_state.process_global_motion_replays();
        tree.start_all_css_animations();
        render_state.sync_shared_motion_states();
        tree.process_pending_scroll_refs();
        ctx.finish_runtime_rebuild();
        tree.process_pending_scroll_refs();

        let recording = Arc::new(SharedRecordingSession::new(RecordingConfig::debug()));
        install_recorder(Arc::clone(&recording));
        recording.start();

        let mut session = Self {
            session_guard: Some(session_guard),
            runtime_mode,
            runtime_cfg,
            ui_builder,
            ctx,
            tree,
            element_registry,
            ref_dirty_flag,
            pending_programmatic_events,
            pending_focus_changes,
            pending_scroll_requests,
            recording,
            render_state,
            css_anim_store,
            last_frame_time_ms: current_time,
            latest_snapshot: None,
            previous_binding_override,
            previous_resource_override,
            restore_resource_override,
            previous_recorder,
        };
        session
            .recording
            .record_trace_entry(TraceEntryKind::Artifact(TraceArtifactRecord {
                kind: "runtime_mode".to_string(),
                path: None,
                message: Some(match runtime_mode {
                    AutomationRuntimeMode::Headless => "headless".to_string(),
                    AutomationRuntimeMode::DesktopHarness => "desktop_harness".to_string(),
                }),
            }));
        session.advance_runtime_frame(current_time, false, true);
        session.capture_snapshot();
        session
    }

    pub fn export_recording(&self) -> RecordingExport {
        self.recording.export()
    }

    #[cfg(test)]
    pub(crate) fn runtime_time_ms(&self) -> u64 {
        self.last_frame_time_ms
    }

    #[cfg(test)]
    pub(crate) fn overlay_scroll_offsets(&self) -> Vec<(String, f32)> {
        self.ctx.overlay_manager().get_scroll_offsets()
    }

    #[cfg(test)]
    pub(crate) fn absolute_bounds_for_id(
        &self,
        id: &str,
    ) -> Option<blinc_layout::element::ElementBounds> {
        let node_id = self.tree.query_by_id(id)?;
        self.tree.get_absolute_bounds(node_id)
    }

    fn record_command(
        &self,
        name: &str,
        locator: &AutomationLocator,
        payload: Option<String>,
    ) -> u64 {
        self.recording
            .record_trace_entry(TraceEntryKind::Command(TraceCommandRecord {
                name: name.to_string(),
                target: Some(locator.describe()),
                payload,
            }))
    }

    fn record_assertion(
        &self,
        code: &str,
        passed: bool,
        target: Option<&str>,
        actual: Option<String>,
        expected: Option<String>,
    ) -> u64 {
        self.recording
            .record_trace_entry(TraceEntryKind::Assertion(TraceAssertionRecord {
                code: code.to_string(),
                passed,
                target: target.map(str::to_string),
                actual,
                expected,
            }))
    }

    fn fail(
        &self,
        code: &str,
        message: &str,
        target: Option<String>,
        trace_sequence: Option<u64>,
    ) -> AutomationFailure {
        AutomationFailure {
            code: code.to_string(),
            message: message.to_string(),
            target,
            trace_sequence,
        }
    }

    fn stop_recording(&self) {
        self.recording.stop();
    }
}

impl<F, E> Drop for AutomationSession<F, E>
where
    F: FnMut(&mut WindowedContext) -> E,
    E: blinc_layout::ElementBuilder + 'static,
{
    fn drop(&mut self) {
        blinc_layout::widgets::blur_all_text_inputs();

        if self.restore_resource_override {
            if let Some(ctx) = BlincContextState::try_get() {
                ctx.restore_resource_override(self.previous_resource_override.take());
            }
        }

        if let Some(ctx) = BlincContextState::try_get() {
            ctx.restore_binding_override(self.previous_binding_override.take());
        }

        if let Some(recorder) = self.previous_recorder.take() {
            install_recorder(recorder);
        } else {
            uninstall_recorder();
        }

        let _ = self.session_guard.take();
    }
}

fn redacted_trace_value(value: &str) -> String {
    let len = value.chars().count();
    if len == 0 {
        "value=<redacted:empty>".to_string()
    } else {
        format!("value=<redacted:{len} chars>")
    }
}

fn subtree_text(
    tree: &blinc_layout::RenderTree,
    node_id: blinc_layout::LayoutNodeId,
) -> Option<String> {
    let mut parts = Vec::new();
    collect_subtree_text(tree, node_id, &mut parts);
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(""))
    }
}

fn text_contains(actual: &str, expected: &str) -> bool {
    actual.to_lowercase().contains(&expected.to_lowercase())
}

fn collect_subtree_text(
    tree: &blinc_layout::RenderTree,
    node_id: blinc_layout::LayoutNodeId,
    parts: &mut Vec<String>,
) {
    if let Some(render_node) = tree.get_render_node(node_id) {
        match &render_node.element_type {
            blinc_layout::renderer::ElementType::Text(data) => parts.push(data.content.clone()),
            blinc_layout::renderer::ElementType::StyledText(data) => {
                parts.push(data.content.clone())
            }
            _ => {}
        }
    }
    for child in tree.layout().children(node_id) {
        collect_subtree_text(tree, child, parts);
    }
}

fn point_in_overlay_bounds(bounds: &[(f32, f32, f32, f32)], x: f32, y: f32) -> bool {
    bounds
        .iter()
        .any(|&(bx, by, width, height)| x >= bx && x < bx + width && y >= by && y < by + height)
}

fn frame_delta_ms(current_time: u64, last_frame_time_ms: u64) -> f32 {
    if last_frame_time_ms == 0 {
        return current_time as f32;
    }

    current_time.saturating_sub(last_frame_time_ms) as f32
}

#[cfg(test)]
mod automation_session_tests {
    use super::frame_delta_ms;

    #[test]
    fn frame_delta_ms_matches_initial_timestamp_when_no_prior_frame() {
        assert_eq!(frame_delta_ms(0, 0), 0.0);
        assert_eq!(frame_delta_ms(32, 0), 32.0);
    }

    #[test]
    fn frame_delta_ms_saturates_when_time_moves_backwards() {
        assert_eq!(frame_delta_ms(10, 25), 0.0);
    }
}
