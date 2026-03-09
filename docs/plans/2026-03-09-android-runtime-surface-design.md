# Android Runtime Surface Design

**Scope:** `extensions/blinc_platform_android` runtime surface only.

## Problem

The current Android runtime exposes JNI entry points and lifecycle hooks, but the
core runtime state is still mostly placeholder logic:

- JNI bridge tracks only raw size/touch booleans
- resize/render/touch handlers do not preserve enough state to drive a host app
- activity lifecycle logs events but does not keep a coherent renderability model

This makes the Android surface hard to integrate even before a full GPU renderer
exists.

## Goal

Make the Android runtime surface truthful and useful as bridge scaffolding by
preserving the state that a renderer or host app actually needs.

## Recommended Approach

### 1. Introduce a host-testable JNI runtime state model

Split state transitions out of raw JNI functions into a pure Rust state object
that tracks:

- current size and scale factor
- focus/render eligibility
- pending redraw requests
- queued touch events in logical coordinates
- last rendered frame metadata

JNI entry points should only translate inputs and delegate to this state object.

### 2. Make render/resize/touch mutate meaningful state

- init marks the surface dirty
- resize marks pending redraw and updates dimensions
- touch input queues translated logical events and requests redraw
- render consumes pending redraw markers and records the rendered surface size

This still does not claim a full GPU pipeline exists, but it removes meaningless
no-op behavior.

### 3. Keep activity lifecycle aligned with bridge semantics

`BlincAndroidApp` should share the same story:

- render only when focused and a window exists
- lifecycle changes should preserve redraw intent without conflating resume with
  actual focus gain
- tests should cover these transitions without Android runtime

## Non-Goals

- Building a complete Vulkan/wgpu renderer
- Implementing full event-router delivery into Blinc layout/input systems
- Adding Android instrumentation tests

## Testing

- Unit tests for the pure JNI runtime state model
- Focused tests for activity renderability transitions
- `cargo test -p blinc_platform_android`
