# Form Components

Components for building forms: inputs, checkboxes, selects, and more.

## Input

Text input field:

```rust
use blinc_cn::prelude::*;
use blinc_layout::widgets::text_input::text_input_data;

let name = text_input_data();

input(&name)
    .placeholder("Enter your name...")
    .on_change(|value| println!("name: {}", value));
```

### Input Types

```rust
use blinc_layout::widgets::text_input::text_input_data;

let name = text_input_data();
let email = text_input_data();
let password = text_input_data();
let age = text_input_data();
let search = text_input_data();

// Text (default)
input(&name).placeholder("Name")

// Email
input(&email).input_type("email").placeholder("Email")

// Password
input(&password).input_type("password").placeholder("Password")

// Number
input(&age).input_type("number").placeholder("Age")

// Search
input(&search).input_type("search").placeholder("Search...")
```

### Input States

```rust
use blinc_layout::widgets::text_input::text_input_data;

let disabled_input = text_input_data();
let error_input = text_input_data();

// Disabled
input(&disabled_input).disabled(true)

// With error
input(&error_input).error("Invalid value")
```

## Textarea

Multi-line text input:

```rust
use blinc_layout::widgets::text_area::text_area_state;

let description = text_area_state();

textarea(&description)
    .placeholder("Enter description...")
    .rows(4)
    .on_change(|value| println!("description: {}", value));
```

## Field

`field()` wraps a single control with label + helper/error text.

```rust
use blinc_layout::widgets::text_input::text_input_data;

let email = text_input_data();

field("Email")
    .required()
    .description("We'll only use this for account notices.")
    .child(
        input(&email)
            .placeholder("name@example.com")
    )
```

## Form

`form()` is a vertical layout container for multiple fields.

```rust
use blinc_layout::widgets::text_input::text_input_data;

let name = text_input_data();
let email = text_input_data();

form()
    .spacing(16.0)
    .max_w(420.0)
    .child(
        field("Name")
            .required()
            .child(input(&name).placeholder("John Doe"))
    )
    .child(
        field("Email")
            .required()
            .child(input(&email).input_type("email").placeholder("john@example.com"))
    )
```

## Checkbox

```rust
checkbox()
    .checked(is_checked)
    .on_change(|checked| println!("checked: {}", checked))
    .child(label("Accept terms and conditions"))
```

### Indeterminate State

```rust
checkbox()
    .checked(some_checked)
    .indeterminate(some_checked && !all_checked)
    .on_change(|checked| toggle_all(checked))
    .child(label("Select all"))
```

## Switch

Toggle switch:

```rust
switch_()
    .checked(is_enabled)
    .on_change(|enabled| println!("enabled: {}", enabled))
```

### With Label

```rust
div()
    .flex_row()
    .items_center()
    .gap(8.0)
    .child(switch_().checked(dark_mode).on_change(|v| println!("dark mode: {}", v)))
    .child(label("Dark mode"))
```

## Radio Group

```rust
radio_group()
    .value(selected)
    .on_change(|value| println!("selected: {}", value))
    .child(
        div().flex_col().gap(8.0)
            .child(radio_item("small").child(label("Small")))
            .child(radio_item("medium").child(label("Medium")))
            .child(radio_item("large").child(label("Large")))
    )
```

## Select

Dropdown selection:

```rust
select()
    .value(selected)
    .on_change(|value| println!("selected: {}", value))
    .child(select_trigger()
        .child(select_value().placeholder("Select option...")))
    .child(select_content()
        .child(select_item("opt1").child(text("Option 1")))
        .child(select_item("opt2").child(text("Option 2")))
        .child(select_item("opt3").child(text("Option 3"))))
```

### Grouped Options

```rust
select()
    .child(select_trigger().child(select_value()))
    .child(select_content()
        .child(select_group()
            .child(select_label("Fruits"))
            .child(select_item("apple").child(text("Apple")))
            .child(select_item("banana").child(text("Banana"))))
        .child(select_separator())
        .child(select_group()
            .child(select_label("Vegetables"))
            .child(select_item("carrot").child(text("Carrot")))
            .child(select_item("broccoli").child(text("Broccoli")))))
```

## Combobox

Searchable select with autocomplete:

```rust
combobox()
    .value(selected)
    .on_change(|value| println!("selected: {}", value))
    .child(combobox_trigger()
        .child(combobox_input().placeholder("Search...")))
    .child(combobox_content()
        .child(combobox_empty().child(text("No results found")))
        .child(combobox_item("react").child(text("React")))
        .child(combobox_item("vue").child(text("Vue")))
        .child(combobox_item("svelte").child(text("Svelte"))))
```

## Slider

Range slider:

```rust
slider()
    .value(volume)
    .min(0.0)
    .max(100.0)
    .step(1.0)
    .on_change(|value| println!("volume: {}", value))
```

### Range Slider

```rust
slider()
    .value_range(min_price, max_price)
    .min(0.0)
    .max(1000.0)
    .on_change_range(|min, max| {
        println!("price range: {} - {}", min, max);
    })
```

## Label

```rust
// Associated with input via for
label("Email").for_id("email-input")

// Direct child of input
checkbox()
    .child(label("Remember me"))
```

## Form Layout Example

```rust
use blinc_layout::widgets::text_input::text_input_data;

let name = text_input_data();
let email = text_input_data();

form()
    .spacing(24.0)
    .max_w(400.0)
    .child(
        field("Name")
            .required()
            .child(input(&name)
                .placeholder("John Doe")
                .on_change(|v| println!("name: {}", v)))
    )
    .child(
        field("Email")
            .required()
            .child(input(&email)
                .input_type("email")
                .placeholder("john@example.com")
                .on_change(|v| println!("email: {}", v)))
    )
    .child(
        button("Submit")
            .on_click(|| submit_form())
    )
```

## Validation

```rust
use blinc_layout::widgets::text_input::text_input_data;

let email = text_input_data();
let show_error = true; // replace with your own validation condition

field("Email")
    .when(show_error, |f| f.error("Invalid email address"))
    .child(
        input(&email)
            .input_type("email")
            .on_change(|v| println!("email: {}", v))
    )
```
