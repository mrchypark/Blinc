//! Tests for blinc_app API

use crate::app::BlincConfig;
use crate::prelude::*;

/// Create test app with sample_count=1 (no MSAA for simple texture targets)
fn create_test_app() -> BlincApp {
    BlincApp::with_config(BlincConfig {
        sample_count: 1,
        ..Default::default()
    })
    .expect("Failed to create test app")
}

/// Create a test texture for rendering (must match renderer's format)
fn create_test_texture(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Test Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

#[test]
fn test_app_creation() {
    let app = BlincApp::new();
    assert!(app.is_ok(), "BlincApp should initialize successfully");
}

#[test]
fn test_simple_render() {
    let mut app = create_test_app();

    let ui = div().w(100.0).h(100.0).bg(Color::RED);

    let (_, view) = create_test_texture(app.device(), 100, 100);
    let result = app.render(&ui, &view, 100.0, 100.0);

    assert!(result.is_ok(), "Simple render should succeed");
}

#[test]
fn test_nested_layout() {
    let mut app = create_test_app();

    let ui = div()
        .w(400.0)
        .h(300.0)
        .flex_col()
        .gap(4.0)
        .p(4.0)
        .child(div().h(50.0).bg(Color::RED))
        .child(div().flex_grow().bg(Color::GREEN))
        .child(div().h(50.0).bg(Color::BLUE));

    let (_, view) = create_test_texture(app.device(), 400, 300);
    let result = app.render(&ui, &view, 400.0, 300.0);

    assert!(result.is_ok(), "Nested layout render should succeed");
}

#[test]
fn test_text_element() {
    let mut app = create_test_app();

    let ui = div()
        .w(200.0)
        .h(100.0)
        .child(text("Hello Blinc!").size(24.0).color(Color::BLACK));

    let (_, view) = create_test_texture(app.device(), 200, 100);
    let result = app.render(&ui, &view, 200.0, 100.0);

    assert!(result.is_ok(), "Text element render should succeed");
}

#[test]
fn test_svg_element() {
    let mut app = create_test_app();

    let svg_source = r#"<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/></svg>"#;

    let ui = div()
        .w(100.0)
        .h(100.0)
        .child(svg(svg_source).size(48.0, 48.0));

    let (_, view) = create_test_texture(app.device(), 100, 100);
    let result = app.render(&ui, &view, 100.0, 100.0);

    assert!(result.is_ok(), "SVG element render should succeed");
}

#[test]
fn test_glass_effect() {
    let mut app = create_test_app();

    let ui = div()
        .w(400.0)
        .h(300.0)
        .child(div().w(100.0).h(100.0).bg(Color::RED))
        .child(
            div()
                .w(200.0)
                .h(150.0)
                .glass()
                .rounded(16.0)
                .child(text("Glass Panel").size(18.0).foreground()),
        );

    let (_, view) = create_test_texture(app.device(), 400, 300);
    let result = app.render(&ui, &view, 400.0, 300.0);

    assert!(result.is_ok(), "Glass effect render should succeed");
}

#[test]
fn test_flex_row_justify() {
    let mut app = create_test_app();

    let ui = div()
        .w(400.0)
        .h(100.0)
        .flex_row()
        .justify_between()
        .child(div().w(50.0).h(50.0).bg(Color::RED))
        .child(div().w(50.0).h(50.0).bg(Color::GREEN))
        .child(div().w(50.0).h(50.0).bg(Color::BLUE));

    let (_, view) = create_test_texture(app.device(), 400, 100);
    let result = app.render(&ui, &view, 400.0, 100.0);

    assert!(result.is_ok(), "Flex row with justify-between should render");
}

#[test]
fn test_render_tree_reuse() {
    let mut app = create_test_app();

    let ui = div().w(200.0).h(200.0).bg(Color::WHITE);

    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(200.0, 200.0);

    let (_, view) = create_test_texture(app.device(), 200, 200);

    for _ in 0..3 {
        let result = app.render_tree(&tree, &view, 200, 200);
        assert!(result.is_ok(), "Render tree reuse should work");
    }
}

#[test]
fn test_complex_ui() {
    let mut app = create_test_app();

    let ui = div()
        .w(400.0)
        .h(300.0)
        .flex_col()
        .p(4.0)
        .gap(4.0)
        .child(
            div()
                .flex_row()
                .justify_between()
                .items_center()
                .child(text("Now Playing").size(14.0))
                .child(div().square(24.0).rounded(12.0).bg(Color::GRAY)),
        )
        .child(div().flex_grow().rounded(8.0).bg(Color::rgba(0.2, 0.2, 0.2, 1.0)))
        .child(
            div()
                .h(60.0)
                .flex_row()
                .justify_center()
                .items_center()
                .gap(8.0)
                .child(div().square(32.0).rounded(16.0).bg(Color::WHITE))
                .child(div().square(48.0).rounded(24.0).bg(Color::WHITE))
                .child(div().square(32.0).rounded(16.0).bg(Color::WHITE)),
        );

    let (_, view) = create_test_texture(app.device(), 400, 300);
    let result = app.render(&ui, &view, 400.0, 300.0);

    assert!(result.is_ok(), "Complex UI should render successfully");
}
