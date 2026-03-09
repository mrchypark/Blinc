# blinc_platform_ios

> **Part of the [Blinc UI Framework](https://project-blinc.github.io/Blinc)**
>
> This crate is a component of Blinc, a GPU-accelerated UI framework for Rust.
> For full documentation and guides, visit the [Blinc documentation](https://project-blinc.github.io/Blinc).

iOS platform implementation for Blinc UI.

For the repo-wide native support contract, see
[`docs/native-readiness.md`](../../docs/native-readiness.md).

## Overview

`blinc_platform_ios` provides UIKit integration, Metal rendering, and touch input handling for iOS and iPadOS applications.

## Supported Platforms

- iOS 14.0+
- iPadOS 14.0+

## Current Support Tier

- Tier 1: partial
- Tier 2: partial
- Tier 3: deferred

This crate currently exposes UIKit integration, rendering hooks, touch handling,
and environment snapshot helpers. It does not yet guarantee full mobile IME,
accessibility parity, or complete production packaging flows.

## Features

- **UIKit Integration**: Native iOS view hierarchy
- **Metal Rendering**: Hardware-accelerated graphics
- **Touch Input**: Full multi-touch support
- **iOS Lifecycle**: Partial runtime lifecycle wiring
- **Safe Area**: Environment snapshot helpers for current window metrics/insets

## Quick Start

```rust
use blinc_platform_ios::ios_main;

#[no_mangle]
pub extern "C" fn main() {
    // Initializes the Rust side of the iOS platform integration.
    ios_main();
}
```

Today, UI construction and lifecycle wiring are still coordinated from the host
UIKit/Xcode application rather than a closure-based Rust entrypoint.

## Project Setup

### Cargo.toml

```toml
[lib]
crate-type = ["staticlib"]

[dependencies]
blinc_platform_ios = "0.1"
```

### Xcode Project

1. Create a new iOS project in Xcode
2. Add your Rust library as a dependency
3. Configure the bridging header
4. Set up the Metal view

### Info.plist

```xml
<key>UILaunchStoryboardName</key>
<string>LaunchScreen</string>
<key>UISupportedInterfaceOrientations</key>
<array>
    <string>UIInterfaceOrientationPortrait</string>
    <string>UIInterfaceOrientationLandscapeLeft</string>
    <string>UIInterfaceOrientationLandscapeRight</string>
</array>
```

## Touch Handling

```rust
fn handle_touch(event: TouchEvent) {
    match event.phase {
        TouchPhase::Began => {
            // Touch started
        }
        TouchPhase::Moved => {
            // Touch moved
        }
        TouchPhase::Ended => {
            // Touch ended
        }
        TouchPhase::Cancelled => {
            // Touch cancelled
        }
    }
}
```

## Safe Area Snapshot

```rust
use blinc_platform_ios::get_safe_area_insets;

let insets = get_safe_area_insets();

// Returns (top, left, bottom, right)
let (top, left, bottom, right) = insets;
```

## Lifecycle Hooks

```rust
// Lifecycle callbacks are currently managed by the host UIKit application.
// Use ios_main() for Rust-side initialization, then forward lifecycle events
// from AppDelegate / SceneDelegate into your app-specific integration layer.
```

## Building

```bash
# Build for iOS Simulator
cargo build --target aarch64-apple-ios-sim

# Build for iOS Device
cargo build --target aarch64-apple-ios --release

# Build universal binary
cargo lipo --release
```

## Requirements

- Xcode 14+
- iOS SDK 14.0+
- Rust with iOS targets:
  ```bash
  rustup target add aarch64-apple-ios
  rustup target add aarch64-apple-ios-sim
  ```

## License

MIT OR Apache-2.0
