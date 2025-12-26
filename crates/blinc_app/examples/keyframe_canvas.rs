//! Keyframe Animation Canvas Demo
//!
//! Demonstrates keyframe animations with the canvas element for:
//! - Spinning loader with rotation keyframes
//! - Pulsing dots animation
//! - Progress bar with eased fill
//! - Bouncing ball animation
//!
//! Run with: cargo run -p blinc_app --example keyframe_canvas --features windowed

use blinc_animation::{AnimatedTimeline, Easing, KeyframeProperties, MultiKeyframeAnimation};
use blinc_app::prelude::*;
use blinc_app::windowed::{WindowedApp, WindowedContext};
use blinc_core::{Brush, Color, CornerRadius, DrawContext, Gradient, Point, Rect};
use std::cell::RefCell;
use std::f32::consts::PI;
use std::rc::Rc;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = WindowConfig {
        title: "Keyframe Canvas Animations".to_string(),
        width: 800,
        height: 600,
        ..Default::default()
    };

    WindowedApp::run(config, |ctx| build_ui(ctx))
}

fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
    div()
        .w(ctx.width)
        .h(ctx.height)
        .bg(Color::rgba(0.1, 0.1, 0.15, 1.0))
        .flex_col()
        .gap(20.0)
        .p(40.0)
        .items_center()
        .child(
            text("Keyframe Canvas Animations")
                .size(32.0)
                .weight(FontWeight::Bold)
                .color(Color::WHITE),
        )
        .child(
            text("Canvas elements with multi-property keyframe animations")
                .size(14.0)
                .color(Color::rgba(0.6, 0.6, 0.7, 1.0)),
        )
        .child(
            div()
                .flex_row()
                .gap(30.0)
                .flex_wrap()
                .justify_center()
                .child(spinning_loader_demo(ctx))
                .child(pulsing_dots_demo(ctx))
                .child(progress_bar_demo(ctx))
                .child(bouncing_ball_demo(ctx)),
        )
}

/// Demo 1: Spinning loader using rotation keyframes
fn spinning_loader_demo(ctx: &WindowedContext) -> Div {
    // Create a looping rotation animation using timeline
    let rotation = Rc::new(RefCell::new({
        let mut timeline = AnimatedTimeline::new(ctx.animation_handle());
        let entry = timeline.add(0, 1000, 0.0, 360.0); // 1 second full rotation
        timeline.set_loop(-1); // Infinite loop
        timeline.start();
        (timeline, entry)
    }));

    let render_rotation = Rc::clone(&rotation);

    demo_card("Spinning Loader").child(
        canvas(move |ctx: &mut dyn DrawContext, bounds| {
            let (timeline, entry) = &*render_rotation.borrow();
            let angle_deg = timeline.get(*entry).unwrap_or(0.0);
            let angle_rad = angle_deg * PI / 180.0;

            let cx = bounds.width / 2.0;
            let cy = bounds.height / 2.0;
            let radius = 30.0;
            let thickness = 4.0;

            // Draw spinning arc using line segments
            let arc_length = PI * 1.5; // 270 degrees
            let segments = 32;

            for i in 0..segments {
                let t1 = i as f32 / segments as f32;
                let t2 = (i + 1) as f32 / segments as f32;

                let a1 = angle_rad + t1 * arc_length;
                let a2 = angle_rad + t2 * arc_length;

                let x1 = cx + radius * a1.cos();
                let y1 = cy + radius * a1.sin();
                let x2 = cx + radius * a2.cos();
                let y2 = cy + radius * a2.sin();

                // Draw line segment as a thin rectangle
                let dx = x2 - x1;
                let dy = y2 - y1;
                let len = (dx * dx + dy * dy).sqrt();

                // Calculate alpha based on position in arc (fade effect)
                let alpha = 0.3 + 0.7 * t1;

                ctx.fill_rect(
                    Rect::new(
                        x1 - thickness / 2.0,
                        y1 - thickness / 2.0,
                        len + thickness,
                        thickness,
                    ),
                    CornerRadius::uniform(thickness / 2.0),
                    Brush::Solid(Color::rgba(0.4, 0.8, 1.0, alpha)),
                );
            }
        })
        .w(100.0)
        .h(100.0),
    )
}

/// Demo 2: Pulsing dots with staggered keyframes
fn pulsing_dots_demo(ctx: &WindowedContext) -> Div {
    // Create three dots with staggered pulse animations
    let dots: Vec<Rc<RefCell<(AnimatedTimeline, _, _)>>> = (0..3)
        .map(|i| {
            let mut timeline = AnimatedTimeline::new(ctx.animation_handle());
            // Stagger start by 200ms per dot
            let offset = i as i32 * 200;
            let scale_entry = timeline.add(offset, 600, 0.5, 1.0);
            let opacity_entry = timeline.add(offset, 600, 0.3, 1.0);
            timeline.set_loop(-1);
            timeline.start();
            Rc::new(RefCell::new((timeline, scale_entry, opacity_entry)))
        })
        .collect();

    let dots_clone = dots.clone();

    demo_card("Pulsing Dots").child(
        canvas(move |ctx: &mut dyn DrawContext, bounds| {
            let cx = bounds.width / 2.0;
            let cy = bounds.height / 2.0;
            let dot_radius = 8.0;
            let spacing = 25.0;

            for (i, dot) in dots_clone.iter().enumerate() {
                let (timeline, scale_entry, opacity_entry) = &*dot.borrow();
                let scale = timeline.get(*scale_entry).unwrap_or(1.0);
                let opacity = timeline.get(*opacity_entry).unwrap_or(1.0);

                let x = cx + (i as f32 - 1.0) * spacing;
                let r = dot_radius * scale;

                ctx.fill_rect(
                    Rect::new(x - r, cy - r, r * 2.0, r * 2.0),
                    CornerRadius::uniform(r),
                    Brush::Solid(Color::rgba(0.4, 1.0, 0.8, opacity)),
                );
            }
        })
        .w(100.0)
        .h(100.0),
    )
}

/// Demo 3: Progress bar with eased fill animation
fn progress_bar_demo(ctx: &WindowedContext) -> Div {
    // Keyframe animation for progress (with ease-in-out)
    let progress = Rc::new(RefCell::new({
        let mut timeline = AnimatedTimeline::new(ctx.animation_handle());
        // Create keyframes: slow start, fast middle, slow end
        let entry = timeline.add(0, 2000, 0.0, 1.0);
        timeline.start();
        (timeline, entry)
    }));

    let render_progress = Rc::clone(&progress);
    let click_progress = Rc::clone(&progress);

    demo_card("Progress Bar")
        .child(
            canvas(move |ctx: &mut dyn DrawContext, bounds| {
                let (timeline, entry) = &*render_progress.borrow();
                let progress_val = timeline.get(*entry).unwrap_or(0.0);

                let bar_width = bounds.width - 20.0;
                let bar_height = 12.0;
                let bar_x = 10.0;
                let bar_y = (bounds.height - bar_height) / 2.0;

                // Background
                ctx.fill_rect(
                    Rect::new(bar_x, bar_y, bar_width, bar_height),
                    CornerRadius::uniform(6.0),
                    Brush::Solid(Color::rgba(0.2, 0.2, 0.25, 1.0)),
                );

                // Filled portion
                let fill_width = bar_width * progress_val;
                if fill_width > 0.0 {
                    ctx.fill_rect(
                        Rect::new(bar_x, bar_y, fill_width, bar_height),
                        CornerRadius::uniform(6.0),
                        Brush::Gradient(Gradient::linear(
                            Point::new(bar_x, bar_y),
                            Point::new(bar_x + fill_width, bar_y),
                            Color::rgba(0.4, 0.8, 1.0, 1.0),
                            Color::rgba(0.6, 0.4, 1.0, 1.0),
                        )),
                    );
                }

                // Percentage text background
                let percent = (progress_val * 100.0) as i32;
                let text_x = bounds.width / 2.0 - 15.0;
                ctx.fill_rect(
                    Rect::new(text_x - 5.0, bar_y + bar_height + 5.0, 40.0, 16.0),
                    CornerRadius::uniform(4.0),
                    Brush::Solid(Color::rgba(0.15, 0.15, 0.2, 0.9)),
                );
            })
            .w(150.0)
            .h(60.0),
        )
        .child(
            text("Click to restart")
                .size(12.0)
                .color(Color::rgba(0.5, 0.5, 0.5, 1.0)),
        )
        .on_click(move |_| {
            // Restart animation on click
            let (timeline, _) = &*click_progress.borrow();
            timeline.restart();
        })
}

/// Demo 4: Bouncing ball with squash and stretch
fn bouncing_ball_demo(ctx: &WindowedContext) -> Div {
    // Bounce animation timeline
    let bounce = Rc::new(RefCell::new({
        let mut timeline = AnimatedTimeline::new(ctx.animation_handle());
        // Y position (0 = top, 1 = bottom)
        let y_entry = timeline.add(0, 800, 0.0, 1.0);
        timeline.set_loop(-1);
        timeline.start();
        (timeline, y_entry)
    }));

    let render_bounce = Rc::clone(&bounce);

    demo_card("Bouncing Ball").child(
        canvas(move |ctx: &mut dyn DrawContext, bounds| {
            let (timeline, y_entry) = &*render_bounce.borrow();
            let t = timeline.get(*y_entry).unwrap_or(0.0);

            let bounce_height = 50.0;
            let ground_y = bounds.height - 25.0;

            // Simple parabolic bounce
            let y = if t < 0.5 {
                // Falling (ease in - accelerating)
                let fall_t = t * 2.0;
                ground_y - bounce_height * (1.0 - fall_t * fall_t)
            } else {
                // Rising (ease out - decelerating)
                let rise_t = (t - 0.5) * 2.0;
                ground_y - bounce_height * (1.0 - (1.0 - rise_t) * (1.0 - rise_t))
            };

            // Squash/stretch based on velocity
            let (scale_x, scale_y) = if t < 0.45 || t > 0.55 {
                // In air - slight stretch
                (0.9, 1.1)
            } else {
                // Near ground - squash
                (1.2, 0.8)
            };

            let cx = bounds.width / 2.0;
            let radius = 15.0;

            // Draw shadow
            let shadow_scale = 1.0 - (ground_y - y) / bounce_height * 0.5;
            let shadow_width = radius * 2.0 * shadow_scale;
            let shadow_height = radius * 0.3 * 2.0 * shadow_scale;
            ctx.fill_rect(
                Rect::new(
                    cx - shadow_width / 2.0,
                    ground_y + 2.0,
                    shadow_width,
                    shadow_height,
                ),
                CornerRadius::uniform(shadow_height / 2.0),
                Brush::Solid(Color::rgba(0.0, 0.0, 0.0, 0.3 * shadow_scale)),
            );

            // Draw ball with squash/stretch
            let ball_width = radius * 2.0 * scale_x;
            let ball_height = radius * 2.0 * scale_y;
            ctx.fill_rect(
                Rect::new(
                    cx - ball_width / 2.0,
                    y - ball_height / 2.0,
                    ball_width,
                    ball_height,
                ),
                CornerRadius::uniform(ball_height.min(ball_width) / 2.0),
                Brush::Gradient(Gradient::linear(
                    Point::new(cx - ball_width / 2.0, y - ball_height / 2.0),
                    Point::new(cx + ball_width / 2.0, y + ball_height / 2.0),
                    Color::rgba(1.0, 0.5, 0.3, 1.0),
                    Color::rgba(0.9, 0.3, 0.2, 1.0),
                )),
            );
        })
        .w(100.0)
        .h(120.0),
    )
}

/// Helper to create a demo card
fn demo_card(title: &str) -> Div {
    div()
        .w(180.0)
        .flex_col()
        .gap(10.0)
        .p(16.0)
        .bg(Color::rgba(0.15, 0.15, 0.2, 1.0))
        .rounded(12.0)
        .items_center()
        .child(
            text(title)
                .size(14.0)
                .weight(FontWeight::SemiBold)
                .color(Color::WHITE),
        )
}
