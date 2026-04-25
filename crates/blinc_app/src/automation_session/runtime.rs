use super::*;

impl<F, E> AutomationSession<F, E>
where
    F: FnMut(&mut WindowedContext) -> E,
    E: blinc_layout::ElementBuilder + 'static,
{
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

    fn apply_pending_runtime_requests(&mut self) -> bool {
        drain_programmatic_runtime_requests(
            &mut self.tree,
            &mut self.ctx.event_router,
            &self.pending_focus_changes,
            &self.pending_scroll_requests,
            &self.pending_programmatic_events,
        )
    }

    pub(super) fn after_interaction(&mut self) {
        self.mark_visible_overlay_content_dirty();
        let current_time = self.last_frame_time_ms.saturating_add(self.frame_step_ms());
        self.advance_runtime_frame(current_time, true, true);
    }

    pub(super) fn after_scroll_interaction(&mut self) {
        let current_time = self.last_frame_time_ms.saturating_add(self.frame_step_ms());
        self.advance_runtime_frame(current_time, true, false);
    }

    pub(super) fn advance_runtime_frame(
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
            tree.transfer_scroll_offsets_from(&self.tree);
            tree.transfer_scroll_physics_from(&self.tree);
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
                    self.tree.process_pending_scroll_refs();
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
                    self.tree.process_pending_scroll_refs();
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

    pub(super) fn capture_snapshot(&mut self) {
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
}
