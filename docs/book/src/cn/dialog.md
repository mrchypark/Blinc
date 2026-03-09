# Dialog

Dialogs in `blinc_cn` are imperative overlay builders.

## Basic Usage

```rust
use blinc_cn::prelude::*;

button("Open Dialog").on_click(|_| {
    dialog()
        .title("Dialog Title")
        .description("Dialog description")
        .confirm_text("Continue")
        .cancel_text("Cancel")
        .on_confirm(|| println!("confirmed"))
        .show();
});
```

## Dialog Builder

```rust
dialog()
    .title("Edit Profile")
    .description("Update your profile information")
    .content(|| {
        div()
            .flex_col()
            .gap(12.0)
            .child(text("Custom content goes here"))
    })
    .footer(|| {
        div()
            .flex_row()
            .gap(8.0)
            .child(button("Cancel").variant(ButtonVariant::Outline))
            .child(button("Save"))
    })
    .show();
```

## Alert Dialog

Use `alert_dialog()` for single-action confirmation flows:

```rust
alert_dialog()
    .title("Delete Account")
    .description("This action cannot be undone.")
    .confirm_text("Delete")
    .on_confirm(|| println!("delete confirmed"))
    .show();
```

## Sheet

```rust
button("Open Sheet").on_click(|_| {
    sheet_right()
        .title("Settings")
        .description("Update application settings")
        .show();
});
```

## Drawer

```rust
button("Open Drawer").on_click(|_| {
    drawer()
        .title("Actions")
        .description("Mobile-style bottom drawer")
        .show();
});
```
