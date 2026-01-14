# Mobile Development

Blinc supports building native mobile applications for both Android and iOS platforms. The same Rust UI code runs on mobile with platform-specific rendering backends (Vulkan for Android, Metal for iOS).

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
    │ (wgpu)  │         │ (Vulkan)  │        │ (Metal) │
    └─────────┘         └───────────┘        └─────────┘
```

## Key Features

- **Shared UI Code**: Write your UI once in Rust, deploy everywhere
- **Native Performance**: GPU-accelerated rendering via Vulkan/Metal
- **Touch Support**: Full multi-touch gesture handling
- **Reactive State**: Same reactive state system as desktop
- **Animations**: Spring physics and keyframe animations work seamlessly

## Supported Platforms

| Platform | Backend | Min Version  | Status |
|----------|---------|--------------|--------|
| Android  | Vulkan  | API 24 (7.0) | Stable |
| iOS      | Metal   | iOS 15+      | Stable |

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
        .bg(0x1a1a2e)
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
                .color(0xffffff)
        })
}

fn counter_button(label: &str, count: State<i32>, delta: i32) -> impl ElementBuilder {
    let label = label.to_string();
    stateful::<ButtonState>()
        .on_state(move |ctx| {
            let bg = match ctx.state() {
                ButtonState::Idle => 0x4a4a5a,
                ButtonState::Hovered => 0x5a5a6a,
                ButtonState::Pressed => 0x3a3a4a,
                ButtonState::Disabled => 0x2a2a2a,
            };
            div()
                .w(80.0).h(50.0)
                .rounded(8.0)
                .bg(bg)
                .items_center()
                .justify_center()
                .child(text(&label).size(24.0).color(0xffffff))
        })
        .on_click(move |_| count.set_rebuild(count.get() + delta))
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
