# Updater Architecture Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an interface-first updater architecture to Blinc, with canonical platform-derived release identity, user-callable CLI release metadata generation, artifact signing and verification, and Android plus macOS updater backends.

**Architecture:** Keep update orchestration in a new shared crate and push platform-specific install behavior into dedicated extension crates. Treat `blinc_cli` as the owner of package/release metadata, load `BlincProject` without lossy conversion for release paths, and keep `crates/blinc_platform` unchanged. Implement real behavior for Android and macOS first, with explicit stubs for Windows and Linux.

**Tech Stack:** Rust 2021, Cargo workspace crates, `serde`/`serde_json`, `thiserror`, `anyhow`, `sha2`, `ed25519-dalek` or `ring`, Blinc CLI/config scaffolding, platform extension crates.

---

## Global Constraints / Guardrails

- Follow TDD for every new behavior: write a failing test first, confirm the failure, then add the minimum code to pass.
- Keep `crates/blinc_platform::Platform` unchanged in this plan.
- Do not add updater-only duplicate identity fields such as Android `expected_package`.
- Canonical release identity must come from existing platform config or platform packaging metadata.
- Expose at least one user-callable CLI release entry point in v1.
- Treat Windows/Linux as capability-limited stubs in v1.

### Task 1: Fix non-Rust scaffolding identity drift and validate `org`

**Files:**
- Modify: `crates/blinc_cli/src/project.rs`
- Test: `crates/blinc_cli/src/project.rs`

**Step 1: Write the failing tests**

Add tests beside the existing `#[cfg(test)]` module in `crates/blinc_cli/src/project.rs`:

```rust
#[test]
fn non_rust_project_uses_org_for_android_apple_and_linux_ids() {
    // create_project(&root, "Demo App", "default", "io.test") ...
    // assert Android, plist, and Linux metainfo files contain io.test.demo_app
}

#[test]
fn non_rust_project_rejects_unsafe_org_chars() {
    // create_project(..., r#"io.test"; rm -rf /"#) should fail
}
```

**Step 2: Run tests to verify failure**

Run:

```bash
cargo test -p blinc_cli non_rust_project_uses_org_for_android_apple_and_linux_ids -- --nocapture
cargo test -p blinc_cli non_rust_project_rejects_unsafe_org_chars -- --nocapture
```

Expected: FAIL because non-Rust scaffolding still emits `com.example.*` and does not validate `org`.

**Step 3: Write minimal implementation**

Update `create_project()` and its helpers to:

- reuse `validate_org_name()`
- propagate `org` into Android files
- propagate `org` into iOS/macOS bundle IDs
- propagate `org` into Linux AppStream metadata

Keep generated identifiers aligned with `BlincProject::with_all_platforms()`.

**Step 4: Run targeted tests**

Run: `cargo test -p blinc_cli project::tests -- --nocapture`
Expected: PASS with existing scaffold tests plus the new identity and validation tests.

**Step 5: Commit**

```bash
git add crates/blinc_cli/src/project.rs
git commit -m "fix: align non-rust scaffold identities and validate org"
```

### Task 2: Add updater config without duplicating platform identity

**Files:**
- Modify: `crates/blinc_cli/src/config.rs`
- Test: `crates/blinc_cli/src/config.rs`

**Step 1: Write the failing tests**

Add config tests such as:

```rust
#[test]
fn updater_config_round_trips_without_identity_overrides() {
    let toml = r#"
        [project]
        name = "Demo"
        version = "0.1.0"

        [platforms.android]
        package = "io.test.demo"

        [platforms.macos]
        bundle_id = "io.test.demo"

        [updates]
        enabled = true
        channel = "stable"
        manifest_url = "https://example.com/manifest.json"
        public_key = "abc"

        [updates.desktop]
        enabled = true

        [updates.android]
        enabled = true
        allow_unknown_sources_prompt = true
    "#;
    // parse + serialize + parse again
}
```

**Step 2: Run test to verify failure**

Run: `cargo test -p blinc_cli updater_config_round_trips_without_identity_overrides -- --nocapture`
Expected: FAIL because `BlincProject` has no updater schema.

**Step 3: Write minimal implementation**

Add new config structs:

- `UpdatesConfig`
- `DesktopUpdateConfig`
- `AndroidUpdateConfig`
- `ReleaseChannel` string representation for config

Do not add any updater-owned package or bundle ID field. Extend `BlincProject` with `#[serde(default)] pub updates: UpdatesConfig`.

**Step 4: Run focused tests**

Run: `cargo test -p blinc_cli updater_config_round_trips_without_identity_overrides -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_cli/src/config.rs
git commit -m "feat: add updater config without duplicate app identity"
```

### Task 3: Preserve full project metadata for release commands

**Files:**
- Modify: `crates/blinc_cli/src/config.rs`
- Modify: `crates/blinc_cli/src/main.rs`
- Create: `crates/blinc_cli/src/release.rs`
- Test: `crates/blinc_cli/src/release.rs`

**Step 1: Write the failing tests**

Add tests that prove release code sees fields missing from the legacy `BlincConfig` projection:

```rust
#[test]
fn release_loader_preserves_updates_and_non_legacy_platforms() {}
```

**Step 2: Run test to verify failure**

Run: `cargo test -p blinc_cli release_loader_preserves_updates_and_non_legacy_platforms -- --nocapture`
Expected: FAIL because current release paths would be forced through lossy `BlincConfig`.

**Step 3: Write minimal implementation**

Add a release-focused loading path that either:

- loads `BlincProject` directly, or
- extends `BlincConfig` and `from_project()` so release commands can access `updates`, `macos`, `windows`, `linux`, and `wasm`

Make the choice explicit in code and tests.

**Step 4: Run focused tests**

Run: `cargo test -p blinc_cli release_loader_preserves_updates_and_non_legacy_platforms -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_cli/src/config.rs crates/blinc_cli/src/main.rs crates/blinc_cli/src/release.rs
git commit -m "refactor: preserve full project metadata for release commands"
```

### Task 4: Define and expose a user-callable CLI release contract

**Files:**
- Modify: `crates/blinc_cli/src/main.rs`
- Modify: `crates/blinc_cli/src/release.rs`
- Test: `crates/blinc_cli/src/release.rs`

**Step 1: Write the failing black-box test**

Add a CLI-level test such as:

```rust
#[test]
fn release_manifest_command_writes_manifest_json() {
    // invoke CLI with a temp project and assert manifest file is created
}
```

**Step 2: Run test to verify failure**

Run: `cargo test -p blinc_cli release_manifest_command_writes_manifest_json -- --nocapture`
Expected: FAIL because the CLI exposes no release command yet.

**Step 3: Write minimal implementation**

Add a user-callable release surface. The minimum acceptable contract is:

- `blinc release manifest`

with explicit input and output arguments. Document that this is the v1 public interface even if packaging is still incomplete.

**Step 4: Run focused tests**

Run: `cargo test -p blinc_cli release_manifest_command_writes_manifest_json -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_cli/src/main.rs crates/blinc_cli/src/release.rs
git commit -m "feat: add user-callable release manifest command"
```

### Task 5: Create the shared updater domain crate

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/blinc_update/Cargo.toml`
- Create: `crates/blinc_update/src/lib.rs`
- Create: `crates/blinc_update/src/error.rs`
- Create: `crates/blinc_update/src/manifest.rs`
- Create: `crates/blinc_update/src/service.rs`
- Create: `crates/blinc_update/src/version.rs`
- Test: `crates/blinc_update/src/lib.rs`

**Step 1: Write the failing tests**

Start with unit tests for:

```rust
#[test]
fn selects_matching_artifact_for_platform_and_arch() {}

#[test]
fn rejects_manifest_with_missing_target_id() {}

#[test]
fn version_check_detects_newer_release() {}
```

**Step 2: Run test to verify failure**

Run: `cargo test -p blinc_update`
Expected: FAIL because crate does not exist yet.

**Step 3: Write minimal implementation**

Create the crate and implement:

- manifest structs using per-artifact `target_id`
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

### Task 6: Add artifact signing to release generation

**Files:**
- Modify: `crates/blinc_cli/Cargo.toml`
- Modify: `crates/blinc_cli/src/release.rs`
- Modify: `crates/blinc_update/src/manifest.rs`
- Test: `crates/blinc_cli/src/release.rs`

**Step 1: Write the failing tests**

Add tests such as:

```rust
#[test]
fn release_manifest_populates_artifact_signatures() {}

#[test]
fn release_manifest_generation_requires_private_key_input() {}
```

**Step 2: Run test to verify failure**

Run:

```bash
cargo test -p blinc_cli release_manifest_populates_artifact_signatures -- --nocapture
cargo test -p blinc_cli release_manifest_generation_requires_private_key_input -- --nocapture
```

Expected: FAIL because release generation does not sign artifacts yet.

**Step 3: Write minimal implementation**

Add release signing support that:

- accepts signing key material from a documented CLI argument or environment variable
- computes artifact signatures during manifest generation
- writes `artifact.signature`
- exposes the matching public key for application configuration
- picks one concrete crypto stack and uses it consistently across release generation and runtime verification

Use fixture keys in tests rather than real release keys.

**Step 4: Run focused tests**

Run: `cargo test -p blinc_cli release -- --nocapture`
Expected: PASS for signing-related tests.

**Step 5: Commit**

```bash
git add crates/blinc_cli/Cargo.toml crates/blinc_cli/src/release.rs crates/blinc_update/src/manifest.rs
git commit -m "feat: sign release artifacts during manifest generation"
```

### Task 7: Add runtime checksum and signature verification helpers

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
- detached signature using the same public-key scheme produced by release generation

Use the same concrete crypto crates chosen in Task 6 rather than a second parallel implementation.

Return typed `UpdateError` values. Keep this layer file-based and deterministic.

**Step 4: Run tests**

Run: `cargo test -p blinc_update verify -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_update/Cargo.toml crates/blinc_update/src/lib.rs crates/blinc_update/src/verify.rs
git commit -m "feat: add updater artifact verification helpers"
```

### Task 8: Add Android updater backend crate

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
fn android_backend_rejects_artifact_with_wrong_target_id() {}

#[test]
fn android_backend_builds_install_intent_for_matching_apk() {}
```

**Step 2: Run tests to verify failure**

Run: `cargo test -p blinc_update_android`
Expected: FAIL because crate and backend do not exist.

**Step 3: Write minimal implementation**

Implement a backend that:

- chooses Android APK artifacts
- validates `artifact.target_id` against `platforms.android.package`
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

### Task 9: Add desktop updater backend crate with macOS reference backend

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
- macOS install intent builder using canonical bundle identity
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

### Task 10: Surface updater config and release contract in generated docs

**Files:**
- Modify: `crates/blinc_cli/src/project.rs`
- Modify: `crates/blinc_cli/README.md`
- Test: `crates/blinc_cli/src/project.rs`

**Step 1: Write the failing tests**

Add a scaffolding test that asserts generated projects include updater configuration examples or placeholders:

```rust
#[test]
fn generated_project_includes_updater_config_and_release_manifest_guidance() {}
```

**Step 2: Run the test**

Run: `cargo test -p blinc_cli generated_project_includes_updater_config_and_release_manifest_guidance -- --nocapture`
Expected: FAIL because templates do not include updater guidance.

**Step 3: Write minimal implementation**

Update generated README and config examples to describe:

- updater config location
- desktop and Android support
- iOS exclusion
- the user-callable `blinc release manifest` entry point

Keep templates simple; do not add fake implementation code.

**Step 4: Run tests**

Run: `cargo test -p blinc_cli generated_project_includes_updater_config_and_release_manifest_guidance -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/blinc_cli/src/project.rs crates/blinc_cli/README.md
git commit -m "docs: surface updater config and release contract"
```

### Task 11: Add end-to-end release and runtime verification

**Files:**
- Modify: `crates/blinc_cli/src/main.rs`
- Modify: `crates/blinc_cli/src/release.rs`
- Test: `crates/blinc_cli/src/release.rs`
- Test: `crates/blinc_update/src/lib.rs`
- Test: `extensions/blinc_update_android/src/lib.rs`
- Test: `extensions/blinc_update_desktop/src/lib.rs`

**Step 1: Write the failing integration-style tests**

Add tests that:

- invoke `blinc release manifest` on a temp project
- parse the resulting manifest through `blinc_update`
- assert signatures and `target_id` fields are present
- assert Android and macOS selectors choose the correct artifacts

**Step 2: Run test to verify failure**

Run: `cargo test -p blinc_cli generated_manifest_round_trips_into_update_domain -- --nocapture`
Expected: FAIL until the CLI contract, manifest schema, and runtime domain align.

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

### Task 12: Final verification, formatting, and documentation pass

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
