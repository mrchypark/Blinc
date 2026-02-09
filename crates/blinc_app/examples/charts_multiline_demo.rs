//! blinc_charts Multi-Line Demo (many series with gaps)
//!
//! Run with:
//! `cargo run -p blinc_app --example charts_multiline_demo --features windowed`
//!
//! Optional:
//! - `BLINC_CHARTS_SERIES` number of series (default: 1000)
//! - `BLINC_CHARTS_POINTS` nominal points per series (default: 160)

use blinc_app::prelude::*;
use blinc_app::windowed::{WindowedApp, WindowedContext};
use blinc_charts::prelude::*;
use blinc_core::Color;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = WindowConfig {
        title: "blinc_charts: Multi-Line Demo".to_string(),
        width: 1200,
        height: 720,
        resizable: true,
        ..Default::default()
    };

    WindowedApp::run(config, |ctx| build_ui(ctx))
}

fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
    let series_n = std::env::var("BLINC_CHARTS_SERIES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&v| v >= 1)
        .unwrap_or(1_000);

    let points_n = std::env::var("BLINC_CHARTS_POINTS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&v| v >= 8)
        .unwrap_or(160);

    let series = make_many_series(series_n, points_n);
    let mut model =
        MultiLineChartModel::new(series).expect("failed to create multi-line chart model");
    // "5-minute grid" feel in demo units: treat dx > 1.6 as a missing-sample gap.
    model.set_gap_dx(1.6);
    let handle = MultiLineChartHandle::new(model);

    div()
        .w(ctx.width)
        .h(ctx.height)
        .bg(Color::rgba(0.06, 0.07, 0.09, 1.0))
        .flex_col()
        .gap(10.0)
        .p(12.0)
        .child(
            div()
                .flex_row()
                .items_center()
                .gap(12.0)
                .child(
                    text("blinc_charts: multi-series line (gapped)")
                        .size(20.0)
                        .color(Color::rgba(0.95, 0.96, 0.98, 1.0)),
                )
                .child(
                    text(format!(
                        "series = {series_n}, nominal points/series = {points_n} (env: BLINC_CHARTS_SERIES / BLINC_CHARTS_POINTS)"
                    ))
                    .size(12.0)
                    .color(Color::rgba(0.7, 0.75, 0.82, 1.0)),
                ),
        )
        .child(
            div()
                .flex_1()
                .rounded(14.0)
                .overflow_clip()
                .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08))
                .child(multi_line_chart(handle)),
        )
}

fn make_many_series(series_n: usize, points_n: usize) -> Vec<TimeSeriesF32> {
    let mut out = Vec::with_capacity(series_n);

    for si in 0..series_n {
        // Deterministic pseudo-random seed.
        let mut seed = (si as u32)
            .wrapping_mul(0x9E37_79B9)
            .wrapping_add(0x7F4A_7C15);

        let mut x = Vec::with_capacity(points_n);
        let mut y = Vec::with_capacity(points_n);

        // x is in "steps" (like 5-minute slots), but we make it numeric.
        for ti in 0..points_n {
            // Sparse missingness: drop ~5% of samples.
            seed = xorshift32(seed);
            if (seed % 20) == 0 {
                continue;
            }

            let t = ti as f32 / (points_n.saturating_sub(1) as f32).max(1.0);

            // Daylight hump: sin(pi*t) with per-series scaling.
            seed = xorshift32(seed);
            let scale = 250.0 + (seed as f32 / u32::MAX as f32) * 250.0;
            let hump = (std::f32::consts::PI * t).sin().max(0.0).powf(1.4) * scale;

            // Midday spikes.
            seed = xorshift32(seed);
            let spike = ((t - 0.5) / 0.03).powi(2);
            let spike = (-spike).exp() * (seed as f32 / u32::MAX as f32) * 250.0;

            // Tiny deterministic jitter.
            seed = xorshift32(seed);
            let jitter = (seed as f32 / u32::MAX as f32 - 0.5) * 12.0;

            x.push(ti as f32);
            y.push(hump + spike + jitter);
        }

        // Ensure each series has at least 2 points so the chart has something to render.
        if x.len() < 2 {
            x.clear();
            y.clear();
            x.push(0.0);
            y.push(0.0);
            x.push(1.0);
            y.push(0.0);
        }

        out.push(TimeSeriesF32::new(x, y).expect("x must be sorted"));
    }

    out
}

fn xorshift32(mut x: u32) -> u32 {
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}
