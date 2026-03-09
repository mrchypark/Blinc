# Component Library Overview

`blinc_cn` is the themed component library for Blinc UI. Its default stylesheet uses the newer CSS
engine surface directly, including token-driven `corner-shape`, overlay `backdrop-filter`,
truncation helpers, and CSS text decoration for link-like affordances.

## Setup

Initialize theme state before building UI:

```rust
use blinc_cn::ensure_default_theme;

fn init_ui() {
    ensure_default_theme();
}
```

If your app uses stylesheet injection through `WindowedContext`, load the default component styles once:

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
        .child(card_content().child(text("Beautiful, themed components.")))
        .child(card_footer().child(button("Get Started")))
}
```

## Design Principles

### Composable

Components stay close to `blinc_layout` builders and can still be customized with regular layout methods.

### Themeable

Components read colors, spacing, radii, and typography from `blinc_theme::ThemeState`.
Common control, container, overlay, and typography-role defaults are derived from semantic theme tokens,
so a preset can move the whole component set together instead of each component carrying its own numbers.

### Practical

The goal is not to hide layout primitives. The goal is to give you better defaults and less repetitive styling.

## Categories

| Category | Components |
|----------|------------|
| Buttons | Button |
| Cards | Card, CardHeader, CardContent, CardFooter |
| Dialogs | Dialog, AlertDialog, Sheet, Drawer |
| Forms | Input, Textarea, Checkbox, Switch, Radio, Select, Slider, Field, Form |
| Navigation | Tabs, DropdownMenu, ContextMenu, Breadcrumb, Sidebar, NavigationMenu |
| Feedback | Alert, Badge, Progress, Spinner, Skeleton, Toast |
| Layout | Avatar, Separator, AspectRatio, ScrollArea, Accordion |
| Data | Tooltip, HoverCard, Popover, Chart, TreeView |
