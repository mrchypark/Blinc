# Card

Cards group related content and actions.

## Basic Usage

```rust
use blinc_cn::prelude::*;

card()
    .child(
        card_header()
            .title("Card Title")
            .description("Card description text"),
    )
    .child(card_content().child(text("Card content goes here.")))
    .child(card_footer().child(button("Action")));
```

## Card Header

`card_header()` exposes convenience methods instead of separate title/description builders.

```rust
card_header()
    .title("Account Settings")
    .description("Manage your account preferences");
```

## Examples

### Simple Card

```rust
card()
    .child(
        card_header()
            .title("Notifications")
            .description("Configure notification settings"),
    )
    .child(
        card_content().child(
            text("Card content can contain any layout tree."),
        ),
    );
```

### Card With Actions

```rust
card()
    .w(360.0)
    .child(
        card_header()
            .title("Login")
            .description("Enter your credentials"),
    )
    .child(card_content().child(text("Form fields go here")))
    .child(
        card_footer()
            .child(button("Cancel").variant(ButtonVariant::Outline))
            .child(button("Continue")),
    );
```
