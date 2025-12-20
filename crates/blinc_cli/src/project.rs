//! Project creation and scaffolding

use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::config::BlincConfig;

/// Create a new Blinc project
pub fn create_project(path: &Path, name: &str, template: &str) -> Result<()> {
    // Create directory structure
    fs::create_dir_all(path.join("src"))?;
    fs::create_dir_all(path.join("assets"))?;

    // Create blinc.toml
    let config = BlincConfig::new(name);
    fs::write(path.join("blinc.toml"), config.to_toml()?)?;

    // Create main file based on template
    let main_content = match template {
        "minimal" => template_minimal(name),
        "counter" => template_counter(name),
        _ => template_default(name),
    };

    fs::write(path.join("src/main.blinc"), main_content)?;

    // Create .gitignore
    fs::write(
        path.join(".gitignore"),
        r#"# Blinc build artifacts
/target/
*.zrtl

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db
"#,
    )?;

    // Create README
    fs::write(
        path.join("README.md"),
        format!(
            r#"# {}

A Blinc UI application.

## Development

```bash
blinc dev
```

## Build

```bash
blinc build --release
```

## Project Structure

```
{}/
├── blinc.toml       # Project configuration
├── src/
│   └── main.blinc   # Application entry point
└── assets/          # Static assets
```
"#,
            name, name
        ),
    )?;

    Ok(())
}

/// Create a new ZRTL plugin project
pub fn create_plugin_project(path: &Path, name: &str) -> Result<()> {
    fs::create_dir_all(path.join("src"))?;

    // Create Cargo.toml for the plugin
    fs::write(
        path.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "staticlib"]

[dependencies]
# Add your plugin dependencies here

[features]
default = []
"#,
            name
        ),
    )?;

    // Create lib.rs
    fs::write(
        path.join("src/lib.rs"),
        format!(
            r#"//! {} - Blinc ZRTL Plugin
//!
//! This plugin can be loaded dynamically or compiled statically.

/// Plugin initialization - called when the plugin is loaded
#[no_mangle]
pub extern "C" fn plugin_init() {{
    // Initialize your plugin here
}}

/// Plugin cleanup - called when the plugin is unloaded
#[no_mangle]
pub extern "C" fn plugin_cleanup() {{
    // Clean up resources here
}}

/// Example exported function
#[no_mangle]
pub extern "C" fn hello() -> *const std::ffi::c_char {{
    static HELLO: &[u8] = b"Hello from {}!\0";
    HELLO.as_ptr() as *const std::ffi::c_char
}}
"#,
            name, name
        ),
    )?;

    // Create README
    fs::write(
        path.join("README.md"),
        format!(
            r#"# {}

A Blinc ZRTL plugin.

## Building

### Dynamic (.zrtl)
```bash
blinc plugin build --mode dynamic
```

### Static
```bash
blinc plugin build --mode static
```

## Usage

Import in your Blinc application:

```blinc
import {} from "{}.zrtl"
```
"#,
            name, name, name
        ),
    )?;

    Ok(())
}

fn template_default(name: &str) -> String {
    format!(
        r#"// {} - Blinc Application
//
// A simple Blinc application with reactive state and animations.

@widget App {{
    @state count: i32 = 0

    @spring scale: f32 = 1.0 {{
        stiffness: 400
        damping: 30
    }}

    @machine button_state {{
        initial: idle

        idle -> hovered: pointer_enter
        hovered -> idle: pointer_leave
        hovered -> pressed: pointer_down
        pressed -> hovered: pointer_up
    }}

    @render {{
        Column {{
            spacing: 20
            align: center

            Text {{
                content: "Welcome to {}"
                font_size: 24
            }}

            Text {{
                content: "Count: {{count}}"
                font_size: 48
            }}

            Button {{
                label: "Increment"
                on_click: {{ count += 1 }}
                scale: scale
            }}
        }}
    }}
}}
"#,
        name, name
    )
}

fn template_minimal(name: &str) -> String {
    format!(
        r#"// {} - Minimal Blinc Application

@widget App {{
    @render {{
        Text {{
            content: "Hello, Blinc!"
        }}
    }}
}}
"#,
        name
    )
}

fn template_counter(name: &str) -> String {
    format!(
        r#"// {} - Counter Example
//
// Demonstrates reactive state and FSM-driven interactions.

@widget Counter {{
    @state count: i32 = 0

    @derived doubled: i32 = count * 2

    @machine state {{
        initial: idle

        idle -> active: pointer_enter
        active -> idle: pointer_leave
    }}

    @spring opacity: f32 = 1.0 {{
        stiffness: 300
        damping: 25
    }}

    @effect {{
        // Animate opacity based on state
        when state == active {{
            opacity = 1.0
        }} else {{
            opacity = 0.7
        }}
    }}

    @render {{
        Column {{
            spacing: 16
            padding: 24

            Row {{
                spacing: 12

                Button {{
                    label: "-"
                    on_click: {{ count -= 1 }}
                }}

                Text {{
                    content: "{{count}}"
                    font_size: 32
                    opacity: opacity
                }}

                Button {{
                    label: "+"
                    on_click: {{ count += 1 }}
                }}
            }}

            Text {{
                content: "Doubled: {{doubled}}"
                font_size: 14
                color: #666
            }}
        }}
    }}
}}

@widget App {{
    @render {{
        Center {{
            Counter {{}}
        }}
    }}
}}
"#,
        name
    )
}
