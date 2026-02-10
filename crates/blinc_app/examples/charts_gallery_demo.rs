//! blinc_charts Gallery Demo
//!
//! Run with:
//! `cargo run -p blinc_app --example charts_gallery_demo --features windowed`
//!
//! This demo is a living gallery: every chart candidate gets at least one section.
//! Some sections are placeholders until their dedicated chart types land.

use blinc_app::demos::charts_gallery::{layout_mode, parse_initial_selected, LayoutMode};
use blinc_app::prelude::*;
use blinc_app::windowed::{WindowedApp, WindowedContext};
use blinc_charts::prelude::*;
use blinc_core::{Brush, Color, DrawContext, Point, Rect, TextStyle};
use blinc_layout::prelude::{ButtonState, NoState};

const ITEMS: &[(&str, &str)] = &[
    ("Line", "Ultra-large time series line"),
    ("Multi-line", "Many series with gap breaks"),
    ("Area", "Single-series area fill"),
    ("Bar / Stacked bar", "Stacked bars (screen-binned)"),
    ("Histogram", "Pre-binned histogram"),
    ("Scatter / Bubble", "Scatter points (capped primitives)"),
    ("Candlestick", "OHLC candles (screen-binned)"),
    ("Heatmap", "2D grid heatmap (screen-sampled)"),
    ("Area (stacked)", "Placeholder"),
    ("Density map / patch_map", "Placeholder"),
    ("Contour / Isobands", "Placeholder"),
    ("Boxplot / Violin / Error bands", "Placeholder"),
    ("Treemap / Sunburst / Icicle / Packing", "Placeholder"),
    ("Graph / Sankey / Chord", "Placeholder"),
    ("Parallel / Polar / Radar", "Placeholder"),
    ("Gauge / Funnel / Streamgraph", "Placeholder"),
    ("Geo", "Placeholder"),
];

#[derive(Clone)]
struct GalleryModels {
    line: LineChartHandle,
    multi: MultiLineChartHandle,
    area: AreaChartHandle,
    bar: BarChartHandle,
    hist: HistogramChartHandle,
    scatter: ScatterChartHandle,
    candle: CandlestickChartHandle,
    heat: HeatmapChartHandle,
}

impl GalleryModels {
    fn new(line_n: usize) -> Self {
        // Keep this initializer pure and deterministic (no context access).
        let line_series = make_series(line_n).expect("failed to create series (x must be sorted)");

        let line = LineChartHandle::new(LineChartModel::new(line_series.clone()));
        let area = AreaChartHandle::new(AreaChartModel::new(line_series.clone()));
        let scatter = ScatterChartHandle::new(ScatterChartModel::new(line_series.clone()));

        let multi = MultiLineChartHandle::new(
            MultiLineChartModel::new(make_multi_series(64, 240))
                .expect("multiline requires series"),
        );
        let bar = BarChartHandle::new(
            BarChartModel::new(make_bar_series(3, 3_000)).expect("bar requires series"),
        );
        let hist =
            HistogramChartHandle::new(HistogramChartModel::new(make_hist_values(100_000)).unwrap());
        let candle = CandlestickChartHandle::new(CandlestickChartModel::new(
            CandleSeries::new(make_candles(120_000)).expect("candles must be sorted"),
        ));
        let heat = HeatmapChartHandle::new(
            HeatmapChartModel::new(320, 160, make_heat_values(320, 160)).expect("valid heatmap"),
        );

        Self {
            line,
            multi,
            area,
            bar,
            hist,
            scatter,
            candle,
            heat,
        }
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = WindowConfig {
        title: "blinc_charts: Gallery".to_string(),
        width: 1200,
        height: 840,
        resizable: true,
        ..Default::default()
    };

    WindowedApp::run(config, |ctx| build_ui(ctx))
}

fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
    // Core datasets (kept deterministic; no RNG needed).
    let line_n = std::env::var("BLINC_CHARTS_N")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&v| v >= 2)
        .unwrap_or(200_000);

    let initial_selected = parse_initial_selected(
        std::env::var("BLINC_GALLERY_SELECTED").ok().as_deref(),
        ITEMS.len(),
    );
    let selected = ctx.use_state_keyed("charts_gallery_selected", move || initial_selected);

    let models = ctx.use_state_keyed("charts_gallery_models", || GalleryModels::new(line_n));

    let layout = layout_mode(ctx.width, ctx.height);

    let header = div()
        .flex_row()
        .items_end()
        .justify_between()
        .child(
            text("blinc_charts gallery")
                .size(24.0)
                .weight(FontWeight::Bold)
                .color(Color::rgba(0.95, 0.96, 0.98, 1.0)),
        )
        .child(
            text(format!("BLINC_CHARTS_N = {line_n}"))
                .size(12.0)
                .color(Color::rgba(0.70, 0.75, 0.82, 1.0)),
        );

    let sidebar = {
        let selected_for_list = selected.clone();
        let selected_signal = selected.signal_id();

        div()
            .w(280.0)
            .h_full()
            .rounded(14.0)
            .bg(Color::rgba(0.04, 0.05, 0.07, 1.0))
            .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.06))
            .p(10.0)
            .flex_col()
            .gap(8.0)
            .child(
                text("Charts")
                    .size(14.0)
                    .weight(FontWeight::SemiBold)
                    .color(Color::rgba(1.0, 1.0, 1.0, 0.80)),
            )
            .child(div().h(1.0).bg(Color::rgba(1.0, 1.0, 1.0, 0.06)))
            .child({
                let mut list = div().flex_1().overflow_y_scroll().flex_col().gap(6.0);
                for (i, (title, _desc)) in ITEMS.iter().enumerate() {
                    let title = *title;
                    let selected_for_state = selected_for_list.clone();
                    let selected_for_click = selected_for_list.clone();

                    list = list.child(
                        stateful::<ButtonState>()
                            .initial(ButtonState::Idle)
                            .deps([selected_signal])
                            .on_state(move |ctx| {
                                let is_selected = selected_for_state.get() == i;
                                let bg = if is_selected {
                                    Color::rgba(0.20, 0.35, 0.60, 0.55)
                                } else {
                                    match ctx.state() {
                                        ButtonState::Idle => Color::rgba(1.0, 1.0, 1.0, 0.04),
                                        ButtonState::Hovered => Color::rgba(1.0, 1.0, 1.0, 0.07),
                                        ButtonState::Pressed => Color::rgba(1.0, 1.0, 1.0, 0.10),
                                        ButtonState::Disabled => Color::rgba(1.0, 1.0, 1.0, 0.02),
                                    }
                                };
                                let fg = if is_selected {
                                    Color::rgba(0.95, 0.96, 0.98, 1.0)
                                } else {
                                    Color::rgba(0.85, 0.88, 0.92, 0.90)
                                };
                                div().px(10.0).py(8.0).rounded(10.0).bg(bg).child(
                                    text(title)
                                        .size(12.0)
                                        .weight(FontWeight::Medium)
                                        .color(fg)
                                        .no_wrap()
                                        .pointer_events_none(),
                                )
                            })
                            .on_click(move |_| {
                                selected_for_click.set(i);
                            }),
                    );
                }
                list
            })
    };

    let tabs = {
        let selected_for_list = selected.clone();
        let selected_signal = selected.signal_id();

        let mut list = div()
            .w_full()
            .overflow_x_scroll()
            .flex_row()
            .gap(8.0)
            .p(8.0)
            .rounded(14.0)
            .bg(Color::rgba(0.04, 0.05, 0.07, 1.0))
            .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.06));

        for (i, (title, _desc)) in ITEMS.iter().enumerate() {
            let title = *title;
            let selected_for_state = selected_for_list.clone();
            let selected_for_click = selected_for_list.clone();

            list = list.child(
                stateful::<ButtonState>()
                    .initial(ButtonState::Idle)
                    .deps([selected_signal])
                    .on_state(move |ctx| {
                        let is_selected = selected_for_state.get() == i;
                        let bg = if is_selected {
                            Color::rgba(0.20, 0.35, 0.60, 0.55)
                        } else {
                            match ctx.state() {
                                ButtonState::Idle => Color::rgba(1.0, 1.0, 1.0, 0.04),
                                ButtonState::Hovered => Color::rgba(1.0, 1.0, 1.0, 0.07),
                                ButtonState::Pressed => Color::rgba(1.0, 1.0, 1.0, 0.10),
                                ButtonState::Disabled => Color::rgba(1.0, 1.0, 1.0, 0.02),
                            }
                        };
                        let fg = if is_selected {
                            Color::rgba(0.95, 0.96, 0.98, 1.0)
                        } else {
                            Color::rgba(0.85, 0.88, 0.92, 0.90)
                        };
                        div().px(10.0).py(8.0).rounded(999.0).bg(bg).child(
                            text(title)
                                .size(12.0)
                                .weight(FontWeight::Medium)
                                .color(fg)
                                .no_wrap()
                                .pointer_events_none(),
                        )
                    })
                    .on_click(move |_| {
                        selected_for_click.set(i);
                    }),
            );
        }

        list
    };

    let main = {
        let selected_for_main = selected.clone();
        let selected_signal = selected.signal_id();

        let models = models.clone();

        stateful::<NoState>()
            .deps([selected_signal])
            .on_state(move |_ctx| {
                let idx = selected_for_main.get();
                let (title, desc) = ITEMS.get(idx).copied().unwrap_or(("Unknown", ""));
                let m = models.try_get().expect("models exist");

                div()
                    .flex_1()
                    .h_full()
                    .rounded(14.0)
                    .bg(Color::rgba(0.04, 0.05, 0.07, 1.0))
                    .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.06))
                    .p(10.0)
                    .flex_col()
                    .gap(10.0)
                    .child(
                        div()
                            .flex_col()
                            .gap(2.0)
                            .child(
                                text(title)
                                    .size(16.0)
                                    .weight(FontWeight::SemiBold)
                                    .color(Color::rgba(0.92, 0.93, 0.95, 1.0)),
                            )
                            .child(
                                text(desc)
                                    .size(12.0)
                                    .color(Color::rgba(0.68, 0.72, 0.78, 1.0)),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .rounded(14.0)
                            .overflow_clip()
                            .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08))
                            .child_box(chart_for(
                                idx, m.line, m.multi, m.area, m.bar, m.hist, m.scatter, m.candle,
                                m.heat,
                            )),
                    )
            })
    };

    let root = div()
        .w(ctx.width)
        .h(ctx.height)
        .bg(Color::rgba(0.06, 0.07, 0.09, 1.0))
        .p(12.0)
        .flex_col()
        .gap(12.0)
        .child(header);

    match layout {
        LayoutMode::Wide => root.child(
            div()
                .flex_1()
                .flex_row()
                .gap(12.0)
                .child(sidebar)
                .child(main),
        ),
        LayoutMode::Narrow => root.child(tabs).child(div().flex_1().child(main)),
    }
}

fn chart_for(
    idx: usize,
    line: LineChartHandle,
    multi: MultiLineChartHandle,
    area: AreaChartHandle,
    bar: BarChartHandle,
    hist: HistogramChartHandle,
    scatter: ScatterChartHandle,
    candle: CandlestickChartHandle,
    heat: HeatmapChartHandle,
) -> Box<dyn ElementBuilder> {
    match idx {
        0 => Box::new(line_chart(line)),
        1 => Box::new(multi_line_chart(multi)),
        2 => Box::new(area_chart(area)),
        3 => Box::new(bar_chart(bar)),
        4 => Box::new(histogram_chart(hist)),
        5 => Box::new(scatter_chart(scatter)),
        6 => Box::new(candlestick_chart(candle)),
        7 => Box::new(heatmap_chart(heat)),
        _ => Box::new(todo_canvas("TODO: not implemented yet")),
    }
}

fn todo_canvas(label: &'static str) -> impl ElementBuilder {
    div()
        .h(220.0)
        .rounded(14.0)
        .overflow_clip()
        .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08))
        .child(
            blinc_layout::canvas::canvas(move |ctx: &mut dyn DrawContext, bounds| {
                ctx.fill_rect(
                    Rect::new(0.0, 0.0, bounds.width, bounds.height),
                    0.0.into(),
                    Brush::Solid(Color::rgba(0.08, 0.09, 0.11, 1.0)),
                );
                let style = TextStyle::new(14.0).with_color(Color::rgba(1.0, 1.0, 1.0, 0.75));
                ctx.draw_text(label, Point::new(14.0, 14.0), &style);
            })
            .w_full()
            .h_full(),
        )
}

fn make_series(n: usize) -> anyhow::Result<TimeSeriesF32> {
    let mut x = Vec::with_capacity(n);
    let mut y = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 * 0.001;
        let v = (t * 1.0).sin() * 0.8 + (t * 0.13).sin() * 0.2 + (t.fract() - 0.5) * 0.05;
        x.push(i as f32);
        y.push(v);
    }
    TimeSeriesF32::new(x, y)
}

fn make_multi_series(series_n: usize, points_n: usize) -> Vec<TimeSeriesF32> {
    let mut out = Vec::with_capacity(series_n);
    for s in 0..series_n {
        let mut x = Vec::with_capacity(points_n);
        let mut y = Vec::with_capacity(points_n);
        let phase = s as f32 * 0.37;
        let mut cur_x = 0.0f32;
        for i in 0..points_n {
            // Force occasional gaps while keeping x sorted (monotonic).
            cur_x += if i % 37 == 0 && i != 0 { 9.0 } else { 1.0 };
            x.push(cur_x);
            let t = i as f32 * 0.06;
            let vv = (t + phase).sin() * 0.7 + (t * 0.17 + phase).sin() * 0.2;
            y.push(vv);
        }
        out.push(TimeSeriesF32::new(x, y).expect("sorted by construction"));
    }
    out
}

fn make_bar_series(series_n: usize, n: usize) -> Vec<TimeSeriesF32> {
    let mut out = Vec::with_capacity(series_n);
    for s in 0..series_n {
        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);
        let phase = s as f32 * 0.85;
        for i in 0..n {
            let t = i as f32 * 0.01;
            let v = ((t + phase).sin() * 0.5 + 0.6).max(0.0);
            x.push(i as f32);
            y.push(v);
        }
        out.push(TimeSeriesF32::new(x, y).expect("sorted by construction"));
    }
    out
}

fn make_hist_values(n: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(n);
    // Two-lobe distribution (deterministic).
    for i in 0..n {
        let t = i as f32 * 0.001;
        let a = (t * 1.7).sin() * 0.6 + (t * 0.11).sin() * 0.15;
        let b = (t * 0.9).cos() * 0.4;
        out.push(a + b);
    }
    out
}

fn make_candles(n: usize) -> Vec<Candle> {
    let mut out = Vec::with_capacity(n);
    let mut last = 0.0f32;
    for i in 0..n {
        let t = i as f32 * 0.01;
        let drift = (t * 0.23).sin() * 0.02;
        let noise = (t * 1.7).sin() * 0.03 + (t * 2.1).cos() * 0.01;
        let close = last + drift + noise;
        let open = last;
        let hi = open.max(close) + (t * 0.7).sin().abs() * 0.04;
        let lo = open.min(close) - (t * 0.9).cos().abs() * 0.04;
        out.push(Candle {
            x: i as f32,
            open,
            high: hi,
            low: lo,
            close,
        });
        last = close;
    }
    out
}

fn make_heat_values(w: usize, h: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    let cx = (w as f32) * 0.55;
    let cy = (h as f32) * 0.45;
    for y in 0..h {
        for x in 0..w {
            let dx = (x as f32 - cx) / (w as f32);
            let dy = (y as f32 - cy) / (h as f32);
            let r2 = dx * dx + dy * dy;
            let v = (-r2 * 38.0).exp() + (dx * 18.0).sin() * (dy * 14.0).cos() * 0.15;
            out[y * w + x] = v;
        }
    }
    out
}
