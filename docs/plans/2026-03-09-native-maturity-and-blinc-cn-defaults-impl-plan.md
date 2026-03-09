# Native Maturity And blinc_cn Defaults Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `blinc_cn` easier to bootstrap correctly, align its docs with the actual API, improve a few poor default visuals, and bring desktop/Android/iOS/template docs in line with implemented reality.

**Architecture:** Keep the implementation surgical. Add explicit bootstrap helpers in `blinc_cn`, update the highest-traffic docs/examples to the real API, tune a small number of component defaults to rely more consistently on theme tokens, and normalize native-template/docs claims without trying to invent missing runtime features.

**Tech Stack:** Rust 1.75, edition 2021, `blinc_cn`, `blinc_theme`, mobile/native templates, mdBook docs.

---

### Task 1: Add Official blinc_cn Bootstrap Helpers

**Files:**
- Modify: `crates/blinc_cn/src/lib.rs`
- Test: `crates/blinc_cn/src/lib.rs`

**Step 1: Add public helpers**

Add helpers to expose:

- `ensure_default_theme()`
- `ensure_theme(bundle, scheme)`
- `default_styles() -> &'static str`

These should use `ThemeState::try_get()` and initialize only when absent.

**Step 2: Add focused tests**

Add tests that verify:

- `default_styles()` returns a non-empty stylesheet
- `ensure_default_theme()` leaves theme state available

Use existing singleton carefully and avoid tests that depend on resetting the singleton.

**Step 3: Run focused tests**

Run: `cargo test -p blinc_cn bootstrap -- --nocapture`

**Step 4: Commit**

```bash
git add crates/blinc_cn/src/lib.rs
git commit -m "feat(blinc_cn): add bootstrap helpers"
```

### Task 2: Fix blinc_cn README To Match Real Setup And API

**Files:**
- Modify: `crates/blinc_cn/README.md`

**Step 1: Rewrite setup section**

Document:

- bootstrap helper usage
- explicit theme initialization option
- default styles helper usage

**Step 2: Replace stale examples**

Update examples to use current APIs:

- `switch(&state)`
- `select(&state).option(...)`
- imperative `dialog().show()`
- current button size enum names
- currently exported card helpers

**Step 3: Verify references**

Run: `rg -n "switch_|dialog_trigger|select_trigger|ButtonSize::Sm|ButtonSize::Default|card_title|card_description" crates/blinc_cn/README.md`
Expected: no matches

**Step 4: Commit**

```bash
git add crates/blinc_cn/README.md
git commit -m "docs(blinc_cn): align readme with actual api"
```

### Task 3: Correct mdBook blinc_cn Pages

**Files:**
- Modify: `docs/book/src/cn/overview.md`
- Modify: `docs/book/src/cn/dialog.md`
- Modify: `docs/book/src/cn/form.md`
- Modify: `docs/book/src/cn/navigation.md`
- Modify: `docs/book/src/cn/card.md`
- Modify: `docs/book/src/cn/button.md`

**Step 1: Update high-traffic pages**

Correct the same stale patterns found in README.

**Step 2: Add bootstrap note**

At least `overview.md` should explain the required `blinc_cn` setup path.

**Step 3: Verify stale API references are removed from touched pages**

Run: `rg -n "switch_|dialog_trigger|dialog_content|select_trigger|select_content|radio_item|card_title|card_description|ButtonSize::Sm|ButtonSize::Default" docs/book/src/cn/{overview,dialog,form,navigation,card,button}.md`

**Step 4: Commit**

```bash
git add docs/book/src/cn/overview.md docs/book/src/cn/dialog.md docs/book/src/cn/form.md docs/book/src/cn/navigation.md docs/book/src/cn/card.md docs/book/src/cn/button.md
git commit -m "docs(book): update blinc_cn component guides"
```

### Task 4: Improve High-Impact blinc_cn Defaults

**Files:**
- Modify: `crates/blinc_cn/src/components/progress.rs`
- Modify: `crates/blinc_cn/src/components/avatar.rs`
- Modify: `crates/blinc_cn/src/components/sidebar.rs`
- Modify: `crates/blinc_cn/src/components/select.rs`
- Modify: `crates/blinc_cn/src/components/tabs.rs`
- Modify: `crates/blinc_cn/src/cn_styles.rs`

**Step 1: Progress**

Make the default track use a more recessed surface/token treatment than the current strong secondary fill.

**Step 2: Avatar**

Improve fallback/empty avatar styling so it remains intentional on cards and surfaces.

**Step 3: Sidebar**

Replace the most cramped hardcoded spacing with token-driven spacing values.

**Step 4: Select/Tabs**

Reduce conflicts between inline Rust appearance defaults and CSS defaults. Prefer a single authoritative default path where practical.

**Step 5: Run focused tests**

Run: `cargo test -p blinc_cn`

**Step 6: Commit**

```bash
git add crates/blinc_cn/src/components/progress.rs crates/blinc_cn/src/components/avatar.rs crates/blinc_cn/src/components/sidebar.rs crates/blinc_cn/src/components/select.rs crates/blinc_cn/src/components/tabs.rs crates/blinc_cn/src/cn_styles.rs
git commit -m "feat(blinc_cn): refine default component styling"
```

### Task 5: Normalize Android/iOS Template Native Bridge Wiring

**Files:**
- Modify: `toolchain/templates/rust/platforms/android/README.md`
- Modify or Create: `toolchain/templates/rust/platforms/android/app/src/main/kotlin/com/blinc/BlincNativeBridge.kt`
- Modify: `toolchain/templates/rust/platforms/ios/BlincApp/AppDelegate.swift`
- Modify: `toolchain/templates/rust/platforms/ios/README.md`

**Step 1: Android template**

Ensure the template either includes the bridge file or the README stops claiming it does.

**Step 2: iOS template**

Make the AppDelegate and README consistent with the example/native bridge setup that is actually required.

**Step 3: Verify template/example consistency**

Run:

```bash
rg -n "BlincNativeBridge" toolchain/templates/rust/platforms/android toolchain/templates/rust/platforms/ios mobile/example/platforms/android mobile/example/platforms/ios
```

**Step 4: Commit**

```bash
git add toolchain/templates/rust/platforms/android/README.md toolchain/templates/rust/platforms/android/app/src/main/kotlin/com/blinc/BlincNativeBridge.kt toolchain/templates/rust/platforms/ios/BlincApp/AppDelegate.swift toolchain/templates/rust/platforms/ios/README.md
git commit -m "fix(templates): align mobile bridge wiring"
```

### Task 6: Correct Desktop/Android/iOS Maturity Claims

**Files:**
- Modify: `docs/book/src/mobile/overview.md`
- Modify: `extensions/blinc_platform_android/README.md`
- Modify: `extensions/blinc_platform_ios/README.md`
- Modify: `crates/blinc_platform/README.md`
- Modify: `README.md`

**Step 1: Reframe support claims**

Adjust wording so docs reflect implemented support rather than planned support.

**Step 2: Clarify current guarantees**

Be explicit about:

- desktop strengths and current limitations
- Android/iOS platform crate limitations vs app/example support
- missing OS-level accessibility bridge and shell/productization features

**Step 3: Verify obvious overstatements removed**

Run:

```bash
rg -n "Stable|automatic safe area|proper app state handling|Vulkan Rendering|Metal Rendering|full native|production-ready" docs/book/src/mobile/overview.md extensions/blinc_platform_android/README.md extensions/blinc_platform_ios/README.md crates/blinc_platform/README.md README.md
```

Review matches manually.

**Step 4: Commit**

```bash
git add docs/book/src/mobile/overview.md extensions/blinc_platform_android/README.md extensions/blinc_platform_ios/README.md crates/blinc_platform/README.md README.md
git commit -m "docs(platform): align native support claims with implementation"
```

### Task 7: Final Verification

**Files:**
- Verify all touched files

**Step 1: Run formatting**

Run: `cargo fmt --all`

**Step 2: Run formatting check**

Run: `cargo fmt --all -- --check`

**Step 3: Run tests**

Run:

```bash
cargo test -p blinc_cn
cargo test -p blinc_theme
```

**Step 4: Summarize any residual gaps**

Note any remaining doc debt or native runtime gaps that were intentionally left out of scope.
