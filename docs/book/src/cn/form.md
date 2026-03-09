# Form Components

Components for building forms: inputs, textareas, switches, selects, sliders, and wrappers.

## Input

```rust
use blinc_cn::prelude::*;
use blinc_layout::widgets::text_input::text_input_data;

let name = text_input_data();

input(&name)
    .label("Name")
    .placeholder("Enter your name...")
    .on_change(|value| println!("name: {}", value));
```

## Textarea

```rust
use blinc_layout::widgets::text_area::text_area_state;

let description = text_area_state();

textarea(&description)
    .label("Description")
    .placeholder("Enter description...")
    .rows(4);
```

## Field

```rust
let email = text_input_data();

field("Email")
    .required()
    .description("We'll only use this for account notices.")
    .child(input(&email).placeholder("name@example.com"));
```

## Form

```rust
let name = text_input_data();
let email = text_input_data();

form()
    .spacing(16.0)
    .max_w(420.0)
    .child(field("Name").required().child(input(&name).placeholder("John Doe")))
    .child(
        field("Email")
            .required()
            .child(input(&email).input_type("email").placeholder("john@example.com")),
    );
```

## Checkbox

```rust
checkbox()
    .checked(true)
    .child(label("Accept terms and conditions"));
```

## Switch

```rust
let dark_mode = blinc_core::State::new(false);

switch(&dark_mode)
    .label("Dark mode")
    .on_change(|enabled| println!("dark mode: {}", enabled));
```

## Radio Group

```rust
let selected = blinc_core::State::new("medium".to_string());

radio_group(&selected)
    .option("small", "Small")
    .option("medium", "Medium")
    .option("large", "Large")
    .on_change(|value| println!("selected: {}", value));
```

## Select

```rust
let framework = blinc_core::State::new(String::new());

select(&framework)
    .label("Framework")
    .placeholder("Choose one")
    .option("react", "React")
    .option("svelte", "Svelte")
    .option("solid", "Solid")
    .on_change(|value| println!("selected: {}", value));
```

## Combobox

```rust
let framework = blinc_core::State::new(String::new());

combobox(&framework)
    .label("Framework")
    .placeholder("Search frameworks...")
    .option("react", "React")
    .option("svelte", "Svelte")
    .option("solid", "Solid");
```

## Slider

```rust
let volume = blinc_core::State::new(50.0);

slider(&volume)
    .min(0.0)
    .max(100.0)
    .step(1.0)
    .on_change(|value| println!("volume: {}", value));
```
