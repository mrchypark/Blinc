# Blinc WebView Embedding Packaging Baseline (macOS/Windows/Linux)

Task 6 baseline for release packaging in the `Blinc-wt-webview-packaging` worktree.

## Source of Truth

- `toolchain/targets/macos.toml`
- `toolchain/targets/windows.toml`
- `toolchain/targets/linux.toml`

## Baseline Output Contract

| Target | Artifact name | Default output path (smoke) | Signing / security assumptions |
| --- | --- | --- | --- |
| macOS | `BlincApp.app` | `target/package/macos/BlincApp.app` | Code signing identity from `[signing].identity` (default `-` for ad-hoc). Optional notarization is controlled by `[notarization]` fields and environment (`BLINC_APPLE_ID`, `BLINC_APPLE_PASSWORD`, `BLINC_APPLE_TEAM_ID`). |
| Windows | `blincapp.exe` and installer `blincapp.msix` | `target/package/windows/blincapp.exe` and `target/package/windows/blincapp.msix` | Release signing from `[signing].certificate` and password via env when enabled. |
| Linux | `blinc-app` and `blinc-app.AppImage` | `target/package/linux/blinc-app` and `target/package/linux/blinc-app.AppImage` | Not required in smoke by default; package metadata exists in `[appimage]`, `[flatpak]`, `[snap]` sections.

## Script Contract (`scripts/package-smoke.sh`)

- Target filter required via one or multiple `--target` values (`macos`, `windows`, `linux`).
- `--dry-run` prints deterministic build command plan and resolved artifact paths.
- `--output-root` defaults to `target/package`.
- `--expect-artifact <path>` verifies an artifact exists; returns non-zero with a clear error if missing.
- `--version` can override version metadata shown in output.

## Non-goals for Task 6

- App-store submission and distribution storefront workflows.
- Runtime code changes to `blinc_platform`, `blinc_app`, or rendering crates.
- Signing certificate bootstrap logic.
