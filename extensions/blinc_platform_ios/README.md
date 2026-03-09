# blinc_platform_ios

> **Part of the [Blinc UI Framework](https://project-blinc.github.io/Blinc)**
>
> This crate is a component of Blinc, a GPU-accelerated UI framework for Rust.
> For full documentation and guides, visit the [Blinc documentation](https://project-blinc.github.io/Blinc).

iOS platform scaffolding for Blinc UI.

## Overview

`blinc_platform_ios` provides the current iOS runtime surface for Blinc:
UIKit integration points, touch forwarding, native bridge helpers, and the
render-loop scaffolding used by generated templates.

## Supported Platforms

- iOS 14.0+
- iPadOS 14.0+

## Features

- **UIKit Integration**: Template-friendly app shell
- **Touch Input**: Touch forwarding into Blinc events
- **Native Bridge**: Rust-to-Swift interoperability
- **Template Support**: Bridge registration from generated projects

## Quick Start

Use the generated iOS template and connect your Rust static library through the
provided bridging header and `BlincNativeBridge`.

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

## Status

The iOS runtime is still under active development. Treat the crate as bridge and
template scaffolding for shared UI work, and validate lifecycle, safe-area, and
rendering behavior in your target app before treating it as production ready.

The current runtime surface now reflects UIKit when available for:

- dark-mode detection
- safe-area inset queries
- initial lifecycle/frame bridge events from the Rust-side event loop surface

`IOSEventLoop` should still be treated as a UIKit-managed integration point, not
as a desktop-style blocking owner loop.

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
