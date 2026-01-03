//! CSS Parser Demo
//!
//! Demonstrates the CSS parser with error collection and colored diagnostics.
//!
//! Run with: cargo run -p blinc_app --example css_parser_demo

use blinc_layout::prelude::{CssParseResult, Stylesheet};
use blinc_theme::ThemeState;

fn main() {
    // Initialize theme (required for theme() function in CSS)
    ThemeState::init_default();

    println!("\n=== CSS Parser Demo ===\n");

    // Example 1: Valid CSS
    println!("1. Parsing valid CSS:");
    println!("{}", "-".repeat(50));
    let valid_css = r#"
#card {
    background: #3498db;
    border-radius: 8px;
    opacity: 0.95;
}

#button-primary {
    background: theme(primary);
    transform: scale(1.0);
    border-radius: theme(radius-default);
}
"#;

    let result = Stylesheet::parse_with_errors(valid_css);
    print_result(&result, valid_css);

    // Example 2: CSS with unknown properties (warnings)
    println!("\n2. Parsing CSS with unknown properties:");
    println!("{}", "-".repeat(50));
    let css_with_unknowns = r#"
#card {
    background: #FF5733;
    margin: 10px;
    padding: 20px;
    opacity: 0.8;
    display: flex;
}
"#;

    let result = Stylesheet::parse_with_errors(css_with_unknowns);
    print_result(&result, css_with_unknowns);

    // Example 3: CSS with invalid values
    println!("\n3. Parsing CSS with invalid values:");
    println!("{}", "-".repeat(50));
    let css_with_invalid = r#"
#widget {
    opacity: not-a-number;
    border-radius: ???;
    background: red;
    transform: invalid(42);
}
"#;

    let result = Stylesheet::parse_with_errors(css_with_invalid);
    print_result(&result, css_with_invalid);

    // Example 4: Mixed valid and invalid
    println!("\n4. Parsing mixed valid/invalid CSS:");
    println!("{}", "-".repeat(50));
    let mixed_css = r#"
#header {
    background: theme(surface);
    border-radius: theme(radius-lg);
    box-shadow: theme(shadow-md);
}

#sidebar {
    background: rgba(255, 128, 0, 0.5);
    unknown-flex: center;
    opacity: 0.9;
}

#footer {
    background: #333333;
    render-layer: foreground;
}
"#;

    let result = Stylesheet::parse_with_errors(mixed_css);
    print_result(&result, mixed_css);

    // Example 5: Show error collection API
    println!("\n5. Error Collection API Demo:");
    println!("{}", "-".repeat(50));
    let css = "#test { unknown: x; opacity: bad; background: blue; weird-prop: y; }";
    let result = Stylesheet::parse_with_errors(css);

    println!("CSS: {}", css);
    println!("\nUsing CssParseResult API:");
    println!("  - has_errors(): {}", result.has_errors());
    println!("  - has_warnings(): {}", result.has_warnings());
    println!("  - errors_only().count(): {}", result.errors_only().count());
    println!("  - warnings_only().count(): {}", result.warnings_only().count());
    println!("  - stylesheet.len(): {}", result.stylesheet.len());

    if !result.errors.is_empty() {
        println!("\nCollected errors/warnings:");
        for (i, err) in result.errors.iter().enumerate() {
            println!("  [{}] {} (line {}, col {})",
                i + 1,
                err.severity,
                err.line,
                err.column
            );
            if let Some(ref prop) = err.property {
                println!("      property: {}", prop);
            }
            if let Some(ref val) = err.value {
                println!("      value: {}", val);
            }
            println!("      message: {}", err.message);
        }
    }

    println!("\n=== Demo Complete ===\n");
}

fn print_result(result: &CssParseResult, _css: &str) {
    // Print stylesheet info
    println!("Parsed {} rule(s):", result.stylesheet.len());
    for id in result.stylesheet.ids() {
        let style = result.stylesheet.get(id).unwrap();
        println!("  #{} => {:?}", id, style);
    }

    // Let the formatter handle colored diagnostics
    if !result.errors.is_empty() {
        println!();
    }
    result.print_colored_diagnostics();
    result.print_summary();
}
