# Native Readiness

This document defines the current native support contract for Blinc.

`native-ready` is not a single yes/no state in this repository. Support is
tracked in tiers so platform behavior, examples, and templates can describe the
same contract.

## Tiers

- `Tier 1`: render, touch/input dispatch, lifecycle wiring, viewport/environment snapshot
- `Tier 2`: text input/IME, permissions, and basic native services
- `Tier 3`: accessibility, advanced OS integration, and release productization

## Platform Matrix

| Platform | Tier 1 | Tier 2 | Tier 3 | Notes |
|----------|--------|--------|--------|-------|
| Android | Partial | Partial | Deferred | Native bridge, sensors, permissions, clipboard/share/haptics paths exist. IME/runtime lifecycle and release productization are still being hardened. |
| iOS | Partial | Partial | Deferred | UIKit bridge, rendering, sensors, and environment snapshot work exists. IME/accessibility parity and some native permission paths are still incomplete. |
| Fuchsia | Unsupported | Unsupported | Unsupported | In-tree runtime is explicitly unsupported. Template files are scaffolding only. |
| HarmonyOS | Deferred | Deferred | Deferred | Not a verification target in the current repo. |

For the concrete mobile packaging contract, artifact paths, and signing inputs,
see [`docs/mobile-release.md`](mobile-release.md).

## Canonical References

- [`mobile/example`](../mobile/example/README.md): canonical native reference app for current bridge/runtime work
- [`examples/counter`](../examples/counter/README.md): scaffold example, not the native readiness reference
- [`toolchain/templates/rust/platforms/android/README.md`](../toolchain/templates/rust/platforms/android/README.md): Android scaffold contract
- [`toolchain/templates/rust/platforms/fuchsia/README.md`](../toolchain/templates/rust/platforms/fuchsia/README.md): deferred/unsupported Fuchsia scaffolding

## Rules

- No mobile runtime path should silently succeed with stub behavior.
- If a platform path is not implemented, it should return an explicit unsupported error or be documented as deferred.
- Example and template documentation should describe the tier they actually satisfy, not an aspirational end state.
