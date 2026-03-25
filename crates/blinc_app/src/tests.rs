//! Visual tests for blinc_app API
//!
//! Tests render to PNG files in test_output/blinc_app/ for visual verification.
//! These tests require a GPU and will be skipped in CI environments without one.

use crate::app::BlincConfig;
use crate::prelude::*;
use blinc_core::{Brush, DrawContext, Path as BlincPath, Point, Rect, Stroke};
use blinc_gpu::GpuPaintContext;
use image::{ImageBuffer, Rgba, RgbaImage};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Test output directory
const OUTPUT_DIR: &str = "test_output/blinc_app";

/// Create test app for rendering tests
/// Returns None if no GPU adapter is available (e.g., in CI without GPU)
fn create_test_app() -> Option<BlincApp> {
    match BlincApp::with_config(BlincConfig {
        sample_count: 1, // SDF handles AA for most elements
        ..Default::default()
    }) {
        Ok(app) => Some(app),
        Err(e) => {
            eprintln!("Skipping test: no GPU available ({e})");
            None
        }
    }
}

/// Macro to skip test if no GPU is available
macro_rules! require_gpu {
    ($app:ident) => {
        let Some(mut $app) = create_test_app() else {
            return; // Skip test if no GPU
        };
    };
}

/// Create a test texture for rendering (must match renderer's format)
fn create_test_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
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

/// Padded bytes per row for wgpu buffer alignment
fn padded_bytes_per_row(width: u32) -> u32 {
    let unpadded = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    unpadded.div_ceil(align) * align
}

/// Read back a rendered texture into an RGBA image (BGRA->RGBA conversion).
fn read_to_rgba_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> RgbaImage {
    let bytes_per_row = padded_bytes_per_row(width);
    let buffer_size = (bytes_per_row * height) as u64;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Readback Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Copy Encoder"),
    });

    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(std::iter::once(encoder.finish()));

    let buffer_slice = buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv().unwrap().expect("Failed to map buffer");

    let data = buffer_slice.get_mapped_range();

    // Create image (convert BGRA to RGBA)
    let mut img: RgbaImage = ImageBuffer::new(width, height);
    for y in 0..height {
        let row_start = (y * bytes_per_row) as usize;
        let row_end = row_start + (width * 4) as usize;
        let row_data = &data[row_start..row_end];

        for x in 0..width {
            let i = (x * 4) as usize;
            // BGRA -> RGBA
            img.put_pixel(
                x,
                y,
                Rgba([
                    row_data[i + 2],
                    row_data[i + 1],
                    row_data[i],
                    row_data[i + 3],
                ]),
            );
        }
    }

    drop(data);
    buffer.unmap();

    img
}

/// Save a rendered texture to PNG
fn save_to_png(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
    path: &Path,
) {
    let img = read_to_rgba_image(device, queue, texture, width, height);

    // Ensure output directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    img.save(path).expect("Failed to save PNG");
}

/// Render a UI element and save to PNG
fn render_to_png(
    app: &mut BlincApp,
    name: &str,
    ui: &impl ElementBuilder,
    width: u32,
    height: u32,
) {
    let (texture, view) = create_test_texture(app.device(), width, height);
    app.render(ui, &view, width as f32, height as f32)
        .expect("Render failed");

    let path = Path::new(OUTPUT_DIR).join(format!("{}.png", name));
    save_to_png(app.device(), app.queue(), &texture, width, height, &path);
    println!("Saved: {:?}", path);
}

/// Render a UI element and return the rendered pixels (RGBA).
fn render_to_image(
    app: &mut BlincApp,
    ui: &impl ElementBuilder,
    width: u32,
    height: u32,
) -> RgbaImage {
    let (texture, view) = create_test_texture(app.device(), width, height);
    app.render(ui, &view, width as f32, height as f32)
        .expect("Render failed");
    read_to_rgba_image(app.device(), app.queue(), &texture, width, height)
}

/// Render a UI element via the motion render path and return rendered pixels (RGBA).
fn render_to_image_with_motion(
    app: &mut BlincApp,
    ui: &impl ElementBuilder,
    width: u32,
    height: u32,
) -> RgbaImage {
    let (texture, view) = create_test_texture(app.device(), width, height);
    let mut tree = RenderTree::from_element(ui);
    tree.compute_layout(width as f32, height as f32);

    let scheduler = Arc::new(Mutex::new(blinc_animation::AnimationScheduler::new()));
    let render_state = blinc_layout::RenderState::new(scheduler);

    app.render_tree_with_motion(&tree, &render_state, &view, width, height)
        .expect("Motion render failed");
    read_to_rgba_image(app.device(), app.queue(), &texture, width, height)
}

#[test]
fn test_simple_red_box() {
    require_gpu!(app);
    let ui = div().w(200.0).h(200.0).bg(Color::RED);
    render_to_png(&mut app, "simple_red_box", &ui, 200, 200);
}

#[test]
fn test_nested_boxes() {
    require_gpu!(app);

    let ui = div()
        .w(400.0)
        .h(300.0)
        .flex_col()
        .gap(4.0)
        .p(4.0)
        .bg(Color::rgba(0.1, 0.1, 0.15, 1.0))
        .child(div().h(80.0).w_full().rounded(8.0).bg(Color::RED))
        .child(div().flex_grow().w_full().rounded(8.0).bg(Color::GREEN))
        .child(div().h(80.0).w_full().rounded(8.0).bg(Color::BLUE));

    render_to_png(&mut app, "nested_boxes", &ui, 400, 300);
}

#[test]
fn test_text_element() {
    require_gpu!(app);

    let ui = div()
        .w(400.0)
        .h(200.0)
        .flex_col()
        .items_center()
        .justify_center()
        .bg(Color::WHITE)
        .child(text("Hello Blinc!").size(32.0).color(Color::BLACK));

    render_to_png(&mut app, "text_element", &ui, 400, 200);
}

#[test]
fn test_svg_icon() {
    require_gpu!(app);

    let svg_source = r##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="#3B82F6"/></svg>"##;

    let ui = div()
        .w(200.0)
        .h(200.0)
        .flex_col()
        .items_center()
        .justify_center()
        .bg(Color::rgba(0.1, 0.1, 0.15, 1.0))
        .child(svg(svg_source).size(100.0, 100.0));

    render_to_png(&mut app, "svg_icon", &ui, 200, 200);
}

#[test]
fn test_stack_z_layer_renders_paths_above_layer_primitives() {
    require_gpu!(app);

    let (width, height) = (220u32, 180u32);
    let line_path = BlincPath::new().move_to(20.0, 160.0).line_to(200.0, 20.0);
    let stroke = Stroke::new(4.0);

    // Stack child #1 (z=1) draws a covering primitive, then a stroked path.
    // Regression test for the interleaved z-layer renderer: paths must respect z-layer.
    let ui = stack()
        .w(width as f32)
        .h(height as f32)
        .child(
            div()
                .w_full()
                .h_full()
                .bg(Color::rgba(0.08, 0.09, 0.12, 1.0)),
        )
        .child(
            div()
                .w_full()
                .h_full()
                .bg(Color::rgba(0.02, 0.02, 0.02, 1.0))
                .child(
                    canvas(move |ctx: &mut dyn DrawContext, _bounds| {
                        ctx.stroke_path(
                            &line_path,
                            &stroke,
                            Brush::Solid(Color::rgb(0.2, 0.6, 1.0)),
                        );
                    })
                    .w(width as f32)
                    .h(height as f32),
                ),
        );

    let img = render_to_image_with_motion(&mut app, &ui, width, height);

    // Heuristic: the stroked path must contribute noticeably bright blue pixels.
    // Backgrounds are near-black, so `max_b` is a strong signal.
    let mut max_b = 0u8;
    let mut bright_blueish = 0usize;
    for p in img.pixels() {
        let [r, g, b, a] = p.0;
        if a == 0 {
            continue;
        }
        max_b = max_b.max(b);
        if b > 150 && g > 60 && r < 160 {
            bright_blueish += 1;
        }
    }
    assert!(
        max_b > 150 && bright_blueish > 0,
        "expected bright blue pixels from stroked path; max_b={max_b} bright_blueish={bright_blueish}"
    );
}

#[test]
fn test_tessellated_path_stroke_is_visible() {
    require_gpu!(app);

    // Force the polyline to fall back to tessellated path rendering by using a rounded clip.
    // This catches regressions where the path pipeline silently stops drawing.
    let ui = div()
        .w(240.0)
        .h(180.0)
        .rounded(16.0)
        .overflow_clip()
        .bg(Color::rgba(0.08, 0.09, 0.11, 1.0))
        .child(
            canvas(|ctx: &mut dyn DrawContext, bounds| {
                // A diagonal line that should be clearly visible.
                let pts = [
                    Point::new(20.0, bounds.height - 20.0),
                    Point::new(bounds.width - 20.0, 20.0),
                ];
                ctx.stroke_polyline(
                    &pts,
                    &Stroke::new(3.0),
                    Brush::Solid(Color::rgba(0.35, 0.65, 1.0, 1.0)),
                );
            })
            .w_full()
            .h_full(),
        );

    // Optional artifact for local debugging.
    if std::env::var_os("BLINC_TEST_WRITE_ARTIFACTS").is_some() {
        render_to_png(&mut app, "debug_canvas_polyline", &ui, 240, 180);
    }

    let img = render_to_image(&mut app, &ui, 240, 180);

    // Count "blue-ish" pixels. Thresholds are conservative to be robust to AA.
    let mut blueish = 0usize;
    for p in img.pixels() {
        let [r, g, b, a] = p.0;
        if a > 32 && b > 160 && g > 150 && r < 180 {
            blueish += 1;
        }
    }

    assert!(
        blueish > 50,
        "expected tessellated stroke to produce visible blue pixels; blueish={blueish}"
    );
}

#[test]
fn test_foreground_tessellated_path_stroke_is_visible() {
    require_gpu!(app);

    let ui = div()
        .w(240.0)
        .h(180.0)
        .rounded(16.0)
        .overflow_clip()
        .bg(Color::rgba(0.08, 0.09, 0.11, 1.0))
        .child(
            canvas(|ctx: &mut dyn DrawContext, bounds| {
                // Force foreground-path codepath.
                ctx.set_foreground_layer(true);

                let pts = [
                    Point::new(20.0, bounds.height - 20.0),
                    Point::new(bounds.width - 20.0, 20.0),
                ];
                ctx.stroke_polyline(
                    &pts,
                    &Stroke::new(3.0),
                    Brush::Solid(Color::rgba(0.35, 0.65, 1.0, 1.0)),
                );

                ctx.set_foreground_layer(false);
            })
            .w_full()
            .h_full(),
        );

    // Optional artifact for local debugging.
    if std::env::var_os("BLINC_TEST_WRITE_ARTIFACTS").is_some() {
        render_to_png(&mut app, "debug_foreground_canvas_polyline", &ui, 240, 180);
    }

    if std::env::var_os("BLINC_DEBUG_TEST").is_some() {
        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(240.0, 180.0);
        let mut ctx = GpuPaintContext::new(240.0, 180.0);
        tree.render_to_layer(&mut ctx, RenderLayer::Background);
        let batch = ctx.take_batch();
        println!(
            "debug batch: prims={} lines={} paths(v/i)={}/{} fg_paths(v/i)={}/{}",
            batch.primitives.len(),
            batch.line_segments.len(),
            batch.paths.vertices.len(),
            batch.paths.indices.len(),
            batch.foreground_paths.vertices.len(),
            batch.foreground_paths.indices.len()
        );
        println!(
            "debug fg_paths flags: use_grad={} use_img={} use_glass={} image_uv={:?} glass_params={:?} glass_tint={:?}",
            batch.foreground_paths.use_gradient_texture,
            batch.foreground_paths.use_image_texture,
            batch.foreground_paths.use_glass_effect,
            batch.foreground_paths.image_uv_bounds,
            batch.foreground_paths.glass_params,
            batch.foreground_paths.glass_tint,
        );
        if let Some(d) = batch.foreground_paths.draws.first() {
            println!(
                "debug fg_paths draw0: start={} count={} clip_bounds={:?} clip_radius={:?} clip_type={}",
                d.index_start, d.index_count, d.clip_bounds, d.clip_radius, d.clip_type
            );
        }
        if let Some(v) = batch.foreground_paths.vertices.first() {
            println!(
                "debug fg_path vertex0: pos={:?} color={:?} end_color={:?} grad_type={}",
                v.position, v.color, v.end_color, v.gradient_type
            );
        }
    }

    let img = render_to_image(&mut app, &ui, 240, 180);

    let mut blueish = 0usize;
    let mut max_r = 0u8;
    let mut max_g = 0u8;
    let mut max_b = 0u8;
    let mut b_hi = 0usize;
    for p in img.pixels() {
        let [r, g, b, a] = p.0;
        max_r = max_r.max(r);
        max_g = max_g.max(g);
        max_b = max_b.max(b);
        if a > 32 && b > 120 {
            b_hi += 1;
        }
        // NOTE: render target is sRGB, so linear colors are gamma-encoded in the readback.
        // Use conservative thresholds that tolerate AA and gamma.
        if a > 32 && b > 160 && g > 150 && r < 200 {
            blueish += 1;
        }
    }

    if std::env::var_os("BLINC_DEBUG_TEST").is_some() {
        println!(
            "debug pixels: max_r={} max_g={} max_b={} b_hi={} blueish={}",
            max_r, max_g, max_b, b_hi, blueish
        );
    }

    assert!(
        blueish > 50,
        "expected foreground tessellated stroke to produce visible blue pixels; blueish={blueish}"
    );
}

#[test]
fn test_foreground_tessellated_path_stroke_is_visible_no_msaa() {
    // Isolate the non-MSAA render path (render_with_clear + foreground paths).
    // `create_test_app` uses sample_count=1 and skips cleanly when no adapter exists.
    require_gpu!(app);

    let ui = div()
        .w(240.0)
        .h(180.0)
        .rounded(16.0)
        .overflow_clip()
        .bg(Color::rgba(0.08, 0.09, 0.11, 1.0))
        .child(
            canvas(|ctx: &mut dyn DrawContext, bounds| {
                ctx.set_foreground_layer(true);

                let pts = [
                    Point::new(20.0, bounds.height - 20.0),
                    Point::new(bounds.width - 20.0, 20.0),
                ];
                ctx.stroke_polyline(
                    &pts,
                    &Stroke::new(3.0),
                    Brush::Solid(Color::rgba(0.35, 0.65, 1.0, 1.0)),
                );

                ctx.set_foreground_layer(false);
            })
            .w_full()
            .h_full(),
        );

    if std::env::var_os("BLINC_TEST_WRITE_ARTIFACTS").is_some() {
        render_to_png(
            &mut app,
            "debug_foreground_canvas_polyline_no_msaa",
            &ui,
            240,
            180,
        );
    }

    let img = render_to_image(&mut app, &ui, 240, 180);

    let mut blueish = 0usize;
    let mut b_hi = 0usize;
    let mut max_b = 0u8;
    for p in img.pixels() {
        let [r, g, b, a] = p.0;
        max_b = max_b.max(b);
        if a > 32 && b > 120 {
            b_hi += 1;
        }
        // Keep thresholds tolerant to backend/color-space differences under coverage.
        if a > 32 && b > 145 && g > 110 && r < 220 {
            blueish += 1;
        }
    }

    assert!(
        max_b > 150 && (blueish > 20 || b_hi > 80),
        "expected foreground tessellated stroke (no MSAA) to produce visible blue pixels; max_b={max_b} blueish={blueish} b_hi={b_hi}"
    );
}

#[test]
fn test_compact_polyline_is_visible() {
    require_gpu!(app);

    // No rounded clip: should take the compact line-segment path.
    let ui = div()
        .w(240.0)
        .h(180.0)
        .bg(Color::rgba(0.08, 0.09, 0.11, 1.0))
        .child(
            canvas(|ctx: &mut dyn DrawContext, bounds| {
                // Sanity check: SDF primitives should render in the same canvas.
                ctx.fill_rect(
                    Rect::new(8.0, 8.0, 16.0, 16.0),
                    0.0.into(),
                    Brush::Solid(Color::rgba(0.95, 0.2, 0.2, 1.0)),
                );

                let pts = [
                    Point::new(20.0, bounds.height - 20.0),
                    Point::new(bounds.width - 20.0, 20.0),
                ];
                ctx.stroke_polyline(
                    &pts,
                    &Stroke::new(3.0),
                    Brush::Solid(Color::rgba(0.35, 0.65, 1.0, 1.0)),
                );
            })
            .w_full()
            .h_full(),
        );

    let img = render_to_image(&mut app, &ui, 240, 180);

    // Confirm the red square exists (catch any readback/format issues early).
    let mut reddish = 0usize;
    for p in img.pixels() {
        let [r, g, b, a] = p.0;
        if a > 32 && r > 200 && g < 160 && b < 160 {
            reddish += 1;
        }
    }
    assert!(
        reddish > 50,
        "expected red sanity pixels; reddish={reddish}"
    );

    let mut blueish = 0usize;
    for p in img.pixels() {
        let [r, g, b, a] = p.0;
        if a > 32 && b > 160 && g > 150 && r < 180 {
            blueish += 1;
        }
    }
    assert!(
        blueish > 50,
        "expected compact polyline to produce visible blue pixels; blueish={blueish}"
    );
}

#[test]
fn test_foreground_compact_polyline_is_visible() {
    require_gpu!(app);

    // No rounded clip: should take the compact line-segment path, but recorded to
    // `foreground_line_segments` via `set_foreground_layer(true)`.
    let ui = div()
        .w(240.0)
        .h(180.0)
        .bg(Color::rgba(0.08, 0.09, 0.11, 1.0))
        .child(
            canvas(|ctx: &mut dyn DrawContext, bounds| {
                // Sanity check: SDF primitives should render in the same canvas.
                ctx.fill_rect(
                    Rect::new(8.0, 8.0, 16.0, 16.0),
                    0.0.into(),
                    Brush::Solid(Color::rgba(0.95, 0.2, 0.2, 1.0)),
                );

                ctx.set_foreground_layer(true);
                let pts = [
                    Point::new(20.0, bounds.height - 20.0),
                    Point::new(bounds.width - 20.0, 20.0),
                ];
                ctx.stroke_polyline(
                    &pts,
                    &Stroke::new(3.0),
                    Brush::Solid(Color::rgba(0.35, 0.65, 1.0, 1.0)),
                );
                ctx.set_foreground_layer(false);
            })
            .w_full()
            .h_full(),
        );

    if std::env::var_os("BLINC_TEST_WRITE_ARTIFACTS").is_some() {
        render_to_png(&mut app, "debug_foreground_compact_polyline", &ui, 240, 180);
    }

    let img = render_to_image(&mut app, &ui, 240, 180);

    // Confirm the red square exists (catch any readback/format issues early).
    let mut reddish = 0usize;
    for p in img.pixels() {
        let [r, g, b, a] = p.0;
        if a > 32 && r > 200 && g < 160 && b < 160 {
            reddish += 1;
        }
    }
    assert!(
        reddish > 50,
        "expected red sanity pixels; reddish={reddish}"
    );

    let mut blueish = 0usize;
    for p in img.pixels() {
        let [r, g, b, a] = p.0;
        if a > 32 && b > 160 && g > 150 && r < 200 {
            blueish += 1;
        }
    }
    assert!(
        blueish > 50,
        "expected foreground compact polyline to produce visible blue pixels; blueish={blueish}"
    );
}

#[test]
fn test_glass_panel() {
    require_gpu!(app);

    let ui = div()
        .w(400.0)
        .h(300.0)
        .bg(Color::rgba(0.2, 0.1, 0.4, 1.0))
        // Background blob
        .child(
            div()
                .absolute()
                .w(150.0)
                .h(150.0)
                .rounded(75.0)
                .bg(Color::rgba(0.95, 0.3, 0.5, 1.0)),
        )
        // Another background blob
        .child(
            div()
                .absolute()
                .mt(4.0)
                .ml(50.0)
                .w(120.0)
                .h(120.0)
                .rounded(60.0)
                .bg(Color::rgba(0.3, 0.8, 0.6, 1.0)),
        )
        // Glass card
        .child(
            div()
                .w(280.0)
                .h(180.0)
                .m(4.0)
                .rounded(20.0)
                .p(4.0)
                .flex_col()
                .gap(2.0)
                .effect(
                    GlassMaterial::new()
                        .blur(25.0)
                        .tint_rgba(0.95, 0.95, 0.98, 0.5)
                        .border(1.0),
                )
                .child(
                    div()
                        .w(200.0)
                        .h(20.0)
                        .rounded(4.0)
                        .bg(Color::rgba(1.0, 1.0, 1.0, 0.8)),
                )
                .child(
                    div()
                        .w(140.0)
                        .h(14.0)
                        .rounded(3.0)
                        .bg(Color::rgba(1.0, 1.0, 1.0, 0.5)),
                )
                .child(
                    div()
                        .flex_grow()
                        .w_full()
                        .rounded(8.0)
                        .bg(Color::rgba(1.0, 1.0, 1.0, 0.15)),
                ),
        );

    render_to_png(&mut app, "glass_panel", &ui, 400, 300);
}

#[test]
fn test_flex_row_justify() {
    require_gpu!(app);

    let ui = div()
        .w(400.0)
        .h(100.0)
        .flex_row()
        .justify_between()
        .items_center()
        .p(4.0)
        .bg(Color::rgba(0.15, 0.15, 0.2, 1.0))
        .child(div().w(60.0).h(60.0).rounded(8.0).bg(Color::RED))
        .child(div().w(60.0).h(60.0).rounded(8.0).bg(Color::GREEN))
        .child(div().w(60.0).h(60.0).rounded(8.0).bg(Color::BLUE));

    render_to_png(&mut app, "flex_row_justify", &ui, 400, 100);
}

#[test]
fn test_card_component() {
    require_gpu!(app);

    let card = div()
        .w(300.0)
        .h(200.0)
        .p(4.0)
        .rounded(16.0)
        .bg(Color::WHITE)
        .flex_col()
        .gap(3.0)
        // Header row
        .child(
            div()
                .w_full()
                .h(48.0)
                .flex_row()
                .gap(3.0)
                .items_center()
                // Avatar
                .child(
                    div()
                        .w(48.0)
                        .h(48.0)
                        .rounded(24.0)
                        .bg(Color::rgba(0.3, 0.5, 0.9, 1.0)),
                )
                // Title area
                .child(
                    div()
                        .flex_grow()
                        .h(48.0)
                        .flex_col()
                        .gap(1.0)
                        .justify_center()
                        .child(
                            div()
                                .w(120.0)
                                .h(14.0)
                                .rounded(3.0)
                                .bg(Color::rgba(0.2, 0.2, 0.25, 1.0)),
                        )
                        .child(
                            div()
                                .w(80.0)
                                .h(10.0)
                                .rounded(2.0)
                                .bg(Color::rgba(0.6, 0.6, 0.65, 1.0)),
                        ),
                ),
        )
        // Content area
        .child(
            div()
                .w_full()
                .flex_grow()
                .rounded(8.0)
                .bg(Color::rgba(0.95, 0.95, 0.97, 1.0)),
        )
        // Button row
        .child(
            div()
                .w_full()
                .h(36.0)
                .flex_row()
                .justify_end()
                .gap(2.0)
                .child(
                    div()
                        .w(80.0)
                        .h(36.0)
                        .rounded(8.0)
                        .bg(Color::rgba(0.9, 0.9, 0.92, 1.0)),
                )
                .child(
                    div()
                        .w(80.0)
                        .h(36.0)
                        .rounded(8.0)
                        .bg(Color::rgba(0.3, 0.5, 0.9, 1.0)),
                ),
        );

    render_to_png(&mut app, "card_component", &card, 300, 200);
}

#[test]
fn test_music_player() {
    require_gpu!(app);
    let scale = 2.0;

    // SVG icons
    let rewind_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 640"><path d="M236.3 107.1C247.9 96 265 92.9 279.7 99.2C294.4 105.5 304 120 304 136L304 272.3L476.3 107.2C487.9 96 505 92.9 519.7 99.2C534.4 105.5 544 120 544 136L544 504C544 520 534.4 534.5 519.7 540.8C505 547.1 487.9 544 476.3 532.9L304 367.7L304 504C304 520 294.4 534.5 279.7 540.8C265 547.1 247.9 544 236.3 532.9L44.3 348.9C36.4 341.4 32 330.9 32 320C32 309.1 36.5 298.7 44.3 291.1L236.3 107.1z" fill="white"/></svg>"#;
    let pause_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 640"><path d="M176 96C149.5 96 128 117.5 128 144L128 496C128 522.5 149.5 544 176 544L240 544C266.5 544 288 522.5 288 496L288 144C288 117.5 266.5 96 240 96L176 96zM400 96C373.5 96 352 117.5 352 144L352 496C352 522.5 373.5 544 400 544L464 544C490.5 544 512 522.5 512 496L512 144C512 117.5 490.5 96 464 96L400 96z" fill="white"/></svg>"#;
    let forward_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 640"><path d="M403.7 107.1C392.1 96 375 92.9 360.3 99.2C345.6 105.5 336 120 336 136L336 272.3L163.7 107.2C152.1 96 135 92.9 120.3 99.2C105.6 105.5 96 120 96 136L96 504C96 520 105.6 534.5 120.3 540.8C135 547.1 152.1 544 163.7 532.9L336 367.7L336 504C336 520 345.6 534.5 360.3 540.8C375 547.1 392.1 544 403.7 532.9L595.7 348.9C603.6 341.4 608 330.9 608 320C608 309.1 603.5 298.7 595.7 291.1L403.7 107.1z" fill="white"/></svg>"#;

    let bar_h = 7.0 * scale;
    let icon_size = 32.0 * scale;

    let ui = div()
        .w(400.0 * scale)
        .h(300.0 * scale)
        .bg(Color::rgba(0.4, 0.2, 0.6, 1.0))
        // Background blobs
        .child(
            div()
                .absolute()
                .w(200.0 * scale)
                .h(200.0 * scale)
                .rounded(100.0 * scale)
                .bg(Color::rgba(0.95, 0.3, 0.5, 1.0)),
        )
        .child(
            div()
                .absolute()
                .ml(50.0)
                .mt(30.0)
                .w(180.0 * scale)
                .h(180.0 * scale)
                .rounded(90.0 * scale)
                .bg(Color::rgba(0.2, 0.8, 0.85, 1.0)),
        )
        // Player card
        .child(
            div()
                .w(340.0 * scale)
                .h(140.0 * scale)
                .m(7.0)
                .rounded(28.0 * scale)
                .flex_col()
                .p(5.0)
                .gap(2.0)
                .effect(
                    GlassMaterial::new()
                        .blur(30.0 * scale)
                        .tint_rgba(0.12, 0.12, 0.14, 0.55)
                        .saturation(0.85)
                        .border(0.6 * scale),
                )
                // Title
                .child(
                    div()
                        .w_full()
                        .h(20.0 * scale)
                        .flex_row()
                        .justify_center()
                        .items_center()
                        .child(
                            text("Blinc UI 0.1.0")
                                .size(14.0 * scale)
                                .color(Color::rgba(1.0, 1.0, 1.0, 0.95)),
                        ),
                )
                // Progress bar
                .child(
                    div()
                        .w_full()
                        .h(bar_h + 8.0 * scale)
                        .flex_row()
                        .items_center()
                        .gap(2.0)
                        .child(
                            div()
                                .w(35.0 * scale)
                                .flex_row()
                                .justify_end()
                                .items_center()
                                .child(
                                    text("0:10")
                                        .size(11.0 * scale)
                                        .color(Color::rgba(1.0, 1.0, 1.0, 0.85)),
                                ),
                        )
                        .child(
                            div()
                                .flex_grow()
                                .h(bar_h)
                                .rounded(bar_h / 2.0)
                                .effect(
                                    GlassMaterial::new()
                                        .blur(25.0 * scale)
                                        .tint_rgba(1.0, 1.0, 1.0, 0.65)
                                        .border(0.0),
                                )
                                .child(
                                    div()
                                        .w(20.0 * scale)
                                        .h_full()
                                        .rounded(bar_h / 2.0)
                                        .bg(Color::WHITE),
                                ),
                        )
                        .child(
                            div()
                                .w(40.0 * scale)
                                .flex_row()
                                .justify_start()
                                .items_center()
                                .child(
                                    text("-3:24")
                                        .size(11.0 * scale)
                                        .color(Color::rgba(1.0, 1.0, 1.0, 0.85)),
                                ),
                        ),
                )
                // Controls
                .child(
                    div()
                        .w_full()
                        .flex_grow()
                        .flex_row()
                        .justify_center()
                        .items_center()
                        .gap(10.0)
                        .child(svg(rewind_svg).square(icon_size))
                        .child(svg(pause_svg).square(icon_size))
                        .child(svg(forward_svg).square(icon_size)),
                ),
        );

    render_to_png(
        &mut app,
        "music_player",
        &ui,
        (400.0 * scale) as u32,
        (300.0 * scale) as u32,
    );
}

#[test]
fn test_render_tree_reuse() {
    require_gpu!(app);

    let ui = div()
        .w(200.0)
        .h(200.0)
        .flex_col()
        .gap(2.0)
        .p(2.0)
        .bg(Color::WHITE)
        .child(div().flex_grow().w_full().rounded(8.0).bg(Color::RED))
        .child(div().flex_grow().w_full().rounded(8.0).bg(Color::GREEN))
        .child(div().flex_grow().w_full().rounded(8.0).bg(Color::BLUE));

    let mut tree = RenderTree::from_element(&ui);
    tree.compute_layout(200.0, 200.0);

    let (texture, view) = create_test_texture(app.device(), 200, 200);

    // Render the same tree 3 times
    for _i in 0..3 {
        app.render_tree(&tree, &view, 200, 200)
            .expect("Render failed");
    }

    let path = Path::new(OUTPUT_DIR).join("render_tree_reuse.png");
    save_to_png(app.device(), app.queue(), &texture, 200, 200, &path);
    println!("Saved: {:?}", path);
}

#[test]
fn headless_runtime_runs_fixed_frame_budget() {
    use crate::headless_runtime::{HeadlessRunConfig, HeadlessRuntime};

    let mut frames = 0u32;
    let cfg = HeadlessRunConfig {
        width: 800,
        height: 600,
        max_frames: 3,
        tick_ms: 16,
        probe_every_frames: 1,
    };

    HeadlessRuntime::run(cfg, |_ctx| {
        frames += 1;
    })
    .expect("headless run should succeed");

    assert_eq!(frames, 3);
}

#[test]
fn parses_wait_and_assert_steps() {
    use crate::headless_scenario::{HeadlessScenario, ScenarioStep};

    let json = r#"{
      "steps": [
        {"type":"wait","ms":100},
        {"type":"assert_exists","id":"login.button"}
      ]
    }"#;

    let scenario = HeadlessScenario::from_json(json).expect("scenario should parse");
    assert!(matches!(scenario.steps[0], ScenarioStep::Wait { ms: 100 }));
    assert!(matches!(
        scenario.steps[1],
        ScenarioStep::AssertExists { ref target }
            if target.id.as_deref() == Some("login.button")
    ));
}

#[test]
fn scenario_parses_click_fill_and_press_steps() {
    use crate::headless_scenario::{HeadlessScenario, ScenarioStep};

    let json = r#"{
      "steps": [
        {"type":"click","id":"login.button"},
        {"type":"fill","id":"login.email","value":"person@example.com"},
        {"type":"press","key":"Enter"}
      ]
    }"#;

    let scenario = HeadlessScenario::from_json(json).expect("scenario should parse");
    assert!(matches!(
        scenario.steps[0],
        ScenarioStep::Click {
            ref target,
            x: None,
            y: None
        } if target.id.as_deref() == Some("login.button")
    ));
    assert!(matches!(
        scenario.steps[1],
        ScenarioStep::Fill { ref target, ref value }
            if target.id.as_deref() == Some("login.email") && value == "person@example.com"
    ));
    assert!(matches!(
        scenario.steps[2],
        ScenarioStep::Press { ref key } if key == "Enter"
    ));
}

#[test]
fn scenario_parses_coordinate_click_steps() {
    use crate::headless_scenario::{HeadlessScenario, ScenarioStep};

    let json = r#"{
      "steps": [
        {"type":"click","x":24.0,"y":48.0}
      ]
    }"#;

    let scenario =
        HeadlessScenario::from_json(json).expect("coordinate click scenario should parse");
    assert!(matches!(
        scenario.steps[0],
        ScenarioStep::Click {
            ref target,
            x: Some(x),
            y: Some(y)
        } if target.is_empty() && (x - 24.0).abs() < f32::EPSILON && (y - 48.0).abs() < f32::EPSILON
    ));
}

#[test]
fn scenario_parses_semantic_locator_steps() {
    use crate::headless_scenario::{HeadlessScenario, ScenarioStep};

    let json = r#"{
      "steps": [
        {"type":"click","role":"button","label":"Increment"},
        {"type":"assert_text_contains","role":"label","text":"Count","value":"Count: 1"}
      ]
    }"#;

    let scenario =
        HeadlessScenario::from_json(json).expect("semantic locator scenario should parse");
    assert!(matches!(
        scenario.steps[0],
        ScenarioStep::Click {
            ref target,
            x: None,
            y: None
        }
            if target.id.is_none()
                && target.semantic.role.as_deref() == Some("button")
                && target.semantic.label.as_deref() == Some("Increment")
    ));
    assert!(matches!(
        scenario.steps[1],
        ScenarioStep::AssertTextContains { ref target, ref value }
            if target.semantic.role.as_deref() == Some("label")
                && target.semantic.text.as_deref() == Some("Count")
                && value == "Count: 1"
    ));
}

#[test]
fn headless_runner_returns_structured_failure_for_unhandled_action_step() {
    use crate::headless_runner::{run_scenario, RunOutcome};

    let scenario_json = r#"{
      "steps": [
        {"type":"click","id":"login.button"}
      ]
    }"#;

    let outcome = run_scenario(scenario_json).expect("runner should return outcome");
    match outcome {
        RunOutcome::Failed { report } => {
            assert_eq!(report.failed_step_index, Some(0));
            assert_eq!(
                report.assertion,
                Some("unsupported_action_step".to_string())
            );
            assert!(
                report
                    .message
                    .as_deref()
                    .is_some_and(|message| message.contains("click")),
                "unexpected message: {:?}",
                report.message
            );
        }
        other => panic!("expected structured failure, got {other:?}"),
    }
}

#[test]
fn assert_text_contains_reports_failure_detail() {
    use crate::headless_assert::{
        evaluate_assert_text_contains, DiagnosticsElement, DiagnosticsSnapshot,
    };

    let mut snapshot = DiagnosticsSnapshot::default();
    snapshot.elements.insert(
        "title".to_string(),
        DiagnosticsElement {
            text: Some("Hello".to_string()),
        },
    );

    let result = evaluate_assert_text_contains("title", "Welcome", &snapshot);
    assert!(matches!(
        result,
        crate::headless_assert::AssertionResult::Failed { .. }
    ));
}

#[test]
fn headless_asserts_can_read_text_from_tree_snapshot_backed_probe() {
    use crate::headless_assert::{
        evaluate_assert_text_contains, AssertionResult, DiagnosticsSnapshot,
    };
    use blinc_recorder::{ElementSnapshot, Rect, Timestamp, TreeSnapshot};

    let mut tree = TreeSnapshot::new(Timestamp::from_micros(10), (1280, 720), 1.0);
    let mut title = ElementSnapshot::new(
        "title".to_string(),
        "Text".to_string(),
        Rect::new(0.0, 0.0, 100.0, 20.0),
    );
    title.text_content = Some("Welcome back".to_string());
    tree.elements.insert("title".to_string(), title);

    let snapshot = DiagnosticsSnapshot::from(tree);
    let result = evaluate_assert_text_contains("title", "Welcome", &snapshot);

    assert!(
        snapshot.tree().is_some(),
        "tree-backed snapshots should retain the source tree"
    );
    assert_eq!(result, AssertionResult::Passed);
}

#[test]
fn missing_element_failure_uses_tree_snapshot_ids() {
    use crate::headless_assert::{evaluate_assert_exists, AssertionResult, DiagnosticsSnapshot};
    use blinc_recorder::{ElementSnapshot, Rect, Timestamp, TreeSnapshot};

    let mut tree = TreeSnapshot::new(Timestamp::from_micros(10), (1280, 720), 1.0);
    let element = ElementSnapshot::new(
        "existing".to_string(),
        "Div".to_string(),
        Rect::new(0.0, 0.0, 100.0, 20.0),
    );
    tree.elements.insert("existing".to_string(), element);

    let snapshot = DiagnosticsSnapshot::from(tree);
    let result = evaluate_assert_exists("missing.node", &snapshot);

    assert_eq!(
        result,
        AssertionResult::Failed {
            code: "missing_element".to_string(),
            message: "missing.node: element not found".to_string(),
        }
    );
}

#[test]
fn runner_stops_on_first_failed_assertion() {
    use crate::headless_assert::DiagnosticsSnapshot;
    use crate::headless_runner::{run_scenario_with_probe, RunOutcome};
    use crate::headless_runtime::HeadlessRunConfig;

    let scenario_json = r#"{
      "steps": [
        {"type":"assert_exists","id":"missing.node"},
        {"type":"tick","frames":10}
      ]
    }"#;

    let outcome = run_scenario_with_probe(scenario_json, HeadlessRunConfig::default(), |_ctx| {
        DiagnosticsSnapshot::default()
    })
    .expect("runner should return outcome");
    assert!(matches!(outcome, RunOutcome::Failed { .. }));
}

#[test]
fn runner_accepts_named_probe_closure_with_captured_state() {
    use crate::headless_assert::{DiagnosticsElement, DiagnosticsSnapshot};
    use crate::headless_runner::{run_scenario_with_owned_probe, RunOutcome};
    use crate::headless_runtime::HeadlessRunConfig;

    let scenario_json = r#"{
      "steps": [
        {"type":"assert_exists","id":"app.title"}
      ]
    }"#;
    let title = "Welcome to Demo".to_string();

    let mut probe = |_ctx| {
        let mut snapshot = DiagnosticsSnapshot::default();
        snapshot.elements.insert(
            "app.title".to_string(),
            DiagnosticsElement {
                text: Some(title.clone()),
            },
        );
        snapshot
    };

    let outcome =
        run_scenario_with_owned_probe(scenario_json, HeadlessRunConfig::default(), &mut probe)
            .expect("runner should accept named closure probe");

    assert!(matches!(outcome, RunOutcome::Passed { .. }));
}

#[test]
fn loaded_runner_accepts_reference_probe_with_captured_state() {
    use crate::headless_assert::{DiagnosticsElement, DiagnosticsSnapshot};
    use crate::headless_runner::{run_loaded_scenario_with_probe, ProbeContext, RunOutcome};
    use crate::headless_runtime::HeadlessRunConfig;
    use crate::headless_scenario::{HeadlessScenario, ScenarioStep, ScenarioTarget};

    let scenario = HeadlessScenario {
        steps: vec![ScenarioStep::AssertExists {
            target: ScenarioTarget {
                id: Some("app.title".to_string()),
                ..Default::default()
            },
        }],
    };
    let title = "Welcome to Demo".to_string();

    let mut probe = |_ctx: &ProbeContext| {
        let mut snapshot = DiagnosticsSnapshot::default();
        snapshot.elements.insert(
            "app.title".to_string(),
            DiagnosticsElement {
                text: Some(title.clone()),
            },
        );
        snapshot
    };

    let outcome =
        run_loaded_scenario_with_probe(&scenario, HeadlessRunConfig::default(), &mut probe)
            .expect("loaded runner should accept reference probe");

    assert!(matches!(outcome, RunOutcome::Passed { .. }));
}

#[test]
fn run_scenario_requires_probe_for_assertions() {
    use crate::headless_runner::run_scenario;

    let scenario_json = r#"{
      "steps": [
        {"type":"assert_exists","id":"missing.node"}
      ]
    }"#;

    let err = run_scenario(scenario_json).expect_err("assert scenario without probe must fail");
    assert!(
        err.to_string().contains("use run_scenario_with_probe"),
        "unexpected error: {err}"
    );
}

#[test]
fn failed_run_writes_machine_readable_report() {
    use crate::headless_report::HeadlessReport;

    let report = HeadlessReport::failed("assert_exists", 0, "missing.node".to_string(), 0, 0);
    let json = serde_json::to_string(&report).expect("report should serialize");

    assert!(json.contains("\"status\":\"failed\""));
    assert!(json.contains("\"assert_exists\""));
}

fn automation_test_guard() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| {
        eprintln!("automation test lock was poisoned; recovering serialized access");
        poisoned.into_inner()
    })
}

fn ensure_automation_theme() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(blinc_theme::ThemeState::init_default);

    if !blinc_core::BlincContextState::is_initialized() {
        blinc_core::BlincContextState::init_with_callback(
            std::sync::Arc::new(std::sync::Mutex::new(blinc_core::ReactiveGraph::new())),
            std::sync::Arc::new(std::sync::Mutex::new(blinc_core::HookState::new())),
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            std::sync::Arc::new(|signal_ids| {
                blinc_layout::check_stateful_deps(signal_ids);
            }),
        );
    }

    let ctx = blinc_core::BlincContextState::get();
    ctx.reseed_for_tests();
    blinc_layout::widgets::reset_text_widget_test_state();
    blinc_layout::click_outside::clear_click_outside_handlers();
    blinc_recorder::uninstall_hooks();
    blinc_recorder::uninstall_recorder();
}

#[test]
fn ensure_automation_theme_reseeds_context_state_between_runs() {
    let _guard = automation_test_guard();
    ensure_automation_theme();

    let ctx = blinc_core::BlincContextState::get();
    ctx.set_query_callback(std::sync::Arc::new(|_| Some(7)));
    ctx.set_focus(Some("stale.focus"));
    ctx.set_viewport_size(640.0, 480.0);
    ctx.set_programmatic_event_callback(std::sync::Arc::new(|_, _| {}));
    ctx.set_element_registry(std::sync::Arc::new(123usize));
    ctx.set_recorder_event_callback(std::sync::Arc::new(|_| {}));
    ctx.set_recorder_snapshot_callback(std::sync::Arc::new(|_| {}));
    ctx.set_recorder_update_callback(std::sync::Arc::new(|_, _| {}));
    let _: blinc_core::State<i32> = ctx.use_state_keyed("stale.counter", || 1);

    let _previous_resources = ctx.set_resource_override(blinc_core::ContextResourceOverride::new(
        std::sync::Arc::new(std::sync::Mutex::new(blinc_core::ReactiveGraph::new())),
        std::sync::Arc::new(std::sync::Mutex::new(blinc_core::HookState::new())),
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
    ));
    let _previous_bindings =
        ctx.set_binding_override(blinc_core::context_state::ContextBindingOverride::default());

    blinc_layout::widgets::set_continuous_redraw_callback(|_| {});
    blinc_layout::click_outside::register_click_outside("stale", "node", || {});

    ensure_automation_theme();

    assert_eq!(ctx.query("node"), None);
    assert_eq!(ctx.focused_element(), None);
    assert_eq!(ctx.viewport_size(), (0.0, 0.0));
    assert!(ctx.programmatic_event_callback().is_none());
    assert!(ctx.element_registry_any().is_none());
    assert!(!ctx.is_recording_events());
    assert!(!ctx.is_recording_snapshots());
    assert!(!ctx.is_recording_updates());
    assert!(
        ctx.debug_keyed_state_entries().is_empty(),
        "expected keyed state inventory to be empty after reseed"
    );
}

#[test]
fn automation_session_executes_click_and_assert_commands_in_headless_mode() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::widgets::blur_all_text_inputs;
    use blinc_recorder::TraceEntryKind;

    let _guard = automation_test_guard();
    ensure_automation_theme();
    blur_all_text_inputs();

    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        let count = ctx.use_state_keyed("count", || 0i32);

        div()
            .w(ctx.width)
            .h(ctx.height)
            .flex_col()
            .gap(16.0)
            .child(
                div()
                    .id("increment")
                    .on_click({
                        let count = count.clone();
                        move |_| count.set(count.get() + 1)
                    })
                    .child(text("Increment")),
            )
            .child(
                div()
                    .id("counter")
                    .child(text(format!("Count: {}", count.get()))),
            )
    });

    session
        .click(AutomationLocator::id("increment"))
        .expect("click should dispatch");
    session
        .assert_text_contains(AutomationLocator::id("counter"), "Count: 1")
        .expect("assertion should observe updated state");

    let export = session.export_recording();
    assert!(
        export
            .trace_entries
            .iter()
            .any(|entry| matches!(entry.kind, TraceEntryKind::Command(_))),
        "expected command trace entries"
    );
    assert!(
        !export.snapshots.is_empty(),
        "expected state snapshots to be captured"
    );
}

#[test]
fn automation_session_assert_text_contains_is_case_insensitive() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        div()
            .w(ctx.width)
            .h(ctx.height)
            .child(div().id("status").child(text("Signed In")))
    });

    session
        .assert_text_contains(AutomationLocator::id("status"), "signed in")
        .expect("text assertions should follow semantic locator case-folding");
}

#[test]
fn semantic_locator_scenarios_execute_against_headless_runtime() {
    use crate::{run_headless_scenario, HeadlessRunConfig, HeadlessScenario, ReportStatus};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let scenario = HeadlessScenario::from_json(
        r#"{
  "steps": [
    {"type":"click","role":"button","label":"Increment"},
    {"type":"assert_text_contains","id":"counter","value":"Count: 1"}
  ]
}"#,
    )
    .expect("semantic locator scenario should parse");

    let run = run_headless_scenario(HeadlessRunConfig::default(), &scenario, |ctx| {
        let count = ctx.use_state_keyed("count", || 0i32);
        let increment_button = ctx.use_state_for("counter.increment.button", ButtonState::Idle);

        div()
            .w(ctx.width)
            .h(ctx.height)
            .flex_col()
            .gap(16.0)
            .child(
                button(increment_button, "Increment")
                    .id("counter.increment")
                    .on_click({
                        let count = count.clone();
                        move |_| count.set(count.get() + 1)
                    }),
            )
            .child(
                div()
                    .id("counter")
                    .child(text(format!("Count: {}", count.get()))),
            )
    })
    .expect("semantic locator run should succeed");

    assert!(matches!(run.report.status, ReportStatus::Passed));
}

#[test]
fn wait_steps_report_elapsed_time_using_advanced_frames() {
    use crate::{run_headless_scenario, HeadlessRunConfig, HeadlessScenario, ReportStatus};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let runtime_cfg = HeadlessRunConfig {
        tick_ms: 16,
        ..HeadlessRunConfig::default()
    };
    let scenario = HeadlessScenario::from_json(
        r#"{
  "steps": [
    {"type":"wait","ms":1},
    {"type":"assert_exists","id":"missing"}
  ]
}"#,
    )
    .expect("scenario should parse");

    let run = run_headless_scenario(runtime_cfg, &scenario, |ctx| {
        div().w(ctx.width).h(ctx.height).child(text("Ready"))
    })
    .expect("scenario execution should finish with a failed report");

    assert!(matches!(run.report.status, ReportStatus::Failed));
    assert_eq!(run.report.failed_step_index, Some(1));
    assert_eq!(run.report.elapsed_frames, 1);
    assert_eq!(run.report.elapsed_ms, 16);
}

#[test]
fn automation_session_returns_trace_linked_failure_details() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_recorder::TraceEntryKind;

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        div().w(ctx.width).h(ctx.height).child(text("Ready"))
    });

    let failure = session
        .assert_exists(AutomationLocator::id("missing"))
        .expect_err("missing locator should fail");

    assert_eq!(failure.code, "locator_not_found");
    assert!(failure.trace_sequence.is_some());

    let export = session.export_recording();
    assert!(
        export.trace_entries.iter().any(|entry| matches!(
            &entry.kind,
            TraceEntryKind::LocatorResolution(resolution)
                if resolution.query == "id=\"missing\"" && resolution.failure_reason.as_deref() == Some("no_match")
        )),
        "expected failed locator resolution in trace"
    );
    assert!(
        export.trace_entries.iter().any(|entry| matches!(
            &entry.kind,
            TraceEntryKind::Assertion(assertion)
                if assertion.code == "assert_exists" && !assertion.passed
        )),
        "expected failed assertion entry in trace"
    );
}

#[test]
fn playbook_parses_named_states_and_transitions() {
    use crate::Playbook;

    let playbook = Playbook::from_yaml(
        r#"
initial_state: idle
states:
  - filling
  - submitted
transitions:
  - name: submit_form
    from: idle
    event: begin
    to: filling
    steps:
      - type: click
        id: login.submit
"#,
    )
    .expect("playbook should parse");

    assert_eq!(playbook.initial_state, "idle");
    assert_eq!(playbook.states, vec!["filling", "submitted"]);
    assert_eq!(playbook.transitions.len(), 1);
    assert_eq!(playbook.transitions[0].name.as_deref(), Some("submit_form"));
}

#[test]
fn playbook_from_path_reports_the_failing_file() {
    use crate::Playbook;

    let path = std::env::temp_dir().join(format!(
        "blinc-invalid-playbook-{}.yaml",
        std::process::id()
    ));
    std::fs::write(&path, "initial_state: [").expect("invalid playbook should be written");

    let err = Playbook::from_path(&path).expect_err("invalid playbook should fail to parse");
    let err_text = err.to_string();
    assert!(
        err_text.contains(path.to_string_lossy().as_ref()),
        "expected parse error to mention the playbook path: {err_text}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn playbook_compiles_into_existing_fsm_runtime_types() {
    use crate::Playbook;

    let playbook = Playbook::from_yaml(
        r#"
initial_state: idle
states:
  - filling
  - submitted
transitions:
  - from: idle
    event: begin
    to: filling
    steps:
      - type: tick
        frames: 1
  - from: filling
    event: submit
    to: submitted
    steps:
      - type: assert_exists
        id: status
"#,
    )
    .expect("playbook should parse");

    let compiled = playbook.compile().expect("playbook should compile");
    compiled
        .validate_execution_order()
        .expect("execution order should validate");

    let (mut runtime, machine) = compiled.instantiate_runtime();
    let begin = *compiled
        .event_ids
        .get("begin")
        .expect("begin event should be assigned");
    let submit = *compiled
        .event_ids
        .get("submit")
        .expect("submit event should be assigned");
    let filling = *compiled
        .state_ids
        .get("filling")
        .expect("filling state should be assigned");
    let submitted = *compiled
        .state_ids
        .get("submitted")
        .expect("submitted state should be assigned");

    assert_eq!(runtime.send(machine, begin), Some(filling));
    assert_eq!(runtime.send(machine, submit), Some(submitted));
}

#[test]
fn playbook_execution_rejects_ambiguous_branches() {
    use crate::Playbook;

    let playbook = Playbook::from_yaml(
        r#"
initial_state: idle
states:
  - filling
  - cancelled
transitions:
  - from: idle
    event: begin
    to: filling
    steps:
      - type: tick
        frames: 1
  - from: idle
    event: cancel
    to: cancelled
    steps:
      - type: tick
        frames: 1
"#,
    )
    .expect("playbook should parse");

    let compiled = playbook.compile().expect("playbook should compile");
    let err = compiled
        .validate_execution_order()
        .expect_err("ambiguous outgoing transitions should be rejected");
    assert!(
        err.to_string().contains("ambiguous"),
        "unexpected error: {err}"
    );
}

#[test]
fn playbook_execution_accepts_explicit_branching_path() {
    use crate::Playbook;

    let playbook = Playbook::from_yaml(
        r#"
initial_state: idle
states:
  - filling
  - cancelled
execution:
  - cancel
transitions:
  - from: idle
    event: begin
    to: filling
    steps:
      - type: tick
        frames: 1
  - from: idle
    event: cancel
    to: cancelled
    steps:
      - type: assert_exists
        id: cancel.banner
"#,
    )
    .expect("playbook should parse");

    let compiled = playbook.compile().expect("playbook should compile");
    let scenario = compiled
        .execution_scenario()
        .expect("explicit execution path should compile into a scenario");
    assert_eq!(scenario.steps.len(), 1);
    assert!(matches!(
        scenario.steps[0],
        crate::headless_scenario::ScenarioStep::AssertExists { ref target }
            if target.id.as_deref() == Some("cancel.banner")
    ));
}

#[test]
fn desktop_session_uses_same_command_types_as_headless() {
    use crate::{AutomationLocator, AutomationRuntimeMode, AutomationSession, HeadlessRunConfig};
    use blinc_recorder::TraceEntryKind;

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let headless_commands = {
        let mut headless = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
            div()
                .w(ctx.width)
                .h(ctx.height)
                .child(div().id("status").child(text("Ready")))
        });

        headless
            .assert_exists(AutomationLocator::id("status"))
            .expect("headless session should succeed");
        assert_eq!(headless.runtime_mode(), AutomationRuntimeMode::Headless);

        headless
            .export_recording()
            .trace_entries
            .into_iter()
            .filter_map(|entry| match entry.kind {
                TraceEntryKind::Command(command) => Some(command.name),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let desktop_commands = {
        let mut desktop =
            AutomationSession::new_desktop_harness(HeadlessRunConfig::default(), |ctx| {
                div()
                    .w(ctx.width)
                    .h(ctx.height)
                    .child(div().id("status").child(text("Ready")))
            });

        desktop
            .assert_exists(AutomationLocator::id("status"))
            .expect("desktop harness session should succeed");
        assert_eq!(
            desktop.runtime_mode(),
            AutomationRuntimeMode::DesktopHarness
        );

        desktop
            .export_recording()
            .trace_entries
            .into_iter()
            .filter_map(|entry| match entry.kind {
                TraceEntryKind::Command(command) => Some(command.name),
                _ => None,
            })
            .collect::<Vec<_>>()
    };

    assert_eq!(headless_commands, desktop_commands);
}

#[test]
fn desktop_and_headless_runs_produce_matching_assertion_outcomes() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let build_ui = |ctx: &mut crate::windowed::WindowedContext| {
        let count = ctx.use_state_keyed("count", || 0i32);
        div()
            .w(ctx.width)
            .h(ctx.height)
            .child(
                div()
                    .id("increment")
                    .on_click({
                        let count = count.clone();
                        move |_| count.set(count.get() + 1)
                    })
                    .child(text("Increment")),
            )
            .child(
                div()
                    .id("counter")
                    .child(text(format!("Count: {}", count.get()))),
            )
    };

    let mut headless = AutomationSession::new_headless(HeadlessRunConfig::default(), build_ui);
    headless
        .click(AutomationLocator::id("increment"))
        .expect("headless click should succeed");
    headless
        .assert_text_contains(AutomationLocator::id("counter"), "Count: 1")
        .expect("headless assertion should pass");
    drop(headless);

    let mut desktop =
        AutomationSession::new_desktop_harness(HeadlessRunConfig::default(), build_ui);
    desktop
        .click(AutomationLocator::id("increment"))
        .expect("desktop click should succeed");
    desktop
        .assert_text_contains(AutomationLocator::id("counter"), "Count: 1")
        .expect("desktop assertion should pass");
}

#[test]
fn automation_session_runs_on_ready_callbacks_in_desktop_harness() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_desktop_harness(HeadlessRunConfig::default(), |ctx| {
        let ready = ctx.use_state_keyed("ready", || false);
        if !ready.get() {
            let ready = ready.clone();
            ctx.on_ready(move || ready.set(true));
        }

        div()
            .w(ctx.width)
            .h(ctx.height)
            .child(
                div()
                    .id("status")
                    .child(text(if ready.get() { "Ready" } else { "Booting" })),
            )
    });

    session
        .tick_frames(1)
        .expect("desktop harness should advance the ready callback");
    session
        .assert_text_contains(AutomationLocator::id("status"), "Ready")
        .expect("on_ready callback should update the rendered tree");
}

#[test]
fn automation_session_renders_overlay_layers_in_desktop_harness() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_desktop_harness(HeadlessRunConfig::default(), |ctx| {
        let overlays = ctx.overlay_manager();

        div().w(ctx.width).h(ctx.height).child(
            div()
                .id("overlay.open")
                .on_click({
                    let overlays = overlays.clone();
                    move |_| {
                        overlays
                            .modal()
                            .content(|| div().id("overlay.content").child(text("Overlay Ready")))
                            .show();
                    }
                })
                .child(text("Open Overlay")),
        )
    });

    session
        .click(AutomationLocator::id("overlay.open"))
        .expect("desktop harness should dispatch overlay open click");
    session
        .assert_text_contains(AutomationLocator::id("overlay.content"), "Overlay Ready")
        .expect("overlay content should be rendered into the automation tree");
}

#[test]
fn automation_session_press_escape_dismisses_overlay_without_focus() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::widgets::overlay::OverlayAnimation;

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        let overlays = ctx.overlay_manager();
        let opened = ctx.use_state_keyed("overlay.auto_opened", || false);
        if !opened.get() {
            let overlays = overlays.clone();
            let opened = opened.clone();
            ctx.on_ready(move || {
                opened.set(true);
                overlays
                    .hover_card()
                    .at(96.0, 96.0)
                    .animation(OverlayAnimation::none())
                    .dismiss_on_hover_leave(false)
                    .dismiss_on_click_outside(true)
                    .size(120.0, 40.0)
                    .content(|| {
                        div()
                            .id("overlay.menu")
                            .w(120.0)
                            .h(40.0)
                            .child(text("Menu"))
                    })
                    .show();
            });
        }

        div().w(ctx.width).h(ctx.height).child(text("Ready"))
    });

    session
        .tick_frames(20)
        .expect("overlay open scheduled from on_ready should rebuild");
    session
        .assert_exists(AutomationLocator::id("overlay.menu"))
        .expect("overlay should be visible before escape");
    session
        .press("Escape")
        .expect("escape should dismiss overlays even without a focused element");
    assert!(
        session
            .assert_exists(AutomationLocator::id("overlay.menu"))
            .is_err(),
        "overlay should be dismissed after escape"
    );
}

#[test]
fn automation_session_click_rejects_targets_consumed_by_dismissable_overlay_backdrops() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::widgets::overlay::OverlayAnimation;

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        let overlays = ctx.overlay_manager();
        let background_clicks = ctx.use_state_keyed("background.clicks", || 0usize);
        let click_count = background_clicks.get();

        div()
            .w(ctx.width)
            .h(ctx.height)
            .child(
                div()
                    .id("background.target")
                    .w(ctx.width)
                    .h(ctx.height)
                    .on_click({
                        let background_clicks = background_clicks.clone();
                        move |_| background_clicks.set(background_clicks.get() + 1)
                    })
                    .child(
                        div()
                            .id("background.count")
                            .child(text(format!("Clicks: {click_count}"))),
                    ),
            )
            .child(
                div()
                    .id("overlay.open")
                    .w(100.0)
                    .h(40.0)
                    .on_click({
                        let overlays = overlays.clone();
                        move |_| {
                            overlays
                                .hover_card()
                                .at(96.0, 96.0)
                                .animation(OverlayAnimation::none())
                                .dismiss_on_hover_leave(false)
                                .dismiss_on_click_outside(true)
                                .size(120.0, 40.0)
                                .content(|| {
                                    div()
                                        .id("overlay.dismissable")
                                        .w(120.0)
                                        .h(40.0)
                                        .child(text("Dismiss me"))
                                })
                                .show();
                        }
                    })
                    .child(text("Open")),
            )
    });

    session
        .click(AutomationLocator::id("overlay.open"))
        .expect("overlay open click should dispatch");
    session
        .tick_frames(20)
        .expect("overlay should advance into the open state");
    session
        .assert_exists(AutomationLocator::id("overlay.dismissable"))
        .expect("dismissable overlay should be visible");
    let failure = session
        .click(AutomationLocator::id("background.target"))
        .expect_err("targeted click should fail when a dismissable overlay consumes the backdrop");
    assert_eq!(failure.code, "target_blocked_by_overlay");

    session
        .assert_exists(AutomationLocator::id("overlay.dismissable"))
        .expect("targeted click should leave the dismissable overlay open");
    session
        .assert_text_contains(AutomationLocator::id("background.count"), "Clicks: 0")
        .expect("background click handler should not run when overlay blocks the targeted click");
}

#[test]
fn automation_session_click_at_coordinates_dismisses_overlay_without_locator() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::widgets::overlay::OverlayAnimation;

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        let overlays = ctx.overlay_manager();

        div().w(ctx.width).h(ctx.height).child(
            div()
                .id("overlay.open")
                .w(100.0)
                .h(40.0)
                .on_click({
                    let overlays = overlays.clone();
                    move |_| {
                        overlays
                            .hover_card()
                            .at(96.0, 96.0)
                            .animation(OverlayAnimation::none())
                            .dismiss_on_hover_leave(false)
                            .dismiss_on_click_outside(true)
                            .size(120.0, 40.0)
                            .content(|| {
                                div()
                                    .id("overlay.dismissable")
                                    .w(120.0)
                                    .h(40.0)
                                    .child(text("Dismiss me"))
                            })
                            .show();
                    }
                })
                .child(text("Open")),
        )
    });

    session
        .click(AutomationLocator::id("overlay.open"))
        .expect("overlay open click should dispatch");
    session
        .tick_frames(20)
        .expect("overlay should advance into the open state");
    session
        .assert_exists(AutomationLocator::id("overlay.dismissable"))
        .expect("overlay should be visible before click-away");
    session
        .click_at(24.0, 24.0)
        .expect("coordinate click should dismiss the overlay");

    assert!(
        session
            .assert_exists(AutomationLocator::id("overlay.dismissable"))
            .is_err(),
        "coordinate click outside the overlay should dismiss it"
    );
}

#[test]
fn automation_session_blocks_clicks_to_targets_behind_persistent_modal_backdrops() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::widgets::overlay::{BackdropConfig, OverlayAnimation};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        let overlays = ctx.overlay_manager();
        let opened = ctx.use_state_keyed("overlay.persistent_opened", || false);
        let clicks = ctx.use_state_keyed("background.blocked_clicks", || 0usize);
        let click_count = clicks.get();
        if !opened.get() {
            let overlays = overlays.clone();
            let opened = opened.clone();
            ctx.on_ready(move || {
                opened.set(true);
                overlays
                    .modal()
                    .animation(OverlayAnimation::none())
                    .backdrop(BackdropConfig::persistent())
                    .size(160.0, 80.0)
                    .content(|| div().id("overlay.blocking").child(text("Blocking")))
                    .show();
            });
        }

        div().w(ctx.width).h(ctx.height).child(
            div()
                .id("background.target")
                .w(120.0)
                .h(40.0)
                .on_click({
                    let clicks = clicks.clone();
                    move |_| clicks.set(clicks.get() + 1)
                })
                .child(text(format!("Clicks: {click_count}"))),
        )
    });

    session
        .tick_frames(20)
        .expect("persistent modal should open from on_ready");
    session
        .assert_exists(AutomationLocator::id("overlay.blocking"))
        .expect("persistent modal should be visible");

    let failure = session
        .click(AutomationLocator::id("background.target"))
        .expect_err("background click should be blocked by the modal backdrop");
    assert_eq!(failure.code, "target_blocked_by_overlay");
    session
        .assert_text_contains(AutomationLocator::id("background.target"), "Clicks: 0")
        .expect("blocked click should not reach the background target");
}

#[test]
fn automation_session_allows_clicks_inside_modal_overlay_content() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::widgets::overlay::{BackdropConfig, OverlayAnimation};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        let overlays = ctx.overlay_manager();
        let opened = ctx.use_state_keyed("overlay.clickable_opened", || false);
        let clicks = ctx.use_state_keyed("overlay.clicks", || 0usize);
        let click_count = clicks.get();
        if !opened.get() {
            let overlays = overlays.clone();
            let opened = opened.clone();
            let clicks = clicks.clone();
            ctx.on_ready(move || {
                opened.set(true);
                overlays
                    .modal()
                    .animation(OverlayAnimation::none())
                    .backdrop(BackdropConfig::persistent())
                    .size(160.0, 80.0)
                    .content(move || {
                        div()
                            .id("overlay.action")
                            .w(160.0)
                            .h(80.0)
                            .on_click({
                                let clicks = clicks.clone();
                                move |_| clicks.set(clicks.get() + 1)
                            })
                            .child(text("Overlay action"))
                    })
                    .show();
            });
        }

        div()
            .w(ctx.width)
            .h(ctx.height)
            .child(text("Background"))
            .child(
                div()
                    .id("overlay.status")
                    .child(text(format!("Overlay clicks: {click_count}"))),
            )
    });

    session
        .tick_frames(20)
        .expect("persistent modal should open from on_ready");
    session
        .assert_exists(AutomationLocator::id("overlay.action"))
        .expect("overlay action should be visible");

    session
        .click(AutomationLocator::id("overlay.action"))
        .expect("overlay content should remain clickable while modal is open");
    session
        .assert_text_contains(AutomationLocator::id("overlay.status"), "Overlay clicks: 1")
        .expect("overlay click should update overlay-local state");
}

#[test]
fn automation_session_allows_coordinate_clicks_inside_modal_overlay_content() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::widgets::overlay::{BackdropConfig, OverlayAnimation};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        let overlays = ctx.overlay_manager();
        let opened = ctx.use_state_keyed("overlay.coordinate_opened", || false);
        let clicks = ctx.use_state_keyed("overlay.coordinate_clicks", || 0usize);
        let click_count = clicks.get();
        if !opened.get() {
            let overlays = overlays.clone();
            let opened = opened.clone();
            let clicks = clicks.clone();
            ctx.on_ready(move || {
                opened.set(true);
                overlays
                    .modal()
                    .animation(OverlayAnimation::none())
                    .backdrop(BackdropConfig::persistent())
                    .size(160.0, 80.0)
                    .content(move || {
                        div()
                            .id("overlay.coordinate-action")
                            .w(160.0)
                            .h(80.0)
                            .on_click({
                                let clicks = clicks.clone();
                                move |_| clicks.set(clicks.get() + 1)
                            })
                            .child(text("Overlay action"))
                    })
                    .show();
            });
        }

        div()
            .w(ctx.width)
            .h(ctx.height)
            .child(text("Background"))
            .child(
                div()
                    .id("overlay.coordinate-status")
                    .child(text(format!("Overlay clicks: {click_count}"))),
            )
    });

    session
        .tick_frames(20)
        .expect("persistent modal should open from on_ready");
    session
        .assert_exists(AutomationLocator::id("overlay.coordinate-action"))
        .expect("overlay action should be visible");

    session
        .click_at(640.0, 360.0)
        .expect("center coordinate should hit the centered modal content");
    session
        .assert_text_contains(
            AutomationLocator::id("overlay.coordinate-status"),
            "Overlay clicks: 1",
        )
        .expect("coordinate click should update overlay-local state");
}

#[test]
fn automation_session_visible_hover_card_occludes_background_targets() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::widgets::overlay::OverlayAnimation;

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        let overlays = ctx.overlay_manager();
        let opened = ctx.use_state_keyed("overlay.hover_card_opened", || false);
        let background_clicks =
            ctx.use_state_keyed("overlay.hover_card_background_clicks", || 0usize);
        let overlay_clicks = ctx.use_state_keyed("overlay.hover_card_overlay_clicks", || 0usize);
        let background_count = background_clicks.get();
        let overlay_count = overlay_clicks.get();
        if !opened.get() {
            let overlays = overlays.clone();
            let opened = opened.clone();
            let overlay_clicks = overlay_clicks.clone();
            ctx.on_ready(move || {
                opened.set(true);
                overlays
                    .hover_card()
                    .at(0.0, 0.0)
                    .animation(OverlayAnimation::none())
                    .dismiss_on_hover_leave(false)
                    .dismiss_on_click_outside(false)
                    .size(160.0, 80.0)
                    .content(move || {
                        div()
                            .id("overlay.hover-card")
                            .w(160.0)
                            .h(80.0)
                            .on_click({
                                let overlay_clicks = overlay_clicks.clone();
                                move |_| overlay_clicks.set(overlay_clicks.get() + 1)
                            })
                            .child(text("Hover card"))
                    })
                    .show();
            });
        }

        div()
            .w(ctx.width)
            .h(ctx.height)
            .child(
                div()
                    .id("background.target")
                    .w(120.0)
                    .h(40.0)
                    .on_click({
                        let background_clicks = background_clicks.clone();
                        move |_| background_clicks.set(background_clicks.get() + 1)
                    })
                    .child(text("Background target")),
            )
            .child(
                div()
                    .id("background.status")
                    .child(text(format!("Background clicks: {background_count}"))),
            )
            .child(
                div()
                    .id("overlay.status")
                    .child(text(format!("Overlay clicks: {overlay_count}"))),
            )
    });

    session
        .tick_frames(20)
        .expect("hover card should open from on_ready");
    session
        .assert_exists(AutomationLocator::id("overlay.hover-card"))
        .expect("hover card should be visible");

    let failure = session
        .click(AutomationLocator::id("background.target"))
        .expect_err("background target behind hover card should be blocked");
    assert_eq!(failure.code, "target_blocked_by_overlay");
    session
        .assert_text_contains(
            AutomationLocator::id("background.status"),
            "Background clicks: 0",
        )
        .expect("blocked hover-card occlusion should leave background untouched");

    session
        .click_at(60.0, 20.0)
        .expect("coordinate click inside hover card should hit overlay content");
    session
        .assert_text_contains(AutomationLocator::id("overlay.status"), "Overlay clicks: 1")
        .expect("coordinate click should reach hover card content");
}

#[test]
fn automation_session_scrolls_targets_inside_modal_overlay_content() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::widgets::overlay::{BackdropConfig, OverlayAnimation};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        let overlays = ctx.overlay_manager();
        let opened = ctx.use_state_keyed("overlay.modal_scroll_opened", || false);
        if !opened.get() {
            let overlays = overlays.clone();
            let opened = opened.clone();
            ctx.on_ready(move || {
                opened.set(true);
                overlays
                    .modal()
                    .animation(OverlayAnimation::none())
                    .backdrop(BackdropConfig::persistent())
                    .size(200.0, 120.0)
                    .content(|| {
                        div()
                            .id("overlay.scroll.host")
                            .w(180.0)
                            .h(80.0)
                            .overflow_y_scroll()
                            .child(div().flex_col().w_full().children((0..24).map(|index| {
                                div()
                                    .id(format!("overlay.item.{index}"))
                                    .w_full()
                                    .h(32.0)
                                    .child(text(format!("Overlay item {index}")))
                            })))
                    })
                    .show();
            });
        }

        div().w(ctx.width).h(ctx.height).child(text("Background"))
    });

    session
        .tick_frames(20)
        .expect("modal should open from on_ready");
    let before_scroll = session
        .absolute_bounds_for_id("overlay.item.15")
        .expect("overlay item should have bounds before scrolling");

    session
        .scroll(
            Some(AutomationLocator::id("overlay.scroll.host")),
            0.0,
            -320.0,
        )
        .expect("scroll should reach the targeted host inside modal content");

    let after_scroll = session
        .absolute_bounds_for_id("overlay.item.15")
        .expect("overlay item should retain bounds after scrolling");
    assert!(
        after_scroll.y < before_scroll.y,
        "expected modal scroll host to move content upward: before={before_scroll:?} after={after_scroll:?}"
    );
}

#[test]
fn automation_session_scrolls_modal_targets_even_with_dismissable_overlay_above_them() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::widgets::overlay::{BackdropConfig, OverlayAnimation};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        let overlays = ctx.overlay_manager();
        let opened = ctx.use_state_keyed("overlay.modal_dismissable_scroll_opened", || false);
        if !opened.get() {
            let overlays = overlays.clone();
            let opened = opened.clone();
            ctx.on_ready(move || {
                opened.set(true);
                overlays
                    .modal()
                    .animation(OverlayAnimation::none())
                    .backdrop(BackdropConfig::persistent())
                    .size(240.0, 220.0)
                    .content(|| {
                        div()
                            .w(220.0)
                            .h(180.0)
                            .flex_col()
                            .child(
                                div()
                                    .id("overlay.modal.header")
                                    .w_full()
                                    .h(56.0)
                                    .child(text("Header")),
                            )
                            .child(
                                div()
                                    .id("overlay.modal.scroll.host")
                                    .w_full()
                                    .h(120.0)
                                    .overflow_y_scroll()
                                    .child(div().flex_col().w_full().children((0..24).map(
                                        |index| {
                                            div()
                                                .id(format!("overlay.modal.item.{index}"))
                                                .w_full()
                                                .h(32.0)
                                                .child(text(format!("Overlay item {index}")))
                                        },
                                    ))),
                            )
                    })
                    .show();
                overlays
                    .hover_card()
                    .at(540.0, 300.0)
                    .animation(OverlayAnimation::none())
                    .dismiss_on_hover_leave(false)
                    .dismiss_on_click_outside(true)
                    .size(120.0, 40.0)
                    .content(|| {
                        div()
                            .id("overlay.dismissable")
                            .w(120.0)
                            .h(40.0)
                            .child(text("Peek"))
                    })
                    .show();
            });
        }

        div().w(ctx.width).h(ctx.height).child(text("Background"))
    });

    session
        .tick_frames(20)
        .expect("modal and dismissable overlay should open from on_ready");
    let before_scroll = session
        .absolute_bounds_for_id("overlay.modal.item.15")
        .expect("modal item should have bounds before scrolling");

    session
        .scroll(
            Some(AutomationLocator::id("overlay.modal.scroll.host")),
            0.0,
            -320.0,
        )
        .expect("scroll should still reach modal content outside the dismissable overlay bounds");

    let after_scroll = session
        .absolute_bounds_for_id("overlay.modal.item.15")
        .expect("modal item should retain bounds after scrolling");
    assert!(
        after_scroll.y < before_scroll.y,
        "expected modal scroll host to keep moving even with a dismissable overlay above it: before={before_scroll:?} after={after_scroll:?}"
    );
}

#[test]
fn automation_session_blocks_scroll_targets_behind_visible_hover_cards() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::widgets::overlay::OverlayAnimation;

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        let overlays = ctx.overlay_manager();
        let opened = ctx.use_state_keyed("overlay.hover_scroll_opened", || false);
        if !opened.get() {
            let overlays = overlays.clone();
            let opened = opened.clone();
            ctx.on_ready(move || {
                opened.set(true);
                overlays
                    .hover_card()
                    .at(0.0, 0.0)
                    .animation(OverlayAnimation::none())
                    .dismiss_on_hover_leave(false)
                    .dismiss_on_click_outside(false)
                    .size(160.0, 80.0)
                    .content(|| {
                        div()
                            .id("overlay.hover-scroll")
                            .w(160.0)
                            .h(80.0)
                            .child(text("Hover card"))
                    })
                    .show();
            });
        }

        div().w(ctx.width).h(ctx.height).child(
            div()
                .id("background.scroll.host")
                .w(120.0)
                .h(40.0)
                .overflow_y_scroll()
                .child(div().flex_col().w_full().children((0..24).map(|index| {
                    div()
                        .id(format!("background.item.{index}"))
                        .w_full()
                        .h(32.0)
                        .child(text(format!("Background item {index}")))
                }))),
        )
    });

    session
        .tick_frames(20)
        .expect("hover card should open from on_ready");
    let before_scroll = session
        .absolute_bounds_for_id("background.item.15")
        .expect("background item should have bounds before blocked scroll");

    let failure = session
        .scroll(
            Some(AutomationLocator::id("background.scroll.host")),
            0.0,
            -320.0,
        )
        .expect_err("visible hover card should occlude background scroll targets");
    assert_eq!(failure.code, "target_blocked_by_overlay");

    let after_scroll = session
        .absolute_bounds_for_id("background.item.15")
        .expect("background item should retain bounds after blocked scroll");
    assert_eq!(
        after_scroll.y, before_scroll.y,
        "blocked scroll should leave background content stationary: before={before_scroll:?} after={after_scroll:?}"
    );
}

#[test]
fn automation_session_scroll_updates_follows_scroll_overlay_positions() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::widgets::overlay::OverlayAnimation;

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        let overlays = ctx.overlay_manager();

        div()
            .w(ctx.width)
            .h(ctx.height)
            .child(
                div()
                    .id("scroll.host")
                    .w(ctx.width)
                    .h(ctx.height)
                    .child(text("Scroll host")),
            )
            .child(
                div()
                    .id("overlay.open")
                    .w(100.0)
                    .h(40.0)
                    .on_click({
                        let overlays = overlays.clone();
                        move |_| {
                            overlays
                                .hover_card()
                                .at(96.0, 96.0)
                                .animation(OverlayAnimation::none())
                                .dismiss_on_hover_leave(false)
                                .dismiss_on_click_outside(true)
                                .follows_scroll(true)
                                .size(120.0, 40.0)
                                .content(|| {
                                    div()
                                        .id("overlay.follow")
                                        .w(120.0)
                                        .h(40.0)
                                        .child(text("Follow"))
                                })
                                .show();
                        }
                    })
                    .child(text("Open")),
            )
    });

    session
        .click(AutomationLocator::id("overlay.open"))
        .expect("overlay open click should dispatch");
    session
        .tick_frames(20)
        .expect("follows_scroll overlay should advance into the open state");
    let before_offsets = session.overlay_scroll_offsets();
    assert!(
        before_offsets
            .iter()
            .any(|(_, offset)| offset.abs() < f32::EPSILON),
        "expected follows_scroll overlay to start with zero offset: {before_offsets:?}"
    );

    session
        .scroll(Some(AutomationLocator::id("scroll.host")), 0.0, -24.0)
        .expect("scroll should update follows_scroll overlays");
    let after_offsets = session.overlay_scroll_offsets();

    assert!(
        after_offsets
            .iter()
            .any(|(_, offset)| (*offset - -24.0).abs() < 0.01),
        "follows_scroll overlay should record the dispatched scroll delta: {after_offsets:?}"
    );
}

#[test]
fn automation_session_blocked_scroll_does_not_mutate_follows_scroll_overlays() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::widgets::overlay::{BackdropConfig, OverlayAnimation};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        let overlays = ctx.overlay_manager();
        let opened = ctx.use_state_keyed("overlay.blocked_follow_scroll_opened", || false);
        if !opened.get() {
            let overlays = overlays.clone();
            let opened = opened.clone();
            ctx.on_ready(move || {
                opened.set(true);
                overlays
                    .modal()
                    .animation(OverlayAnimation::none())
                    .backdrop(BackdropConfig::persistent())
                    .size(180.0, 96.0)
                    .content(|| div().id("overlay.blocking.modal").child(text("Blocking")))
                    .show();
                overlays
                    .hover_card()
                    .at(96.0, 96.0)
                    .animation(OverlayAnimation::none())
                    .dismiss_on_hover_leave(false)
                    .dismiss_on_click_outside(true)
                    .follows_scroll(true)
                    .size(120.0, 40.0)
                    .content(|| {
                        div()
                            .id("overlay.follow.blocked")
                            .w(120.0)
                            .h(40.0)
                            .child(text("Follow"))
                    })
                    .show();
            });
        }

        div().w(ctx.width).h(ctx.height).child(
            div()
                .id("background.scroll.host")
                .w(ctx.width)
                .h(ctx.height)
                .child(text("Scroll host")),
        )
    });

    session
        .tick_frames(20)
        .expect("modal and follows_scroll overlay should open from on_ready");
    let before_offsets = session.overlay_scroll_offsets();
    assert!(
        before_offsets
            .iter()
            .any(|(_, offset)| offset.abs() < f32::EPSILON),
        "expected follows_scroll overlay to start with zero offset: {before_offsets:?}"
    );

    let failure = session
        .scroll(
            Some(AutomationLocator::id("background.scroll.host")),
            0.0,
            -24.0,
        )
        .expect_err("blocked background scroll should fail before mutating overlay offsets");
    assert_eq!(failure.code, "target_blocked_by_overlay");

    let after_offsets = session.overlay_scroll_offsets();
    assert_eq!(
        after_offsets, before_offsets,
        "blocked scroll should not mutate follows_scroll overlay offsets: before={before_offsets:?} after={after_offsets:?}"
    );
}

#[test]
fn automation_session_processes_scroll_ref_updates_in_desktop_harness() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::selector::ScrollRef;

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let observed_scroll_ref = std::sync::Arc::new(std::sync::Mutex::new(None::<ScrollRef>));
    let observed_scroll_ref_for_ui = observed_scroll_ref.clone();
    let mut session =
        AutomationSession::new_desktop_harness(HeadlessRunConfig::default(), move |ctx| {
            let scroll_ref = ctx.use_scroll_ref("automation.list");
            if observed_scroll_ref_for_ui
                .lock()
                .expect("scroll ref slot should lock")
                .is_none()
            {
                *observed_scroll_ref_for_ui
                    .lock()
                    .expect("scroll ref slot should lock") = Some(scroll_ref.clone());
            }

            div()
                .w(ctx.width)
                .h(ctx.height)
                .flex_col()
                .child(
                    div()
                        .id("scroll.jump")
                        .on_click({
                            let scroll_ref = scroll_ref.clone();
                            move |_| scroll_ref.scroll_to("scroll.item.24")
                        })
                        .child(text("Jump")),
                )
                .child(
                    scroll()
                        .bind(&scroll_ref)
                        .h(120.0)
                        .child(div().flex_col().children((0..32).map(|index| {
                            div()
                                .id(format!("scroll.item.{index}"))
                                .h(40.0)
                                .child(text(format!("Item {index}")))
                        }))),
                )
        });

    session
        .click(AutomationLocator::id("scroll.jump"))
        .expect("desktop harness should dispatch scroll ref jump");
    session
        .tick_frames(1)
        .expect("desktop harness should process pending scroll ref commands");

    let offset = observed_scroll_ref
        .lock()
        .expect("scroll ref slot should lock")
        .as_ref()
        .expect("scroll ref should be captured")
        .offset()
        .1;
    assert!(
        offset > 0.0,
        "expected scroll ref to advance vertically, got {offset}"
    );
}

#[test]
fn automation_session_scroll_into_view_routes_through_runtime_callback() {
    use crate::{AutomationSession, HeadlessRunConfig};
    use blinc_layout::selector::{query, ScrollRef};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let observed_scroll_ref = std::sync::Arc::new(std::sync::Mutex::new(None::<ScrollRef>));
    let observed_scroll_ref_for_ui = observed_scroll_ref.clone();
    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), move |ctx| {
        let scroll_ref = ctx.use_scroll_ref("automation.query.list");
        if observed_scroll_ref_for_ui
            .lock()
            .expect("scroll ref slot should lock")
            .is_none()
        {
            *observed_scroll_ref_for_ui
                .lock()
                .expect("scroll ref slot should lock") = Some(scroll_ref.clone());
        }

        div()
            .w(ctx.width)
            .h(ctx.height)
            .child(
                scroll()
                    .bind(&scroll_ref)
                    .h(120.0)
                    .child(div().flex_col().children((0..32).map(|index| {
                        div()
                            .id(format!("query.scroll.item.{index}"))
                            .h(40.0)
                            .child(text(format!("Item {index}")))
                    }))),
            )
    });

    query("query.scroll.item.24")
        .expect("query target should exist")
        .scroll_into_view();
    session
        .tick_frames(1)
        .expect("headless automation should process scroll-into-view requests");

    let offset = observed_scroll_ref
        .lock()
        .expect("scroll ref slot should lock")
        .as_ref()
        .expect("scroll ref should be captured")
        .offset()
        .1;
    assert!(
        offset > 0.0,
        "expected scroll_into_view to update scroll position"
    );
}

#[test]
fn automation_session_seeds_viewport_for_query_visibility() {
    use crate::{AutomationSession, HeadlessRunConfig};
    use blinc_layout::selector::query;

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let _session = AutomationSession::new_headless(
        HeadlessRunConfig {
            width: 160,
            height: 120,
            ..HeadlessRunConfig::default()
        },
        |ctx| {
            div()
                .w(ctx.width)
                .h(ctx.height)
                .child(div().id("visible").w(80.0).h(24.0).child(text("Visible")))
        },
    );

    assert!(
        query("visible")
            .expect("visible element should exist")
            .is_visible(),
        "expected visibility queries to use the automation viewport"
    );
}

#[test]
fn automation_session_click_keeps_query_focus_state_in_sync() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::selector::query;

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let email = text_input_state_with_placeholder("Email");
    let email_for_ui = email.clone();
    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), move |ctx| {
        div().w(ctx.width).h(ctx.height).child(
            text_input(&email_for_ui)
                .id("login.email")
                .placeholder("Email")
                .w(240.0),
        )
    });

    session
        .click(AutomationLocator::id("login.email"))
        .expect("click should focus the text input");
    assert!(
        query("login.email")
            .expect("email input should exist")
            .is_focused(),
        "expected query focus state to follow runtime focus"
    );

    query("login.email")
        .expect("email input should exist")
        .blur();
    session
        .tick_frames(1)
        .expect("blur should flow back through the runtime");
    assert!(
        !query("login.email")
            .expect("email input should exist")
            .is_focused(),
        "expected query focus state to clear after blur"
    );
}

#[test]
fn automation_session_fill_updates_text_input_value() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_recorder::TraceEntryKind;

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let email = text_input_state_with_placeholder("Email");
    let email_for_ui = email.clone();
    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), move |ctx| {
        div().w(ctx.width).h(ctx.height).child(
            text_input(&email_for_ui)
                .id("login.email")
                .placeholder("Email")
                .w(240.0),
        )
    });

    session
        .fill(AutomationLocator::id("login.email"), "person@example.com")
        .expect("fill should succeed");

    assert_eq!(
        email.lock().expect("email state should lock").value,
        "person@example.com"
    );

    let export = session.export_recording();
    let fill_payloads = export
        .trace_entries
        .iter()
        .filter_map(|entry| match &entry.kind {
            TraceEntryKind::Command(command) if command.name == "fill" => {
                command.payload.as_deref()
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        fill_payloads
            .iter()
            .all(|payload| !payload.contains("person@example.com") && payload.contains("redacted")),
        "expected fill trace payloads to be redacted: {fill_payloads:?}"
    );
}

#[test]
fn automation_session_fill_replaces_existing_text_input_value() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let email = text_input_state_with_placeholder("Email");
    email.lock().expect("email state should lock").value = "old@example.com".to_string();
    let email_for_ui = email.clone();
    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), move |ctx| {
        div().w(ctx.width).h(ctx.height).child(
            text_input(&email_for_ui)
                .id("login.email")
                .placeholder("Email")
                .w(240.0),
        )
    });

    session
        .fill(AutomationLocator::id("login.email"), "person@example.com")
        .expect("fill should replace existing text");

    assert_eq!(
        email.lock().expect("email state should lock").value,
        "person@example.com"
    );
}

#[test]
fn automation_session_fill_blocks_targets_occluded_by_modal_backdrop() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::widgets::overlay::{BackdropConfig, OverlayAnimation};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let email = text_input_state_with_placeholder("Email");
    let email_for_ui = email.clone();
    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), move |ctx| {
        let overlays = ctx.overlay_manager();
        let opened = ctx.use_state_keyed("overlay.fill_blocked_opened", || false);
        if !opened.get() {
            let overlays = overlays.clone();
            let opened = opened.clone();
            ctx.on_ready(move || {
                opened.set(true);
                overlays
                    .modal()
                    .animation(OverlayAnimation::none())
                    .backdrop(BackdropConfig::persistent())
                    .size(180.0, 96.0)
                    .content(|| div().id("overlay.fill.blocking").child(text("Blocking")))
                    .show();
            });
        }

        div().w(ctx.width).h(ctx.height).child(
            text_input(&email_for_ui)
                .id("login.email")
                .placeholder("Email")
                .w(240.0),
        )
    });

    session
        .tick_frames(20)
        .expect("persistent modal should open from on_ready");
    session
        .assert_exists(AutomationLocator::id("overlay.fill.blocking"))
        .expect("blocking modal should be visible");

    let failure = session
        .fill(AutomationLocator::id("login.email"), "person@example.com")
        .expect_err("fill should not type into an occluded background input");
    assert_eq!(failure.code, "target_blocked_by_overlay");
    assert_eq!(email.lock().expect("email state should lock").value, "");
}

#[test]
fn automation_session_press_types_printable_text_into_focused_input() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let email = text_input_state_with_placeholder("Email");
    let email_for_ui = email.clone();
    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), move |ctx| {
        div().w(ctx.width).h(ctx.height).child(
            text_input(&email_for_ui)
                .id("login.email")
                .placeholder("Email")
                .w(240.0),
        )
    });

    session
        .click(AutomationLocator::id("login.email"))
        .expect("click should focus input");
    session
        .press("a")
        .expect("press should type printable input");

    assert_eq!(email.lock().expect("email state should lock").value, "a");
}

#[test]
fn automation_session_press_blocks_focused_targets_occluded_by_modal_backdrop() {
    use crate::{AutomationSession, HeadlessRunConfig};
    use blinc_layout::selector::query;
    use blinc_layout::widgets::overlay::{BackdropConfig, OverlayAnimation};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let email = text_input_state_with_placeholder("Email");
    let email_for_ui = email.clone();
    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), move |ctx| {
        let overlays = ctx.overlay_manager();
        let opened = ctx.use_state_keyed("overlay.press_blocked_opened", || false);
        if !opened.get() {
            let overlays = overlays.clone();
            let opened = opened.clone();
            ctx.on_ready(move || {
                blinc_core::BlincContextState::get().set_focus(Some("login.email"));
                opened.set(true);
                overlays
                    .modal()
                    .animation(OverlayAnimation::none())
                    .backdrop(BackdropConfig::persistent())
                    .size(180.0, 96.0)
                    .content(|| div().id("overlay.press.blocking").child(text("Blocking")))
                    .show();
            });
        }

        div().w(ctx.width).h(ctx.height).child(
            text_input(&email_for_ui)
                .id("login.email")
                .placeholder("Email")
                .w(240.0),
        )
    });

    session
        .tick_frames(20)
        .expect("persistent modal should open from on_ready");
    assert!(
        query("login.email")
            .expect("email input should exist")
            .is_focused(),
        "expected modal test to keep the background input focused"
    );

    let failure = session
        .press("a")
        .expect_err("press should not type into a focused input hidden behind a blocking modal");
    assert_eq!(failure.code, "target_blocked_by_overlay");
    assert_eq!(email.lock().expect("email state should lock").value, "");
}

#[test]
fn automation_session_press_emits_text_input_before_key_up() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let events_for_ui = events.clone();
    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), move |ctx| {
        let events_for_key_down = events_for_ui.clone();
        let events_for_text = events_for_ui.clone();
        let events_for_key_up = events_for_ui.clone();
        div()
            .id("key-target")
            .w(240.0)
            .h(48.0)
            .child(text("Focus me"))
            .on_key_down(move |_ctx| {
                events_for_key_down
                    .lock()
                    .expect("events should lock")
                    .push("key_down".to_string());
            })
            .on_text_input(move |_ctx| {
                events_for_text
                    .lock()
                    .expect("events should lock")
                    .push("text_input".to_string());
            })
            .on_key_up(move |_ctx| {
                events_for_key_up
                    .lock()
                    .expect("events should lock")
                    .push("key_up".to_string());
            })
    });

    session
        .click(AutomationLocator::id("key-target"))
        .expect("click should focus the event target");
    session
        .press("a")
        .expect("press should dispatch key events");

    assert_eq!(
        *events.lock().expect("events should lock"),
        vec![
            "key_down".to_string(),
            "text_input".to_string(),
            "key_up".to_string()
        ]
    );
}

#[test]
fn automation_session_preserves_text_input_caret_across_runtime_ticks() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let email = text_input_state_with_placeholder("Email");
    let email_for_ui = email.clone();
    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), move |ctx| {
        div().w(ctx.width).h(ctx.height).child(
            text_input(&email_for_ui)
                .id("login.email")
                .placeholder("Email")
                .w(240.0),
        )
    });

    session
        .click(AutomationLocator::id("login.email"))
        .expect("click should focus input");
    session.press("a").expect("first press should type");
    session
        .tick_frames(1)
        .expect("tick should not reset text input caret");
    session.press("b").expect("second press should append");

    assert_eq!(email.lock().expect("email state should lock").value, "ab");
}

#[test]
fn automation_session_tick_frames_runs_registered_tick_callbacks() {
    use crate::{AutomationSession, HeadlessRunConfig};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let frames_seen = std::sync::Arc::new(std::sync::Mutex::new(0u32));
    let frames_for_ui = frames_seen.clone();
    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), move |ctx| {
        let frames = frames_for_ui.clone();
        ctx.use_tick_callback_for("automation_tick_counter", move |_dt| {
            *frames.lock().expect("tick counter should lock") += 1;
        });
        div().w(ctx.width).h(ctx.height).child(text("Ticking"))
    });

    session
        .tick_frames(3)
        .expect("tick_frames should advance the active session");

    assert!(
        *frames_seen.lock().expect("tick counter should lock") >= 3,
        "expected tick callback to run for the session under test"
    );
}

#[test]
fn automation_session_tick_frames_respects_probe_sampling_interval() {
    use crate::{AutomationSession, HeadlessRunConfig};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_headless(
        HeadlessRunConfig {
            probe_every_frames: 2,
            ..HeadlessRunConfig::default()
        },
        |ctx| div().w(ctx.width).h(ctx.height).child(text("Ticking")),
    );

    session
        .tick_frames(5)
        .expect("tick_frames should honor probe sampling");

    let export = session.export_recording();
    assert_eq!(
        export.snapshots.len(),
        4,
        "expected initial snapshot plus samples at frames 2, 4, and 5"
    );
}

#[test]
fn automation_session_tick_frames_advances_logical_runtime_time() {
    use crate::{AutomationSession, HeadlessRunConfig};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let mut session = AutomationSession::new_headless(
        HeadlessRunConfig {
            tick_ms: 25,
            ..HeadlessRunConfig::default()
        },
        |ctx| div().w(ctx.width).h(ctx.height).child(text("Ticking")),
    );

    assert_eq!(session.runtime_time_ms(), 0);
    session
        .tick_frames(4)
        .expect("tick_frames should advance logical runtime time deterministically");
    assert_eq!(session.runtime_time_ms(), 100);
}

#[test]
fn automation_session_records_snapshot_and_trace_exports_as_artifacts() {
    use crate::{AutomationSession, HeadlessRunConfig};
    use blinc_recorder::TraceEntryKind;

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time must be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("blinc-automation-artifacts-{nonce}"));
    let snapshot_path = root.join("tree.json");
    let trace_path = root.join("trace.json");

    let session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        div().w(ctx.width).h(ctx.height).child(text("Ready"))
    });

    session
        .write_snapshot_to_path(&snapshot_path)
        .expect("snapshot export should succeed");
    session
        .write_trace_to_path(&trace_path)
        .expect("trace export should succeed");

    let export = session.export_recording();
    let written_export: blinc_recorder::RecordingExport = serde_json::from_str(
        &std::fs::read_to_string(&trace_path).expect("trace export should be readable"),
    )
    .expect("trace export should deserialize");
    let artifact_kinds = export
        .trace_entries
        .iter()
        .filter_map(|entry| match &entry.kind {
            TraceEntryKind::Artifact(artifact) => Some(artifact.kind.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        artifact_kinds.contains(&"snapshot_export"),
        "expected snapshot export artifact in trace entries: {artifact_kinds:?}"
    );
    assert!(
        artifact_kinds.contains(&"trace_export"),
        "expected trace export artifact in trace entries: {artifact_kinds:?}"
    );
    let live_trace_export = export
        .trace_entries
        .iter()
        .find(|entry| matches!(&entry.kind, TraceEntryKind::Artifact(artifact) if artifact.kind == "trace_export"))
        .expect("live export should contain trace_export artifact");
    let written_trace_export = written_export
        .trace_entries
        .iter()
        .find(|entry| matches!(&entry.kind, TraceEntryKind::Artifact(artifact) if artifact.kind == "trace_export"))
        .expect("written export should contain trace_export artifact");
    assert_eq!(
        live_trace_export.timestamp, written_trace_export.timestamp,
        "trace export artifact should use the same timestamp in memory and on disk"
    );
    assert_eq!(
        live_trace_export.sequence, written_trace_export.sequence,
        "trace export artifact should use the same sequence in memory and on disk"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn automation_session_snapshot_captures_semantic_and_view_model_state() {
    use crate::{AutomationSession, HeadlessRunConfig};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
        let count = ctx.use_state_keyed("counter.count", || 1i32);
        let increment_button = ctx.use_state_for("counter.increment.button", ButtonState::Idle);

        div()
            .w(ctx.width)
            .h(ctx.height)
            .flex_col()
            .gap(16.0)
            .child(button(increment_button, "Increment").id("counter.increment"))
            .child(
                div()
                    .id("counter.value")
                    .child(text(format!("Count: {}", count.get()))),
            )
    });

    let snapshot = session
        .export_recording()
        .snapshots
        .last()
        .cloned()
        .expect("automation session should capture an initial snapshot");
    let button = snapshot
        .elements
        .get("counter.increment")
        .expect("button element should exist in the captured snapshot");
    let semantic = button
        .semantic
        .as_ref()
        .expect("button snapshot should include semantic metadata");
    assert_eq!(semantic.tag.as_deref(), Some("button"));
    assert_eq!(semantic.role.as_deref(), Some("Button"));
    assert_eq!(semantic.name.as_deref(), Some("Increment"));
    assert!(
        snapshot.view_model_states.iter().any(|entry| {
            entry.key == "counter.count"
                && entry.value_summary == "1"
                && entry.type_name.contains("i32")
        }),
        "expected keyed count state in snapshot inventory: {:?}",
        snapshot.view_model_states
    );
    assert!(
        snapshot
            .view_model_states
            .iter()
            .any(|entry| entry.type_name.contains("ButtonState")),
        "expected button state in snapshot inventory: {:?}",
        snapshot.view_model_states
    );
}

#[test]
fn automation_session_click_targets_scrolled_descendants_using_visible_bounds() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let clicked = std::sync::Arc::new(std::sync::Mutex::new(None::<usize>));
    let clicked_for_ui = clicked.clone();
    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), move |ctx| {
        div().w(ctx.width).h(ctx.height).child(
            div()
                .id("scroll.host")
                .w(ctx.width)
                .h(120.0)
                .overflow_y_scroll()
                .child(div().flex_col().w_full().children((0..32).map(|index| {
                    let clicked = clicked_for_ui.clone();
                    div()
                        .id(format!("scroll.item.{index}"))
                        .w_full()
                        .h(40.0)
                        .on_click(move |_| {
                            *clicked.lock().expect("clicked slot should lock") = Some(index);
                        })
                        .child(text(format!("Item {index}")))
                }))),
        )
    });

    let before_scroll = session
        .absolute_bounds_for_id("scroll.item.15")
        .expect("target item should have bounds before scrolling");
    session
        .scroll(Some(AutomationLocator::id("scroll.host")), 0.0, -520.0)
        .expect("scroll should move the target into view");
    let after_scroll = session
        .absolute_bounds_for_id("scroll.item.15")
        .expect("target item should keep bounds after scrolling");
    assert!(
        after_scroll.y < before_scroll.y,
        "expected direct scroll to move the target upward: before={before_scroll:?} after={after_scroll:?}"
    );
    session
        .click(AutomationLocator::id("scroll.item.15"))
        .expect("click should use post-scroll bounds");

    assert_eq!(
        *clicked.lock().expect("clicked slot should lock"),
        Some(15),
        "expected click to land on the visible scrolled descendant"
    );
}

#[test]
fn automation_session_scroll_into_view_handles_nested_offsets_before_clicking() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};
    use blinc_layout::selector::{query, ScrollRef};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let clicked = std::sync::Arc::new(std::sync::Mutex::new(false));
    let clicked_for_ui = clicked.clone();
    let observed_scroll_ref = std::sync::Arc::new(std::sync::Mutex::new(None::<ScrollRef>));
    let observed_scroll_ref_for_ui = observed_scroll_ref.clone();
    let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), move |ctx| {
        let scroll_ref = ctx.use_scroll_ref("automation.nested.scroll");
        if observed_scroll_ref_for_ui
            .lock()
            .expect("scroll ref slot should lock")
            .is_none()
        {
            *observed_scroll_ref_for_ui
                .lock()
                .expect("scroll ref slot should lock") = Some(scroll_ref.clone());
        }

        div().w(ctx.width).h(ctx.height).child(
            scroll().bind(&scroll_ref).h(120.0).child(
                div()
                    .flex_col()
                    .child(div().h(32.0))
                    .children((0..10).map(|index| {
                        div()
                            .id(format!("nested.spacer.{index}"))
                            .h(36.0)
                            .child(text(format!("Spacer {index}")))
                    }))
                    .child(
                        div().pt(18.0).pl(12.0).child(
                            div()
                                .id("nested.target")
                                .h(40.0)
                                .on_click({
                                    let clicked = clicked_for_ui.clone();
                                    move |_| {
                                        *clicked.lock().expect("clicked slot should lock") = true;
                                    }
                                })
                                .child(text("Target")),
                        ),
                    ),
            ),
        )
    });

    query("nested.target")
        .expect("nested target should exist")
        .scroll_into_view();
    session
        .tick_frames(1)
        .expect("scroll_into_view should be processed during the next frame");
    session
        .click(AutomationLocator::id("nested.target"))
        .expect("nested target should be clickable after scroll_into_view");

    let offset = observed_scroll_ref
        .lock()
        .expect("scroll ref slot should lock")
        .as_ref()
        .expect("scroll ref should be captured")
        .offset()
        .1;
    assert!(offset > 0.0, "expected nested scroll_ref offset to advance");
    assert!(
        *clicked.lock().expect("clicked slot should lock"),
        "expected click after nested scroll_into_view to reach the target"
    );
}

#[test]
fn desktop_harness_preserves_existing_programmatic_event_callback() {
    use crate::{AutomationSession, HeadlessRunConfig};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let callback_hits = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let callback_hits_for_callback = callback_hits.clone();
    blinc_core::BlincContextState::get().set_programmatic_event_callback(std::sync::Arc::new(
        move |id, event| {
            callback_hits_for_callback
                .lock()
                .expect("callback hits should lock")
                .push(format!("{id}:{event:?}"));
        },
    ));

    {
        let _session =
            AutomationSession::new_desktop_harness(HeadlessRunConfig::default(), |ctx| {
                div().w(ctx.width).h(ctx.height).child(text("Ready"))
            });
        blinc_core::BlincContextState::get()
            .dispatch_programmatic_event("status", blinc_core::ProgrammaticElementEvent::Custom(7));
    }

    assert_eq!(
        callback_hits
            .lock()
            .expect("callback hits should lock")
            .len(),
        1,
        "desktop harness should not replace an existing programmatic event callback"
    );
}

#[test]
fn automation_session_restores_previous_thread_local_recorder() {
    use crate::{AutomationSession, HeadlessRunConfig};
    use blinc_recorder::{get_recorder, install_recorder, RecordingConfig, SharedRecordingSession};

    let _guard = automation_test_guard();
    ensure_automation_theme();

    let previous = std::sync::Arc::new(SharedRecordingSession::new(RecordingConfig::debug()));
    install_recorder(previous.clone());

    {
        let _session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
            div().w(ctx.width).h(ctx.height).child(text("Ready"))
        });
        let installed = get_recorder().expect("automation recorder should be installed");
        assert!(
            !std::sync::Arc::ptr_eq(&installed, &previous),
            "session should replace the active recorder while running"
        );
    }

    let restored = get_recorder().expect("previous recorder should be restored");
    assert!(std::sync::Arc::ptr_eq(&restored, &previous));
    blinc_recorder::uninstall_recorder();
}

#[test]
fn automation_session_keeps_global_keyed_state_isolated_from_session_override() {
    use crate::{AutomationLocator, AutomationSession, HeadlessRunConfig};

    let _guard = automation_test_guard();
    ensure_automation_theme();
    if !blinc_core::BlincContextState::is_initialized() {
        blinc_core::BlincContextState::init_with_callback(
            std::sync::Arc::new(std::sync::Mutex::new(blinc_core::ReactiveGraph::new())),
            std::sync::Arc::new(std::sync::Mutex::new(blinc_core::HookState::new())),
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            std::sync::Arc::new(|signal_ids| {
                blinc_layout::check_stateful_deps(signal_ids);
            }),
        );
    }

    let base_counter = blinc_core::context_state::use_state_keyed("global-counter", || 41i32);
    base_counter.set(42);

    {
        let mut session = AutomationSession::new_headless(HeadlessRunConfig::default(), |ctx| {
            let count = blinc_core::context_state::use_state_keyed("global-counter", || 0i32);
            div()
                .w(ctx.width)
                .h(ctx.height)
                .child(
                    div()
                        .id("increment")
                        .on_click({
                            let count = count.clone();
                            move |_| count.set(count.get() + 1)
                        })
                        .child(text("Increment")),
                )
                .child(
                    div()
                        .id("value")
                        .child(text(format!("Count: {}", count.get()))),
                )
        });

        session
            .click(AutomationLocator::id("increment"))
            .expect("session click should succeed");
        session
            .assert_text_contains(AutomationLocator::id("value"), "Count: 1")
            .expect("session should observe isolated automation state");
    }

    assert_eq!(
        base_counter.get(),
        42,
        "automation session should not clear or reuse the global keyed-state store"
    );
}

#[test]
fn render_tree_can_query_text_input_by_id() {
    let _guard = automation_test_guard();
    ensure_automation_theme();

    let email = text_input_state_with_placeholder("Email");
    let ui = div().child(
        text_input(&email)
            .id("login.email")
            .placeholder("Email")
            .w(240.0),
    );
    let mut tree = crate::RenderTree::from_element(&ui);
    tree.compute_layout(400.0, 200.0);

    assert!(
        tree.query_by_id("login.email").is_some(),
        "text input id should be queryable through the element registry"
    );
}
