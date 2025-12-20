# Zyntax UI Framework Research & Design Proposal

## Executive Summary

This document synthesizes research on leading declarative UI frameworks (Jetpack Compose, QML, Zed's GPUI), reactive state management systems (Leptos, SolidJS signals), state machine integration (XState), and best-in-class animation libraries (Framer Motion, anime.js) to inform the design of a next-generation UI framework DSL that is:

1. **Embeddable in Rust** - Native performance with zero-cost abstractions
2. **AOT-compilable via Zyntax** - Leverage your compiler infrastructure for native binaries
3. **Best-in-class reactive** - Fine-grained reactivity without VDOM overhead
4. **State machine powered** - Built-in FSM for complex widget transient states
5. **Animation-first** - Keyframe and spring physics rivaling Framer Motion

---

## Part 1: Framework Analysis

### 1.1 Jetpack Compose Architecture

**Key Innovations:**
- **Compiler-transformed functions**: `@Composable` annotation changes function semantics, injecting a `$composer` parameter
- **Slot Table**: Gap-buffer data structure storing composition state (positions, parameters, remembered values)
- **Positional Memoization**: Uses call-site position as implicit keys for efficient skip/update decisions
- **Smart Recomposition**: Only re-executes composables whose inputs have changed

**DSL Pattern:**
```kotlin
@Composable
fun Counter() {
    var count by remember { mutableStateOf(0) }
    Button(onClick = { count++ }) {
        Text("Count: $count")
    }
}
```

**What to Learn:**
- Compiler transforms enable declarative code that compiles to efficient imperative operations
- The `remember` pattern for scoped state that survives recomposition
- Idempotence requirement: same inputs → same output tree
- Stability inference for automatic skip optimization

### 1.2 QML Architecture

**Key Innovations:**
- **Property Bindings**: Declarative reactive connections that auto-update
- **Signal/Slot System**: First-class event handling with Qt's proven pattern
- **Built-in State Machine Framework**: Based on Harel's Statecharts (SCXML)
- **JavaScript Expression Bindings**: Inline logic in property values

**DSL Pattern:**
```qml
Rectangle {
    id: root
    width: parent.width * 0.8  // Reactive binding
    height: calculateHeight()
    
    states: [
        State { name: "pressed"; PropertyChanges { target: root; color: "red" } },
        State { name: "released"; PropertyChanges { target: root; color: "blue" } }
    ]
    
    transitions: [
        Transition { from: "released"; to: "pressed"; NumberAnimation { property: "opacity" } }
    ]
}
```

**What to Learn:**
- Declarative state machines as first-class language constructs
- Property bindings with automatic dependency tracking
- Seamless animation integration via transitions
- JSON-like object literal syntax is highly readable

### 1.3 Zed's GPUI Architecture

**Key Innovations:**
- **Hybrid Immediate/Retained Mode**: Best of both worlds
- **GPU-First Rendering**: Custom shaders for rectangles, shadows, text (SDFs)
- **Entity-Based State Management**: Central `AppContext` owns all state via smart pointers
- **Tailwind-Style API**: Familiar, composable styling

**DSL Pattern:**
```rust
impl Render for MyView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .size(px(500.0))
            .child(format!("Hello, {}!", &self.text))
    }
}
```

**What to Learn:**
- Entity ownership model works well with Rust's borrow checker
- Builder pattern for declarative composition
- Views as thin render functions over state entities
- SDF-based GPU primitives for high-performance 2D rendering

### 1.4 Leptos/Floem Fine-Grained Reactivity

**Key Innovations:**
- **Signal Primitives**: Atomic units of trackable state
- **Automatic Dependency Tracking**: Effects subscribe to signals on read
- **No VDOM**: Direct DOM/widget tree updates from signal changes
- **Computed (Memo)**: Derived values that cache and invalidate

**Pattern:**
```rust
let (count, set_count) = signal(0);
let double = Memo::new(move |_| count.get() * 2);

Effect::new(move |_| {
    println!("Count changed: {}", count.get());
});
```

**What to Learn:**
- Push-pull hybrid: signals push invalidation, effects pull values
- Lazy evaluation: computations only run when observed
- Batching: multiple signal updates can be coalesced
- Reactive graph algorithm (based on Reactively)

---

## Part 2: Animation System Analysis

### 2.1 Framer Motion Architecture

**Core Concepts:**
1. **Motion Values**: Observable values that can be animated
2. **Spring Physics**: `stiffness`, `damping`, `mass` for natural motion
3. **Keyframe Sequences**: Arrays with wildcard (`null`) support
4. **Variants**: Named animation states for orchestration
5. **Layout Animations**: Automatic FLIP animations for layout changes

**API Design:**
```javascript
// Spring animation
animate({ x: 100 }, { type: "spring", stiffness: 300, damping: 20 })

// Keyframes with timing
animate({ 
    x: [0, 100, 50, 200],
    transition: { times: [0, 0.3, 0.6, 1], duration: 2 }
})

// Variants for orchestration
const variants = {
    hidden: { opacity: 0, y: 20 },
    visible: { opacity: 1, y: 0, transition: { staggerChildren: 0.1 } }
}
```

**What to Learn:**
- Smart defaults: physical properties default to springs, visual to tweens
- Interruptible animations that inherit velocity
- `visualDuration` concept for coordinating springs with timed animations
- Gesture animations (drag, hover, tap) as first-class primitives

### 2.2 anime.js Architecture

**Core Concepts:**
1. **Timeline Composition**: Sequential/parallel animation orchestration
2. **Stagger Functions**: Automatic delay distribution across targets
3. **Property Keyframes**: Per-property animation sequences
4. **Easing Presets + Custom**: Extensive built-in + cubic bezier support
5. **Relative Values**: `+=100`, `-=50` for additive animations

**API Design:**
```javascript
const tl = createTimeline({ defaults: { ease: 'outQuad', duration: 800 } });

tl.add('#box1', { translateX: 100, rotate: '1turn' })
  .add('#box2', { scale: [0.5, 1], opacity: [0, 1] }, '-=400')  // Offset
  .add('#box3', { translateY: stagger(50, { from: 'center' }) });
```

**What to Learn:**
- Timeline offsets with relative positioning (`-=`, `+=`, labels)
- Stagger with grid support for 2D layouts
- Function-based values for dynamic per-target animation
- Modular architecture (v4): tree-shakeable imports

---

## Part 3: State Machine Integration

### 3.1 XState for UI

**Statechart Features:**
- **Hierarchical States**: Nested states for complex UI modes
- **Parallel States**: Concurrent state regions (e.g., loading + validation)
- **Guards**: Conditional transitions based on context
- **Actions**: Side effects on entry/exit/transition
- **Context**: Extended state data alongside finite states

**Widget State Machine Pattern:**
```javascript
const buttonMachine = createMachine({
    id: 'button',
    initial: 'idle',
    context: { pressCount: 0 },
    states: {
        idle: {
            on: { 
                POINTER_DOWN: 'pressed',
                FOCUS: 'focused'
            }
        },
        pressed: {
            entry: 'startRipple',
            on: {
                POINTER_UP: { target: 'idle', actions: 'handleClick' },
                POINTER_LEAVE: 'idle'
            }
        },
        focused: {
            on: {
                BLUR: 'idle',
                KEYDOWN_ENTER: { target: 'pressed', actions: 'handleClick' }
            }
        },
        disabled: {
            type: 'final'
        }
    }
});
```

**What to Learn:**
- States as behavior modes, not just data flags
- Transitions are explicit, preventing invalid states
- Entry/exit actions for animation triggers
- Visual debugging with statechart diagrams

---

## Part 4: Proposed DSL Design

### 4.1 Design Principles

1. **Declarative-First**: Describe UI structure and behavior, not imperative steps
2. **Compile-Time Optimization**: Zyntax AOT compiles reactive graphs and state machines
3. **Zero-Cost Abstractions**: No runtime overhead for unused features
4. **GPU-Accelerated**: Follow GPUI's SDF-based rendering approach
5. **Embeddable**: Single Rust struct that owns the entire UI runtime

### 4.2 Proposed DSL Syntax: `.zui` Format

```zui
// zyntax-ui DSL for Button component

@widget Button {
    // Properties with types and defaults
    @prop label: String = ""
    @prop disabled: Bool = false
    @prop on_click: Callback<()> = {}
    
    // Reactive state (fine-grained signals)
    @state hover: Bool = false
    @state pressed: Bool = false
    @state ripple_origin: Point = Point::zero()
    
    // Derived/computed values
    @derived background_color: Color = 
        if disabled { Color::gray(0.3) }
        else if pressed { theme.primary.darken(0.2) }
        else if hover { theme.primary.lighten(0.1) }
        else { theme.primary }
    
    // Built-in state machine for transient states
    @machine interaction {
        initial: idle
        
        states {
            idle {
                on POINTER_ENTER => hovered
                on FOCUS => focused
            }
            hovered {
                entry: { hover.set(true) }
                exit: { hover.set(false) }
                on POINTER_LEAVE => idle
                on POINTER_DOWN => pressed { 
                    actions: [capture_ripple_origin]
                }
            }
            pressed {
                entry: { 
                    pressed.set(true)
                    spawn_animation(ripple_expand)
                }
                exit: { pressed.set(false) }
                on POINTER_UP => hovered { 
                    guard: pointer_inside
                    actions: [trigger_on_click, spawn_animation(ripple_fade)]
                }
                on POINTER_UP => idle {
                    actions: [spawn_animation(ripple_fade)]
                }
                on POINTER_LEAVE => pressed_outside
            }
            pressed_outside {
                on POINTER_ENTER => pressed
                on POINTER_UP => idle
            }
            focused {
                on BLUR => idle
                on KEY_ENTER => pressed
            }
            disabled {
                // Final state, only transitions via prop change
            }
        }
        
        guards {
            pointer_inside: (event) => bounds.contains(event.position)
        }
        
        actions {
            capture_ripple_origin: (event) => {
                ripple_origin.set(event.position - bounds.origin)
            }
            trigger_on_click: () => on_click.emit(())
        }
    }
    
    // Keyframe animation definitions
    @animation ripple_expand {
        duration: 400ms
        easing: ease_out_cubic
        
        keyframes {
            from { ripple_scale: 0.0, ripple_opacity: 0.3 }
            to { ripple_scale: 2.5, ripple_opacity: 0.0 }
        }
    }
    
    @animation ripple_fade {
        duration: 200ms
        easing: ease_out
        
        keyframes {
            to { ripple_opacity: 0.0 }
        }
    }
    
    // Spring-physics animation for hover scale
    @spring hover_spring {
        stiffness: 400
        damping: 25
        target: if hover { 1.02 } else { 1.0 }
    }
    
    // Render tree - declarative, reactive
    @render {
        Container(
            background: background_color,
            border_radius: 8.px,
            padding: EdgeInsets::symmetric(16.px, 12.px),
            transform: Transform::scale(hover_spring.value),
            shadow: if pressed { Shadow::none() } else { Shadow::sm() },
        ) {
            // Ripple effect layer
            if ripple_opacity > 0.0 {
                Circle(
                    center: ripple_origin,
                    radius: ripple_scale * max(bounds.width, bounds.height),
                    fill: Color::white().with_alpha(ripple_opacity),
                )
            }
            
            // Label
            Text(
                content: label,
                style: TextStyle::button(),
                color: if disabled { Color::gray(0.5) } else { Color::white() },
            )
        }
    }
}
```

### 4.3 Core DSL Constructs

#### Properties (`@prop`)
```zui
@prop name: Type = default_value
```
- Compile to struct fields
- Type-checked at compile time
- Support for callbacks, generics

#### Reactive State (`@state`)
```zui
@state counter: Int = 0
```
- Compile to fine-grained signals
- Automatic dependency tracking
- Thread-safe with atomic updates

#### Derived Values (`@derived`)
```zui
@derived total: Int = items.iter().sum()
```
- Computed/memo values
- Lazy evaluation, cached
- Auto-invalidate on dependency change

#### State Machines (`@machine`)
```zui
@machine name {
    initial: state_name
    states { ... }
    guards { ... }
    actions { ... }
}
```
- Compile to efficient match-based FSM
- Support hierarchical and parallel states
- Generate visual statechart diagrams

#### Animations (`@animation`, `@spring`)
```zui
@animation name {
    duration: 300ms
    easing: ease_out
    keyframes { ... }
}

@spring name {
    stiffness: 300
    damping: 20
    target: computed_value
}
```
- Keyframe animations with timing control
- Spring physics with interruptible values
- Gesture-driven animations

#### Render Tree (`@render`)
```zui
@render {
    Widget(props...) { children... }
}
```
- Declarative widget composition
- Conditional rendering
- List rendering with keys

---

## Part 5: Implementation Architecture

### 5.1 Compilation Pipeline (Zyntax-Agnostic)

**Key Principle**: Zyntax remains language-context free. The UI framework is entirely a ZRTL plugin - the DSL compiles to standard function calls that invoke plugin APIs at runtime.

```
┌─────────────────┐      ┌──────────────────┐      ┌───────────────────┐
│  .zui DSL       │  →   │  Zyn Grammar     │  →   │  TypedAST         │
│  Source Files   │      │  (zui.zyn)       │      │  (standard nodes) │
└─────────────────┘      └──────────────────┘      └───────────────────┘
                                                            │
                              ┌─────────────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│              Zyntax Compiler (Unchanged, Language-Agnostic)         │
│  ┌──────────────┐    ┌──────────────┐    ┌───────────────────────┐ │
│  │ TypedAST →   │ →  │ Standard     │ →  │ Cranelift/LLVM        │ │
│  │ HIR Lowering │    │ HIR          │    │ Native Code           │ │
│  └──────────────┘    └──────────────┘    └───────────────────────┘ │
│                                                                     │
│  DSL constructs compile to extern function calls:                   │
│  • @state → zui_signal_create(), zui_signal_get(), zui_signal_set() │
│  • @machine → zui_fsm_create(), zui_fsm_send()                      │
│  • @animation → zui_anim_create(), zui_spring_create()              │
│  • @render → zui_widget_create(), zui_widget_child()                │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              │ Links against ZRTL plugin
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    ZRTL: zyntax_ui Plugin                           │
│  ┌──────────────┐  ┌───────────────┐  ┌───────────────────────────┐│
│  │ Reactive     │  │ FSM           │  │ Animation                 ││
│  │ Graph Engine │  │ Runtime       │  │ Scheduler                 ││
│  └──────────────┘  └───────────────┘  └───────────────────────────┘│
│  ┌──────────────┐  ┌───────────────┐  ┌───────────────────────────┐│
│  │ Layout       │  │ GPU Renderer  │  │ Event                     ││
│  │ (Taffy)      │  │ (wgpu/metal)  │  │ Dispatcher                ││
│  └──────────────┘  └───────────────┘  └───────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
```

### 5.2 DSL-to-Function-Call Compilation

The `.zui` grammar emits standard TypedAST nodes that compile to extern function calls. Zyntax sees these as regular function calls - it has no knowledge of UI concepts.

**Example: `@state counter: Int = 0` compiles to:**

```
// TypedAST (what zui.zyn grammar emits)
{
  "kind": "let_stmt",
  "name": "counter",
  "type": { "kind": "named_type", "name": "SignalHandle" },
  "value": {
    "kind": "call_expr",
    "callee": { "kind": "variable", "name": "zui_signal_create_i32" },
    "args": [{ "kind": "int_literal", "value": 0 }]
  }
}

// Which Zyntax compiles to HIR as a normal extern call:
let counter: *SignalHandle = call @zui_signal_create_i32(0)
```

**Example: `@machine` state machine compiles to:**

```
// The grammar emits a struct literal + function calls
{
  "kind": "let_stmt",
  "name": "interaction_fsm",
  "value": {
    "kind": "call_expr", 
    "callee": "zui_fsm_create",
    "args": [
      // FSM definition as a struct or serialized descriptor
      { "kind": "struct_literal", "type": "FsmDescriptor", ... }
    ]
  }
}

// State transitions become:
call @zui_fsm_send(interaction_fsm, event_id)
```

### 5.3 ZRTL Plugin: `zyntax_ui`

The entire UI runtime is provided as a ZRTL plugin. Compiled code makes extern calls to these exported functions.

```rust
// crates/zyntax_ui/src/lib.rs
use zyntax_zrtl::{zrtl_plugin, ExportFn, PluginContext};

/// ZRTL Plugin providing reactive UI runtime
#[zrtl_plugin]
pub struct ZyntaxUiPlugin {
    runtime: UiRuntime,
}

impl ZyntaxUiPlugin {
    pub fn new() -> Self {
        Self {
            runtime: UiRuntime::new(),
        }
    }
}

// ============================================================================
// EXPORTED FUNCTIONS (called by compiled .zui code)
// ============================================================================

// --- Reactive Signals ---

#[no_mangle]
pub extern "C" fn zui_signal_create_i32(initial: i32) -> *mut SignalHandle {
    with_runtime(|rt| rt.reactive.create_signal(initial))
}

#[no_mangle]
pub extern "C" fn zui_signal_get_i32(handle: *mut SignalHandle) -> i32 {
    with_runtime(|rt| rt.reactive.get_signal(handle))
}

#[no_mangle]
pub extern "C" fn zui_signal_set_i32(handle: *mut SignalHandle, value: i32) {
    with_runtime(|rt| rt.reactive.set_signal(handle, value))
}

#[no_mangle]
pub extern "C" fn zui_derived_create(
    compute_fn: extern "C" fn() -> i32,
    dep_count: usize,
    deps: *const *mut SignalHandle,
) -> *mut DerivedHandle {
    with_runtime(|rt| rt.reactive.create_derived(compute_fn, deps, dep_count))
}

#[no_mangle]
pub extern "C" fn zui_effect_create(
    effect_fn: extern "C" fn(),
    dep_count: usize,
    deps: *const *mut SignalHandle,
) -> *mut EffectHandle {
    with_runtime(|rt| rt.reactive.create_effect(effect_fn, deps, dep_count))
}

// --- State Machines ---

#[no_mangle]
pub extern "C" fn zui_fsm_create(
    initial_state: u32,
    transition_table: *const FsmTransition,
    transition_count: usize,
) -> *mut FsmHandle {
    with_runtime(|rt| rt.fsm.create(initial_state, transition_table, transition_count))
}

#[no_mangle]
pub extern "C" fn zui_fsm_send(handle: *mut FsmHandle, event: u32) -> u32 {
    with_runtime(|rt| rt.fsm.send(handle, event))
}

#[no_mangle]
pub extern "C" fn zui_fsm_current_state(handle: *mut FsmHandle) -> u32 {
    with_runtime(|rt| rt.fsm.current_state(handle))
}

#[no_mangle]
pub extern "C" fn zui_fsm_on_enter(
    handle: *mut FsmHandle,
    state: u32,
    callback: extern "C" fn(),
) {
    with_runtime(|rt| rt.fsm.on_enter(handle, state, callback))
}

#[no_mangle]
pub extern "C" fn zui_fsm_on_exit(
    handle: *mut FsmHandle,
    state: u32,
    callback: extern "C" fn(),
) {
    with_runtime(|rt| rt.fsm.on_exit(handle, state, callback))
}

// --- Animations ---

#[no_mangle]
pub extern "C" fn zui_spring_create(
    stiffness: f32,
    damping: f32,
    mass: f32,
    initial: f32,
) -> *mut SpringHandle {
    with_runtime(|rt| rt.animation.create_spring(stiffness, damping, mass, initial))
}

#[no_mangle]
pub extern "C" fn zui_spring_set_target(handle: *mut SpringHandle, target: f32) {
    with_runtime(|rt| rt.animation.spring_set_target(handle, target))
}

#[no_mangle]
pub extern "C" fn zui_spring_value(handle: *mut SpringHandle) -> f32 {
    with_runtime(|rt| rt.animation.spring_value(handle))
}

#[no_mangle]
pub extern "C" fn zui_spring_velocity(handle: *mut SpringHandle) -> f32 {
    with_runtime(|rt| rt.animation.spring_velocity(handle))
}

#[no_mangle]
pub extern "C" fn zui_keyframe_create(
    duration_ms: u32,
    easing: u32,  // Enum: Linear, EaseIn, EaseOut, EaseInOut, CubicBezier
    keyframe_count: usize,
    keyframes: *const Keyframe,
) -> *mut KeyframeHandle {
    with_runtime(|rt| rt.animation.create_keyframe(duration_ms, easing, keyframes, keyframe_count))
}

#[no_mangle]
pub extern "C" fn zui_keyframe_start(handle: *mut KeyframeHandle) {
    with_runtime(|rt| rt.animation.keyframe_start(handle))
}

#[no_mangle]
pub extern "C" fn zui_keyframe_value(handle: *mut KeyframeHandle) -> f32 {
    with_runtime(|rt| rt.animation.keyframe_value(handle))
}

#[no_mangle]
pub extern "C" fn zui_timeline_create() -> *mut TimelineHandle {
    with_runtime(|rt| rt.animation.create_timeline())
}

#[no_mangle]
pub extern "C" fn zui_timeline_add(
    timeline: *mut TimelineHandle,
    animation: *mut KeyframeHandle,
    offset_ms: i32,  // Relative offset, can be negative
) {
    with_runtime(|rt| rt.animation.timeline_add(timeline, animation, offset_ms))
}

// --- Widget Tree ---

#[no_mangle]
pub extern "C" fn zui_widget_create(widget_type: u32) -> *mut WidgetHandle {
    with_runtime(|rt| rt.widgets.create(widget_type))
}

#[no_mangle]
pub extern "C" fn zui_widget_set_prop_f32(
    widget: *mut WidgetHandle,
    prop_id: u32,
    value: f32,
) {
    with_runtime(|rt| rt.widgets.set_prop_f32(widget, prop_id, value))
}

#[no_mangle]
pub extern "C" fn zui_widget_set_prop_color(
    widget: *mut WidgetHandle,
    prop_id: u32,
    r: u8, g: u8, b: u8, a: u8,
) {
    with_runtime(|rt| rt.widgets.set_prop_color(widget, prop_id, r, g, b, a))
}

#[no_mangle]
pub extern "C" fn zui_widget_add_child(
    parent: *mut WidgetHandle,
    child: *mut WidgetHandle,
) {
    with_runtime(|rt| rt.widgets.add_child(parent, child))
}

#[no_mangle]
pub extern "C" fn zui_widget_set_root(widget: *mut WidgetHandle) {
    with_runtime(|rt| rt.widgets.set_root(widget))
}

// --- Event Handling ---

#[no_mangle]
pub extern "C" fn zui_on_event(
    widget: *mut WidgetHandle,
    event_type: u32,
    handler: extern "C" fn(*const EventData),
) {
    with_runtime(|rt| rt.events.register(widget, event_type, handler))
}

// --- Render Loop ---

#[no_mangle]
pub extern "C" fn zui_run() {
    with_runtime(|rt| rt.run_event_loop())
}

#[no_mangle]
pub extern "C" fn zui_request_frame() {
    with_runtime(|rt| rt.request_frame())
}
```

### 5.4 Zyn Grammar Semantic Actions

The `zui.zyn` grammar transforms DSL constructs into these function calls:

```zyn
// @state counter: Int = 0
// Emits: let counter = zui_signal_create_i32(0)
state_def = { "@state" ~ IDENT ~ ":" ~ type_expr ~ "=" ~ expr }
  -> TypedStatement {
      "define": "let_stmt",
      "args": {
          "name": "$1",
          "value": {
              "define": "call_expr",
              "args": {
                  "callee": { "concat": ["zui_signal_create_", "$2"] },
                  "arguments": ["$3"]
              }
          }
      }
  }

// @spring hover_scale { stiffness: 400, damping: 25, target: ... }
spring_def = { "@spring" ~ IDENT ~ "{" ~ spring_body ~ "}" }
  -> TypedStatement {
      "commands": [
          {
              "define": "let_stmt",
              "args": {
                  "name": "$1",
                  "value": {
                      "define": "call_expr",
                      "args": {
                          "callee": "zui_spring_create",
                          "arguments": ["$stiffness", "$damping", "$mass", "$initial"]
                      }
                  }
              }
          },
          // Effect to update target when dependencies change
          {
              "define": "call_expr",
              "args": {
                  "callee": "zui_effect_create",
                  "arguments": ["$target_update_fn", "$dep_count", "$deps"]
              }
          }
      ]
  }

// @machine interaction { initial: idle, states { ... } }
machine_def = { "@machine" ~ IDENT ~ "{" ~ machine_body ~ "}" }
  -> TypedStatement {
      "commands": [
          // Create transition table as static data
          { "define": "static_array", "store": "transitions", ... },
          // Create FSM instance
          {
              "define": "let_stmt",
              "args": {
                  "name": "$1",
                  "value": {
                      "define": "call_expr",
                      "args": {
                          "callee": "zui_fsm_create",
                          "arguments": ["$initial_state_id", "$transitions", "$transition_count"]
                      }
                  }
              }
          },
          // Register entry/exit callbacks
          { "for_each": "$states_with_entry", "emit": "zui_fsm_on_enter(...)" },
          { "for_each": "$states_with_exit", "emit": "zui_fsm_on_exit(...)" }
      ]
  }
```

### 5.5 ZRTL Plugin Internal Architecture

The `zyntax_ui` plugin contains the full UI runtime implementation. Here's the internal design:

```rust
// crates/zyntax_ui/src/runtime.rs

/// The UI runtime - owns all reactive state, animations, and rendering
pub struct UiRuntime {
    pub reactive: ReactiveGraph,
    pub fsm: FsmRuntime,
    pub animation: AnimationScheduler,
    pub widgets: WidgetTree,
    pub layout: LayoutEngine,  // Taffy-based
    pub renderer: GpuRenderer,  // wgpu + SDF shaders
    pub events: EventDispatcher,
}

// ============================================================================
// REACTIVE SYSTEM (Fine-grained signals, inspired by Leptos/SolidJS)
// ============================================================================

pub struct ReactiveGraph {
    signals: Slab<SignalNode>,
    effects: Slab<EffectNode>,
    derived: Slab<DerivedNode>,
    batch_depth: AtomicU32,
    pending_effects: Vec<EffectId>,
}

struct SignalNode {
    value: Box<dyn Any>,
    version: u64,
    subscribers: Vec<SubscriberId>,
}

impl ReactiveGraph {
    /// Update a signal and schedule dependent effects
    pub fn set_signal<T: 'static>(&mut self, handle: SignalId, value: T) {
        let node = &mut self.signals[handle];
        node.value = Box::new(value);
        node.version += 1;
        
        // Mark all subscribers as dirty
        for sub in &node.subscribers {
            match sub {
                SubscriberId::Effect(id) => {
                    self.effects[*id].dirty = true;
                    self.pending_effects.push(*id);
                }
                SubscriberId::Derived(id) => {
                    self.derived[*id].dirty = true;
                }
            }
        }
        
        // If not in a batch, flush effects immediately
        if self.batch_depth.load(Ordering::Relaxed) == 0 {
            self.flush_effects();
        }
    }
    
    /// Execute all pending effects in topological order
    fn flush_effects(&mut self) {
        self.pending_effects.sort_by_key(|id| self.effects[*id].depth);
        for effect_id in self.pending_effects.drain(..) {
            let effect = &mut self.effects[effect_id];
            if effect.dirty {
                effect.dirty = false;
                (effect.callback)();
            }
        }
    }
}

// ============================================================================
// STATE MACHINE RUNTIME (Harel Statecharts)
// ============================================================================

pub struct FsmRuntime {
    machines: Slab<FsmInstance>,
}

struct FsmInstance {
    current_state: u32,
    transitions: Vec<FsmTransition>,
    entry_callbacks: HashMap<u32, extern "C" fn()>,
    exit_callbacks: HashMap<u32, extern "C" fn()>,
}

impl FsmRuntime {
    pub fn send(&mut self, handle: FsmId, event: u32) -> u32 {
        let fsm = &mut self.machines[handle];
        let current = fsm.current_state;
        
        for trans in &fsm.transitions {
            if trans.from_state == current && trans.event == event {
                if let Some(guard) = trans.guard {
                    if !guard() { continue; }
                }
                
                // Exit → Transition → Enter
                if let Some(on_exit) = fsm.exit_callbacks.get(&current) {
                    on_exit();
                }
                fsm.current_state = trans.to_state;
                if let Some(on_enter) = fsm.entry_callbacks.get(&trans.to_state) {
                    on_enter();
                }
                return trans.to_state;
            }
        }
        current
    }
}

// ============================================================================
// ANIMATION SYSTEM (Spring physics + Keyframes, inspired by Framer Motion)
// ============================================================================

pub struct AnimationScheduler {
    springs: Slab<SpringInstance>,
    keyframes: Slab<KeyframeInstance>,
    timelines: Slab<TimelineInstance>,
    active: Vec<AnimationId>,
    last_frame: Instant,
}

struct SpringInstance {
    value: f32,
    velocity: f32,
    target: f32,
    stiffness: f32,
    damping: f32,
    mass: f32,
}

impl SpringInstance {
    /// RK4 integration for physically-accurate spring physics
    fn step(&mut self, dt: f32) {
        let k1_v = self.acceleration(self.value, self.velocity);
        let k1_x = self.velocity;
        
        let k2_v = self.acceleration(
            self.value + k1_x * dt * 0.5, 
            self.velocity + k1_v * dt * 0.5
        );
        let k2_x = self.velocity + k1_v * dt * 0.5;
        
        let k3_v = self.acceleration(
            self.value + k2_x * dt * 0.5, 
            self.velocity + k2_v * dt * 0.5
        );
        let k3_x = self.velocity + k2_v * dt * 0.5;
        
        let k4_v = self.acceleration(
            self.value + k3_x * dt, 
            self.velocity + k3_v * dt
        );
        let k4_x = self.velocity + k3_v * dt;
        
        self.velocity += (k1_v + 2.0 * k2_v + 2.0 * k3_v + k4_v) * dt / 6.0;
        self.value += (k1_x + 2.0 * k2_x + 2.0 * k3_x + k4_x) * dt / 6.0;
    }
    
    fn acceleration(&self, x: f32, v: f32) -> f32 {
        let spring_force = -self.stiffness * (x - self.target);
        let damping_force = -self.damping * v;
        (spring_force + damping_force) / self.mass
    }
}

impl AnimationScheduler {
    /// Called every frame (target 120fps for smooth animations)
    pub fn tick(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;
        
        // Update all active springs
        for spring in self.springs.iter_mut() {
            spring.1.step(dt);
        }
        
        // Update all active keyframe animations
        for anim in self.keyframes.iter_mut() {
            if anim.1.playing {
                anim.1.current_time += dt * 1000.0;
                if anim.1.current_time >= anim.1.duration_ms as f32 {
                    anim.1.playing = false;
                }
            }
        }
    }
}

// ============================================================================
// GPU RENDERER (SDF-based like Zed's GPUI)
// ============================================================================

pub struct GpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    
    // SDF shaders for primitives
    rect_pipeline: wgpu::RenderPipeline,    // Rounded rects, borders
    shadow_pipeline: wgpu::RenderPipeline,  // Gaussian blur via erf()
    text_pipeline: wgpu::RenderPipeline,    // Glyph atlas rendering
    
    glyph_cache: GlyphCache,
}

impl GpuRenderer {
    /// Batch primitives and render in minimal draw calls
    pub fn render(&mut self, root: &WidgetNode, layout: &LayoutResult) {
        let primitives = self.collect_primitives(root, layout);
        
        // Sort by z-order, then batch by type
        let mut rects = Vec::new();
        let mut shadows = Vec::new();
        let mut glyphs = Vec::new();
        
        for prim in primitives {
            match prim {
                Primitive::Rect(r) => rects.push(r),
                Primitive::Shadow(s) => shadows.push(s),
                Primitive::Text(t) => glyphs.extend(self.shape_text(&t)),
            }
        }
        
        // Render in order: shadows → rects → text
        self.draw_batch(&self.shadow_pipeline, &shadows);
        self.draw_batch(&self.rect_pipeline, &rects);
        self.draw_batch(&self.text_pipeline, &glyphs);
    }
}
```

---

## Part 6: Widget Library Design

### 6.1 Core Widgets with FSM States

| Widget | States | Key Animations |
|--------|--------|----------------|
| **Button** | idle, hovered, pressed, focused, disabled | ripple, scale spring |
| **Checkbox** | unchecked, checking, checked, unchecking | checkmark draw, scale |
| **Toggle** | off, transitioning, on | thumb slide spring |
| **TextField** | empty, focused, filled, error, disabled | label float, border color |
| **Dropdown** | closed, opening, open, closing | height expand, arrow rotate |
| **Modal** | hidden, entering, visible, exiting | fade + scale spring |
| **Tabs** | idle, switching | underline slide, content fade |
| **Accordion** | collapsed, expanding, expanded, collapsing | height spring, rotate chevron |
| **Tooltip** | hidden, delay, showing, visible, hiding | fade + offset |
| **Slider** | idle, dragging, snapping | thumb scale, track fill |

### 6.2 Animation Presets

```zui
@presets animations {
    // Framer Motion-inspired defaults
    spring_default: Spring { stiffness: 400, damping: 25, mass: 1.0 }
    spring_bouncy: Spring { stiffness: 300, damping: 10, mass: 0.8 }
    spring_stiff: Spring { stiffness: 700, damping: 30, mass: 1.0 }
    
    // Timing presets
    ease_out: cubic_bezier(0.0, 0.0, 0.2, 1.0)
    ease_in_out: cubic_bezier(0.4, 0.0, 0.2, 1.0)
    
    // Duration presets (Material Design)
    duration_short: 150ms
    duration_medium: 300ms
    duration_long: 500ms
}
```

---

## Part 7: Zyntax Integration

### 7.1 Zyn Grammar for `.zui` Files

Create a `zui.zyn` grammar that parses the DSL and emits standard TypedAST nodes (function calls, structs, etc.):

```zyn
@language {
    name: "ZyntaxUI",
    version: "1.0",
    file_extensions: [".zui"],
    entry_point: "widget_file"
}

// Import the ZRTL plugin functions as externs
@preamble {
    // These are resolved at link time against zyntax_ui plugin
    "extern": [
        "zui_signal_create_i32", "zui_signal_get_i32", "zui_signal_set_i32",
        "zui_derived_create", "zui_effect_create",
        "zui_fsm_create", "zui_fsm_send", "zui_fsm_on_enter", "zui_fsm_on_exit",
        "zui_spring_create", "zui_spring_set_target", "zui_spring_value",
        "zui_keyframe_create", "zui_keyframe_start", "zui_timeline_create",
        "zui_widget_create", "zui_widget_set_prop_f32", "zui_widget_add_child",
        "zui_on_event", "zui_run"
    ]
}

widget_file = { SOI ~ widget_def* ~ EOI }
  -> TypedProgram {
      "define": "program",
      "args": { "declarations": "$all" }
  }

widget_def = { "@widget" ~ IDENT ~ "{" ~ widget_body ~ "}" }
  -> TypedDeclaration {
      "commands": [
          // Widget becomes a struct
          { "define": "struct_def", "args": { "name": "$1", "fields": "$props" } },
          // Plus an init function that sets up signals/fsm/animations
          { "define": "function", "args": { 
              "name": { "concat": ["$1", "_init"] },
              "body": "$init_code"
          }},
          // Plus a render function
          { "define": "function", "args": {
              "name": { "concat": ["$1", "_render"] },
              "body": "$render_code"
          }}
      ]
  }

widget_body = { (prop_def | state_def | derived_def | machine_def | 
                 animation_def | spring_def | render_def)* }

prop_def = { "@prop" ~ IDENT ~ ":" ~ type_expr ~ ("=" ~ expr)? }
  -> StructField {
      "define": "field",
      "args": { "name": "$1", "type": "$2", "default": "$3" }
  }

state_def = { "@state" ~ IDENT ~ ":" ~ type_expr ~ "=" ~ expr }
  -> TypedStatement {
      "define": "let_stmt",
      "args": {
          "name": "$1",
          "type": "SignalHandle",
          "value": {
              "define": "call_expr",
              "args": {
                  "callee": { "concat": ["zui_signal_create_", { "type_suffix": "$2" }] },
                  "arguments": ["$3"]
              }
          }
      }
  }

derived_def = { "@derived" ~ IDENT ~ ":" ~ type_expr ~ "=" ~ expr }
  -> TypedStatement {
      "commands": [
          // Create a closure for the compute function
          { "define": "let_stmt", "args": {
              "name": { "concat": ["$1", "_compute"] },
              "value": { "define": "lambda", "body": "$3" }
          }, "store": "compute_fn" },
          // Create the derived signal
          { "define": "let_stmt", "args": {
              "name": "$1",
              "value": {
                  "define": "call_expr",
                  "args": {
                      "callee": "zui_derived_create",
                      "arguments": ["$compute_fn", { "deps_from_expr": "$3" }]
                  }
              }
          }}
      ]
  }

machine_def = { "@machine" ~ IDENT ~ "{" ~ "initial" ~ ":" ~ IDENT ~ 
                "states" ~ "{" ~ state_block* ~ "}" ~ 
                ("guards" ~ "{" ~ guard_block* ~ "}")? ~
                ("actions" ~ "{" ~ action_block* ~ "}")? ~ "}" }
  -> TypedStatement {
      "commands": [
          // Build transition table as array literal
          { "define": "let_stmt", "args": {
              "name": { "concat": ["$1", "_transitions"] },
              "value": { "build_transition_table": "$states" }
          }, "store": "trans_table" },
          // Create FSM
          { "define": "let_stmt", "args": {
              "name": "$1",
              "value": {
                  "define": "call_expr",
                  "args": {
                      "callee": "zui_fsm_create",
                      "arguments": [
                          { "state_id": "$initial" },
                          "$trans_table",
                          { "len": "$trans_table" }
                      ]
                  }
              }
          }},
          // Register entry/exit callbacks
          { "for_each_entry_action": "$states", 
            "emit": { "define": "call_expr", "callee": "zui_fsm_on_enter", ... }
          }
      ]
  }

spring_def = { "@spring" ~ IDENT ~ "{" ~ 
               "stiffness" ~ ":" ~ NUMBER ~
               "damping" ~ ":" ~ NUMBER ~
               ("mass" ~ ":" ~ NUMBER)? ~
               "target" ~ ":" ~ expr ~ "}" }
  -> TypedStatement {
      "commands": [
          // Create spring
          { "define": "let_stmt", "args": {
              "name": "$1",
              "value": {
                  "define": "call_expr",
                  "args": {
                      "callee": "zui_spring_create",
                      "arguments": ["$stiffness", "$damping", "$mass_or_1", "$initial_target"]
                  }
              }
          }},
          // Create effect to update target when dependencies change
          { "define": "call_expr", "args": {
              "callee": "zui_effect_create",
              "arguments": [
                  { "define": "lambda", "body": {
                      "define": "call_expr",
                      "callee": "zui_spring_set_target",
                      "arguments": ["$1", "$target_expr"]
                  }},
                  { "deps_from_expr": "$target_expr" }
              ]
          }}
      ]
  }

animation_def = { "@animation" ~ IDENT ~ "{" ~
                  "duration" ~ ":" ~ duration_lit ~
                  "easing" ~ ":" ~ IDENT ~
                  "keyframes" ~ "{" ~ keyframe* ~ "}" ~ "}" }
  -> TypedStatement {
      "define": "let_stmt",
      "args": {
          "name": "$1",
          "value": {
              "define": "call_expr",
              "args": {
                  "callee": "zui_keyframe_create",
                  "arguments": [
                      { "duration_ms": "$duration" },
                      { "easing_id": "$easing" },
                      { "len": "$keyframes" },
                      { "define": "array", "elements": "$keyframes" }
                  ]
              }
          }
      }
  }

render_def = { "@render" ~ "{" ~ widget_tree ~ "}" }
  -> TypedFunction {
      "define": "function",
      "args": {
          "name": "render",
          "params": [{ "name": "self", "type": "*Self" }],
          "body": { "widget_tree_to_calls": "$widget_tree" }
      }
  }

// Widget tree nodes compile to zui_widget_* calls
widget_node = { IDENT ~ "(" ~ prop_list? ~ ")" ~ ("{" ~ widget_node* ~ "}")? }
  -> TypedExpression {
      "commands": [
          { "define": "let_stmt", "args": {
              "name": "$generated_id",
              "value": { "define": "call_expr", "callee": "zui_widget_create", 
                         "arguments": [{ "widget_type_id": "$1" }] }
          }, "store": "widget" },
          { "for_each": "$props", "emit": {
              "define": "call_expr",
              "callee": { "prop_setter_for_type": "$prop_type" },
              "arguments": ["$widget", { "prop_id": "$prop_name" }, "$prop_value"]
          }},
          { "for_each": "$children", "emit": {
              "define": "call_expr",
              "callee": "zui_widget_add_child",
              "arguments": ["$widget", "$child_widget"]
          }},
          { "result": "$widget" }
      ]
  }

// Primitives
IDENT = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }
NUMBER = @{ ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? }
duration_lit = @{ NUMBER ~ ("ms" | "s") }
WHITESPACE = _{ " " | "\t" | "\n" | "\r" }
COMMENT = _{ "//" ~ (!"\n" ~ ANY)* }
```

### 7.2 Compilation Flow

```
1. Parse .zui file with zui.zyn grammar
   ↓
2. Grammar emits standard TypedAST
   - structs for widget definitions
   - functions for init/render
   - call_expr nodes for all ZRTL function calls
   ↓
3. Zyntax compiles TypedAST → HIR → Native Code
   - All zui_* calls are marked as extern
   - Zyntax has NO knowledge of UI concepts
   ↓
4. Link against zyntax_ui ZRTL plugin
   - Plugin provides all zui_* function implementations
   ↓
5. Run
   - Plugin initializes GPU renderer, event loop
   - Compiled code calls into plugin for all UI operations
```

### 7.3 Example: Button Widget Compilation

**Input (.zui):**
```zui
@widget Button {
    @prop label: String = ""
    @state pressed: Bool = false
    
    @spring scale {
        stiffness: 400
        damping: 25
        target: if pressed { 0.95 } else { 1.0 }
    }
    
    @render {
        Container(background: Color::blue()) {
            Text(content: label)
        }
    }
}
```

**Output (TypedAST, simplified):**
```json
{
  "kind": "program",
  "declarations": [
    {
      "kind": "struct_def",
      "name": "Button",
      "fields": [
        { "name": "label", "type": "String" },
        { "name": "pressed", "type": "*SignalHandle" },
        { "name": "scale", "type": "*SpringHandle" }
      ]
    },
    {
      "kind": "function",
      "name": "Button_init",
      "params": [{ "name": "self", "type": "*Button" }, { "name": "label", "type": "String" }],
      "body": {
        "kind": "block",
        "statements": [
          {
            "kind": "assignment",
            "target": { "kind": "field_access", "object": "self", "field": "label" },
            "value": { "kind": "variable", "name": "label" }
          },
          {
            "kind": "assignment",
            "target": { "kind": "field_access", "object": "self", "field": "pressed" },
            "value": {
              "kind": "call_expr",
              "callee": "zui_signal_create_bool",
              "args": [{ "kind": "bool_literal", "value": false }]
            }
          },
          {
            "kind": "assignment",
            "target": { "kind": "field_access", "object": "self", "field": "scale" },
            "value": {
              "kind": "call_expr",
              "callee": "zui_spring_create",
              "args": [
                { "kind": "float_literal", "value": 400.0 },
                { "kind": "float_literal", "value": 25.0 },
                { "kind": "float_literal", "value": 1.0 },
                { "kind": "float_literal", "value": 1.0 }
              ]
            }
          },
          {
            "kind": "call_expr",
            "callee": "zui_effect_create",
            "args": ["_scale_target_updater", 1, ["self.pressed"]]
          }
        ]
      }
    },
    {
      "kind": "function",
      "name": "Button_render",
      "params": [{ "name": "self", "type": "*Button" }],
      "body": { "...widget tree compiled to zui_widget_* calls..." }
    }
  ]
}
```

This TypedAST is completely standard - Zyntax compiles it like any other language, producing native code with extern calls to `zui_*` functions.

---

## Part 8: Comparison with Existing Solutions

| Feature | GPUI | Leptos | QML | Compose | **ZyntaxUI** |
|---------|------|--------|-----|---------|--------------|
| Language | Rust | Rust | QML/JS | Kotlin | Custom DSL → Rust |
| Reactivity | Entity-based | Fine-grained signals | Property bindings | Recomposition | Fine-grained signals |
| State Machines | Manual | Manual | Built-in (SCXML) | Manual | Built-in (Harel) |
| Animation | Manual | CSS/JS | Property Animation | Compose Animation | Keyframes + Springs |
| GPU Rendering | ✅ SDF | DOM | OpenGL/Vulkan | Skia | wgpu/Metal SDFs |
| AOT Compile | ✅ Rust | WASM | ❌ | ❌ | ✅ Zyntax |
| Embeddable | Moderate | Web only | C++/QML | JVM | ✅ Single Rust crate |

---

## Part 9: Implementation Roadmap

### Phase 1: Core Infrastructure (Q1 2026)
- [ ] Zyn grammar for `.zui` DSL
- [ ] Reactive signal system in Rust
- [ ] Basic widget tree and layout (Taffy integration)
- [ ] Software renderer for prototyping

### Phase 2: State Machines & Animation (Q2 2026)
- [ ] FSM compiler and runtime
- [ ] Keyframe animation scheduler
- [ ] Spring physics engine
- [ ] GPU renderer (wgpu backend)

### Phase 3: Widget Library (Q3 2026)
- [ ] 10 core widgets with full FSM/animation
- [ ] Theming system
- [ ] Accessibility support
- [ ] Developer tools (state inspector, animation debugger)

### Phase 4: Production Hardening (Q4 2026)
- [ ] Performance optimization
- [ ] Platform-specific rendering (Metal, DX12, Vulkan)
- [ ] Comprehensive test suite
- [ ] Documentation and examples

---

## Appendix A: Key References

1. **Jetpack Compose Internals** - Jorge Castillo's book on compiler/runtime architecture
2. **QML State Machine Framework** - Qt documentation on SCXML integration
3. **GPUI README** - Zed's hybrid immediate/retained mode design
4. **Framer Motion API** - Spring physics and keyframe animation patterns
5. **anime.js v4** - Timeline composition and stagger functions
6. **XState Documentation** - Statecharts for UI logic
7. **Leptos Reactive System** - Fine-grained signals in Rust
8. **Floem** - Native Rust UI with signals (Lapce project)
9. **Reactively Algorithm** - Push-pull reactive graph update strategy

---

## Appendix B: Example Widget Implementation

Complete example of a Toggle widget:

```zui
@widget Toggle {
    @prop value: Bool = false
    @prop on_change: Callback<Bool> = {}
    @prop disabled: Bool = false
    
    @state thumb_x: Signal<f32> = if value { 20.0 } else { 0.0 }
    
    @spring thumb_spring {
        stiffness: 500
        damping: 30
        target: if value { 20.0 } else { 0.0 }
    }
    
    @derived track_color: Color = 
        if disabled { Color::gray(0.3) }
        else if value { theme.primary }
        else { Color::gray(0.5) }
    
    @machine toggle_fsm {
        initial: idle
        
        states {
            idle {
                on TAP if !disabled => toggling
            }
            toggling {
                entry: {
                    let new_value = !value;
                    on_change.emit(new_value);
                    thumb_spring.animate_to(if new_value { 20.0 } else { 0.0 });
                }
                after 300ms => idle
            }
        }
    }
    
    @render {
        Container(
            width: 44.px,
            height: 24.px,
            border_radius: 12.px,
            background: track_color,
            cursor: if disabled { Cursor::NotAllowed } else { Cursor::Pointer },
        ) {
            // Thumb
            Container(
                width: 20.px,
                height: 20.px,
                border_radius: 10.px,
                background: Color::white(),
                shadow: Shadow::sm(),
                transform: Transform::translate(thumb_spring.value + 2.0, 2.0),
            )
        }
    }
}
```

This compiles to approximately:
- 1 reactive signal
- 1 spring animator
- 1 2-state FSM
- 2 GPU draw calls (track rect + thumb circle with SDF)