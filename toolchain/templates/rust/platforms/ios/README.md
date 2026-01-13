# iOS Platform Setup

## Prerequisites

- Xcode 15.0+
- Rust with iOS targets: `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`
- cargo-lipo: `cargo install cargo-lipo`

## Building the Rust Library

```bash
# From project root
cargo lipo --release
```

This creates a universal static library at `target/universal/release/lib{{project_name_snake}}.a`

## Xcode Project Setup

1. Create a new iOS App project in Xcode
2. Add the Swift files from `BlincApp/` to your project
3. Add the bridging header:
   - Go to Build Settings > Swift Compiler - General
   - Set "Objective-C Bridging Header" to `$(SRCROOT)/BlincApp/Blinc-Bridging-Header.h`
4. Link the Rust static library:
   - Go to Build Phases > Link Binary With Libraries
   - Add `lib{{project_name_snake}}.a` from `target/universal/release/`
5. Add required frameworks:
   - Metal.framework
   - MetalKit.framework
   - QuartzCore.framework

## Running

1. Build the Rust library: `cargo lipo --release`
2. Open the Xcode project
3. Select your target device/simulator
4. Build and run (Cmd+R)
