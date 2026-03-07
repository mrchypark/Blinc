# Updater Architecture Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an interface-first updater architecture to Blinc, with shared updater domain types, desktop and Android backend boundaries, updater-aware project configuration, and CLI release metadata generation.

**Architecture:** Keep update orchestration in a new shared crate and push platform-specific install behavior into dedicated extension crates. Treat `blinc_cli` as the owner of package/release metadata while keeping `crates/blinc_platform` unchanged. Implement real behavior for Android and macOS first, with explicit stubs for Windows and Linux.

**Tech Stack:** Rust 2021, Cargo workspace crates, `serde`/`serde_json`, `thiserror`, `anyhow`, Blinc CLI/config scaffolding, platform extension crates.

---

## Global Constraints / Guardrails

- Follow TDD for every new behavior: write a failing test first, confirm the failure, then add the minimum code to pass.
- Keep `crates/blinc_platform::Platform` unchanged in this plan.
- Do not claim desktop support generically unless the backend is explicitly implemented.
- Treat Windows/Linux as capability-limited stubs in v1.
- Fix project identifier propagation before building updater metadata on top of it.

### Task 1: Fix scaffolded app identifiers before updater work

**Files:**
- Modify: `crates/blinc_cli/src/project.rs`
- Modify: `crates/blinc_cli/src/config.rs`
- Test: `crates/blinc_cli/src/project.rs`

**Step 1: Write the failing tests**

Add tests beside the existing `#[cfg(test)]` module in `crates/blinc_cli/src/project.rs`:

```rust
#[test]
fn non_rust_project_uses_org_for_android_and_apple_ids() {
    // create_project(&root, "Demo App", "default", "io.test") ...
    // assert generated Android and plist files contain io.test.demo_app
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_cli non_rust_project_uses_org_for_android_and_apple_ids -- --nocapture`
Expected: FAIL because generated files still contain `com.example`.

**Step 3: Write minimal implementation**

Update `create_platform_files`, `create_android_files`, `create_ios_files`, and `create_macos_files` so they accept and use `org`, not a hardcoded prefix. Keep generated identifiers consistent with `BlincProject::with_all_platforms`.

**Step 4: Run targeted tests to verify they pass**

Run: `cargo test -p blinc_cli project::tests -- --nocapture`
Expected: PASS with existing scaffold tests plus the new org propagation test.

**Step 5: Commit**

```bash
git add crates/blinc_cli/src/project.rs crates/blinc_cli/src/config.rs
git commit -m "fix: align scaffolded app identifiers with project config"
```

### Task 2: Add updater configuration to `.blincproj`

**Files:**
- Modify: `crates/blinc_cli/src/config.rs`
- Test: `crates/blinc_cli/src/config.rs` or `crates/blinc_cli/src/project.rs`

**Step 1: Write the failing tests**

Add round-trip tests for config serialization:

```rust
#[test]
fn updater_config_round_trips_with_desktop_and_android_overrides() {
    let toml = r#"
        [project]
        name = "Demo"
        version = "0.1.0"

        [updates]
        enabled = true
        channel = "stable"
        manifest_url = "https://example.com/manifest.json"
        public_key = "abc"

        [updates.desktop]
        enabled = true

        [updates.android]
        enabled = true
        expected_package = "io.test.demo"
    "#;
    // parse + serialize + parse again
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_cli updater_config_round_trips_with_desktop_and_android_overrides -- --nocapture`
Expected: FAIL because `BlincProject` has no updater schema.

**Step 3: Write minimal implementation**

Add new config structs:

- `UpdatesConfig`
- `DesktopUpdateConfig`
- `AndroidUpdateConfig`
- `ReleaseChannel` string representation for config

Extend `BlincProject` with `#[serde(default)] pub updates: UpdatesConfig`.

**Step 4: Run focused tests**

Run: `cargo test -p blinc_cli updater_config_round_trips_with_desktop_and_android_overrides -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_cli/src/config.rs
git commit -m "feat: add updater settings to blinc project config"
```

### Task 3: Create the shared updater domain crate

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/blinc_update/Cargo.toml`
- Create: `crates/blinc_update/src/lib.rs`
- Create: `crates/blinc_update/src/error.rs`
- Create: `crates/blinc_update/src/manifest.rs`
- Create: `crates/blinc_update/src/service.rs`
- Create: `crates/blinc_update/src/version.rs`
- Test: `crates/blinc_update/src/lib.rs` or dedicated unit test modules

**Step 1: Write the failing tests**

Start with unit tests for:

```rust
#[test]
fn selects_matching_artifact_for_platform_and_arch() {}

#[test]
fn rejects_manifest_with_missing_signature() {}

#[test]
fn version_check_detects_newer_release() {}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p blinc_update`
Expected: FAIL because crate does not exist yet.

**Step 3: Write minimal implementation**

Create the crate and implement:

- manifest structs
- artifact selector helper
- update state enum
- backend trait with install/check/download shape
- version comparison helper

Keep networking out of scope for now; test pure domain behavior first.

**Step 4: Run tests**

Run: `cargo test -p blinc_update`
Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml crates/blinc_update
git commit -m "feat: add shared updater domain crate"
```

### Task 4: Add release manifest generation to the CLI

**Files:**
- Modify: `crates/blinc_cli/src/main.rs`
- Create: `crates/blinc_cli/src/release.rs`
- Modify: `crates/blinc_cli/Cargo.toml`
- Test: `crates/blinc_cli/src/release.rs`

**Step 1: Write the failing tests**

Add tests for release manifest generation:

```rust
#[test]
fn generates_manifest_with_expected_app_id_channel_and_artifacts() {}

#[test]
fn rejects_release_when_updates_enabled_but_public_key_is_missing() {}
```

**Step 2: Run tests to verify failure**

Run: `cargo test -p blinc_cli generates_manifest_with_expected_app_id_channel_and_artifacts -- --nocapture`
Expected: FAIL because release manifest logic is missing.

**Step 3: Write minimal implementation**

Add a new module that:

- reads `BlincProject`
- builds `blinc_update::UpdateManifest`
- serializes JSON
- validates required updater fields when updates are enabled

Expose the smallest possible CLI entry point first, even if it is an internal command helper used by future `package`/`release` commands.

**Step 4: Run tests**

Run: `cargo test -p blinc_cli release -- --nocapture`
Expected: PASS for manifest generation tests.

**Step 5: Commit**

```bash
git add crates/blinc_cli/src/main.rs crates/blinc_cli/src/release.rs crates/blinc_cli/Cargo.toml
git commit -m "feat: generate updater release manifests from the CLI"
```

### Task 5: Add artifact checksum and signature verification helpers

**Files:**
- Modify: `crates/blinc_update/Cargo.toml`
- Modify: `crates/blinc_update/src/lib.rs`
- Create: `crates/blinc_update/src/verify.rs`
- Test: `crates/blinc_update/src/verify.rs`

**Step 1: Write the failing tests**

Add tests like:

```rust
#[test]
fn accepts_matching_sha256_and_signature() {}

#[test]
fn rejects_artifact_when_checksum_differs() {}
```

**Step 2: Run the tests**

Run: `cargo test -p blinc_update verify -- --nocapture`
Expected: FAIL because verification helpers are missing.

**Step 3: Write minimal implementation**

Add pure helpers to verify:

- SHA-256 checksum
- detached signature using the chosen public-key scheme

Return typed `UpdateError` values. Keep this layer file-based and deterministic.

**Step 4: Run tests**

Run: `cargo test -p blinc_update verify -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_update/Cargo.toml crates/blinc_update/src/lib.rs crates/blinc_update/src/verify.rs
git commit -m "feat: add updater artifact verification helpers"
```

### Task 6: Add Android updater backend crate

**Files:**
- Modify: `Cargo.toml`
- Create: `extensions/blinc_update_android/Cargo.toml`
- Create: `extensions/blinc_update_android/src/lib.rs`
- Create: `extensions/blinc_update_android/src/backend.rs`
- Test: `extensions/blinc_update_android/src/lib.rs`

**Step 1: Write the failing tests**

Start with backend-level tests that avoid real Android runtime dependencies:

```rust
#[test]
fn android_backend_rejects_manifest_artifact_with_wrong_package_id() {}

#[test]
fn android_backend_builds_install_intent_for_matching_apk() {}
```

**Step 2: Run tests to verify failure**

Run: `cargo test -p blinc_update_android`
Expected: FAIL because crate and backend do not exist.

**Step 3: Write minimal implementation**

Implement a backend that:

- chooses Android APK artifacts
- validates expected package name from config
- returns an `InstallIntent` representing Android package installer handoff

Do not attempt real install execution in unit tests.

**Step 4: Run tests**

Run: `cargo test -p blinc_update_android`
Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml extensions/blinc_update_android
git commit -m "feat: add android updater backend"
```

### Task 7: Add desktop updater backend crate with macOS reference backend

**Files:**
- Modify: `Cargo.toml`
- Create: `extensions/blinc_update_desktop/Cargo.toml`
- Create: `extensions/blinc_update_desktop/src/lib.rs`
- Create: `extensions/blinc_update_desktop/src/backend.rs`
- Create: `extensions/blinc_update_desktop/src/macos.rs`
- Create: `extensions/blinc_update_desktop/src/windows.rs`
- Create: `extensions/blinc_update_desktop/src/linux.rs`
- Test: `extensions/blinc_update_desktop/src/lib.rs`

**Step 1: Write the failing tests**

Add tests such as:

```rust
#[test]
fn macos_backend_emits_bundle_replace_install_intent() {}

#[test]
fn windows_backend_reports_unsupported_for_now() {}

#[test]
fn linux_backend_reports_unsupported_for_now() {}
```

**Step 2: Run the tests**

Run: `cargo test -p blinc_update_desktop`
Expected: FAIL because crate and backends do not exist.

**Step 3: Write minimal implementation**

Implement:

- shared desktop backend dispatch
- macOS install intent builder
- explicit unsupported/stub responses for Windows/Linux

Avoid real installer side effects in tests. Model installation as a handoff plan, not an in-process overwrite.

**Step 4: Run tests**

Run: `cargo test -p blinc_update_desktop`
Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml extensions/blinc_update_desktop
git commit -m "feat: add desktop updater backend with macos reference path"
```

### Task 8: Thread updater config through generated projects and templates

**Files:**
- Modify: `crates/blinc_cli/src/project.rs`
- Modify: `crates/blinc_cli/README.md`
- Test: `crates/blinc_cli/src/project.rs`

**Step 1: Write the failing tests**

Add a scaffolding test that asserts generated projects include updater configuration examples or placeholders:

```rust
#[test]
fn generated_project_includes_updater_config_placeholders() {}
```

**Step 2: Run the test**

Run: `cargo test -p blinc_cli generated_project_includes_updater_config_placeholders -- --nocapture`
Expected: FAIL because templates do not include updater guidance.

**Step 3: Write minimal implementation**

Update generated README and config examples to describe:

- updater config location
- desktop and Android support
- iOS exclusion

Keep templates simple; do not add fake implementation code.

**Step 4: Run tests**

Run: `cargo test -p blinc_cli generated_project_includes_updater_config_placeholders -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_cli/src/project.rs crates/blinc_cli/README.md
git commit -m "docs: surface updater configuration in generated projects"
```

### Task 9: Add end-to-end CLI plus crate verification

**Files:**
- Modify: `crates/blinc_cli/src/main.rs`
- Modify: `crates/blinc_cli/src/release.rs`
- Test: `crates/blinc_cli/src/release.rs`
- Test: `crates/blinc_update/src/lib.rs`
- Test: `extensions/blinc_update_android/src/lib.rs`
- Test: `extensions/blinc_update_desktop/src/lib.rs`

**Step 1: Write the failing integration-style test**

Add a temporary-dir test that:

- creates a sample project config
- generates a manifest
- parses it through `blinc_update`
- selects the correct platform artifact

**Step 2: Run test to verify failure**

Run: `cargo test -p blinc_cli generated_manifest_round_trips_into_update_domain -- --nocapture`
Expected: FAIL until the pieces connect cleanly.

**Step 3: Write minimal integration glue**

Wire the CLI generator and shared domain so manifest output matches runtime expectations exactly. Remove any duplicate schema definitions.

**Step 4: Run full targeted verification**

Run:

```bash
cargo test -p blinc_cli
cargo test -p blinc_update
cargo test -p blinc_update_android
cargo test -p blinc_update_desktop
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_cli/src/main.rs crates/blinc_cli/src/release.rs crates/blinc_update extensions/blinc_update_android extensions/blinc_update_desktop Cargo.toml
git commit -m "test: verify updater manifest flow across CLI and backends"
```

### Task 10: Final verification, formatting, and documentation pass

**Files:**
- Modify: `docs/plans/2026-03-07-updater-design.md` if implementation changed scope
- Modify: `docs/plans/2026-03-07-updater-impl-plan.md` if task ordering changed

**Step 1: Run formatting**

Run:

```bash
cargo fmt --all
cargo fmt --all -- --check
```

Expected: second command exits successfully.

**Step 2: Run workspace verification for touched packages**

Run:

```bash
cargo test -p blinc_cli
cargo test -p blinc_update
cargo test -p blinc_update_android
cargo test -p blinc_update_desktop
```

Expected: PASS.

**Step 3: Review unsupported behavior explicitly**

Verify Windows/Linux return explicit unsupported or stub results and that no docs imply full support.

**Step 4: Update docs if implementation diverged**

Make only the minimal documentation changes needed to reflect the shipped behavior.

**Step 5: Commit**

```bash
git add docs/plans/2026-03-07-updater-design.md docs/plans/2026-03-07-updater-impl-plan.md
git commit -m "docs: finalize updater implementation notes"
```
