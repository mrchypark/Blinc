# Desktop Productization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Finish the desktop platform with productization hooks for capture utilities, updater plumbing, best-effort runtime observability notes, and developer-facing documentation/examples.

**Architecture:** Reuse the now-expanded shared platform contracts, add only narrow productization abstractions, and validate with backend tests plus documentation smoke checks instead of building a large opinionated app-management layer. Sleep/wake and display-topology handling are not treated as immediate `winit` features here; this plan records best-effort hooks and a follow-up OS-specific backlog item instead of promising a uniform implementation.

**Tech Stack:** Rust 2021, `winit`, desktop helper crates, docs and example updates.

---

### Task 1: Add Red Tests For Productization Hooks

**Files:**
- Create: `crates/blinc_platform/tests/productization_api.rs`
- Modify: `crates/blinc_platform/src/event.rs`
- Modify: `crates/blinc_platform/src/lib.rs`

**Step 1: Write the failing tests**

- Add a test for capture request value objects for screen, window, and region capture.
- Add a test for updater status objects (`Idle`, `Checking`, `UpdateAvailable`, `Downloading`, `ReadyToInstall`, `Error`).
- Add a test for a best-effort desktop observability note type that can record unsupported lifecycle capabilities without pretending they are implemented uniformly.

**Step 2: Run targeted tests to verify failure**

Run:
- `cargo test -p blinc_platform --test productization_api -- --nocapture`

Expected: compile failures because none of these productization hooks exist.

**Step 3: Commit the red tests**

```bash
git add crates/blinc_platform/tests/productization_api.rs crates/blinc_platform/src/event.rs crates/blinc_platform/src/lib.rs
git commit -m "test: add desktop productization contracts"
```

### Task 2: Add Shared Productization Modules

**Files:**
- Create: `crates/blinc_platform/src/capture.rs`
- Create: `crates/blinc_platform/src/updater.rs`
- Create: `crates/blinc_platform/src/desktop_runtime_notes.rs`
- Modify: `crates/blinc_platform/src/lib.rs`

**Step 1: Add shared capture types**

- Introduce `CaptureTarget`, `CaptureRequest`, and `CaptureResult`.
- Keep the API request/response based so desktop backends can opt into platform-specific capture permissions later.

**Step 2: Add updater status types**

- Introduce `UpdateChannel`, `UpdateStatus`, and `UpdateCommand`.
- Keep download/install orchestration outside the shared layer.

**Step 3: Add best-effort runtime observability notes**

- Add a small shared note or capability type that can report unsupported or platform-specific lifecycle hooks without turning them into guaranteed cross-platform events.

**Step 4: Re-run shared API tests**

Run:
- `cargo test -p blinc_platform --test productization_api -- --nocapture`

Expected: PASS.

**Step 5: Commit the shared productization types**

```bash
git add crates/blinc_platform/src/capture.rs crates/blinc_platform/src/updater.rs crates/blinc_platform/src/desktop_runtime_notes.rs crates/blinc_platform/src/lib.rs crates/blinc_platform/tests/productization_api.rs
git commit -m "feat: add desktop productization abstractions"
```

### Task 3: Implement Desktop Capture, Updater Hooks, And Runtime Notes

**Files:**
- Create: `extensions/blinc_platform_desktop/src/capture.rs`
- Create: `extensions/blinc_platform_desktop/src/updater.rs`
- Modify: `extensions/blinc_platform_desktop/src/event_loop.rs`
- Create: `extensions/blinc_platform_desktop/src/runtime_notes.rs`
- Modify: `extensions/blinc_platform_desktop/src/lib.rs`
- Modify: `extensions/blinc_platform_desktop/Cargo.toml`
- Create: `extensions/blinc_platform_desktop/tests/productization_runtime.rs`
- Modify: `extensions/blinc_platform_desktop/tests/support.rs`

**Step 1: Write the failing backend tests**

- Add a test for a capture backend stub returning deterministic metadata.
- Add a test for updater state-machine transitions without calling a real update service.
- Add a test for runtime notes reporting that sleep/wake and display-topology support are unsupported or OS-specific on the current backend.

**Step 2: Run targeted backend tests to verify failure**

Run:
- Linux/CI: `xvfb-run --auto-servernum cargo test -p blinc_platform_desktop --test productization_runtime -- --nocapture`
- macOS/Windows/local GUI session: `cargo test -p blinc_platform_desktop --test productization_runtime -- --nocapture`

Expected: failures because the backend has no capture or updater modules yet.

**Step 3: Implement minimal backend adapters**

- Add a capture trait adapter and a backend stub implementation behind a concrete `desktop-capture-stub` feature declared in `extensions/blinc_platform_desktop/Cargo.toml`.
- Add updater polling hooks with a mockable backend.
- Add runtime notes that mark sleep/wake and display-topology as OS-specific follow-up work rather than claiming uniform support.

**Step 4: Re-run targeted backend tests**

Run:
- Linux/CI: `xvfb-run --auto-servernum cargo test -p blinc_platform_desktop --test productization_runtime -- --nocapture`
- macOS/Windows/local GUI session: `cargo test -p blinc_platform_desktop --test productization_runtime -- --nocapture`
- `cargo test -p blinc_platform --test productization_api -- --nocapture`

Expected: PASS.

**Step 5: Commit the backend productization hooks**

```bash
git add extensions/blinc_platform_desktop/src/capture.rs extensions/blinc_platform_desktop/src/updater.rs extensions/blinc_platform_desktop/src/event_loop.rs extensions/blinc_platform_desktop/src/runtime_notes.rs extensions/blinc_platform_desktop/src/lib.rs extensions/blinc_platform_desktop/tests/support.rs extensions/blinc_platform_desktop/tests/productization_runtime.rs extensions/blinc_platform_desktop/Cargo.toml
git commit -m "feat: add desktop productization hooks"
```

### Task 4: Update Docs, Scaffolds, And Examples

**Files:**
- Modify: `extensions/blinc_platform_desktop/README.md`
- Modify: `README.md`
- Modify: `crates/blinc_cli/src/project.rs`
- Create: `examples/desktop_platform_showcase/README.md`

**Step 1: Write the failing documentation checklist**

- Add a short checklist in the commit description or task notes covering new APIs that must be documented: window controls, dialogs, drag-drop, shortcuts, accessibility, capture, updater.

**Step 2: Update desktop docs and scaffolds**

- Document the new desktop capability surface and any new optional dependencies/features.
- Update project scaffolding notes if desktop apps now need extra config for updater setup, capture permissions, or Linux `xvfb-run` based verification.
- Add an explicit note that sleep/wake and display-topology remain OS-specific follow-up work.

**Step 3: Add a showcase example README**

- Document how to manually verify window controls, file drop, menu commands, IME, and notifications in one sample app directory.

**Step 4: Run final verification**

Run:
- `cargo fmt --all`
- `cargo fmt --all -- --check`
- `cargo test -p blinc_platform -- --nocapture`
- Linux/CI: `xvfb-run --auto-servernum cargo test -p blinc_platform_desktop -- --nocapture`
- macOS/Windows/local GUI session: `cargo test -p blinc_platform_desktop -- --nocapture`

Expected: docs are updated and the final verification pass is green.

**Step 5: Commit the docs and scaffolding updates**

```bash
git add extensions/blinc_platform_desktop/README.md README.md crates/blinc_cli/src/project.rs examples/desktop_platform_showcase/README.md
git commit -m "docs: describe desktop platform capabilities"
```
