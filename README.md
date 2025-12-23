# Blinc

[![Build Status](https://github.com/project-blinc/Blinc/actions/workflows/ci.yml/badge.svg)](https://github.com/project-blinc/Blinc/actions/workflows/ci.yml)
[![Tests](https://github.com/project-blinc/Blinc/actions/workflows/ci.yml/badge.svg?event=push)](https://github.com/project-blinc/Blinc/actions/workflows/ci.yml)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-green.svg)](LICENSE)

![Blinc UI](glass_music_player.png)

**A GPU-accelerated, cross-platform UI framework** with a GPUI-inspired builder API, glassmorphism effects, spring physics animations, and native rendering on Desktop, Android, and iOS.

## Features

- **GPU-Accelerated Rendering** - SDF-based primitives via wgpu with automatic batching
- **Glassmorphism Effects** - Apple-style vibrancy with backdrop blur and frosted glass
- **GPUI-Style Builder API** - Declarative, chainable element builders (`div()`, `text()`, `svg()`, `image()`)
- **Flexbox Layout** - Powered by Taffy with 100+ Tailwind-style builder methods
- **Spring Physics** - Interruptible animations with RK4 integration (Framer Motion quality)
- **Cross-Platform** - Desktop (macOS, Windows, Linux), Android, iOS
- **Fine-Grained Reactivity** - Signal-based state without VDOM overhead
- **State Machines** - Harel statecharts for complex widget interactions
- **Image Support** - PNG, JPEG, GIF, WebP, BMP with cross-platform asset loading
- **SVG Rendering** - Vector graphics with fill/stroke support
- **Text Rendering** - HarfBuzz shaping, glyph atlas, proper anchoring

## Quick Start

### Installation

```bash
# Build from source
git clone https://github.com/project-blinc/Blinc
cd Blinc
cargo build --release
```

### Hello World

```rust
use blinc_app::prelude::*;
use blinc_app::windowed::WindowedApp;

fn main() -> Result<()> {
    WindowedApp::run(WindowConfig::default(), |ctx| {
        div()
            .w(ctx.width).h(ctx.height)
            .bg([0.1, 0.1, 0.15, 1.0])
            .flex_center()
            .child(
                text("Hello Blinc!")
                    .size(48.0)
                    .color([1.0, 1.0, 1.0, 1.0])
            )
    })
}
```

### Glassmorphism Example

```rust
div()
    .w(ctx.width).h(ctx.height)
    .bg([0.2, 0.3, 0.5, 1.0])
    .flex_center()
    .child(
        // Glass card with backdrop blur
        div()
            .glass()
            .rounded(16.0)
            .p(24.0)
            .child(text("Frosted Glass").size(24.0))
            .child(text("With backdrop blur").size(14.0).color([0.8, 0.8, 0.8, 1.0]))
    )
```

### Image Support

```rust
div()
    .w(400.0).h(300.0)
    .flex_center()
    .child(
        image("assets/photo.png")
            .w(200.0)
            .h(150.0)
            .rounded(8.0)
    )
```

### Layout with Flexbox

```rust
div()
    .w_full().h_full()
    .flex_col()
    .gap(16.0)
    .p(24.0)
    .children([
        // Header
        div().h(60.0).bg([0.2, 0.2, 0.25, 1.0]).rounded(8.0),
        // Content area
        div()
            .flex_1()
            .flex_row()
            .gap(16.0)
            .children([
                div().w(200.0).bg([0.15, 0.15, 0.2, 1.0]).rounded(8.0), // Sidebar
                div().flex_1().bg([0.18, 0.18, 0.22, 1.0]).rounded(8.0), // Main
            ]),
        // Footer
        div().h(40.0).bg([0.2, 0.2, 0.25, 1.0]).rounded(8.0),
    ])
```

## Architecture

```text
┌─────────────────────────────────────────────────────────────────────┐
│                           blinc_app                                  │
│          High-level API: BlincApp, WindowedApp, RenderContext        │
├─────────────────────────────────────────────────────────────────────┤
│  blinc_layout         │  blinc_gpu           │  blinc_paint          │
│  Flexbox (Taffy)      │  SDF Rendering       │  Canvas API           │
│  Element Builders     │  Glass/Blur          │  Paths/Shapes         │
│  RenderTree           │  MSAA                │  Transforms           │
├─────────────────────────────────────────────────────────────────────┤
│  blinc_text           │  blinc_svg           │  blinc_image          │
│  Font Loading         │  SVG Parsing         │  Image Decoding       │
│  Text Shaping         │  Vector Rendering    │  Texture Management   │
│  Glyph Atlas          │  Fill/Stroke         │  Cross-platform Load  │
├─────────────────────────────────────────────────────────────────────┤
│  blinc_core           │  blinc_animation     │  blinc_platform       │
│  Signals/Reactivity   │  Springs (RK4)       │  Window/Event Traits  │
│  State Machines       │  Keyframes           │  Input Events         │
│  Brush/Color Types    │  Timelines           │  Asset Loading        │
├─────────────────────────────────────────────────────────────────────┤
│     blinc_platform_desktop    │  blinc_platform_android  │   _ios   │
│     winit + wgpu              │  NDK + Vulkan            │  UIKit   │
└─────────────────────────────────────────────────────────────────────┘
```

## Crates

| Crate                        | Description                                 |
| ---------------------------- | ------------------------------------------- |
| **blinc_app** | High-level app framework with windowed runner |
| **blinc_core** | Reactive signals, state machines, brush types |
| **blinc_layout** | Flexbox layout engine with GPUI-style builders |
| **blinc_gpu** | GPU rendering: SDF shapes, glass effects, MSAA |
| **blinc_paint** | Canvas/paint API for custom drawing |
| **blinc_text** | Text shaping, font loading, glyph atlas |
| **blinc_image** | Image loading and cross-platform assets |
| **blinc_svg** | SVG parsing and rendering |
| **blinc_animation** | Spring physics and keyframe animations |
| **blinc_platform** | Cross-platform traits and asset loading |
| **blinc_platform_desktop** | Desktop backend (winit) |
| **blinc_platform_android** | Android backend (NDK) |
| **blinc_platform_ios** | iOS backend (UIKit/Metal) |
| **blinc_cli** | Command-line tooling |

## Builder API Reference

### Layout Methods

```rust
// Size
.w(100.0)  .h(100.0)  .size(100.0, 100.0)
.w_full()  .h_full()  .w_auto()  .h_auto()
.min_w()   .max_w()   .min_h()   .max_h()

// Flexbox
.flex_row()    .flex_col()
.flex_center() .flex_1()
.justify_start() .justify_center() .justify_end() .justify_between()
.items_start()   .items_center()   .items_end()   .items_stretch()
.gap(16.0)     .gap_x(8.0)      .gap_y(8.0)

// Spacing
.p(16.0)    .px(8.0)    .py(8.0)
.pt(4.0)    .pb(4.0)    .pl(4.0)    .pr(4.0)
.m(16.0)    .mx(8.0)    .my(8.0)
```

### Styling Methods

```rust
// Background
.bg([r, g, b, a])
.bg_gradient(Gradient::linear(...))
.glass()  .glass_light()  .glass_dark()

// Border & Corners
.rounded(8.0)
.border(1.0, [r, g, b, a])

// Shadow
.shadow_sm()  .shadow_md()  .shadow_lg()  .shadow_xl()

// Opacity & Clipping
.opacity(0.8)
.clip()
```

### Elements

```rust
div()                      // Container element
text("Hello")              // Text element
    .size(16.0)
    .color([1.0, 1.0, 1.0, 1.0])
    .anchor(TextAnchor::Center)

svg(svg_string)            // SVG element
    .w(24.0).h(24.0)

image("path/to/image.png") // Image element
    .w(200.0).h(150.0)
    .rounded(8.0)
```

## Platform Support

| Platform | Status | Backend |
|----------|--------|---------|
| macOS | Stable | wgpu (Metal) |
| Windows | Stable | wgpu (DX12/Vulkan) |
| Linux | Stable | wgpu (Vulkan) |
| Android | Stable | wgpu (Vulkan), ~530KB |
| iOS | In Progress | wgpu (Metal) |

## Roadmap

### Completed

- GPU-accelerated SDF rendering
- Glassmorphism with backdrop blur
- GPUI-style builder API
- Flexbox layout (Taffy)
- Text rendering with shaping
- SVG rendering
- Image support with cross-platform loading
- Spring physics animations
- Reactive signals and state machines
- Desktop and Android platforms

### In Progress

- iOS platform completion
- Widget library (Button, Checkbox, Toggle, etc.)
- Theming system

### Future

- **Zyntax DSL** - `.blinc` file syntax with compile-time optimization
- Hot reload during development
- Developer tools (inspector, animation debugger)
- IDE integration (VS Code extension, LSP)

## License

Apache License 2.0 - see [LICENSE](LICENSE)
