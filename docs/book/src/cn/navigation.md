# Navigation Components

Components for tabs, breadcrumbs, menus, pagination, and sidebars.

## Tabs

```rust
use blinc_cn::prelude::*;

let active_tab = blinc_core::State::new(String::new());

tabs(&active_tab)
    .tab("account", "Account", || div().child(text("Account settings")))
    .tab("password", "Password", || div().child(text("Password settings")))
    .tab("settings", "Settings", || div().child(text("Other settings")));
```

## Dropdown Menu

```rust
dropdown_menu()
    .item("edit", "Edit", || println!("edit"))
    .item("duplicate", "Duplicate", || println!("duplicate"))
    .separator()
    .item("delete", "Delete", || println!("delete"));
```

## Breadcrumb

```rust
breadcrumb()
    .item("Home", || println!("home"))
    .item("Products", || println!("products"))
    .current("Details");
```

## Pagination

```rust
let page = blinc_core::State::new(1usize);

pagination(&page)
    .total_pages(10)
    .on_change(|next| println!("page: {}", next));
```

## Sidebar

```rust
let collapsed = blinc_core::State::new(false);

sidebar(&collapsed)
    .section("Main")
    .item_active("Dashboard", icons::HOME, || println!("dashboard"))
    .item("Projects", icons::FOLDER, || println!("projects"))
    .section("Settings")
    .item("Preferences", icons::SETTINGS, || println!("prefs"));
```

## Navigation Menu

```rust
navigation_menu()
    .item("Docs", || println!("docs"))
    .item("Pricing", || println!("pricing"))
    .item("About", || println!("about"));
```
