# Blinc WebView Embed Packaging Release Runbook (Task 8)

Final verification and rollback runbook for the `Blinc-wt-webview-packaging` worktree.

## Scope

- Consolidate final release checks for formatting, compile/test/build, and packaging smoke.
- Capture rollback procedures for compile-time and runtime fallback behavior.
- Keep package-smoke checks deterministic by using `--dry-run`.

## Final Verification Commands

Run from repository root:

```bash
cargo fmt --all
cargo fmt --all -- --check
cargo check --workspace --all-features
cargo test --workspace --all-features
cargo build --workspace --all-features
./scripts/package-smoke.sh --dry-run --target macos --target windows --target linux
./scripts/package-smoke.sh --dry-run --target macos --expect-artifact target/package/macos/__missing__.app
```

## Expected Results

- `cargo fmt --all` completes without changing tracked formatting policy.
- `cargo fmt --all -- --check` exits with status `0`.
- `cargo check --workspace --all-features` exits with status `0`.
- `cargo test --workspace --all-features` exits with status `0`.
- `cargo build --workspace --all-features` exits with status `0`.
- `scripts/package-smoke.sh --dry-run ...` prints target metadata and deterministic artifact paths for macOS/Windows/Linux.
- Negative artifact check exits non-zero with:
  - `error: expected artifact not found: target/package/macos/__missing__.app`

## Rollback Procedures

### 1) Compile-time rollback: disable `webview` feature

Use this when the embedded WebView integration must be excluded from build output.

```bash
cargo check -p blinc_app --no-default-features --features windowed
```

What this achieves:

- Removes `webview` feature-gated code paths from compilation.
- Keeps desktop `windowed` flow compile-valid without embedded webview dependency activation.

### 2) Runtime rollback: unset WebView env controls

Use this when you want runtime behavior to avoid custom webview URL/origin inputs.

```bash
unset BLINC_WEBVIEW_URL
unset BLINC_WEBVIEW_ALLOW_ORIGINS
```

What this achieves:

- `BLINC_WEBVIEW_URL` unset: no explicit startup URL override is provided.
- `BLINC_WEBVIEW_ALLOW_ORIGINS` unset: policy falls back to default restrictive behavior.
- Desktop lifecycle remains graceful when webview backend is unavailable (`PlatformError::Unavailable` is non-fatal).

## Release Operator Notes

- `scripts/package-smoke.sh` supports dry-run validation for `macos`, `windows`, and `linux` targets and reports expected artifact locations.
- Keep negative artifact check in the final gate to ensure missing outputs fail loudly before release publication.
- If verification regresses, run rollback steps first, then re-run the command block above to confirm stabilization.
