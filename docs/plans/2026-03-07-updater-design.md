# Updater Architecture Design (Blinc)

## Problem

Blinc currently has desktop runtime crates and platform scaffolding, but it does not have a real packaging or release pipeline for application updates. The current desktop extension is responsible for windowing and input only, while `blinc build` does not yet produce installable artifacts. That means update support cannot be treated as a small add-on inside `blinc_platform_desktop`; it needs a dedicated domain model plus packaging and release metadata support.

There is also a correctness gap in project scaffolding: `.blincproj` records organization-aware package and bundle identifiers, but the non-Rust project templates still emit `com.example.*` identifiers into generated Android/iOS/macOS files. Any updater design built on top of unstable app identifiers will fail later when signatures, manifests, and installation handoff rely on those IDs.

## Scope

- Desktop self-update support
- Android private/development distribution update support
- Shared updater interfaces, state, manifest schema, and verification rules
- CLI support for package/release metadata generation

## Explicit Non-Goals

- iOS in-app self-update
- Background auto-download in v1
- Full Windows/Linux installer implementation in v1
- Replacing the existing `blinc_platform::Platform` trait with an update-aware trait

## Options Considered

### Option A: Add updater code directly to platform crates

- Put desktop update code in `extensions/blinc_platform_desktop`
- Put Android update code in `extensions/blinc_platform_android`

**Pros**
- Fastest path to a spike
- Fewer new packages

**Cons**
- Mixes runtime platform APIs with release/distribution concerns
- Hard to test and reason about separately
- Makes future signing/release policies leak into low-level windowing/native bridge code

### Option B: Create a shared updater domain crate plus platform backends (Chosen)

- Add a new shared crate for updater types and orchestration
- Implement platform-specific backends in separate extension crates
- Extend `blinc_cli` to emit manifests and release metadata

**Pros**
- Keeps clear package boundaries
- Matches the current `crates/` plus `extensions/` architecture
- Allows desktop and Android to share state, verification, and manifest parsing
- Lets Windows/Linux remain stubs while macOS and Android become reference implementations

**Cons**
- More initial package scaffolding
- Requires CLI and config work before the runtime feature feels complete

### Option C: External-only updater helper

- App only exposes UI
- All update work is delegated to an external CLI/helper

**Pros**
- Simpler app runtime
- Strong separation from installer logic

**Cons**
- Weak in-app state tracking
- Poor Android UX for private APK distribution
- Pushes too much product behavior out of the framework

## Decision

Choose **Option B**.

Blinc should introduce a dedicated updater domain package, keep platform-specific installation logic in separate extension packages, and let `blinc_cli` own artifact packaging plus release manifest generation. The updater should be interface-first, with platform-specific installation behavior hidden behind backends.

## Architectural Boundaries

### 1. Shared updater domain

Add a new crate:

- `crates/blinc_update`

Responsibilities:

- version comparison policy
- release channels
- manifest parsing and validation
- download lifecycle state machine
- signature and checksum verification
- public API traits for check/download/install orchestration

Representative types:

- `UpdateService`
- `UpdateBackend`
- `UpdateCheckRequest`
- `UpdateManifest`
- `ReleaseChannel`
- `UpdateState`
- `UpdateError`
- `InstallIntent`

This crate should not know about `winit`, Android intents, AppKit, MSI/MSIX, or package installers.

### 2. Platform backends

Add extension crates:

- `extensions/blinc_update_desktop`
- `extensions/blinc_update_android`

Responsibilities:

- platform-specific capability checks
- installer handoff
- relaunch/restart hooks
- storage path decisions for downloaded artifacts

`extensions/blinc_update_desktop` should expose separate modules for:

- `macos`
- `windows`
- `linux`

V1 expectation:

- `macos`: real implementation
- `windows`: explicit stub or capability-limited implementation
- `linux`: explicit stub or capability-limited implementation

`extensions/blinc_update_android` should implement APK-based private/development distribution update handoff and validation rules.

### 3. Existing platform abstraction stays unchanged

Do **not** extend `crates/blinc_platform::Platform` for v1.

Reason:

- `Platform` is currently about window/event-loop lifecycle
- updater responsibilities are product distribution concerns
- adding updater APIs there would force unrelated platform crates to implement a surface they do not own

The updater should be created and injected separately by application code or helper APIs in `blinc_app` later if needed.

### 4. CLI owns package/release metadata

`crates/blinc_cli` must become the source of truth for:

- package artifact generation
- release manifest generation
- release signing metadata
- channel-aware output layout

This implies `blinc build` alone is not sufficient. The CLI should evolve toward:

- `build`
- `package`
- `release`

with `release` producing both artifacts and update metadata.

## Configuration Model

Extend `.blincproj` with a dedicated updater section.

Example shape:

```toml
[updates]
enabled = true
channel = "stable"
manifest_url = "https://example.com/releases/manifest.json"
public_key = "base64-ed25519-public-key"

[updates.desktop]
enabled = true
restart_strategy = "prompt"

[updates.android]
enabled = true
allow_unknown_sources_prompt = true
expected_package = "com.example.myapp"
```

Rules:

- global fields live under `[updates]`
- only desktop and Android platform overrides are supported in v1
- iOS has no updater section beyond possible future release metadata
- `manifest_url` and `public_key` are required when updates are enabled

## Release Manifest

Use a shared JSON manifest for runtime consumption.

Example:

```json
{
  "schema_version": 1,
  "app_id": "com.example.myapp",
  "channel": "stable",
  "version": "1.2.3",
  "published_at": "2026-03-07T00:00:00Z",
  "notes_url": "https://example.com/releases/1.2.3",
  "artifacts": [
    {
      "platform": "macos",
      "arch": "universal",
      "url": "https://example.com/releases/myapp-1.2.3-macos.zip",
      "size": 12345678,
      "sha256": "hex",
      "signature": "base64"
    },
    {
      "platform": "android",
      "arch": "arm64-v8a",
      "url": "https://example.com/releases/myapp-1.2.3.apk",
      "size": 9876543,
      "sha256": "hex",
      "signature": "base64"
    }
  ]
}
```

Rules:

- signatures are per artifact, not only per manifest
- the runtime must verify both checksum and signature before handoff
- `app_id` must match the package/bundle identifier expected by the target platform

## Runtime Flow

V1 uses an explicit user-driven flow:

1. Check for updates
2. Show availability and release notes
3. Ask for explicit approval
4. Download artifact
5. Verify checksum and signature
6. Hand off to platform installer path
7. Prompt to restart or continue with a deferred restart

No background auto-download in v1.

## Platform-Specific Behavior

### Desktop

Desktop should never rely on “overwrite the currently running executable in place” as a generic strategy.

Instead:

- `macos`: install via app bundle replacement or helper handoff
- `windows`: eventually install via MSI/MSIX/external installer strategy
- `linux`: eventually install via AppImage/package-manager/packaged artifact strategy

The shared updater service only emits an install intent; the platform backend decides how to perform or delegate installation.

### Android

Android private/development distribution should use APK-based update handoff:

- verify downloaded APK metadata
- confirm package identifier matches expected app
- hand off to Android package installer intent
- report install handoff success/failure, not “update finished inside the app”

The framework should treat Android install as a controlled handoff, not a self-managed binary replacement.

### iOS

iOS is excluded from the in-app updater scope.

## Required Precondition Fixes

Before updater implementation starts, fix the project scaffolding mismatch:

- `.blincproj` already stores organization-aware IDs
- generated non-Rust Android/iOS/macOS files must stop hardcoding `com.example.*`

Without this fix, any release manifest keyed by app ID will drift from generated projects.

## Rollout Strategy

1. Fix identifier propagation in CLI scaffolding and config serialization
2. Introduce `blinc_update` with tests and no platform side effects
3. Add CLI release manifest generation and signing placeholders
4. Implement Android private-distribution updater backend
5. Implement desktop shared backend surface
6. Implement macOS reference backend
7. Add Windows/Linux stubs with explicit unsupported or not-yet-implemented responses

## Testing Strategy

### Unit tests

- version comparison
- manifest parsing
- signature/checksum validation
- config serialization/deserialization
- scaffolded project identifier propagation

### Integration tests

- CLI release manifest generation
- Android handoff request construction
- macOS install intent construction
- unsupported backend behavior for Windows/Linux stubs

### Manual verification

- generate a sample app with non-default org/package IDs
- produce package/release artifacts
- validate manifest contents
- exercise explicit update flow on Android private build and macOS sample app

## Acceptance Criteria

- Blinc has a dedicated updater domain crate with a stable public interface
- `.blincproj` can describe updater settings for desktop and Android
- non-Rust project scaffolding emits correct org-aware package/bundle identifiers
- CLI can generate signed release metadata for updater consumption
- Android private/development distribution update flow works through explicit install handoff
- macOS has a reference desktop backend
- Windows/Linux expose explicit unsupported or stubbed behavior rather than pretending to support updates
