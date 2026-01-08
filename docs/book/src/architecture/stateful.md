# Stateful Elements & FSM

Blinc uses **Finite State Machines (FSM)** to manage interactive UI state. This provides predictable state transitions for widgets like buttons, checkboxes, and text fields.

## Finite State Machines

### Core Concepts

An FSM defines:

- **States**: Discrete conditions the element can be in
- **Events**: Inputs that trigger transitions
- **Transitions**: Rules mapping (state, event) -> new_state

```rust
// State IDs and Event IDs are u32
type StateId = u32;
type EventId = u32;

struct Transition {
    from_state: StateId,
    event: EventId,
    to_state: StateId,
    guard: Option<Box<dyn Fn() -> bool>>,  // Conditional transition
    action: Option<Box<dyn Fn()>>,          // Side effect
}
```

### FSM Builder

```rust
let fsm = StateMachine::builder(initial_state)
    .on(State::Idle, Event::PointerEnter, State::Hovered)
    .on(State::Hovered, Event::PointerLeave, State::Idle)
    .on(State::Hovered, Event::PointerDown, State::Pressed)
    .on(State::Pressed, Event::PointerUp, State::Hovered)
    .on_enter(State::Pressed, || {
        println!("Button pressed!");
    })
    .build();
```

### Entry/Exit Callbacks

```rust
.on_enter(state, || { /* called when entering state */ })
.on_exit(state, || { /* called when leaving state */ })
```

### Guard Conditions

Transitions can be conditional:

```rust
.transition(
    Transition::new(State::Idle, Event::Click, State::Active)
        .with_guard(|| is_enabled())
)
```

---

## StateTransitions Trait

For type-safe state definitions, implement `StateTransitions`:

```rust
use blinc_layout::stateful::StateTransitions;
use blinc_core::events::event_types::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
enum ButtonState {
    #[default]
    Idle,
    Hovered,
    Pressed,
    Disabled,
}

impl StateTransitions for ButtonState {
    fn on_event(&self, event: u32) -> Option<Self> {
        match (self, event) {
            (ButtonState::Idle, POINTER_ENTER) => Some(ButtonState::Hovered),
            (ButtonState::Hovered, POINTER_LEAVE) => Some(ButtonState::Idle),
            (ButtonState::Hovered, POINTER_DOWN) => Some(ButtonState::Pressed),
            (ButtonState::Pressed, POINTER_UP) => Some(ButtonState::Hovered),
            _ => None,
        }
    }
}
```

### Available Event Types

```rust
// Pointer events
POINTER_ENTER    // Mouse enters element
POINTER_LEAVE    // Mouse leaves element
POINTER_DOWN     // Mouse button pressed
POINTER_UP       // Mouse button released
POINTER_MOVE     // Mouse moved over element

// Keyboard events
KEY_DOWN         // Key pressed
KEY_UP           // Key released
TEXT_INPUT       // Character typed

// Focus events
FOCUS            // Element gained focus
BLUR             // Element lost focus

// Other
SCROLL           // Scroll event
DRAG             // Drag motion
DRAG_END         // Drag completed
```

---

## Stateful Elements

### Creating Stateful Elements

```rust
use blinc_layout::prelude::*;

fn interactive_card() -> impl ElementBuilder {
    stateful::<ButtonState>()
        .w(200.0)
        .h(120.0)
        .rounded(12.0)
        .on_state(|ctx| {
            let bg = match ctx.state() {
                ButtonState::Idle => Color::rgba(0.15, 0.15, 0.2, 1.0),
                ButtonState::Hovered => Color::rgba(0.18, 0.18, 0.25, 1.0),
                ButtonState::Pressed => Color::rgba(0.12, 0.12, 0.16, 1.0),
                ButtonState::Disabled => Color::rgba(0.1, 0.1, 0.12, 0.5),
            };
            div().bg(bg).child(text("Hover me").color(Color::WHITE))
        })
}
```

### How It Works

1. **Builder creation**: `stateful::<S>()` creates a StatefulBuilder for state type S
2. **Key generation**: Automatic key based on call site location
3. **Event routing**: Pointer/keyboard events are routed to the FSM
4. **State transition**: FSM computes new state from (current_state, event)
5. **Callback invocation**: `on_state` callback runs with StateContext
6. **Visual update**: Returned Div is merged onto container

### StateContext API

The callback receives a `StateContext` with these methods:

```rust
.on_state(|ctx| {
    // Get current state
    let state = ctx.state();

    // Get triggering event (if any)
    if let Some(event) = ctx.event() {
        // Handle specific event types
        match event.event_type {
            POINTER_UP => println!("Clicked!"),
            _ => {}
        }
    }

    // Create scoped signals
    let counter = ctx.use_signal("counter", || 0);

    // Create animated values (spring physics)
    let opacity = ctx.use_animated_value("opacity", 1.0);

    // Create animated timelines (keyframe sequences)
    let timeline = ctx.use_timeline("fade");

    // Access dependency values by index
    let value: i32 = ctx.dep(0).unwrap_or_default();

    // Get dependency as State handle
    let state_handle = ctx.dep_as_state::<i32>(0);

    // Dispatch events
    ctx.dispatch(CUSTOM_EVENT);

    div()
})
```

---

## Built-in State Types

### ButtonState

```rust
enum ButtonState {
    Idle,      // Default
    Hovered,   // Mouse over
    Pressed,   // Mouse down
    Disabled,  // Non-interactive
}
```

Transitions:

- Idle → Hovered (pointer enter)
- Hovered → Idle (pointer leave)
- Hovered → Pressed (pointer down)
- Pressed → Hovered (pointer up)

### NoState

For elements that only need dependency tracking:

```rust
stateful::<NoState>()
    .deps([signal.signal_id()])
    .on_state(|_ctx| {
        div().child(text("Rebuilds on signal change"))
    })
```

### ToggleState

```rust
enum ToggleState {
    Off,
    On,
}
```

Transitions:

- Off → On (click)
- On → Off (click)

### CheckboxState

```rust
enum CheckboxState {
    UncheckedIdle,
    UncheckedHovered,
    CheckedIdle,
    CheckedHovered,
}
```

### TextFieldState

```rust
enum TextFieldState {
    Idle,
    Hovered,
    Focused,
    FocusedHovered,
    Disabled,
}
```

### ScrollState

```rust
enum ScrollState {
    Idle,
    Scrolling,
    Decelerating,
    Bouncing,
}
```

---

## Signal Dependencies

Stateful elements can depend on external signals using `.deps()`:

```rust
fn counter_display(count: State<i32>) -> impl ElementBuilder {
    stateful::<ButtonState>()
        .deps([count.signal_id()])  // Re-run on_state when count changes
        .on_state(move |ctx| {
            // Access via captured variable
            let value = count.get();

            // Or via context by index
            let value_alt: i32 = ctx.dep(0).unwrap_or_default();

            div().child(
                text(&format!("Count: {}", value)).color(Color::WHITE)
            )
        })
}
```

### Accessing Dependencies

Two patterns for accessing dependency values:

```rust
// Pattern 1: Capture in closure
let my_signal = use_state(|| 42);

stateful::<ButtonState>()
    .deps([my_signal.signal_id()])
    .on_state(move |ctx| {
        let value = my_signal.get();  // Via captured variable
        div()
    })

// Pattern 2: Access via context
stateful::<ButtonState>()
    .deps([my_signal.signal_id()])
    .on_state(|ctx| {
        let value: i32 = ctx.dep(0).unwrap_or_default();  // Via index
        div()
    })
```

### When to Use `.deps()`

| Without `.deps()` | With `.deps()` |
| ----------------- | -------------- |
| Only runs on state transitions | Also runs when dependencies change |
| Hover/press only | External data + hover/press |

---

## Scoped State Management

StateContext provides scoped utilities that persist across rebuilds:

### Scoped Signals

```rust
stateful::<ButtonState>()
    .on_state(|ctx| {
        // Signal keyed as "{stateful_key}:signal:click_count"
        let clicks = ctx.use_signal("click_count", || 0);

        div()
            .child(text(&format!("Clicks: {}", clicks.get())))
            .on_click(move |_| clicks.update(|n| n + 1))
    })
```

### Animated Values

```rust
stateful::<ButtonState>()
    .on_state(|ctx| {
        // Spring-animated value keyed to this stateful
        let scale = ctx.use_animated_value("scale", 1.0);

        // With custom spring config
        let opacity = ctx.use_animated_value_with_config(
            "opacity",
            1.0,
            SpringConfig::bouncy(),
        );

        // Set target based on state
        match ctx.state() {
            ButtonState::Hovered => {
                scale.lock().unwrap().set_target(1.1);
            }
            _ => {
                scale.lock().unwrap().set_target(1.0);
            }
        }

        let s = scale.lock().unwrap().get();
        div().transform(Transform::scale(s, s))
    })
```

---

## Custom State Machines

For complex interactions, define your own states:

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
enum DragState {
    #[default]
    Idle,
    Hovering,
    Pressing,
    Dragging,
}

impl StateTransitions for DragState {
    fn on_event(&self, event: u32) -> Option<Self> {
        match (self, event) {
            (DragState::Idle, POINTER_ENTER) => Some(DragState::Hovering),
            (DragState::Hovering, POINTER_LEAVE) => Some(DragState::Idle),
            (DragState::Hovering, POINTER_DOWN) => Some(DragState::Pressing),
            (DragState::Pressing, DRAG) => Some(DragState::Dragging),
            (DragState::Pressing, POINTER_UP) => Some(DragState::Hovering),
            (DragState::Dragging, DRAG_END) => Some(DragState::Idle),
            _ => None,
        }
    }
}

fn draggable_element() -> impl ElementBuilder {
    stateful::<DragState>()
        .on_state(|ctx| {
            let bg = match ctx.state() {
                DragState::Idle => Color::BLUE,
                DragState::Hovering => Color::CYAN,
                DragState::Pressing => Color::YELLOW,
                DragState::Dragging => Color::GREEN,
            };
            div().w(100.0).h(100.0).bg(bg)
        })
}
```

---

## Event Routing

### Event Flow

```text
Platform Event (pointer, keyboard)
    │
    ├── Hit test: which element?
    │
    ├── EventRouter dispatches to element
    │
    ├── StateMachine receives event
    │   └── Computes transition
    │
    └── on_state callback invoked
```

### Event Context

Handlers receive event details:

```rust
.on_click(|ctx| {
    println!("Clicked at ({}, {})", ctx.local_x, ctx.local_y);
})
.on_key_down(|ctx| {
    if ctx.ctrl && ctx.key_code == 83 {  // Ctrl+S
        save();
    }
})
```

---

## Performance

### Why FSM Over Signals?

| Signals for visual state | FSM for visual state |
| ------------------------ | -------------------- |
| Triggers full rebuild | Updates only affected element |
| Creates new VDOM | Mutates existing element |
| O(tree size) | O(1) |

### Minimal Updates

Stateful elements only update their own RenderProps:

```rust
// State change only affects this element
.on_state(|ctx| {
    div().bg(new_color)  // Updates RenderProps
    // No layout recomputation
    // No tree diff
    // Just visual update
})
```

### Queued Updates

State changes queue updates efficiently:

```rust
static PENDING_PROP_UPDATES: Vec<(NodeId, RenderProps)>;

// Stateful callback queues update
fn on_state(ctx) -> Div {
    div().bg(color)
    // Queues: (node_id, updated_props)
}

// Processed in batch by windowed app
for (node_id, props) in drain_pending() {
    render_tree.update_props(node_id, props);
}
```
