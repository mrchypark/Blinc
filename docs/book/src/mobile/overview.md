# Mobile Development

Blinc can share Rust UI code across desktop, Android, and iOS targets. Mobile
support currently provides generated platform projects, native bridge wiring,
touch forwarding, and renderer scaffolding, but it should still be treated as a
preview surface rather than a finished production target.

## Cross-Platform Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                      Your Blinc App                          │
│         (Shared Rust UI code, state, animations)             │
└─────────────────────────────┬───────────────────────────────┘
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
    ┌────▼────┐         ┌─────▼─────┐        ┌────▼────┐
    │ Desktop │         │  Android  │        │   iOS   │
    │ (wgpu)  │         │ (NDK)     │        │ (UIKit) │
    └─────────┘         └───────────┘        └─────────┘
```

## Key Features

- **Shared UI Code**: Reuse Rust UI trees and state across platforms
- **Generated Platform Projects**: Android Gradle and iOS Xcode templates
- **Native Bridge Wiring**: Template projects register common mobile helpers
- **Touch Support**: Touch input is forwarded into the shared event system
- **Reactive State**: Same reactive state system as desktop
- **Animations**: Shared animation/state APIs are available on mobile too

## Supported Platforms

| Platform | Runtime | Min Version  | Status |
|----------|---------|--------------|--------|
| Android  | NDK + JNI bridge | API 24 (7.0) | Preview |
| iOS      | UIKit + native bridge | iOS 15+ | Preview |

## Project Structure

A typical Blinc mobile project looks like this:

```text
my-app/
├── Cargo.toml           # Rust dependencies
├── blinc.toml           # Blinc project config
├── src/
│   └── main.rs          # Shared UI code
├── platforms/
│   ├── android/         # Android-specific files
│   │   ├── app/
│   │   │   └── src/main/
│   │   │       ├── AndroidManifest.xml
│   │   │       └── kotlin/.../MainActivity.kt
│   │   └── build.gradle.kts
│   └── ios/             # iOS-specific files
│       ├── BlincApp/
│       │   ├── AppDelegate.swift
│       │   ├── BlincViewController.swift
│       │   └── Info.plist
│       └── BlincApp.xcodeproj/
└── build-android.sh     # Build scripts
```

## Quick Start

### 1. Create a new mobile project

```bash
blinc new my-app --template rust
cd my-app
```

### 2. Write your UI

```rust
use blinc_app::prelude::*;

fn app(ctx: &mut WindowedContext) -> impl ElementBuilder {
    let count = ctx.use_state_keyed("count", || 0i32);

    div()
        .w(ctx.width)
        .h(ctx.height)
        .bg(Color::from_hex(0x1a1a2e))
        .flex_col()
        .items_center()
        .justify_center()
        .gap(20.0)
        .child(counter_display(count.clone()))
        .child(counter_button("+", count.clone(), 1))
}

fn counter_display(count: State<i32>) -> impl ElementBuilder {
    // Stateful elements with deps update incrementally when dependencies change
    stateful::<NoState>()
        .deps([count.signal_id()])
        .on_state(move |_ctx| {
            text(format!("Count: {}", count.get()))
                .size(48.0)
                .color(Color::WHITE)
        })
}

fn counter_button(label: &str, count: State<i32>, delta: i32) -> impl ElementBuilder {
    let label = label.to_string();
    stateful::<ButtonState>()
        .on_state(move |ctx| {
            let bg = match ctx.state() {
                ButtonState::Idle => Color::from_hex(0x4a4a5a),
                ButtonState::Hovered => Color::from_hex(0x5a5a6a),
                ButtonState::Pressed => Color::from_hex(0x3a3a4a),
                ButtonState::Disabled => Color::from_hex(0x2a2a2a),
            };
            div()
                .w(80.0).h(50.0)
                .rounded(8.0)
                .bg(bg)
                .items_center()
                .justify_center()
                .child(text(&label).size(24.0).color(Color::WHITE))
        })
        .on_click(move |_| count.set(count.get() + delta))
}
```

### 3. Build and run

```bash
# Android
blinc run android

# iOS
blinc run ios
```

## Next Steps

- [Android Development](./android.md) - Set up Android toolchain and build
- [iOS Development](./ios.md) - Set up iOS toolchain and build
- [CLI Reference](./cli.md) - Full CLI command reference

## Current Scope

- The mobile templates handle project wiring and native bridge registration.
- Runtime coverage is still uneven across rendering, lifecycle, and platform
  services, so validate target behavior on-device before treating it as
  production-ready.
