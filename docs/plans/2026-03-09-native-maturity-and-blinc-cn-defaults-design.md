# Native Maturity And blinc_cn Defaults Design

**Scope:** desktop, Android, iOS, and `blinc_cn`. HarmonyOS is explicitly out of scope for this pass.

## Problem

The repository currently has two high-friction issues:

1. `blinc_cn` does not provide a reliable low-friction path to attractive default UI. Theme/bootstrap requirements are implicit, docs are out of sync with the actual API, and several default component visuals are inconsistent or too raw.
2. Native platform documentation, templates, and exposed APIs overstate maturity for desktop, Android, and iOS. In practice, users can easily copy documented examples that do not compile, do not wire required native bridges, or imply runtime capabilities that are not implemented.

## Goals

- Make `blinc_cn` usable with a small, explicit bootstrap path that does not rely on hidden setup.
- Align `blinc_cn` README/book examples with the real API surface.
- Improve the default visual behavior of a small set of high-impact components so the stock result is more coherent.
- Reduce documentation overstatement for desktop, Android, and iOS.
- Ensure generated mobile templates are consistent with the currently supported native bridge/runtime story.

## Non-Goals

- Building the missing Android/iOS renderer/runtime pieces from scratch.
- Adding a full OS accessibility bridge for desktop or mobile.
- Expanding the platform abstraction with new large native capability surfaces in this pass.
- Touching HarmonyOS.

## Recommended Approach

### 1. Add an official `blinc_cn` bootstrap surface

Introduce a small public bootstrap API in `blinc_cn` that:

- initializes `ThemeState` with a sensible preset when not already initialized
- exposes the default CSS string through a stable, documented helper
- makes docs/examples show the supported setup path instead of hidden assumptions

This does not remove manual control. It gives users a default path while preserving explicit theme/preset override APIs.

### 2. Treat docs/API alignment as a product bug

Update `crates/blinc_cn/README.md` and the component book pages so every example reflects the actual exported API:

- imperative `dialog().show()` usage
- `switch(&state)` rather than removed helpers
- current `select(&state).option(...)` style
- current card/header/footer functions and size enum names
- explicit bootstrap/setup instructions

The success criterion is that a new user can follow the docs without reverse-engineering the crate source.

### 3. Improve a small set of default visuals instead of redesigning everything

Target only components with high leverage and clear default-quality issues:

- `Progress`: make the track recede visually instead of competing with the fill
- `Avatar`: make fallback/empty states look intentional
- `Sidebar`: replace cramped hardcoded spacing with token-driven defaults
- `Tabs` and `Select`: reduce mismatches between CSS defaults and inline Rust styling

This keeps the pass surgical and lowers regression risk.

### 4. Reframe native docs around implemented reality

For desktop, Android, and iOS:

- document current supported behavior accurately
- stop implying full native maturity where crates still contain placeholders/stubs
- make template READMEs and generated scaffolds match what is actually wired

For Android/iOS templates specifically, ensure native bridge files and initialization steps are consistent across template and example projects.

## Design Details

### blinc_cn bootstrap

Add public helpers with behavior like:

- `ensure_default_theme()`: initialize `ThemeState::init_default()` only if absent
- `ensure_theme(bundle, scheme)`: initialize custom theme only if absent
- `default_styles() -> &'static str`: return `CN_STYLES`

This avoids forcing initialization inside every component while still giving the ecosystem a canonical setup point.

### Visual token policy

Prefer token-driven values over local hardcoded spacing/colors when the value represents shared system semantics. Keep local constants only when they are intrinsic to a control’s geometry and not a theme concern.

### Native maturity framing

Desktop remains the strongest runtime, but documentation should clearly separate:

- currently implemented runtime/window/input behavior
- planned shell/productization capabilities

Android and iOS docs should distinguish:

- app-level/mobile example support
- platform crate completeness
- template-level wiring guarantees

## Testing Strategy

- Unit tests for new bootstrap helpers.
- Targeted tests for adjusted component defaults where cheap to assert structurally.
- `cargo test -p blinc_cn`
- `cargo test -p blinc_theme`
- focused checks for template file presence and native bridge wiring via file-level verification
- repository formatting gate before completion:
  - `cargo fmt --all`
  - `cargo fmt --all -- --check`

## Risks

- Docs may still reference stale APIs in pages not touched by this batch.
- Visual default changes can subtly affect demos/tests.
- Template updates can diverge from examples again if not normalized around a single bridge story.

## Rollout

Batch 1:

- `blinc_cn` bootstrap helpers
- README/book corrections
- high-impact visual default fixes

Batch 2:

- template/native bridge normalization
- desktop/Android/iOS documentation corrections
- small supporting tests
