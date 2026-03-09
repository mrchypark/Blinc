# blinc_cn

> **Part of the [Blinc UI Framework](https://project-blinc.github.io/Blinc)**
>
> This crate is a component of Blinc, a GPU-accelerated UI framework for Rust.
> For full documentation and guides, visit the [Blinc documentation](https://project-blinc.github.io/Blinc).

Component library for Blinc UI with themed, token-driven components built on top of `blinc_layout`.

## Overview

`blinc_cn` provides a broad set of reusable UI components on top of the Blinc theme system.
It is best used with an explicit bootstrap step so theme tokens and component styles are available
before the first render.
The default stylesheet is now driven by semantic component tokens from `blinc_theme`, so presets
control control sizing, container spacing, overlay chrome, and typography roles more consistently.

## Installation

```toml
[dependencies]
blinc_cn = { path = "../blinc_cn" }
blinc_theme = { path = "../blinc_theme" }
```

## Recommended Setup

Initialize theme state once at startup:

```rust
use blinc_cn::ensure_default_theme;

fn init_ui() {
    ensure_default_theme();
}
```

Or choose a specific preset:

```rust
use blinc_cn::ensure_theme;
use blinc_theme::{ColorScheme, ThemePreset};

fn init_ui() {
    ensure_theme(ThemePreset::Slate.bundle(), ColorScheme::Light);
}
```

When using CSS stylesheets in a `WindowedContext`, load the default component stylesheet once:

```rust
ctx.add_css(blinc_cn::default_styles());
```

## Quick Start

```rust
use blinc_cn::prelude::*;

fn build_ui() -> impl ElementBuilder {
    card()
        .w(360.0)
        .child(
            card_header()
                .title("Welcome")
                .description("Get started with blinc_cn"),
        )
        .child(
            card_content().child(
                text("Theme-aware components with sensible defaults."),
            ),
        )
        .child(
            card_footer().child(
                button("Continue").variant(ButtonVariant::Primary),
            ),
        )
}
```

## Buttons

```rust
button("Primary")
button("Outline").variant(ButtonVariant::Outline)
button("Danger").variant(ButtonVariant::Destructive)
button("Small").size(ButtonSize::Small)
button("Large").size(ButtonSize::Large)
button("").size(ButtonSize::Icon).icon(icons::SETTINGS)
```

## Form Components

```rust
use blinc_layout::widgets::text_area::text_area_state;
use blinc_layout::widgets::text_input::text_input_data;

let email = text_input_data();
let notes = text_area_state();

form()
    .max_w(420.0)
    .child(
        field("Email")
            .required()
            .child(input(&email).placeholder("name@example.com")),
    )
    .child(
        field("Notes")
            .description("Optional")
            .child(textarea(&notes).rows(4).placeholder("Write a short note")),
    )
```

```rust
let enabled = blinc_core::State::new(false);
switch(&enabled).on_change(|value| println!("enabled: {value}"));
```

```rust
let selected = blinc_core::State::new(String::new());

select(&selected)
    .label("Framework")
    .placeholder("Choose one")
    .option("react", "React")
    .option("svelte", "Svelte")
    .on_change(|value| println!("selected: {value}"));
```

## Dialogs

`dialog()` and `alert_dialog()` are imperative builders that display overlays with `.show()`.

```rust
button("Open Dialog").on_click(|_| {
    dialog()
        .title("Confirm")
        .description("Apply these changes?")
        .confirm_text("Apply")
        .on_confirm(|| println!("confirmed"))
        .show();
});
```

## Navigation

```rust
let tab_state = blinc_core::State::new(String::new());

tabs(&tab_state)
    .tab("account", "Account", || div().child(text("Account settings")))
    .tab("billing", "Billing", || div().child(text("Billing settings")));
```

```rust
let collapsed = blinc_core::State::new(false);

sidebar(&collapsed)
    .section("Main")
    .item_active("Dashboard", icons::HOME, || {})
    .item("Settings", icons::SETTINGS, || {});
```

## Components

- Buttons: `button`
- Cards: `card`, `card_header`, `card_content`, `card_footer`
- Feedback: `alert`, `badge`, `progress`, `spinner`, `skeleton`, `toast`
- Forms: `input`, `textarea`, `checkbox`, `switch`, `radio_group`, `select`, `combobox`, `slider`, `field`, `form`
- Navigation: `tabs`, `dropdown_menu`, `context_menu`, `breadcrumb`, `sidebar`, `navigation_menu`, `pagination`, `menubar`
- Overlays: `dialog`, `alert_dialog`, `sheet`, `drawer`, `tooltip`, `popover`, `hover_card`
- Layout and display: `avatar`, `separator`, `scroll_area`, `accordion`, `aspect_ratio`, `chart`, `tree_view`
