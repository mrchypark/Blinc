# blinc_cn Latest CSS Defaults Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Apply the newer CSS engine features directly to `blinc_cn` default components so the shipped defaults exercise the modern styling surface instead of leaving it only in demos.

**Architecture:** Extend semantic component tokens only where they help express new defaults, then update `CN_STYLES` with reusable modern CSS patterns and add class targets in component builders where those styles need to land on text or decorative children.

**Tech Stack:** Rust, `blinc_theme`, `blinc_cn`, Blinc CSS engine

---

### Task 1: Extend semantic tokens for shape-forward defaults

**Files:**
- Modify: `crates/blinc_theme/src/tokens/component.rs`
- Modify: `crates/blinc_theme/src/state.rs`

1. Add semantic `corner_shape_*` fields for control, container, and overlay tokens.
2. Derive them from primitive theme scales.
3. Export them as CSS variables from `ThemeState`.

### Task 2: Upgrade `CN_STYLES` to use recent CSS features

**Files:**
- Modify: `crates/blinc_cn/src/cn_styles.rs`

1. Add reusable utility classes for truncation and decorative children.
2. Apply `corner-shape` to controls, containers, overlays, and navigation surfaces.
3. Apply `backdrop-filter` to overlay/dialog-like defaults.
4. Apply `text-decoration` to link-like states and `text-overflow` to label helpers.
5. Use a low-risk `mix-blend-mode` on a decorative/supporting component default.

### Task 3: Add class targets in component builders

**Files:**
- Modify: `crates/blinc_cn/src/components/button.rs`
- Modify: `crates/blinc_cn/src/components/select.rs`
- Modify: `crates/blinc_cn/src/components/combobox.rs`
- Modify: `crates/blinc_cn/src/components/navigation_menu.rs`
- Modify: `crates/blinc_cn/src/components/sidebar.rs`
- Modify: `crates/blinc_cn/src/components/breadcrumb.rs`
- Modify: `crates/blinc_cn/src/components/dropdown_menu.rs`
- Modify: `crates/blinc_cn/src/components/context_menu.rs`
- Modify: `crates/blinc_cn/src/components/menubar.rs`

1. Add explicit classes for labels, values, shortcuts, and decorative icons.
2. Keep behavior unchanged except where CSS utilities require a target.

### Task 4: Update docs and verify

**Files:**
- Modify: `crates/blinc_cn/README.md`
- Modify: `docs/book/src/cn/overview.md`

1. Mention that default styles now lean on newer CSS engine features.
2. Run:
   - `cargo fmt --all`
   - `cargo fmt --all -- --check`
   - `cargo test -p blinc_theme`
   - `cargo test -p blinc_cn`

### Task 5: Review loop

**Files:**
- No code target

1. Request subagent review on the diff.
2. Fix findings.
3. Re-run verification before closing the batch.
