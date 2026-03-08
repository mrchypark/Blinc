use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use anyhow::{bail, Result};
use blinc_animation::AnimationScheduler;
use blinc_core::context_state::{
    AnyElementRegistry, BoundsCallback, ContextBindingOverride, ContextResourceOverride,
    FocusCallback, HookState, ProgrammaticEventCallback, QueryCallback, ScrollCallback,
    SharedHookState, SharedReactiveGraph,
};
use blinc_core::reactive::{ReactiveGraph, SignalId};
use blinc_core::{BlincContextState, DirtyFlag, MotionAnimationState};
use blinc_core::{ProgrammaticElementEvent, ScrollIntoViewOptions};
use blinc_layout::recorder_bridge::{capture_tree_snapshot, to_tree_snapshot};
use blinc_layout::selector::SharedElementRegistry;
use blinc_layout::selector::{
    dispatch_programmatic_event_to_node, drain_programmatic_runtime_requests,
    resolve_semantic_locator, sync_context_focus_from_runtime, sync_focus_node_to_runtime,
    sync_focus_to_runtime, SemanticLocator,
};
use blinc_layout::widgets::overlay::OverlayManagerExt;
use blinc_layout::{CssAnimationStore, RenderState, SharedMotionStates, UpdateResult};
use blinc_platform::AccessibilityRole;
use blinc_recorder::{
    get_recorder, install_recorder, uninstall_recorder, RecordingConfig, RecordingExport,
    SharedRecordingSession, TraceArtifactRecord, TraceAssertionRecord, TraceCommandRecord,
    TraceEntry, TraceEntryKind, TraceLocatorResolution, TreeSnapshot,
};

use crate::headless_report::HeadlessReport;
use crate::headless_runtime::HeadlessRunConfig;
use crate::headless_runtime::HeadlessRuntime;
use crate::headless_scenario::{HeadlessScenario, ScenarioStep, ScenarioTarget};
use crate::windowed::{
    RefDirtyFlag, SharedAnimationScheduler, SharedReadyCallbacks, WindowedContext,
};

type AutomationResult<T> = std::result::Result<T, AutomationFailure>;
type PendingProgrammaticEvents = Arc<Mutex<Vec<(String, ProgrammaticElementEvent)>>>;
type PendingFocusChanges = Arc<Mutex<Vec<Option<String>>>>;
type PendingScrollRequests = Arc<Mutex<Vec<(String, ScrollIntoViewOptions)>>>;
type AutomationSessionGuard = std::sync::MutexGuard<'static, ()>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AutomationLocator {
    Id(String),
    Semantic(SemanticLocator),
}

impl AutomationLocator {
    pub fn id(id: impl Into<String>) -> Self {
        Self::Id(id.into())
    }

    pub fn semantic(locator: SemanticLocator) -> Self {
        Self::Semantic(locator)
    }

    pub fn describe(&self) -> String {
        match self {
            AutomationLocator::Id(id) => format!("id={id:?}"),
            AutomationLocator::Semantic(locator) => locator.describe(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutomationFailure {
    pub code: String,
    pub message: String,
    pub target: Option<String>,
    pub trace_sequence: Option<u64>,
}

impl Display for AutomationFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl Error for AutomationFailure {}

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

#[derive(Clone, Debug)]
pub struct AutomationRun {
    pub report: HeadlessReport,
    pub export: RecordingExport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutomationRuntimeMode {
    Headless,
    DesktopHarness,
}

#[derive(Clone)]
struct ContextBindingsSnapshot {
    query_callback: Option<QueryCallback>,
    bounds_callback: Option<BoundsCallback>,
    focus_callback: Option<FocusCallback>,
    scroll_callback: Option<ScrollCallback>,
    programmatic_event_callback: Option<ProgrammaticEventCallback>,
    element_registry: Option<AnyElementRegistry>,
    focused_element: Option<String>,
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

        let session_guard = AUTOMATION_SESSION_LOCK
            .lock()
            .unwrap_or_else(|err| err.into_inner());
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

    pub fn click(&mut self, locator: AutomationLocator) -> AutomationResult<()> {
        self.record_command("click", &locator, None);
        let resolved = self.resolve_target(&locator)?;
        let target = resolved.target.clone();
        let Some((local_x, local_y, mouse_x, mouse_y)) = self.target_center(&resolved) else {
            return Err(self.fail(
                "target_not_interactable",
                "click target has no bounds",
                target,
                None,
            ));
        };
        let overlays = self.ctx.overlay_manager();
        let overlay_occludes_target = overlays.has_visible_overlays();
        let overlay_blocks_background = overlays.has_blocking_overlay();
        let overlay_handles_backdrop =
            overlay_blocks_background || overlays.has_dismissable_overlay();
        if overlay_occludes_target {
            let overlay_bounds = overlays.get_visible_overlay_bounds();
            let overlay_layer_id = self.overlay_layer_id();
            let point_in_overlay_bounds =
                point_in_overlay_bounds(&overlay_bounds, mouse_x, mouse_y);
            if (overlay_blocks_background && !point_in_overlay_bounds)
                || (overlay_handles_backdrop && overlays.is_backdrop_click(mouse_x, mouse_y))
                || (point_in_overlay_bounds
                    && !self.target_matches_overlay_hit(
                        resolved.node_id,
                        mouse_x,
                        mouse_y,
                        &overlay_bounds,
                        overlay_layer_id,
                    ))
            {
                return Err(self.fail(
                    "target_blocked_by_overlay",
                    "click target is occluded by an active overlay",
                    target,
                    None,
                ));
            }
        }
        blinc_layout::widgets::blur_all_text_inputs();
        let dispatched = self.dispatch_runtime_event(
            &resolved,
            ProgrammaticElementEvent::Click {
                x: local_x,
                y: local_y,
            },
        );
        if !dispatched {
            return Err(self.fail(
                "target_not_interactable",
                "click did not dispatch any runtime events",
                target,
                None,
            ));
        }
        self.after_interaction();
        Ok(())
    }

    pub fn click_at(&mut self, mouse_x: f32, mouse_y: f32) -> AutomationResult<()> {
        self.recording
            .record_trace_entry(TraceEntryKind::Command(TraceCommandRecord {
                name: "click".to_string(),
                target: None,
                payload: Some(format!("x={mouse_x},y={mouse_y}")),
            }));

        let overlays = self.ctx.overlay_manager();
        let overlay_occludes_target = overlays.has_visible_overlays();
        let overlay_blocks_background = overlays.has_blocking_overlay();
        let overlay_handles_backdrop =
            overlay_blocks_background || overlays.has_dismissable_overlay();
        let overlay_bounds = overlay_occludes_target
            .then(|| overlays.get_visible_overlay_bounds())
            .unwrap_or_default();
        let overlay_layer_id = overlay_occludes_target
            .then(|| self.overlay_layer_id())
            .flatten();
        let point_in_overlay_bounds = point_in_overlay_bounds(&overlay_bounds, mouse_x, mouse_y);
        if overlay_occludes_target {
            if overlay_handles_backdrop && overlays.handle_click_at(mouse_x, mouse_y) {
                self.after_interaction();
                return Ok(());
            }
            if (overlay_blocks_background && !point_in_overlay_bounds)
                || (overlay_handles_backdrop && overlays.is_backdrop_click(mouse_x, mouse_y))
            {
                return Err(self.fail(
                    "target_blocked_by_overlay",
                    "click coordinates are occluded by an active overlay",
                    None,
                    None,
                ));
            }
        }

        let hit = if overlay_occludes_target {
            self.ctx.event_router.hit_test_with_occlusion(
                &self.tree,
                mouse_x,
                mouse_y,
                &overlay_bounds,
                overlay_layer_id,
            )
        } else {
            self.ctx.event_router.hit_test(&self.tree, mouse_x, mouse_y)
        };
        let Some(hit) = hit else {
            return Err(self.fail(
                if overlay_occludes_target && (point_in_overlay_bounds || overlay_blocks_background)
                {
                    "target_blocked_by_overlay"
                } else {
                    "target_not_interactable"
                },
                if overlay_occludes_target && (point_in_overlay_bounds || overlay_blocks_background)
                {
                    "click coordinates are occluded by an active overlay"
                } else {
                    "click coordinates did not hit any element"
                },
                None,
                None,
            ));
        };

        blinc_layout::widgets::blur_all_text_inputs();
        let resolved = ResolvedTarget {
            node_id: hit.node,
            target: self.tree.element_registry().get_id(hit.node),
        };
        if !self.dispatch_runtime_event(
            &resolved,
            ProgrammaticElementEvent::Click {
                x: hit.local_x,
                y: hit.local_y,
            },
        ) {
            return Err(self.fail(
                "target_not_interactable",
                "click coordinates did not dispatch any runtime events",
                resolved.target,
                None,
            ));
        }
        self.after_interaction();
        Ok(())
    }

    pub fn fill(&mut self, locator: AutomationLocator, value: &str) -> AutomationResult<()> {
        self.record_command("fill", &locator, Some(redacted_trace_value(value)));
        let resolved = self.resolve_target(&locator)?;
        self.ensure_target_is_unoccluded(&resolved, "fill")?;
        self.ensure_target_focused(&resolved)?;
        let select_all_key = parse_key("A").expect("select-all key should parse");
        let backspace_key = parse_key("Backspace").expect("backspace key should parse");
        self.dispatch_key_event(&resolved, select_all_key.key_code, select_all_modifiers())?;
        self.dispatch_key_event(&resolved, backspace_key.key_code, 0)?;
        for ch in value.chars() {
            self.dispatch_text_input_event(&resolved, ch, 0)?;
        }
        self.after_interaction();
        Ok(())
    }

    pub fn assert_exists(&mut self, locator: AutomationLocator) -> AutomationResult<()> {
        self.record_command("assert_exists", &locator, None);
        match self.resolve_target(&locator) {
            Ok(resolved) => {
                self.record_assertion(
                    "assert_exists",
                    true,
                    resolved.target.as_deref(),
                    None,
                    None,
                );
                Ok(())
            }
            Err(mut failure) => {
                failure.trace_sequence = Some(self.record_assertion(
                    "assert_exists",
                    false,
                    failure.target.as_deref(),
                    None,
                    Some(locator.describe()),
                ));
                Err(failure)
            }
        }
    }

    pub fn assert_text_contains(
        &mut self,
        locator: AutomationLocator,
        expected: &str,
    ) -> AutomationResult<()> {
        self.record_command("assert_text_contains", &locator, Some(expected.to_string()));
        let resolved = self.resolve_target(&locator)?;
        let actual = subtree_text(&self.tree, resolved.node_id).unwrap_or_default();
        if text_contains(&actual, expected) {
            self.record_assertion(
                "assert_text_contains",
                true,
                resolved.target.as_deref(),
                Some(actual),
                Some(expected.to_string()),
            );
            Ok(())
        } else {
            let trace_sequence = self.record_assertion(
                "assert_text_contains",
                false,
                resolved.target.as_deref(),
                Some(actual.clone()),
                Some(expected.to_string()),
            );
            Err(AutomationFailure {
                code: "assertion_failed".to_string(),
                message: format!("expected text containing {expected:?}, got {actual:?}"),
                target: resolved.target,
                trace_sequence: Some(trace_sequence),
            })
        }
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

    pub fn press(&mut self, key: &str) -> AutomationResult<()> {
        self.recording
            .record_trace_entry(TraceEntryKind::Command(TraceCommandRecord {
                name: "press".to_string(),
                target: Some(key.to_string()),
                payload: None,
            }));
        let parsed = parse_key(key).ok_or_else(|| {
            self.fail(
                "unsupported_key",
                &format!("unsupported key {key:?}"),
                None,
                None,
            )
        })?;
        if parsed.key_code == 27 && self.ctx.overlay_manager().handle_escape() {
            self.after_interaction();
            return Ok(());
        }
        let Some(node_id) = self.ctx.event_router.focused() else {
            return Err(self.fail(
                "focus_required",
                "press requires a focused element",
                None,
                None,
            ));
        };
        let resolved = ResolvedTarget {
            node_id,
            target: self.tree.element_registry().get_id(node_id),
        };
        self.ensure_target_is_unoccluded(&resolved, "press")?;
        self.dispatch_key_down_event(&resolved, parsed.key_code, parsed.modifiers)?;
        if let Some(text) = parsed.text.filter(|_| parsed.modifiers & 0b1010 == 0) {
            self.dispatch_text_input_event(&resolved, text, parsed.modifiers)?;
        }
        self.dispatch_key_up_event(&resolved, parsed.key_code, parsed.modifiers)?;
        self.after_interaction();
        Ok(())
    }

    pub fn scroll(
        &mut self,
        locator: Option<AutomationLocator>,
        dx: f32,
        dy: f32,
    ) -> AutomationResult<()> {
        self.recording
            .record_trace_entry(TraceEntryKind::Command(TraceCommandRecord {
                name: "scroll".to_string(),
                target: locator.as_ref().map(AutomationLocator::describe),
                payload: Some(format!("dx={dx},dy={dy}")),
            }));
        let resolved = match locator {
            Some(locator) => self.resolve_target(&locator)?,
            None => {
                let Some(node_id) = self.ctx.event_router.focused() else {
                    return Err(self.fail(
                        "scroll_target_required",
                        "scroll without an id requires a focused element",
                        None,
                        None,
                    ));
                };
                ResolvedTarget {
                    node_id,
                    target: None,
                }
            }
        };
        let Some(bounds) = self.tree.get_absolute_bounds(resolved.node_id) else {
            return Err(self.fail(
                "target_not_interactable",
                "scroll target has no bounds",
                resolved.target,
                None,
            ));
        };
        let local_x = bounds.width * 0.5;
        let local_y = bounds.height * 0.5;
        let mouse_x = bounds.x + local_x;
        let mouse_y = bounds.y + local_y;
        let overlays = self.ctx.overlay_manager();
        let overlay_occludes_target = overlays.has_visible_overlays();
        let overlay_blocks_background_scroll = overlays.has_blocking_overlay();
        if overlay_occludes_target {
            let overlay_bounds = overlays.get_visible_overlay_bounds();
            let overlay_layer_id = self.overlay_layer_id();
            let point_in_overlay_bounds =
                point_in_overlay_bounds(&overlay_bounds, mouse_x, mouse_y);
            if (overlay_blocks_background_scroll && !point_in_overlay_bounds)
                || (point_in_overlay_bounds
                    && !self.target_matches_overlay_hit(
                        resolved.node_id,
                        mouse_x,
                        mouse_y,
                        &overlay_bounds,
                        overlay_layer_id,
                    ))
            {
                return Err(self.fail(
                    "target_blocked_by_overlay",
                    "scroll target is occluded by an active overlay",
                    resolved.target,
                    None,
                ));
            }
        }
        let overlay_effect = self.handle_overlay_scroll(dy);
        let dispatched =
            self.dispatch_runtime_event(&resolved, ProgrammaticElementEvent::Scroll { dx, dy });
        if !dispatched && !overlay_effect {
            return Err(self.fail(
                "target_not_interactable",
                &format!(
                    "scroll target at ({:.1}, {:.1}, {:.1}, {:.1}) did not dispatch",
                    bounds.x, bounds.y, bounds.width, bounds.height
                ),
                resolved.target,
                None,
            ));
        }
        self.after_scroll_interaction();
        Ok(())
    }

    pub fn tick_frames(&mut self, frames: u32) -> Result<()> {
        if frames == 0 {
            return Ok(());
        }
        let probe_every = self.runtime_cfg.probe_every_frames.max(1);
        let mut current_time = self.last_frame_time_ms;
        for frame_index in 0..frames {
            if let Ok(animations) = self.ctx.animations.lock() {
                let _ = animations.tick();
            }
            current_time = current_time.saturating_add(self.frame_step_ms());
            let should_sample = (frame_index + 1) % probe_every == 0 || frame_index + 1 == frames;
            self.advance_runtime_frame(current_time, should_sample, false);
        }
        Ok(())
    }

    pub fn write_snapshot_to_path(&self, path: &std::path::Path) -> Result<()> {
        let Some(snapshot) = self.latest_snapshot.as_ref() else {
            bail!("no snapshot captured yet");
        };
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(path, serde_json::to_string_pretty(snapshot)?)?;
        self.recording
            .record_trace_entry(TraceEntryKind::Artifact(TraceArtifactRecord {
                kind: "snapshot_export".to_string(),
                path: Some(path.display().to_string()),
                message: Some("wrote snapshot".to_string()),
            }));
        Ok(())
    }

    pub fn write_trace_to_path(&self, path: &std::path::Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let artifact = TraceArtifactRecord {
            kind: "trace_export".to_string(),
            path: Some(path.display().to_string()),
            message: Some("wrote trace".to_string()),
        };
        let artifact_entry = self
            .recording
            .prepare_trace_entry(TraceEntryKind::Artifact(artifact.clone()));
        let mut export = self.export_recording();
        if let Some(entry) = artifact_entry.clone() {
            export.trace_entries.push(entry);
        }
        std::fs::write(path, serde_json::to_string_pretty(&export)?)?;
        if let Some(entry) = artifact_entry {
            let _ = self.recording.append_trace_entry(entry);
        }
        Ok(())
    }

    fn resolve_target(&mut self, locator: &AutomationLocator) -> AutomationResult<ResolvedTarget> {
        match locator {
            AutomationLocator::Id(id) => {
                let trace_sequence =
                    self.recording
                        .record_trace_entry(TraceEntryKind::LocatorResolution(
                            TraceLocatorResolution {
                                query: format!("id={id:?}"),
                                matched_target: self.tree.query_by_id(id).map(|_| id.clone()),
                                candidate_targets: self
                                    .tree
                                    .query_by_id(id)
                                    .map(|_| vec![id.clone()])
                                    .unwrap_or_default(),
                                failure_reason: self
                                    .tree
                                    .query_by_id(id)
                                    .is_none()
                                    .then(|| "no_match".to_string()),
                            },
                        ));
                let Some(node_id) = self.tree.query_by_id(id) else {
                    return Err(AutomationFailure {
                        code: "locator_not_found".to_string(),
                        message: format!("no element found for id {id:?}"),
                        target: Some(id.clone()),
                        trace_sequence: Some(trace_sequence),
                    });
                };
                Ok(ResolvedTarget {
                    node_id,
                    target: Some(id.clone()),
                })
            }
            AutomationLocator::Semantic(locator) => {
                let resolution = resolve_semantic_locator(&self.tree, locator);
                let Some(node_id) = resolution.matched_node_id else {
                    return Err(AutomationFailure {
                        code: resolution
                            .failure_reason
                            .clone()
                            .unwrap_or_else(|| "locator_resolution_failed".to_string()),
                        message: format!("semantic locator failed: {}", resolution.query),
                        target: resolution.matched_target.clone(),
                        trace_sequence: None,
                    });
                };
                Ok(ResolvedTarget {
                    node_id,
                    target: resolution.matched_target,
                })
            }
        }
    }

    fn target_center(&self, resolved: &ResolvedTarget) -> Option<(f32, f32, f32, f32)> {
        let bounds = self.tree.get_absolute_bounds(resolved.node_id)?;
        let local_x = bounds.width * 0.5;
        let local_y = bounds.height * 0.5;
        Some((local_x, local_y, bounds.x + local_x, bounds.y + local_y))
    }

    fn ensure_target_is_unoccluded(
        &self,
        resolved: &ResolvedTarget,
        action: &str,
    ) -> AutomationResult<()> {
        let Some((_, _, mouse_x, mouse_y)) = self.target_center(resolved) else {
            return Err(self.fail(
                "target_not_interactable",
                &format!("{action} target has no bounds"),
                resolved.target.clone(),
                None,
            ));
        };

        let overlays = self.ctx.overlay_manager();
        if !overlays.has_visible_overlays() {
            return Ok(());
        }

        let overlay_bounds = overlays.get_visible_overlay_bounds();
        let overlay_layer_id = self.overlay_layer_id();
        let point_in_overlay_bounds = point_in_overlay_bounds(&overlay_bounds, mouse_x, mouse_y);
        if (overlays.has_blocking_overlay() && !point_in_overlay_bounds)
            || (point_in_overlay_bounds
                && !self.target_matches_overlay_hit(
                    resolved.node_id,
                    mouse_x,
                    mouse_y,
                    &overlay_bounds,
                    overlay_layer_id,
                ))
        {
            return Err(self.fail(
                "target_blocked_by_overlay",
                &format!("{action} target is occluded by an active overlay"),
                resolved.target.clone(),
                None,
            ));
        }

        Ok(())
    }

    fn overlay_layer_id(&self) -> Option<blinc_layout::tree::LayoutNodeId> {
        self.tree
            .query_by_id(blinc_layout::widgets::overlay::OVERLAY_LAYER_ID)
    }

    fn target_matches_overlay_hit(
        &self,
        node_id: blinc_layout::tree::LayoutNodeId,
        x: f32,
        y: f32,
        overlay_bounds: &[(f32, f32, f32, f32)],
        overlay_layer_id: Option<blinc_layout::tree::LayoutNodeId>,
    ) -> bool {
        let Some(hit) = self.ctx.event_router.hit_test_with_occlusion(
            &self.tree,
            x,
            y,
            overlay_bounds,
            overlay_layer_id,
        ) else {
            return false;
        };

        if hit.node == node_id || hit.ancestors.contains(&node_id) {
            return true;
        }

        let root_id = self.tree.root();
        self.tree
            .element_registry()
            .ancestors(node_id)
            .into_iter()
            .any(|ancestor| Some(ancestor) != root_id && ancestor == hit.node)
    }

    fn handle_overlay_scroll(&mut self, delta_y: f32) -> bool {
        let updated = self.ctx.overlay_manager().handle_scroll(delta_y);
        if updated {
            self.sync_overlay_scroll_offsets();
        }
        updated
    }

    fn sync_overlay_scroll_offsets(&mut self) {
        let overlays = self.ctx.overlay_manager();
        for (element_id, offset_y) in overlays.get_scroll_offsets() {
            if let Some(node_id) = self.tree.query_by_id(&element_id) {
                self.tree.set_scroll_offset(node_id, 0.0, offset_y);
            }
        }
    }

    fn ensure_target_focused(&mut self, resolved: &ResolvedTarget) -> AutomationResult<()> {
        if self.ctx.event_router.focused() == Some(resolved.node_id) {
            return Ok(());
        }

        if sync_focus_node_to_runtime(
            &mut self.tree,
            &mut self.ctx.event_router,
            Some(resolved.node_id),
        ) {
            return Ok(());
        }

        Err(self.fail(
            "focus_dispatch_failed",
            "target could not be focused through the runtime",
            resolved.target.clone(),
            None,
        ))
    }

    fn dispatch_key_event(
        &mut self,
        resolved: &ResolvedTarget,
        key: u32,
        modifiers: u8,
    ) -> AutomationResult<()> {
        self.dispatch_key_down_event(resolved, key, modifiers)?;
        self.dispatch_key_up_event(resolved, key, modifiers)
    }

    fn dispatch_key_down_event(
        &mut self,
        resolved: &ResolvedTarget,
        key: u32,
        modifiers: u8,
    ) -> AutomationResult<()> {
        if self.dispatch_runtime_event(
            resolved,
            ProgrammaticElementEvent::KeyDown { key, modifiers },
        ) {
            Ok(())
        } else {
            Err(self.fail(
                "key_dispatch_failed",
                "key press did not dispatch to the runtime",
                resolved.target.clone(),
                None,
            ))
        }
    }

    fn dispatch_key_up_event(
        &mut self,
        resolved: &ResolvedTarget,
        key: u32,
        modifiers: u8,
    ) -> AutomationResult<()> {
        if self.dispatch_runtime_event(resolved, ProgrammaticElementEvent::KeyUp { key, modifiers })
        {
            Ok(())
        } else {
            Err(self.fail(
                "key_dispatch_failed",
                "key release did not dispatch to the runtime",
                resolved.target.clone(),
                None,
            ))
        }
    }

    fn dispatch_text_input_event(
        &mut self,
        resolved: &ResolvedTarget,
        text: char,
        modifiers: u8,
    ) -> AutomationResult<()> {
        if self.dispatch_runtime_event(
            resolved,
            ProgrammaticElementEvent::TextInput { text, modifiers },
        ) {
            Ok(())
        } else {
            Err(self.fail(
                "text_input_dispatch_failed",
                "text input did not dispatch to the runtime",
                resolved.target.clone(),
                None,
            ))
        }
    }

    fn dispatch_runtime_event(
        &mut self,
        resolved: &ResolvedTarget,
        event: ProgrammaticElementEvent,
    ) -> bool {
        dispatch_programmatic_event_to_node(
            &mut self.tree,
            &mut self.ctx.event_router,
            resolved.node_id,
            event,
        )
    }

    fn apply_pending_runtime_requests(&mut self) -> bool {
        drain_programmatic_runtime_requests(
            &mut self.tree,
            &mut self.ctx.event_router,
            &self.pending_focus_changes,
            &self.pending_scroll_requests,
            &self.pending_programmatic_events,
        )
    }

    fn after_interaction(&mut self) {
        self.mark_visible_overlay_content_dirty();
        let current_time = self.last_frame_time_ms.saturating_add(self.frame_step_ms());
        self.advance_runtime_frame(current_time, true, true);
    }

    fn after_scroll_interaction(&mut self) {
        let current_time = self.last_frame_time_ms.saturating_add(self.frame_step_ms());
        self.advance_runtime_frame(current_time, true, false);
    }

    fn advance_runtime_frame(
        &mut self,
        current_time: u64,
        capture_snapshot: bool,
        force_rebuild: bool,
    ) {
        self.prepare_runtime_frame(current_time);

        let _ = self.apply_pending_runtime_requests();
        let _ = self.tree.tick_scroll_physics(current_time);
        self.tree.process_pending_scroll_refs();
        self.apply_stateful_updates();
        self.rebuild_runtime_tree(force_rebuild);
        self.finalize_runtime_frame(current_time);

        if let Some(focused_id) = BlincContextState::try_get().and_then(|ctx| ctx.focused_element())
        {
            let runtime_focused_id = self
                .ctx
                .event_router
                .focused()
                .and_then(|node_id| self.tree.element_registry().get_id(node_id));
            if runtime_focused_id.as_deref() != Some(focused_id.as_str()) {
                sync_focus_to_runtime(
                    &mut self.tree,
                    &mut self.ctx.event_router,
                    Some(&focused_id),
                );
            }
        } else {
            let text_focus = blinc_layout::widgets::text_input::focused_text_input_node_id()
                .or_else(blinc_layout::widgets::text_input::focused_text_area_node_id);
            if text_focus != self.ctx.event_router.focused() {
                self.ctx.event_router.set_focus(text_focus);
            }
        }
        sync_context_focus_from_runtime(&self.tree, &self.ctx.event_router);
        if capture_snapshot {
            self.capture_snapshot();
        }
    }

    fn prepare_runtime_frame(&mut self, current_time: u64) {
        if blinc_layout::widgets::take_needs_css_reparse() {
            self.ctx.reparse_css();
        }

        self.render_state.process_global_motion_exit_starts();
        self.render_state.process_global_motion_exit_cancels();
        self.render_state.process_global_motion_starts();
        self.render_state.sync_shared_motion_states();

        self.ctx.prepare_windowless_frame(current_time);
        let overlay_content_dirty = self.ctx.overlay_manager().is_dirty();
        if overlay_content_dirty {
            if let Some(overlay_node_id) = self
                .element_registry
                .get(blinc_layout::widgets::overlay::OVERLAY_LAYER_ID)
            {
                let overlay_content = self.ctx.overlay_manager().build_overlay_layer();
                blinc_layout::queue_subtree_rebuild(overlay_node_id, overlay_content);
            }
            let _ = self.ctx.overlay_manager().take_dirty();
        }
    }

    fn apply_stateful_updates(&mut self) {
        let has_stateful_updates = blinc_layout::take_needs_redraw();
        let has_pending_rebuilds = blinc_layout::has_pending_subtree_rebuilds();
        if !has_stateful_updates && !has_pending_rebuilds {
            return;
        }

        let prop_updates = blinc_layout::take_pending_prop_updates();
        for (node_id, props) in &prop_updates {
            self.tree
                .update_render_props(*node_id, |render_props| *render_props = props.clone());
        }

        if self.tree.process_pending_subtree_rebuilds() {
            self.tree.apply_stylesheet_layout_overrides();
            self.tree.compute_layout(
                self.runtime_cfg.width as f32,
                self.runtime_cfg.height as f32,
            );
            self.render_state.begin_stable_motion_frame();
            self.tree
                .initialize_motion_animations(&mut self.render_state);
            self.render_state.end_stable_motion_frame();
            self.render_state.process_global_motion_replays();
            self.tree.start_all_css_animations();
        }
    }

    fn rebuild_runtime_tree(&mut self, force_rebuild: bool) {
        let needs_rebuild = force_rebuild
            || self.tree.needs_rebuild()
            || self.ref_dirty_flag.swap(false, Ordering::SeqCst)
            || blinc_layout::widgets::take_needs_rebuild();
        let needs_relayout = force_rebuild || blinc_layout::widgets::take_needs_relayout();

        self.render_state.begin_stable_motion_frame();
        if !needs_rebuild {
            self.tree
                .initialize_motion_animations(&mut self.render_state);
            self.render_state.end_stable_motion_frame();
            return;
        }

        blinc_layout::reset_call_counters();
        blinc_layout::clear_stateful_base_updaters();
        blinc_layout::click_outside::clear_click_outside_handlers();
        self.render_state.reset_stable_motions_for_rebuild();

        let user_ui = (self.ui_builder)(&mut self.ctx);
        let ui = self.ctx.compose_runtime_ui(user_ui);

        if let Some(ref stylesheet) = self.ctx.stylesheet {
            self.tree.set_stylesheet_arc(stylesheet.clone());
        }

        if needs_relayout {
            let mut tree = blinc_layout::RenderTree::from_element_with_registry(
                &ui,
                Arc::clone(&self.element_registry),
            );
            tree.set_animations(&self.ctx.animations);
            tree.set_css_anim_store(Arc::clone(&self.css_anim_store));
            tree.set_scale_factor(self.ctx.scale_factor as f32);
            if let Some(ref stylesheet) = self.ctx.stylesheet {
                tree.set_stylesheet_arc(stylesheet.clone());
            }
            tree.apply_all_stylesheet_styles();
            tree.compute_layout(
                self.runtime_cfg.width as f32,
                self.runtime_cfg.height as f32,
            );
            tree.transfer_scroll_state_from(&self.tree);
            tree.initialize_motion_animations(&mut self.render_state);
            self.render_state.end_stable_motion_frame();
            self.render_state.process_global_motion_replays();
            tree.start_all_css_animations();
            self.tree = tree;
        } else {
            match self.tree.incremental_update(&ui) {
                UpdateResult::NoChanges | UpdateResult::VisualOnly => {
                    self.tree
                        .initialize_motion_animations(&mut self.render_state);
                    self.render_state.end_stable_motion_frame();
                }
                UpdateResult::LayoutChanged => {
                    self.tree.apply_stylesheet_layout_overrides();
                    self.tree.compute_layout(
                        self.runtime_cfg.width as f32,
                        self.runtime_cfg.height as f32,
                    );
                    self.tree.restore_bound_scroll_ref_offsets();
                    self.tree
                        .initialize_motion_animations(&mut self.render_state);
                    self.render_state.end_stable_motion_frame();
                }
                UpdateResult::ChildrenChanged => {
                    self.tree.apply_stylesheet_base_styles();
                    self.tree.apply_stylesheet_layout_overrides();
                    self.tree.compute_layout(
                        self.runtime_cfg.width as f32,
                        self.runtime_cfg.height as f32,
                    );
                    self.tree.restore_bound_scroll_ref_offsets();
                    self.tree
                        .initialize_motion_animations(&mut self.render_state);
                    self.render_state.end_stable_motion_frame();
                    self.render_state.process_global_motion_replays();
                    self.tree.start_all_css_animations();
                }
            }
        }

        self.ctx.finish_runtime_rebuild();
        self.sync_overlay_scroll_offsets();
        self.tree.process_pending_scroll_refs();
    }

    fn mark_visible_overlay_content_dirty(&self) {
        self.ctx.overlay_manager().mark_content_dirty();
    }

    fn finalize_runtime_frame(&mut self, current_time: u64) {
        self.render_state.process_global_motion_exit_cancels();
        self.render_state.process_global_motion_exit_starts();
        self.render_state.process_global_motion_starts();
        let _ = self.render_state.tick(current_time);

        let dt_ms = frame_delta_ms(current_time, self.last_frame_time_ms);
        let css_active = {
            let store = self.tree.css_anim_store();
            let mut animations = store.lock().unwrap();
            let (animating, transitioning) = animations.tick(dt_ms);
            drop(animations);
            animating || transitioning || self.tree.css_has_active()
        };
        self.last_frame_time_ms = current_time;
        self.render_state.sync_shared_motion_states();
        let _ = blinc_theme::ThemeState::get().tick();

        if self.tree.stylesheet().is_some() {
            let state_changed = self
                .tree
                .apply_stylesheet_state_styles(&self.ctx.event_router);
            if state_changed {
                self.tree.compute_layout(
                    self.runtime_cfg.width as f32,
                    self.runtime_cfg.height as f32,
                );
            }
        }
        sync_context_focus_from_runtime(&self.tree, &self.ctx.event_router);

        if css_active || !self.tree.css_transitions_empty() {
            self.tree.apply_all_css_animation_props();
            self.tree.apply_all_css_transition_props();
            if self.tree.apply_animated_layout_props() {
                self.tree.compute_layout(
                    self.runtime_cfg.width as f32,
                    self.runtime_cfg.height as f32,
                );
            }
        }
        self.sync_overlay_scroll_offsets();
        self.tree.process_pending_scroll_refs();
    }

    fn capture_snapshot(&mut self) {
        let hovered_nodes = self
            .ctx
            .event_router
            .hovered_nodes()
            .collect::<std::collections::HashSet<_>>();
        let snapshot = capture_tree_snapshot(
            &self.tree,
            self.ctx.event_router.focused(),
            &hovered_nodes,
            self.runtime_cfg.width,
            self.runtime_cfg.height,
        );
        let snapshot = to_tree_snapshot(snapshot);
        self.recording.record_snapshot(snapshot.clone());
        self.latest_snapshot = Some(snapshot);
    }

    fn frame_step_ms(&self) -> u64 {
        u64::from(self.runtime_cfg.tick_ms.max(1))
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

pub fn run_headless_scenario<F, E>(
    runtime_cfg: HeadlessRunConfig,
    scenario: &HeadlessScenario,
    ui_builder: F,
) -> Result<AutomationRun>
where
    F: FnMut(&mut WindowedContext) -> E,
    E: blinc_layout::ElementBuilder + 'static,
{
    run_scenario_with_mode(
        AutomationRuntimeMode::Headless,
        runtime_cfg,
        scenario,
        ui_builder,
    )
}

pub fn run_desktop_harness_scenario<F, E>(
    runtime_cfg: HeadlessRunConfig,
    scenario: &HeadlessScenario,
    ui_builder: F,
) -> Result<AutomationRun>
where
    F: FnMut(&mut WindowedContext) -> E,
    E: blinc_layout::ElementBuilder + 'static,
{
    run_scenario_with_mode(
        AutomationRuntimeMode::DesktopHarness,
        runtime_cfg,
        scenario,
        ui_builder,
    )
}

fn run_scenario_with_mode<F, E>(
    runtime_mode: AutomationRuntimeMode,
    runtime_cfg: HeadlessRunConfig,
    scenario: &HeadlessScenario,
    ui_builder: F,
) -> Result<AutomationRun>
where
    F: FnMut(&mut WindowedContext) -> E,
    E: blinc_layout::ElementBuilder + 'static,
{
    let mut session = match runtime_mode {
        AutomationRuntimeMode::Headless => AutomationSession::new_headless(runtime_cfg, ui_builder),
        AutomationRuntimeMode::DesktopHarness => {
            AutomationSession::new_desktop_harness(runtime_cfg, ui_builder)
        }
    };
    let mut elapsed_frames: u64 = 0;
    let mut elapsed_ms: u64 = 0;

    for (step_index, step) in scenario.steps.iter().enumerate() {
        let result = match step {
            ScenarioStep::Wait { ms } => {
                let frames = wait_frames(*ms, runtime_cfg.tick_ms);
                session.tick_frames(frames)?;
                elapsed_frames += frames as u64;
                elapsed_ms += ms;
                Ok(())
            }
            ScenarioStep::Tick { frames } => {
                session.tick_frames(*frames)?;
                elapsed_frames += *frames as u64;
                elapsed_ms += runtime_cfg.tick_ms.saturating_mul(*frames as u64);
                Ok(())
            }
            ScenarioStep::Click { target, x, y } => match (x, y, target.is_empty()) {
                (Some(x), Some(y), true) => session.click_at(*x, *y),
                (None, None, _) => session.click(automation_locator_from_target(target)?),
                _ => Err(AutomationFailure {
                    code: "invalid_locator".to_string(),
                    message: "coordinate click steps require both x and y without locator fields"
                        .to_string(),
                    target: None,
                    trace_sequence: None,
                }),
            },
            ScenarioStep::Fill { target, value } => {
                session.fill(automation_locator_from_target(target)?, value)
            }
            ScenarioStep::Press { key } => session.press(key),
            ScenarioStep::Scroll { target, dx, dy } => {
                let locator = if target.id.is_some() || target.has_semantic_fields() {
                    Some(automation_locator_from_target(target)?)
                } else {
                    None
                };
                session.scroll(locator, *dx, *dy)
            }
            ScenarioStep::Snapshot { path } => {
                if let Some(path) = path.as_deref() {
                    session.write_snapshot_to_path(std::path::Path::new(path))?;
                }
                Ok(())
            }
            ScenarioStep::ExportTrace { path } => {
                if let Some(path) = path.as_deref() {
                    session.write_trace_to_path(std::path::Path::new(path))?;
                }
                Ok(())
            }
            ScenarioStep::AssertExists { target } => {
                session.assert_exists(automation_locator_from_target(target)?)
            }
            ScenarioStep::AssertTextContains { target, value } => {
                session.assert_text_contains(automation_locator_from_target(target)?, value)
            }
        };

        if let Err(failure) = result {
            session.recording.stop();
            return Ok(AutomationRun {
                report: HeadlessReport::failed(
                    &failure.code,
                    step_index,
                    failure.message,
                    elapsed_frames,
                    elapsed_ms,
                ),
                export: session.export_recording(),
            });
        }
    }

    session.recording.stop();
    Ok(AutomationRun {
        report: HeadlessReport::passed(elapsed_frames, elapsed_ms),
        export: session.export_recording(),
    })
}

fn shared_state_for_automation(
    _runtime_mode: AutomationRuntimeMode,
) -> (
    SharedReactiveGraph,
    SharedHookState,
    RefDirtyFlag,
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

fn snapshot_context_bindings(ctx: &BlincContextState) -> ContextBindingsSnapshot {
    ContextBindingsSnapshot {
        query_callback: ctx.query_callback(),
        bounds_callback: ctx.bounds_callback(),
        focus_callback: ctx.focus_callback(),
        scroll_callback: ctx.scroll_callback(),
        programmatic_event_callback: ctx.programmatic_event_callback(),
        element_registry: ctx.element_registry_any(),
        focused_element: ctx.focused_element(),
    }
}

fn configure_context_callbacks(
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

struct ParsedKey {
    key_code: u32,
    modifiers: u8,
    text: Option<char>,
}

fn parse_key(input: &str) -> Option<ParsedKey> {
    let normalized = input.trim();
    match normalized {
        "Enter" => Some(ParsedKey {
            key_code: 13,
            modifiers: 0,
            text: None,
        }),
        "Tab" => Some(ParsedKey {
            key_code: 9,
            modifiers: 0,
            text: None,
        }),
        "Escape" => Some(ParsedKey {
            key_code: 27,
            modifiers: 0,
            text: None,
        }),
        "Backspace" => Some(ParsedKey {
            key_code: 8,
            modifiers: 0,
            text: None,
        }),
        "Delete" => Some(ParsedKey {
            key_code: 127,
            modifiers: 0,
            text: None,
        }),
        "ArrowLeft" => Some(ParsedKey {
            key_code: 37,
            modifiers: 0,
            text: None,
        }),
        "ArrowRight" => Some(ParsedKey {
            key_code: 39,
            modifiers: 0,
            text: None,
        }),
        "ArrowUp" => Some(ParsedKey {
            key_code: 38,
            modifiers: 0,
            text: None,
        }),
        "ArrowDown" => Some(ParsedKey {
            key_code: 40,
            modifiers: 0,
            text: None,
        }),
        _ if normalized.chars().count() == 1 => Some(ParsedKey {
            key_code: normalized.chars().next()? as u32,
            modifiers: 0,
            text: normalized.chars().next(),
        }),
        _ => None,
    }
}

fn parse_accessibility_role(input: &str) -> Option<AccessibilityRole> {
    match input.trim().to_ascii_lowercase().as_str() {
        "window" => Some(AccessibilityRole::Window),
        "group" => Some(AccessibilityRole::Group),
        "label" => Some(AccessibilityRole::Label),
        "button" => Some(AccessibilityRole::Button),
        "checkbox" => Some(AccessibilityRole::Checkbox),
        "text_input" | "textinput" | "textbox" => Some(AccessibilityRole::TextInput),
        "text_area" | "textarea" => Some(AccessibilityRole::TextArea),
        "image" => Some(AccessibilityRole::Image),
        _ => None,
    }
}

fn automation_locator_from_target(target: &ScenarioTarget) -> AutomationResult<AutomationLocator> {
    if let Some(id) = target.id.as_ref() {
        return Ok(AutomationLocator::id(id.clone()));
    }

    if !target.has_semantic_fields() {
        return Err(AutomationFailure {
            code: "invalid_locator".to_string(),
            message: "scenario step requires id or semantic locator fields".to_string(),
            target: None,
            trace_sequence: None,
        });
    }

    let semantic = &target.semantic;
    let mut locator = if let Some(role) = semantic.role.as_deref() {
        let Some(role) = parse_accessibility_role(role) else {
            return Err(AutomationFailure {
                code: "invalid_locator".to_string(),
                message: format!("unsupported accessibility role {role:?}"),
                target: None,
                trace_sequence: None,
            });
        };
        SemanticLocator::role(role)
    } else {
        SemanticLocator::default()
    };

    if let Some(text) = semantic.text.as_deref() {
        locator = locator.with_text(text);
    }
    if let Some(label) = semantic.label.as_deref() {
        locator = locator.with_label(label);
    }
    if let Some(placeholder) = semantic.placeholder.as_deref() {
        locator = locator.with_placeholder(placeholder);
    }
    if let Some(tag) = semantic.tag.as_deref() {
        locator = locator.with_tag(tag);
    }
    if let Some(within) = semantic.within.as_deref() {
        locator = locator.within(within);
    }
    if let Some(nth) = semantic.nth {
        locator = locator.nth(nth);
    }

    Ok(AutomationLocator::semantic(locator))
}

fn select_all_modifiers() -> u8 {
    if cfg!(target_os = "macos") {
        0b1000
    } else {
        0b0010
    }
}

fn wait_frames(ms: u64, tick_ms: u64) -> u32 {
    if ms == 0 {
        return 1;
    }
    let frames = (ms + tick_ms.saturating_sub(1)) / tick_ms.max(1);
    frames.max(1) as u32
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
