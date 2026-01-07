# Overlay System Architecture Redesign

## Problem Statement

The current overlay system uses a **separate render tree** from the main UI, which causes:

1. **Visual updates don't work** - Hover/press state changes in overlays queue subtree rebuilds to the wrong tree
2. **Motion animation jitter** - Full overlay tree rebuilds reset motion animations (new InstanceKeys)
3. **Triggers full UI rebuild** - Opening/closing overlays can trigger main tree rebuilds unnecessarily
4. **Complex event routing** - Two separate EventRouters with complex coordination logic
5. **State synchronization issues** - Overlay content captures state in closures, doesn't track reactive changes

## Design Goals

1. **Never trigger full UI rebuild** - Overlay changes should only affect overlay content
2. **Incremental visual updates** - Hover/press states update without rebuilding
3. **Smooth motion animations** - No jitter on open/close, stable InstanceKeys
4. **Unified event routing** - Single hit-test tree, simpler event dispatch
5. **Reactive content** - Overlay content responds to signal/state changes naturally

---

## Proposed Architecture: Unified Tree with Overlay Layer

### Core Concept

Instead of a separate overlay tree, overlays become **children of a root Stack** in the main UI tree:

```
Root Stack (viewport-sized)
├── Main UI Content (user's UI)
└── Overlay Layer (managed by OverlayManager)
    ├── Overlay 1 (backdrop + content)
    ├── Overlay 2 (backdrop + content)
    └── ...
```

### Key Changes

#### 1. Overlay Content as Part of Main Tree

**Current:**
```rust
// windowed.rs - Two separate trees
let ui = ui_builder(ctx);                    // Main tree
let overlay_tree = overlay_manager.build_overlay_tree();  // Separate tree
```

**Proposed:**
```rust
// windowed.rs - Single tree with overlay layer
let user_ui = ui_builder(ctx);
let root = stack()
    .w(ctx.width).h(ctx.height)
    .child(user_ui)
    .child(ctx.overlay_layer());  // Returns Div with all overlays
```

#### 2. OverlayManager Changes

**New method: `build_overlay_layer() -> Option<Div>`**
- Returns a Div containing all visible overlays (not a RenderTree)
- Each overlay wrapped in its own container with backdrop + positioning
- Returns `None` when no visible overlays (no extra node in tree)

**Remove: `build_overlay_tree() -> Option<RenderTree>`**
- No longer needed - overlays are part of main tree

**New: Incremental overlay updates**
- Track which overlays changed (added/removed/state changed)
- Use subtree rebuild mechanism for overlay content changes
- Visual-only updates (hover) use existing prop update path

#### 3. Event Routing Simplification

**Remove: `overlay_event_router`**
- Single `event_router` handles all events
- Hit testing naturally respects z-order (Stack children render last = on top)
- Backdrop click detection via event handlers on backdrop Div

**Backdrop handling:**
```rust
// Backdrop is just a Div with click handler
div()
    .absolute().inset(0.0)
    .bg(Color::BLACK.with_alpha(0.5))
    .on_click(|_| overlay_manager.handle_backdrop_click())
```

#### 4. Motion Animation Stability

**Problem:** Rebuilding overlay tree creates new motion() containers with new UUIDs.

**Solution:**
- Overlays in main tree use same incremental update path
- InstanceKey stability preserved across rebuilds
- `OVERLAY_CLOSING` flag still used to signal exit animations

**Key insight:** Since overlays are now part of the main tree's incremental update system:
- Adding overlay = subtree rebuild (adds new children to overlay layer)
- Removing overlay = subtree rebuild (removes children)
- Hover/press = visual prop update only (no rebuild)
- Close animation = motion handles exit, then subtree rebuild removes

---

## Implementation Plan

### Phase 1: OverlayManager API Changes

**Files:** `crates/blinc_layout/src/widgets/overlay.rs`

1. **Add `build_overlay_layer()` method**
   - Returns `Option<Div>` instead of `Option<RenderTree>`
   - Builds all visible overlays as children of a container Div
   - Container is absolute-positioned, full viewport size

2. **Add overlay layer node tracking**
   - Store the LayoutNodeId of the overlay layer root
   - Used for queuing subtree rebuilds when overlays change

3. **Change dirty flag semantics**
   - `dirty` flag now queues a subtree rebuild for the overlay layer
   - Instead of triggering full overlay tree rebuild

4. **Update `OverlayManagerExt` trait**
   - Add `build_overlay_layer()`
   - Deprecate `build_overlay_tree()`

### Phase 2: Windowed App Integration

**Files:** `crates/blinc_app/src/windowed.rs`

1. **Modify tree building (PHASE 2)**
   ```rust
   let user_ui = ui_builder(ctx);
   let overlay_layer = ctx.overlay_manager.build_overlay_layer();

   let root = if let Some(overlays) = overlay_layer {
       stack()
           .w(ctx.width).h(ctx.height)
           .child(user_ui)
           .child(overlays)
   } else {
       // No overlays - just user UI (avoid extra Stack wrapper)
       user_ui
   };
   ```

2. **Remove separate overlay rendering (PHASE 4b)**
   - Delete `render_overlay_tree_with_motion()` call
   - Delete overlay tree building/caching
   - Delete `overlay_tree` field from `WindowedContext`

3. **Simplify event routing**
   - Remove `overlay_event_router`
   - Remove overlay-specific event dispatch logic
   - Single event router handles everything

4. **Update overlay state management**
   - `overlay_manager.update()` still called for state transitions
   - But instead of triggering overlay tree rebuild, queues subtree rebuild

### Phase 3: Subtree Rebuild for Overlay Changes

**Files:** `crates/blinc_layout/src/stateful.rs`, `crates/blinc_layout/src/widgets/overlay.rs`

1. **Track overlay layer node ID**
   - When overlay layer is built, store its LayoutNodeId
   - OverlayManager holds reference to this ID

2. **Queue subtree rebuild on overlay changes**
   ```rust
   // When overlay added/removed:
   fn mark_dirty(&self) {
       if let Some(layer_id) = self.overlay_layer_node_id {
           queue_subtree_rebuild(layer_id, self.build_overlay_layer_content());
       }
       self.dirty.store(true, Ordering::SeqCst);
   }
   ```

3. **Visual-only updates for overlay content**
   - Hover/press in overlay content uses `queue_visual_subtree_rebuild`
   - Same mechanism as main UI

### Phase 4: Backdrop and Positioning

**Files:** `crates/blinc_layout/src/widgets/overlay.rs`

1. **Backdrop as Div with event handler**
   ```rust
   fn build_backdrop(&self, overlay: &ActiveOverlay) -> Div {
       let mgr = self.clone();
       let handle = overlay.handle;

       div()
           .absolute().inset(0.0)
           .bg(backdrop_color.with_alpha(opacity))
           .on_click(move |_| {
               if overlay.config.dismiss_on_backdrop_click {
                   mgr.close(handle);
               }
           })
   }
   ```

2. **Overlay positioning via Div styles**
   - `Centered`: Use flexbox centering on container
   - `AtPoint`: Absolute positioning with left/top
   - `Corner`: Absolute with appropriate edge positioning
   - `RelativeToAnchor`: Query anchor bounds, position relative

3. **Z-ordering via child order**
   - Overlays sorted by z_priority
   - Added to overlay layer in order (last = on top)

### Phase 5: Motion Animation Integration

**Files:** `crates/blinc_layout/src/widgets/overlay.rs`, `crates/blinc_layout/src/motion.rs`

1. **Preserve OVERLAY_CLOSING mechanism**
   - Still set flag before building closing overlay content
   - Motion containers check flag to start exit animation

2. **Stable InstanceKeys**
   - Overlay content built with stable keys (already works)
   - No tree rebuild = keys preserved = animations smooth

3. **Animation timing coordination**
   - Overlay state transitions based on animation duration
   - Motion animations run independently
   - Both use same render state tick

### Phase 6: Cleanup and Migration

1. **Remove deprecated code**
   - `build_overlay_tree()` method
   - `overlay_tree` field in WindowedContext
   - `overlay_event_router` field
   - Separate overlay render pass in windowed.rs

2. **Update components**
   - Select, DropdownMenu, ContextMenu - should work unchanged
   - Toast notifications - same API
   - Modal/Dialog - same API

3. **Update tests**
   - Overlay tests need updating for new architecture
   - Add tests for incremental overlay updates

---

## Detailed Design: Overlay Layer Structure

### Tree Structure

```
overlay_layer (Stack, absolute, full viewport)
├── overlay_0 (Stack, for single overlay)
│   ├── backdrop (Div, absolute, full size, click handler)
│   └── content_wrapper (Div, positioned per OverlayPosition)
│       └── motion() (handles enter/exit animation)
│           └── user_content (from content builder)
├── overlay_1
│   ├── backdrop
│   └── content_wrapper
│       └── motion()
│           └── user_content
└── ...
```

### Overlay Layer Build Logic

```rust
pub fn build_overlay_layer(&self) -> Option<Div> {
    if !self.has_visible_overlays() {
        return None;
    }

    let (width, height) = self.viewport;
    let mut layer = stack().w(width).h(height).absolute().inset(0.0);

    for overlay in self.overlays_sorted() {
        if overlay.is_visible() {
            layer = layer.child(self.build_single_overlay(overlay));
        }
    }

    Some(layer.into_div())  // Convert Stack to Div
}

fn build_single_overlay(&self, overlay: &ActiveOverlay) -> Div {
    let is_closing = overlay.state == OverlayState::Closing;
    set_overlay_closing(is_closing);

    // Build content with motion wrapper
    let content = (overlay.content_builder)();

    set_overlay_closing(false);

    // Wrap in positioning container
    let positioned = self.apply_position(content, &overlay.config);

    // Build backdrop if configured
    if let Some(backdrop_config) = &overlay.config.backdrop {
        let backdrop = self.build_backdrop(overlay, backdrop_config);
        stack()
            .w_full().h_full()
            .child(backdrop)
            .child(positioned)
            .into_div()
    } else {
        positioned
    }
}
```

---

## Migration Strategy

### Breaking Changes

1. **`build_overlay_tree()` removed** - Use `build_overlay_layer()` instead
2. **`overlay_tree` field removed** - No longer exists
3. **`overlay_event_router` removed** - Single event router

### Non-Breaking (API Compatible)

1. **OverlayManagerExt trait** - All builder methods unchanged
2. **Overlay builders** - `.modal()`, `.dropdown()`, etc. unchanged
3. **Content builders** - Same `|| -> Div` signature
4. **Show/close API** - Same `.show()`, `.close()` methods

### Migration Steps for Consumers

1. If directly accessing `overlay_tree` - remove, not needed
2. If using custom overlay rendering - update to new architecture
3. If relying on separate event routing - events now unified

---

## Performance Considerations

### Improvements

1. **Single tree traversal** - No separate overlay tree walk
2. **Unified layout computation** - One `compute_layout()` call
3. **Shared render state** - Motion animations in same registry
4. **Incremental updates** - Subtree rebuilds instead of full tree

### Potential Concerns

1. **Stack wrapper overhead** - Extra node when overlays present
   - Mitigation: Only add Stack when overlays visible

2. **Overlay layer in main tree** - Slightly larger tree
   - Mitigation: Overlay layer is leaf-heavy, not deep

3. **Subtree rebuild scope** - Overlay changes rebuild overlay layer
   - Mitigation: Only overlay layer children affected, not main UI

---

## Testing Plan

1. **Unit tests**
   - Overlay layer building
   - Positioning calculations
   - Backdrop click handling
   - State transitions

2. **Integration tests**
   - Open/close animations smooth
   - Hover states update correctly
   - Multiple overlays z-ordering
   - Backdrop dismiss works

3. **Visual tests**
   - No animation jitter
   - Correct positioning
   - Proper backdrop opacity animation

4. **Performance tests**
   - No full UI rebuilds on overlay changes
   - Hover updates are visual-only
   - Animation frame rate stable

---

## Success Criteria

1. **Opening overlay does NOT trigger main UI rebuild**
2. **Closing overlay does NOT trigger main UI rebuild**
3. **Hover in overlay updates visually without rebuild**
4. **Motion animations play smoothly without jitter**
5. **All existing overlay tests pass**
6. **Select, DropdownMenu, ContextMenu work correctly**
