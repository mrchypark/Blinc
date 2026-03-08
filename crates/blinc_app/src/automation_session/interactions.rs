use super::*;

const MODIFIER_CTRL: u8 = 0b0010;
const MODIFIER_META: u8 = 0b1000;
const TEXT_INPUT_SHORTCUT_MODIFIERS: u8 = MODIFIER_CTRL | MODIFIER_META;

impl<F, E> AutomationSession<F, E>
where
    F: FnMut(&mut WindowedContext) -> E,
    E: blinc_layout::ElementBuilder + 'static,
{
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
        let select_all_key = parse_key("A").ok_or_else(|| {
            self.fail(
                "internal_error",
                "could not parse select-all key",
                resolved.target.clone(),
                None,
            )
        })?;
        let backspace_key = parse_key("Backspace").ok_or_else(|| {
            self.fail(
                "internal_error",
                "could not parse backspace key",
                resolved.target.clone(),
                None,
            )
        })?;
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
        if let Some(text) = parsed
            .text
            .filter(|_| parsed.modifiers & TEXT_INPUT_SHORTCUT_MODIFIERS == 0)
        {
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

    pub(super) fn sync_overlay_scroll_offsets(&mut self) {
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
}
