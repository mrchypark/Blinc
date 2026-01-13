# Blinc Mobile SDK Proposal

## Overview

This document describes the Mobile SDK architecture for building Blinc applications on Android and iOS. The SDK enables Rust-first UI development with native platform integration.

## Design Principles

1. **Rust-First UI** - All UI logic, state, and rendering defined in Rust
2. **Native Shell** - Platform code (Kotlin/Swift) handles lifecycle and input
3. **Minimal FFI Surface** - Small, stable C API between Rust and native
4. **Shared Core** - Same `WindowedContext` API across desktop/mobile

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Your Blinc App                           │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  UI Builder (Rust)                                       │   │
│  │  - div(), text(), button() composables                   │   │
│  │  - Reactive state (Signal, Derived)                      │   │
│  │  - Event handlers (on_click, on_scroll)                  │   │
│  └─────────────────────────────────────────────────────────┘   │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                      blinc_app (Rust)                           │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐    │
│  │ WindowedCtx  │  │ EventRouter  │  │ AnimationScheduler │    │
│  └──────────────┘  └──────────────┘  └────────────────────┘    │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐    │
│  │ RenderTree   │  │ ReactiveGraph│  │ OverlayManager     │    │
│  └──────────────┘  └──────────────┘  └────────────────────┘    │
└────────────────────────────┬────────────────────────────────────┘
                             │ C FFI
         ┌───────────────────┴───────────────────┐
         ▼                                       ▼
┌─────────────────────┐             ┌─────────────────────┐
│   Android (Kotlin)  │             │    iOS (Swift)      │
│  ┌───────────────┐  │             │  ┌───────────────┐  │
│  │ BlincActivity │  │             │  │ BlincViewCtrl │  │
│  │ - Surface     │  │             │  │ - CAMetalLayer│  │
│  │ - Touch       │  │             │  │ - Touch       │  │
│  │ - Lifecycle   │  │             │  │ - Lifecycle   │  │
│  └───────────────┘  │             │  └───────────────┘  │
│  ┌───────────────┐  │             │  ┌───────────────┐  │
│  │ Vulkan/OpenGL │  │             │  │ Metal         │  │
│  └───────────────┘  │             │  └───────────────┘  │
└─────────────────────┘             └─────────────────────┘
```

---

## Android SDK

### Integration Pattern

Android apps use the `android-activity` crate with Blinc's `AndroidApp::run()`.

#### Cargo.toml

```toml
[package]
name = "my_blinc_app"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
blinc_app = { path = "../blinc_app", features = ["android"] }

[target.'cfg(target_os = "android")'.dependencies]
android-activity = { version = "0.6", features = ["native-activity"] }
```

#### Rust Entry Point

```rust
// src/lib.rs
use blinc_app::prelude::*;
use blinc_app::android::AndroidApp;

#[no_mangle]
fn android_main(app: android_activity::AndroidApp) {
    AndroidApp::run(app, |ctx| {
        // Your UI here - same API as desktop
        div()
            .w(ctx.width).h(ctx.height)
            .bg([0.1, 0.1, 0.15, 1.0])
            .flex_center()
            .child(
                button("Tap Me")
                    .on_click(|| println!("Clicked!"))
            )
    }).expect("App failed");
}
```

#### Event Flow

```
┌──────────────────────────────────────────────────────────────┐
│                    Android Event Flow                         │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  MotionEvent (NDK)                                           │
│       │                                                       │
│       ▼                                                       │
│  input_events_iter()  ─────────────────────────────────────► │
│       │                                                       │
│       ▼                                                       │
│  Convert to logical coords (physical / scale_factor)         │
│       │                                                       │
│       ▼                                                       │
│  EventRouter.on_mouse_down/move/up()                         │
│       │                                                       │
│       ▼                                                       │
│  Hit testing against RenderTree                              │
│       │                                                       │
│       ▼                                                       │
│  Dispatch to on_click/on_hover callbacks                     │
│       │                                                       │
│       ▼                                                       │
│  State updates trigger rebuild                               │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

#### Touch Actions Supported

| MotionAction | EventRouter Method | Description |
|--------------|-------------------|-------------|
| `Down` | `on_mouse_down()` | First finger touches |
| `Move` | `on_mouse_move()` | Finger moves |
| `Up` | `on_mouse_up()` | Finger lifts |
| `Cancel` | `on_mouse_leave()` | System cancelled |
| `PointerDown` | `on_mouse_down()` | Additional finger |
| `PointerUp` | `on_mouse_up()` | Additional finger lifts |

#### Building for Android

```bash
# Install Android NDK and set ANDROID_NDK_HOME

# Build for arm64
cargo ndk -t arm64-v8a build --release

# Build APK (requires gradle setup)
./gradlew assembleRelease
```

#### Kotlin Embedding (Optional)

For embedding Blinc in an existing Kotlin app:

```kotlin
// BlincBridge.kt
package com.example.myapp

import android.view.Surface

object BlincBridge {
    init {
        System.loadLibrary("my_blinc_app")
    }

    external fun nativeInit(surface: Surface): Long
    external fun nativeRenderFrame(handle: Long)
    external fun nativeOnTouch(handle: Long, action: Int, x: Float, y: Float): Boolean
    external fun nativeDestroy(handle: Long)
}
```

```rust
// JNI bridge in Rust
#[no_mangle]
pub extern "system" fn Java_com_example_myapp_BlincBridge_nativeInit(
    env: jni::JNIEnv,
    _class: jni::objects::JClass,
    surface: jni::objects::JObject,
) -> jni::sys::jlong {
    // Create renderer with Surface
}
```

---

## iOS SDK

### Integration Pattern

iOS apps use Swift as the shell with Blinc's C FFI.

#### Cargo.toml

```toml
[package]
name = "my_blinc_app"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["staticlib"]

[dependencies]
blinc_app = { path = "../blinc_app", features = ["ios"] }
```

#### Rust App with FFI

```rust
// src/lib.rs
use blinc_app::prelude::*;
use blinc_app::windowed::WindowedContext;
use std::sync::atomic::{AtomicI32, Ordering};

static COUNTER: AtomicI32 = AtomicI32::new(0);

/// UI builder - called each frame via blinc_build_frame
#[no_mangle]
pub extern "C" fn my_app_build_ui(ctx: *mut WindowedContext) {
    if ctx.is_null() { return; }
    let ctx = unsafe { &mut *ctx };

    let count = COUNTER.load(Ordering::SeqCst);

    let ui = div()
        .w(ctx.width).h(ctx.height)
        .bg([0.1, 0.1, 0.15, 1.0])
        .flex_center()
        .child(
            div()
                .on_click(|| { COUNTER.fetch_add(1, Ordering::SeqCst); })
                .child(text(&format!("Count: {}", count)).size(32.0))
        );

    ctx.build_element(&ui);
}

/// Initialize app - call once from Swift
#[no_mangle]
pub extern "C" fn my_app_init() {
    use blinc_app::ios::blinc_set_ui_builder;
    blinc_set_ui_builder(my_app_build_ui);
}
```

#### Swift Integration

```swift
// AppDelegate.swift
import UIKit

@main
class AppDelegate: UIResponder, UIApplicationDelegate {
    var window: UIWindow?

    func application(_ application: UIApplication,
                     didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        // Initialize Rust app
        my_app_init()

        window = UIWindow(frame: UIScreen.main.bounds)
        window?.rootViewController = BlincViewController()
        window?.makeKeyAndVisible()
        return true
    }
}
```

#### Bridging Header

```c
// Blinc-Bridging-Header.h

#include <stdint.h>
#include <stdbool.h>

typedef struct IOSRenderContext IOSRenderContext;
typedef struct WindowedContext WindowedContext;
typedef void (*UIBuilderFn)(WindowedContext* ctx);

// Context lifecycle
IOSRenderContext* blinc_create_context(uint32_t width, uint32_t height, double scale_factor);
void blinc_destroy_context(IOSRenderContext* ctx);

// Frame loop
bool blinc_needs_render(IOSRenderContext* ctx);
void blinc_build_frame(IOSRenderContext* ctx);
void blinc_set_ui_builder(UIBuilderFn builder);

// Input
void blinc_handle_touch(IOSRenderContext* ctx, uint64_t touch_id, float x, float y, int32_t phase);
void blinc_set_focused(IOSRenderContext* ctx, bool focused);

// Size
void blinc_update_size(IOSRenderContext* ctx, uint32_t width, uint32_t height, double scale_factor);
float blinc_get_width(IOSRenderContext* ctx);
float blinc_get_height(IOSRenderContext* ctx);

// Your app's functions
void my_app_init(void);
void my_app_build_ui(WindowedContext* ctx);
```

#### Event Flow

```
┌──────────────────────────────────────────────────────────────┐
│                      iOS Event Flow                           │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  UITouch (Swift)                                             │
│       │                                                       │
│       ▼                                                       │
│  touchesBegan/Moved/Ended/Cancelled                          │
│       │                                                       │
│       ▼                                                       │
│  blinc_handle_touch(ctx, id, x, y, phase)  ──────────────►   │
│       │                                                       │
│       ▼                                                       │
│  EventRouter.on_mouse_down/move/up()                         │
│       │                                                       │
│       ▼                                                       │
│  Hit testing against RenderTree                              │
│       │                                                       │
│       ▼                                                       │
│  Dispatch to on_click/on_hover callbacks                     │
│       │                                                       │
│       ▼                                                       │
│  State updates → blinc_needs_render() returns true           │
│       │                                                       │
│       ▼                                                       │
│  CADisplayLink triggers blinc_build_frame()                  │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

#### Touch Phase Values

| Phase | Value | Swift Method |
|-------|-------|--------------|
| Began | 0 | `touchesBegan` |
| Moved | 1 | `touchesMoved` |
| Ended | 2 | `touchesEnded` |
| Cancelled | 3 | `touchesCancelled` |

#### Building for iOS

```bash
# Build for iOS simulator (arm64)
cargo build --release --target aarch64-apple-ios-sim --features ios

# Build for iOS device
cargo build --release --target aarch64-apple-ios --features ios

# Create universal library (optional)
lipo -create \
  target/aarch64-apple-ios/release/libmy_blinc_app.a \
  target/aarch64-apple-ios-sim/release/libmy_blinc_app.a \
  -output libmy_blinc_app_universal.a
```

---

## FFI Reference

### Core Types

| Rust Type | C Type | Description |
|-----------|--------|-------------|
| `IOSRenderContext` | `IOSRenderContext*` | Opaque render context |
| `WindowedContext` | `WindowedContext*` | UI building context |
| `UIBuilderFn` | `void (*)(WindowedContext*)` | UI builder callback |

### iOS FFI Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `blinc_create_context` | `(u32, u32, f64) -> *mut` | Create context |
| `blinc_destroy_context` | `(*mut) -> ()` | Destroy context |
| `blinc_needs_render` | `(*mut) -> bool` | Check if render needed |
| `blinc_build_frame` | `(*mut) -> ()` | Build UI frame |
| `blinc_set_ui_builder` | `(fn) -> ()` | Register builder |
| `blinc_handle_touch` | `(*mut, u64, f32, f32, i32) -> ()` | Handle touch |
| `blinc_update_size` | `(*mut, u32, u32, f64) -> ()` | Update size |
| `blinc_set_focused` | `(*mut, bool) -> ()` | Set focus |
| `blinc_get_width` | `(*mut) -> f32` | Get logical width |
| `blinc_get_height` | `(*mut) -> f32` | Get logical height |
| `blinc_tick_animations` | `(*mut) -> bool` | Tick animations |
| `blinc_mark_dirty` | `(*mut) -> ()` | Force rebuild |

---

## GPU Rendering

### Current State

GPU rendering uses wgpu with platform-specific backends:
- **Android**: Vulkan (primary), OpenGL ES (fallback)
- **iOS**: Metal

### Integration Points

The `BlincViewController.renderFrame()` currently clears to a background color. Full GPU rendering requires:

1. Pass Metal layer/drawable to Rust via FFI
2. Create wgpu surface from CAMetalLayer
3. Render RenderTree using blinc_gpu

### Future: GPU FFI

```c
// Proposed additions
void* blinc_get_render_commands(IOSRenderContext* ctx);
void blinc_render_to_metal(IOSRenderContext* ctx, void* metal_layer);
```

---

## Best Practices

### State Management

```rust
// Use atomic types for FFI-accessible state
static COUNTER: AtomicI32 = AtomicI32::new(0);

// Or use Mutex for complex state
lazy_static! {
    static ref APP_STATE: Mutex<AppState> = Mutex::new(AppState::default());
}
```

### Error Handling

```rust
// Always null-check FFI pointers
#[no_mangle]
pub extern "C" fn my_function(ctx: *mut IOSRenderContext) {
    if ctx.is_null() { return; }
    // ...
}
```

### Memory Management

- Rust owns the `IOSRenderContext` after `blinc_create_context`
- Swift/Kotlin must call `blinc_destroy_context` to free
- Never use context after destruction

### Thread Safety

- All FFI calls must be on the main thread
- Use `Ordering::SeqCst` for cross-FFI atomics
- Animation scheduler runs on background thread

---

## Project Structure

```
my_blinc_app/
├── Cargo.toml
├── src/
│   └── lib.rs              # Rust app + FFI exports
├── ios/
│   ├── MyApp.xcodeproj/
│   ├── MyApp/
│   │   ├── AppDelegate.swift
│   │   ├── BlincViewController.swift
│   │   └── Blinc-Bridging-Header.h
│   └── libmy_blinc_app.a   # Built Rust library
└── android/
    ├── app/
    │   └── src/main/
    │       ├── AndroidManifest.xml
    │       └── java/.../MainActivity.kt
    ├── build.gradle
    └── libs/
        └── arm64-v8a/
            └── libmy_blinc_app.so
```

---

## Migration from Desktop

Desktop Blinc apps using `WindowedApp::run()` can migrate to mobile:

| Desktop | Android | iOS |
|---------|---------|-----|
| `WindowedApp::run(config, \|ctx\| ...)` | `AndroidApp::run(app, \|ctx\| ...)` | `blinc_set_ui_builder(fn)` |
| winit event loop | android-activity poll | CADisplayLink |
| Mouse events | Touch → mouse | Touch → mouse |
| wgpu (any backend) | wgpu (Vulkan) | wgpu (Metal) |

The `WindowedContext` API is identical across all platforms.

---

## Roadmap

### Phase 1: Touch Interactivity (Complete)
- [x] Android touch event routing
- [x] iOS touch event routing
- [x] Multi-touch support (basic)
- [x] Swift FFI bridge

### Phase 2: GPU Rendering (In Progress)
- [ ] wgpu Metal interop for iOS
- [ ] Pass CAMetalLayer to Rust
- [ ] Render RenderTree to Metal

### Phase 3: Advanced Features
- [ ] Keyboard input (Android)
- [ ] Text input / IME
- [ ] Safe area insets (iOS)
- [ ] Gesture recognizers (pinch, rotate)
- [ ] Accessibility

### Phase 4: Developer Experience
- [ ] CLI tooling (`blinc new --platform ios`)
- [ ] Hot reload on device
- [ ] Debug inspector
