# Blinc CN Theme-Token-First Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Move `blinc_cn` visual defaults to semantic theme tokens so presets drive coherent component defaults without per-component magic numbers.

**Architecture:** Add a semantic component-token layer to `blinc_theme`, expose it through `Theme` and `ThemeState`, then refactor `blinc_cn` CSS and component builders to consume that layer. Keep `blinc_layout` as the primitive/widget substrate and only adjust docs there.

**Tech Stack:** Rust 2021, `blinc_theme`, `blinc_cn`, CSS variable export via `ThemeState`

---

### Task 1: Add semantic component token model to `blinc_theme`

**Files:**
- Create: `crates/blinc_theme/src/tokens/component.rs`
- Modify: `crates/blinc_theme/src/tokens/mod.rs`
- Modify: `crates/blinc_theme/src/theme.rs`
- Modify: `crates/blinc_theme/src/state.rs`
- Test: `crates/blinc_theme/tests/presets.rs`

**Steps:**
1. Add a `ComponentTokens` struct that groups semantic defaults for controls, containers, overlays, badge/chip, and typography roles.
2. Derive default values from existing primitive tokens instead of introducing arbitrary numbers.
3. Export the new tokens from `tokens/mod.rs`.
4. Extend `Theme` with `components(&self) -> &ComponentTokens`.
5. Cache component tokens inside `ThemeState` and add a getter.
6. Extend CSS variable export so semantic component values can be referenced from `cn_styles.rs`.
7. Add or update tests proving presets expose stable semantic defaults.

### Task 2: Wire component tokens into theme implementations and presets

**Files:**
- Modify: `crates/blinc_theme/src/themes/blinc.rs`
- Modify: `crates/blinc_theme/src/themes/platform/macos.rs`
- Modify: `crates/blinc_theme/src/themes/platform/windows.rs`
- Modify: `crates/blinc_theme/src/themes/platform/linux.rs`
- Modify: `crates/blinc_theme/src/presets/mod.rs`
- Test: `crates/blinc_theme/tests/presets.rs`

**Steps:**
1. Update each theme struct to store or derive `ComponentTokens`.
2. Ensure shadcn-style presets derive semantic control/container/overlay values from their preset radii and spacing.
3. Keep platform themes coherent without overfitting them to `blinc_cn`.
4. Add tests for radius/padding/typography role expectations in the shadcn-like presets.

### Task 3: Refactor `blinc_cn` shared CSS defaults to semantic tokens

**Files:**
- Modify: `crates/blinc_cn/src/cn_styles.rs`
- Modify: `crates/blinc_cn/src/lib.rs`
- Test: `crates/blinc_cn/tests` if needed, otherwise component-local tests

**Steps:**
1. Replace common hard-coded literals in CSS with semantic theme CSS variables.
2. Introduce component-variable fallbacks that point to semantic component variables first.
3. Keep user override points stable.
4. Update any bootstrap docs/comments that still reference old registration flow.

### Task 4: Refactor control components to semantic control tokens

**Files:**
- Modify: `crates/blinc_cn/src/components/button.rs`
- Modify: `crates/blinc_cn/src/components/input.rs`
- Modify: `crates/blinc_cn/src/components/textarea.rs`
- Modify: `crates/blinc_cn/src/components/select.rs`
- Modify: `crates/blinc_cn/src/components/combobox.rs`
- Modify: `crates/blinc_cn/src/components/label.rs`
- Test: component-local tests in those files

**Steps:**
1. Replace size-specific font/radius/padding defaults with semantic control tokens.
2. Keep explicit builder overrides working.
3. Update tests to assert token-driven fallback behavior.

### Task 5: Refactor surfaces and overlays to semantic container/overlay tokens

**Files:**
- Modify: `crates/blinc_cn/src/components/card.rs`
- Modify: `crates/blinc_cn/src/components/alert.rs`
- Modify: `crates/blinc_cn/src/components/dialog.rs`
- Modify: `crates/blinc_cn/src/components/drawer.rs`
- Modify: `crates/blinc_cn/src/components/dropdown_menu.rs`
- Modify: `crates/blinc_cn/src/components/context_menu.rs`
- Modify: `crates/blinc_cn/src/components/menubar.rs`
- Modify: `crates/blinc_cn/src/components/popover.rs`
- Modify: `crates/blinc_cn/src/components/tooltip.rs`
- Test: existing local tests plus any new overlay default assertions

**Steps:**
1. Move overlay paddings/radii/font sizes to semantic overlay tokens.
2. Move card/dialog/drawer paddings and gaps to semantic container tokens.
3. Keep geometry-only exceptions local if truly required.

### Task 6: Refactor remaining visual outliers and docs

**Files:**
- Modify: `crates/blinc_cn/src/components/avatar.rs`
- Modify: `crates/blinc_cn/src/components/progress.rs`
- Modify: `crates/blinc_cn/src/components/sidebar.rs`
- Modify: `crates/blinc_cn/src/components/tabs.rs`
- Modify: `crates/blinc_cn/README.md`
- Modify: `docs/book/src/cn/*.md`
- Modify: `crates/blinc_layout/README.md`

**Steps:**
1. Reduce residual local constants in the visual outlier components.
2. Document that `blinc_cn` owns polished defaults while `blinc_layout` owns primitives/widgets.
3. Update examples to emphasize preset-driven defaults.

### Task 7: Verify and review

**Files:**
- Modify only if fixes are needed from review

**Steps:**
1. Run `cargo fmt --all`.
2. Run `cargo fmt --all -- --check`.
3. Run `cargo test -p blinc_theme`.
4. Run `cargo test -p blinc_cn`.
5. Request subagent review focused on token architecture regressions and default consistency.
6. Apply findings before closing the task.
