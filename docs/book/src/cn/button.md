# Button

Buttons trigger actions or events.

## Basic Usage

```rust
use blinc_cn::prelude::*;

button("Click me")
    .on_click(|_| println!("clicked"));
```

## Variants

```rust
button("Save").variant(ButtonVariant::Primary)
button("Cancel").variant(ButtonVariant::Secondary)
button("Delete").variant(ButtonVariant::Destructive)
button("Edit").variant(ButtonVariant::Outline)
button("More").variant(ButtonVariant::Ghost)
button("Learn more").variant(ButtonVariant::Link)
```

## Sizes

```rust
button("Small").size(ButtonSize::Small)
button("Medium").size(ButtonSize::Medium)
button("Large").size(ButtonSize::Large)
button("").size(ButtonSize::Icon).icon(icons::SETTINGS)
```

## Icons

```rust
button("Settings").icon(icons::SETTINGS)

button("Next")
    .icon(icons::ARROW_RIGHT)
    .icon_position(IconPosition::End);
```

## Disabled State

```rust
button("Disabled")
    .disabled(true);
```
