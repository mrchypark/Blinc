# Blinc Project Plan

## Overview

Blinc is a native UI framework powered by Zyntax, featuring:

- **Declarative DSL** (`.blinc` / `.bl`) with compile-time optimization
- **Fine-grained Reactivity** via signals (no VDOM)
- **Built-in State Machines** (Harel statecharts) for widget interactions
- **Animation-first Design** with spring physics and keyframes
- **GPU Rendering** via SDF-based primitives (wgpu/Metal)
- **Paint/Canvas System** for custom 2D drawing
- **Cross-platform** targeting Android, iOS, and Desktop

---

## Phase 1: Core Infrastructure

### 1.1 Toolchain Foundation

**Goal**: Establish the build system and cross-platform compilation pipeline.

#### Tasks

- [ ] **CLI Scaffolding** (`blinc_cli`)
  - Implement `blinc build` command with target selection
  - Implement `blinc dev` with file watcher (notify crate)
  - Implement `blinc new` for project scaffolding
  - Add `blinc plugin build` for ZRTL plugin compilation

- [ ] **Zyntax Integration**
  - Integrate `zyntax_embed` for JIT compilation
  - Configure grammar loading from `grammars/blinc.zyn`
  - Set up ZRTL plugin discovery and loading
  - Implement hot-reload via grammar recompilation

- [ ] **Target Configurations**
  - Create `toolchain/targets/android.toml` with NDK settings
  - Create `toolchain/targets/ios.toml` with Xcode settings
  - Create `toolchain/targets/macos.toml`
  - Create `toolchain/targets/windows.toml`
  - Create `toolchain/targets/linux.toml`

### 1.2 Blinc Grammar (`blinc.zyn`)

**Goal**: Define the complete Blinc DSL grammar that compiles to ZRTL function calls.

#### DSL Constructs

| Construct | Syntax | Compiles To |
|-----------|--------|-------------|
| `@widget` | `@widget Name { ... }` | Struct + init/render functions |
| `@prop` | `@prop name: Type = default` | Struct field |
| `@state` | `@state name: Type = value` | `blinc_signal_create_*()` |
| `@derived` | `@derived name: Type = expr` | `blinc_derived_create()` |
| `@machine` | `@machine name { states { ... } }` | `blinc_fsm_create()` |
| `@spring` | `@spring name { stiffness, damping, target }` | `blinc_spring_create()` |
| `@animation` | `@animation name { duration, keyframes }` | `blinc_keyframe_create()` |
| `@render` | `@render { Widget(...) { ... } }` | `blinc_widget_*()` calls |
| `@paint` | `@paint (ctx) { ... }` | `blinc_paint_*()` calls |

#### Tasks

- [ ] Define grammar metadata (`@language` block)
- [ ] Implement widget definition parsing
- [ ] Implement property declarations
- [ ] Implement reactive state (`@state`, `@derived`)
- [ ] Implement state machines (`@machine`)
- [ ] Implement animations (`@spring`, `@animation`)
- [ ] Implement render tree (`@render`)
- [ ] Implement paint context (`@paint`)
- [ ] Add semantic actions to emit ZRTL function calls

### 1.3 Reactive System (`blinc_core`)

**Goal**: Fine-grained signal-based reactivity inspired by Leptos/SolidJS.

#### Architecture

```
Signal → Subscribers → Effects/Derived
         (lazy)        (push invalidation, pull values)
```

#### Tasks

- [ ] Implement `Signal<T>` with version tracking
- [ ] Implement `Derived<T>` (memoized computed values)
- [ ] Implement `Effect` (side effects on signal change)
- [ ] Implement automatic dependency tracking
- [ ] Implement batched updates
- [ ] Implement reactive graph topological sorting
- [ ] Export ZRTL C-ABI functions

### 1.4 State Machine Runtime (`blinc_core`)

**Goal**: Harel statecharts for complex widget interactions.

#### Features

- Hierarchical states (nested)
- Parallel states (concurrent regions)
- Guards (conditional transitions)
- Entry/exit actions
- Transition actions

#### Tasks

- [ ] Implement `StateMachine` with transition table
- [ ] Implement state entry/exit callbacks
- [ ] Implement guard conditions
- [ ] Implement parallel state regions
- [ ] Implement hierarchical state resolution
- [ ] Export ZRTL C-ABI functions

---

## Phase 2: Animation & Layout

### 2.1 Animation System (`blinc_animation`)

**Goal**: Framer Motion-quality animations with spring physics.

#### Spring Physics

- RK4 integration for accuracy
- Configurable stiffness, damping, mass
- Interruptible with velocity inheritance
- Auto-settle detection

#### Keyframe Animations

- Timed sequences with easing
- Multi-property support
- Wildcard keyframes (from current value)

#### Timeline Orchestration

- Sequential/parallel composition
- Relative offsets (`-=`, `+=`)
- Stagger functions for lists

#### Tasks

- [ ] Implement `Spring` with RK4 integration
- [ ] Implement `KeyframeAnimation` with interpolation
- [ ] Implement `Timeline` with offsets
- [ ] Implement `AnimationScheduler` for frame updates
- [ ] Add easing function library (cubic bezier support)
- [ ] Implement stagger utilities
- [ ] Export ZRTL C-ABI functions

### 2.2 Layout Engine (`blinc_layout`)

**Goal**: Flexbox layout via Taffy.

#### Tasks

- [ ] Integrate Taffy layout engine
- [ ] Map Blinc style properties to Taffy styles
- [ ] Implement layout tree management
- [ ] Implement dirty tracking for incremental layout
- [ ] Support percentage, pixel, and auto sizing
- [ ] Export ZRTL C-ABI functions

---

## Phase 3: Rendering

### 3.1 GPU Renderer (`blinc_gpu`)

**Goal**: High-performance SDF-based GPU rendering.

#### Render Pipeline

1. Collect primitives from widget tree
2. Sort by z-order
3. Batch by primitive type
4. Render: shadows → backgrounds → borders → content

#### SDF Shaders

- Rounded rectangles with variable corner radii
- Circles and ellipses
- Gaussian blur shadows (via erf approximation)
- Gradients (linear, radial, conic)

#### Tasks

- [ ] Set up wgpu device and surface
- [ ] Implement rounded rectangle SDF shader
- [ ] Implement shadow shader (Gaussian blur)
- [ ] Implement gradient shader
- [ ] Implement primitive batching
- [ ] Implement texture atlas for caching
- [ ] Optimize draw call batching

### 3.2 Paint/Canvas System (`blinc_paint`)

**Goal**: Full 2D drawing API for custom graphics.

#### API Design

```rust
ctx.fill_rect(x, y, w, h, color);
ctx.stroke_path(path, stroke_style);
ctx.draw_text(text, x, y, font);
ctx.draw_sdf(shape, position, fill);
ctx.push_clip(rect);
ctx.push_transform(matrix);
```

#### Tasks

- [ ] Implement `PaintContext` with command recording
- [ ] Implement path building API
- [ ] Implement color and gradient types
- [ ] Implement shape primitives (rect, circle, rounded rect)
- [ ] Implement transform stack
- [ ] Implement clip stack
- [ ] Integrate with GPU renderer for execution
- [ ] Export ZRTL C-ABI functions

### 3.3 Text Rendering

**Goal**: High-quality text with SDF glyphs.

#### Tasks

- [ ] Integrate font loading (fontdb or similar)
- [ ] Implement glyph rasterization to SDF
- [ ] Implement glyph atlas with LRU eviction
- [ ] Implement text shaping (harfbuzz or similar)
- [ ] Implement text layout (line breaking, alignment)
- [ ] Implement text shader

---

## Phase 4: Platform Integration

### 4.1 Desktop Platform (`blinc_platform_desktop`)

**Goal**: Native windowing for macOS, Windows, Linux.

#### Tasks

- [ ] Implement window creation via winit
- [ ] Implement event loop integration
- [ ] Implement keyboard input handling
- [ ] Implement mouse/trackpad input
- [ ] Implement DPI scaling
- [ ] Implement clipboard access
- [ ] Implement system theme detection

### 4.2 Android Platform (`blinc_platform_android`)

**Goal**: Native Android integration.

#### Tasks

- [ ] Implement NativeActivity integration
- [ ] Implement JNI bridge for system APIs
- [ ] Implement touch input handling
- [ ] Implement Vulkan/GLES surface creation
- [ ] Implement lifecycle management (pause/resume)
- [ ] Implement soft keyboard handling
- [ ] Create Gradle project template

### 4.3 iOS Platform (`blinc_platform_ios`)

**Goal**: Native iOS integration.

#### Tasks

- [ ] Implement UIKit application delegate
- [ ] Implement Metal surface creation
- [ ] Implement touch input handling
- [ ] Implement safe area insets
- [ ] Implement keyboard handling
- [ ] Implement lifecycle management
- [ ] Create Xcode project template

---

## Phase 5: Widget Library

### 5.1 Core Widgets

| Widget | States | Animations |
|--------|--------|------------|
| Button | idle, hovered, pressed, focused, disabled | ripple, scale |
| Checkbox | unchecked, checking, checked, unchecking | checkmark draw |
| Toggle | off, transitioning, on | thumb slide |
| TextField | empty, focused, filled, error | label float |
| Dropdown | closed, opening, open, closing | height expand |
| Modal | hidden, entering, visible, exiting | fade + scale |
| Tabs | idle, switching | underline slide |
| Accordion | collapsed, expanding, expanded | height spring |
| Tooltip | hidden, delay, showing, visible | fade + offset |
| Slider | idle, dragging | thumb scale |
| ScrollView | idle, scrolling, momentum | content offset |

#### Tasks

- [ ] Implement Button with FSM and ripple
- [ ] Implement Checkbox with animation
- [ ] Implement Toggle with spring animation
- [ ] Implement TextField with floating label
- [ ] Implement Dropdown with expand animation
- [ ] Implement Modal with backdrop
- [ ] Implement Tabs with indicator animation
- [ ] Implement ScrollView with momentum

### 5.2 Theming System

#### Tasks

- [ ] Define theme schema (colors, typography, spacing)
- [ ] Implement theme provider pattern
- [ ] Implement dark/light mode switching
- [ ] Implement theme inheritance

---

## Phase 6: Developer Experience

### 6.1 Hot Reload

**Goal**: Sub-second iteration during development.

#### Architecture

```
File Change → Grammar Recompile → JIT Update → State Preserved
```

#### Tasks

- [ ] Implement file watcher with debouncing
- [ ] Implement incremental grammar compilation
- [ ] Implement widget tree diffing
- [ ] Implement state preservation across reloads

### 6.2 Developer Tools

#### Tasks

- [ ] Implement widget inspector overlay
- [ ] Implement state machine visualizer
- [ ] Implement animation timeline debugger
- [ ] Implement reactive graph viewer
- [ ] Implement performance profiler

### 6.3 IDE Integration

#### Tasks

- [ ] Create VS Code extension with syntax highlighting
- [ ] Implement LSP server for autocomplete
- [ ] Implement error diagnostics
- [ ] Implement go-to-definition

---

## Phase 7: Production Hardening

### 7.1 Performance

#### Tasks

- [ ] Profile and optimize hot paths
- [ ] Implement layout caching
- [ ] Implement render tree diffing
- [ ] Optimize memory allocations (arena allocators)
- [ ] Implement GPU texture atlasing

### 7.2 Testing

#### Tasks

- [ ] Unit tests for reactive system
- [ ] Unit tests for state machines
- [ ] Unit tests for animation
- [ ] Integration tests for widget rendering
- [ ] Visual regression tests
- [ ] Performance benchmarks

### 7.3 Documentation

#### Tasks

- [ ] API reference documentation
- [ ] Tutorial: Getting Started
- [ ] Tutorial: Building Your First App
- [ ] Guide: Custom Widgets
- [ ] Guide: Animations
- [ ] Guide: Paint/Canvas
- [ ] Guide: Platform Integration

---

## Technical Decisions

### Why Zyntax?

- **AOT Compilation**: Native binaries without runtime overhead
- **JIT for Development**: Instant hot-reload during development
- **Custom DSL**: Grammar-defined language without forking a compiler
- **ZRTL Plugins**: Modular runtime with dynamic/static linking

### Why SDF Rendering?

- **Resolution Independent**: Sharp at any scale
- **GPU Efficient**: Simple fragment shaders
- **Smooth Edges**: Built-in anti-aliasing
- **Flexible**: Combine shapes with boolean operations

### Why Built-in State Machines?

- **Explicit States**: No impossible state combinations
- **Visual Debugging**: Generate statechart diagrams
- **Animation Triggers**: Entry/exit actions drive animations
- **Testable**: State machines are easily unit tested

### Why Fine-Grained Reactivity?

- **No VDOM Diffing**: Direct updates to affected widgets
- **Minimal Re-renders**: Only dependent computations update
- **Predictable**: Clear dependency graph
- **Performant**: O(1) signal updates

---

## File Structure

```
blinc/
├── Cargo.toml                    # Workspace
├── blinc.toml                    # Toolchain config
├── grammars/
│   └── blinc.zyn                 # DSL grammar
├── crates/
│   ├── blinc_cli/                # CLI toolchain
│   ├── blinc_core/               # Reactivity + FSM
│   ├── blinc_animation/          # Springs + Keyframes
│   ├── blinc_layout/             # Flexbox (Taffy)
│   ├── blinc_gpu/                # GPU renderer
│   ├── blinc_paint/              # Canvas API
│   ├── blinc_widgets/            # Widget library
│   └── blinc_embed/              # Embedding SDK
├── extensions/                   # ZRTL plugins
│   ├── blinc_platform_desktop/
│   ├── blinc_platform_android/
│   └── blinc_platform_ios/
├── toolchain/
│   ├── targets/                  # Platform configs
│   └── templates/                # Project templates
└── examples/
    └── counter/
        └── main.blinc
```

---

## Success Metrics

1. **Performance**: 120 FPS on target devices
2. **Hot Reload**: < 100ms from save to update
3. **Binary Size**: < 5MB for minimal app
4. **Memory**: < 50MB for typical app
5. **Developer Experience**: Intuitive DSL, helpful errors
