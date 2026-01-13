//! Image Layer Test
//!
//! Tests the rendering order of images vs primitives (paths, backgrounds).
//! This helps debug z-order issues where images may render above/below other elements.
//!
//! **Solution for rendering elements ON TOP of images:**
//! Use `.foreground()` on any element that needs to render above images.
//! The render order is: Background primitives → Images → Foreground primitives
//!
//! Run with: cargo run -p blinc_app --example image_layer_test --features windowed

use blinc_app::prelude::*;
use blinc_app::windowed::{WindowedApp, WindowedContext};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = WindowConfig {
        title: "Image Layer Test".to_string(),
        width: 800,
        height: 600,
        resizable: true,
        ..Default::default()
    };

    WindowedApp::run(config, |ctx| build_ui(ctx))
}

fn build_ui(_ctx: &WindowedContext) -> impl ElementBuilder {
    let image_path = "crates/blinc_app/examples/assets/avatar.jpg";

    div()
        .w_full()
        .h_full()
        .bg(Color::rgb(0.1, 0.1, 0.15))
        .flex_col()
        .gap(8.0)
        .p(20.0)
        .child(
            text("Image Layer Test - Testing image vs primitive z-order")
                .size(24.0)
                .weight(FontWeight::Bold)
                .color(Color::WHITE),
        )
        .child(
            div()
                .flex_row()
                .gap(16.0)
                .child(test_case_1(image_path))
                .child(test_case_2(image_path))
                .child(test_case_3(image_path))
                .child(test_case_4(image_path)),
        )
        .child(
            div()
                .flex_row()
                .gap(16.0)
                .child(test_case_5(image_path))
                .child(test_case_6(image_path)),
        )
}

/// Test 1: Image with border on parent container
fn test_case_1(src: &str) -> impl ElementBuilder {
    div()
        .flex_col()
        .gap(4.0)
        .child(text("1: Border on parent").size(14.0).color(Color::WHITE))
        .child(
            div()
                .w(100.0)
                .h(100.0)
                .border(4.0, Color::RED)
                .rounded(8.0)
                .overflow_clip()
                .child(img(src).size(100.0, 100.0).cover()),
        )
        .child(
            text("Border should be visible around image")
                .size(11.0)
                .color(Color::rgba(1.0, 1.0, 1.0, 0.6)),
        )
}

/// Test 2: Image with sibling overlay div (after image)
fn test_case_2(src: &str) -> impl ElementBuilder {
    div()
        .flex_col()
        .gap(4.0)
        .child(text("2: Sibling overlay after").size(14.0).color(Color::WHITE))
        .child(
            div()
                .w(100.0)
                .h(100.0)
                .relative()
                .child(img(src).size(100.0, 100.0).cover())
                .child(
                    div()
                        .w(30.0)
                        .h(30.0)
                        .bg(Color::GREEN)
                        .rounded(15.0)
                        .absolute()
                        .bottom(4.0)
                        .right(4.0),
                ),
        )
        .child(
            text("Green circle should be ON TOP of image")
                .size(11.0)
                .color(Color::rgba(1.0, 1.0, 1.0, 0.6)),
        )
}

/// Test 3: Image with sibling overlay div using foreground layer
fn test_case_3(src: &str) -> impl ElementBuilder {
    div()
        .flex_col()
        .gap(4.0)
        .child(text("3: Overlay with .foreground()").size(14.0).color(Color::WHITE))
        .child(
            div()
                .w(100.0)
                .h(100.0)
                .relative()
                .child(img(src).size(100.0, 100.0).cover())
                .child(
                    div()
                        .w(30.0)
                        .h(30.0)
                        .bg(Color::BLUE)
                        .rounded(15.0)
                        .absolute()
                        .bottom(4.0)
                        .right(4.0)
                        .foreground(), // Use foreground layer
                ),
        )
        .child(
            text("Blue circle (.foreground) should be ON TOP")
                .size(11.0)
                .color(Color::rgba(1.0, 1.0, 1.0, 0.6)),
        )
}

/// Test 4: Stack with image first, overlay second
fn test_case_4(src: &str) -> impl ElementBuilder {
    div()
        .flex_col()
        .gap(4.0)
        .child(text("4: Stack (image first)").size(14.0).color(Color::WHITE))
        .child(
            stack()
                .w(100.0)
                .h(100.0)
                .child(img(src).size(100.0, 100.0).cover())
                .child(
                    div()
                        .w(30.0)
                        .h(30.0)
                        .bg(Color::YELLOW)
                        .rounded(15.0)
                        .absolute()
                        .bottom(4.0)
                        .right(4.0),
                ),
        )
        .child(
            text("Yellow circle should be ON TOP (stack order)")
                .size(11.0)
                .color(Color::rgba(1.0, 1.0, 1.0, 0.6)),
        )
}

/// Test 5: Plain div background under image
fn test_case_5(src: &str) -> impl ElementBuilder {
    div()
        .flex_col()
        .gap(4.0)
        .child(text("5: Bg div under image").size(14.0).color(Color::WHITE))
        .child(
            div()
                .w(100.0)
                .h(100.0)
                .bg(Color::MAGENTA)
                .rounded(8.0)
                .flex_row()
                .items_center()
                .justify_center()
                .child(img(src).size(80.0, 80.0).cover().rounded(8.0)),
        )
        .child(
            text("Magenta bg should show as border around image")
                .size(11.0)
                .color(Color::rgba(1.0, 1.0, 1.0, 0.6)),
        )
}

/// Test 6: Text over image (wrapped in div for positioning)
fn test_case_6(src: &str) -> impl ElementBuilder {
    div()
        .flex_col()
        .gap(4.0)
        .child(text("6: Text over image").size(14.0).color(Color::WHITE))
        .child(
            div()
                .w(100.0)
                .h(100.0)
                .relative()
                .child(img(src).size(100.0, 100.0).cover().rounded(8.0))
                .child(
                    div()
                        .absolute()
                        .bottom(8.0)
                        .left(8.0)
                        .child(
                            text("HELLO")
                                .size(16.0)
                                .weight(FontWeight::Bold)
                                .color(Color::WHITE),
                        ),
                ),
        )
        .child(
            text("Text should appear ON TOP of image")
                .size(11.0)
                .color(Color::rgba(1.0, 1.0, 1.0, 0.6)),
        )
}
