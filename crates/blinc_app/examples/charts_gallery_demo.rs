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
    (
        "Area (stacked)",
        "Aligned stacked area / streamgraph (pan/zoom/brush)",
    ),
    (
        "Density map / patch_map",
        "2D histogram density (pan/zoom/brush)",
    ),
    (
        "Contour / Isobands",
        "Marching-squares contours (pan/zoom/brush)",
    ),
    (
        "Boxplot / Violin / Error bands",
        "Boxplot per group (pan/zoom/brush)",
    ),
    (
        "Treemap / Sunburst / Icicle / Packing",
        "Hierarchy layouts (hover)",
    ),
    ("Graph / Sankey / Chord", "Network layouts (pan/zoom/hover)"),
    ("Parallel / Polar / Radar", "Radar chart (hover)"),
    ("Gauge / Funnel / Streamgraph", "Gauge + funnel"),
    ("Geo", "Shape rendering (pan/zoom/hover)"),
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
    stacked_area: StackedAreaChartHandle,
    density: DensityMapChartHandle,
    contour: ContourChartHandle,
    stats: StatisticsChartHandle,
    hierarchy: HierarchyChartHandle,
    network: NetworkChartHandle,
    polar: PolarChartHandle,
    gauge: GaugeChartHandle,
    funnel: FunnelChartHandle,
    geo: GeoChartHandle,
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

        let stacked_area = StackedAreaChartHandle::new(
            StackedAreaChartModel::new(make_stacked_series(6, (line_n / 2).max(20_000)))
                .expect("stacked area requires aligned series"),
        );
        let density = DensityMapChartHandle::new(
            DensityMapChartModel::new(make_density_points((line_n / 2).max(60_000)))
                .expect("density requires points"),
        );
        let contour = ContourChartHandle::new(
            ContourChartModel::new(240, 120, make_contour_values(240, 120)).expect("contour grid"),
        );
        let stats = StatisticsChartHandle::new(
            StatisticsChartModel::new(make_statistics_groups(18, 600)).expect("stats groups"),
        );
        let hierarchy = HierarchyChartHandle::new(
            HierarchyChartModel::new(make_hierarchy_tree()).expect("hierarchy tree"),
        );
        let network = NetworkChartHandle::new(
            NetworkChartModel::new_graph(make_graph_labels(48), make_graph_edges(48))
                .expect("network graph"),
        );
        let polar = PolarChartHandle::new(
            PolarChartModel::new_radar(make_radar_dimensions(), make_radar_series())
                .expect("radar data"),
        );
        let gauge =
            GaugeChartHandle::new(GaugeChartModel::new(0.0, 100.0, 72.0).expect("gauge model"));
        let funnel = FunnelChartHandle::new(
            FunnelChartModel::new(make_funnel_stages()).expect("funnel stages"),
        );
        let geo = GeoChartHandle::new(GeoChartModel::new(make_geo_shapes()).expect("geo shapes"));

        Self {
            line,
            multi,
            area,
            bar,
            hist,
            scatter,
            candle,
            heat,
            stacked_area,
            density,
            contour,
            stats,
            hierarchy,
            network,
            polar,
            gauge,
            funnel,
            geo,
        }
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let (mut width, mut height) = (1200u32, 840u32);
    if let Ok(v) = std::env::var("BLINC_WINDOW_SIZE") {
        if let Some((w, h)) = v.split_once('x') {
            if let (Ok(w), Ok(h)) = (w.trim().parse::<u32>(), h.trim().parse::<u32>()) {
                if w >= 320 && h >= 240 {
                    width = w;
                    height = h;
                }
            }
        }
    }

    let config = WindowConfig {
        title: "blinc_charts: Gallery".to_string(),
        width,
        height,
        resizable: true,
        ..Default::default()
    };

    WindowedApp::run(config, |ctx| build_ui(ctx))
}

fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
    if std::env::var_os("BLINC_DEBUG_GALLERY_CTX").is_some() {
        tracing::info!(
            "charts_gallery_demo: ctx.width={:.1} ctx.height={:.1} scale_factor={:.3}",
            ctx.width,
            ctx.height,
            ctx.scale_factor
        );
    }
    if std::env::var_os("BLINC_DEBUG_GALLERY_BOUNDS").is_some() {
        use std::sync::atomic::{AtomicBool, Ordering};
        static REGISTERED: AtomicBool = AtomicBool::new(false);
        if !REGISTERED.swap(true, Ordering::Relaxed) {
            ctx.query("charts_gallery_tabs").on_ready(|b| {
                tracing::info!(
                    "charts_gallery_demo: tabs bounds x={:.1} y={:.1} w={:.1} h={:.1}",
                    b.x,
                    b.y,
                    b.width,
                    b.height
                );
            });
            ctx.query("charts_gallery_main").on_ready(|b| {
                tracing::info!(
                    "charts_gallery_demo: main bounds x={:.1} y={:.1} w={:.1} h={:.1}",
                    b.x,
                    b.y,
                    b.width,
                    b.height
                );
            });
            ctx.query("charts_gallery_canvas").on_ready(|b| {
                tracing::info!(
                    "charts_gallery_demo: canvas bounds x={:.1} y={:.1} w={:.1} h={:.1}",
                    b.x,
                    b.y,
                    b.width,
                    b.height
                );
            });
            ctx.query("charts_gallery_sidebar").on_ready(|b| {
                tracing::info!(
                    "charts_gallery_demo: sidebar bounds x={:.1} y={:.1} w={:.1} h={:.1}",
                    b.x,
                    b.y,
                    b.width,
                    b.height
                );
            });
            ctx.query("charts_gallery_sidebar_scroll").on_ready(|b| {
                tracing::info!(
                    "charts_gallery_demo: sidebar_scroll bounds x={:.1} y={:.1} w={:.1} h={:.1}",
                    b.x,
                    b.y,
                    b.width,
                    b.height
                );
            });
            ctx.query("charts_gallery_sidebar_item_0").on_ready(|b| {
                tracing::info!(
                    "charts_gallery_demo: sidebar_item_0 bounds x={:.1} y={:.1} w={:.1} h={:.1}",
                    b.x,
                    b.y,
                    b.width,
                    b.height
                );
            });
            ctx.query("charts_gallery_sidebar_item_10").on_ready(|b| {
                tracing::info!(
                    "charts_gallery_demo: sidebar_item_10 bounds x={:.1} y={:.1} w={:.1} h={:.1}",
                    b.x,
                    b.y,
                    b.width,
                    b.height
                );
            });
        }
    }

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
    let dense = ctx.height < 460.0;

    let header = div()
        .flex_row()
        .items_end()
        .justify_between()
        .child(
            text("blinc_charts gallery")
                .size(if dense { 18.0 } else { 24.0 })
                .weight(FontWeight::Bold)
                .color(Color::rgba(0.95, 0.96, 0.98, 1.0))
                .no_wrap(),
        )
        .child(
            text(format!("BLINC_CHARTS_N = {line_n}"))
                .size(if dense { 10.0 } else { 12.0 })
                .color(Color::rgba(0.70, 0.75, 0.82, 1.0))
                .no_wrap(),
        );

    let sidebar = {
        let selected_for_list = selected.clone();
        let selected_signal = selected.signal_id();

        div()
            .id("charts_gallery_sidebar")
            .w(280.0)
            .h_full()
            .rounded(14.0)
            .bg(Color::rgba(0.04, 0.05, 0.07, 1.0))
            .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.06))
            .p(if dense { 4.0 } else { 10.0 })
            .flex_col()
            .gap(if dense { 3.0 } else { 8.0 })
            .child(
                text("Charts")
                    .size(14.0)
                    .weight(FontWeight::SemiBold)
                    .color(Color::rgba(1.0, 1.0, 1.0, 0.80)),
            )
            .child(div().h(1.0).bg(Color::rgba(1.0, 1.0, 1.0, 0.06)))
            .child({
                // In a flex column, allow the scroll container to shrink below its content size.
                // (Similar to CSS `min-height: 0` for scrollables inside flex.)
                let mut list = div().flex_col().gap(6.0);
                for (i, (title, _desc)) in ITEMS.iter().enumerate() {
                    let title = *title;
                    let item_id = format!("charts_gallery_sidebar_item_{i}");
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
                                div()
                                    .id(item_id.clone())
                                    .px(10.0)
                                    .py(8.0)
                                    .rounded(10.0)
                                    .bg(bg)
                                    .child(
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
                div()
                    .flex_1()
                    .min_h(0.0)
                    .w_full()
                    .id("charts_gallery_sidebar_scroll")
                    .overflow_y_scroll()
                    .child(list)
            })
    };

    let tabs = {
        let selected_for_list = selected.clone();
        let selected_signal = selected.signal_id();

        // In narrow layout, the pill list can get tall and squeeze the plot area.
        // When the window height is short, keep tabs to a single row (horizontal scroll)
        // so the main chart remains visible.
        let compact_tabs = dense;

        let mut list = div()
            .id("charts_gallery_tabs")
            .w_full()
            .flex_row()
            .gap(if dense { 2.0 } else { 8.0 })
            .p(if dense { 2.0 } else { 8.0 })
            .rounded(14.0)
            .bg(Color::rgba(0.04, 0.05, 0.07, 1.0))
            .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.06));

        if compact_tabs {
            // Keep tabs short in small-height windows so the chart stays visible.
            // Use a fixed height to prevent accidental wrapping from consuming vertical space.
            list = list.max_h(52.0).overflow_x_scroll();
        } else {
            // In taller layouts, show pills in multiple rows and allow vertical wheel scrolling
            // so users can reach items beyond the first row.
            list = list.max_h(140.0).overflow_y_scroll().flex_wrap();
        }

        for (i, (title, _desc)) in ITEMS.iter().enumerate() {
            let title = *title;
            let item_id = format!("charts_gallery_tab_item_{i}");
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
                        div()
                            .id(item_id.clone())
                            .px(if dense { 4.0 } else { 10.0 })
                            .py(if dense { 3.0 } else { 8.0 })
                            .rounded(999.0)
                            .bg(bg)
                            .child(
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
                    .id("charts_gallery_main")
                    .flex_1()
                    .h_full()
                    .rounded(14.0)
                    .bg(Color::rgba(0.04, 0.05, 0.07, 1.0))
                    .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.06))
                    .p(if dense { 4.0 } else { 10.0 })
                    .flex_col()
                    .gap(if dense { 3.0 } else { 10.0 })
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
                            .id("charts_gallery_canvas")
                            .flex_1()
                            .rounded(14.0)
                            .overflow_clip()
                            .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08))
                            .child_box(chart_for(
                                idx,
                                m.line,
                                m.multi,
                                m.area,
                                m.bar,
                                m.hist,
                                m.scatter,
                                m.candle,
                                m.heat,
                                m.stacked_area,
                                m.density,
                                m.contour,
                                m.stats,
                                m.hierarchy,
                                m.network,
                                m.polar,
                                m.gauge,
                                m.funnel,
                                m.geo,
                            )),
                    )
            })
    };

    let root = div()
        .w(ctx.width)
        .h(ctx.height)
        .bg(Color::rgba(0.06, 0.07, 0.09, 1.0))
        .p(if dense { 3.0 } else { 12.0 })
        .flex_col()
        .gap(if dense { 3.0 } else { 12.0 })
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
    stacked_area: StackedAreaChartHandle,
    density: DensityMapChartHandle,
    contour: ContourChartHandle,
    stats: StatisticsChartHandle,
    hierarchy: HierarchyChartHandle,
    network: NetworkChartHandle,
    polar: PolarChartHandle,
    gauge: GaugeChartHandle,
    funnel: FunnelChartHandle,
    geo: GeoChartHandle,
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
        8 => Box::new(stacked_area_chart(stacked_area)),
        9 => Box::new(density_map_chart(density)),
        10 => Box::new(contour_chart(contour)),
        11 => Box::new(statistics_chart(stats)),
        12 => Box::new(hierarchy_chart(hierarchy)),
        13 => Box::new(network_chart(network)),
        14 => Box::new(polar_chart(polar)),
        15 => Box::new(
            div()
                .w_full()
                .h_full()
                .flex_row()
                .gap(10.0)
                .child(
                    div()
                        .flex_1()
                        .h_full()
                        .child_box(Box::new(gauge_chart(gauge))),
                )
                .child(
                    div()
                        .flex_1()
                        .h_full()
                        .child_box(Box::new(funnel_chart(funnel))),
                ),
        ),
        16 => Box::new(geo_chart(geo)),
        _ => Box::new(todo_canvas("TODO: unknown index")),
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

fn make_stacked_series(series_n: usize, n: usize) -> Vec<TimeSeriesF32> {
    let n = n.max(2);
    let mut x = Vec::with_capacity(n);
    for i in 0..n {
        x.push(i as f32);
    }

    let mut out = Vec::with_capacity(series_n.max(1));
    for s in 0..series_n.max(1) {
        let mut y = Vec::with_capacity(n);
        let phase = s as f32 * 0.7;
        for i in 0..n {
            let t = i as f32 * 0.01;
            let v = (t + phase).sin() * 0.7 + (t * 0.17 + phase).sin() * 0.3 + 1.2;
            y.push(v.max(0.0));
        }
        out.push(TimeSeriesF32::new(x.clone(), y).expect("sorted by construction"));
    }
    out
}

fn make_density_points(n: usize) -> Vec<Point> {
    let n = n.max(1);
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 * 0.0025;
        // Two-lobe mixture with gentle warp (deterministic).
        let (cx, cy) = if i % 2 == 0 {
            (0.4, 0.55)
        } else {
            (0.62, 0.45)
        };
        let dx = (t * 3.1).sin() * 0.22 + (t * 0.17).cos() * 0.08;
        let dy = (t * 2.7).cos() * 0.18 + (t * 0.13).sin() * 0.07;
        let x = cx + dx + (t * 0.9).sin() * 0.03;
        let y = cy + dy + (t * 1.1).cos() * 0.03;
        out.push(Point::new(x, y));
    }
    out
}

fn make_contour_values(w: usize, h: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    let cx = (w as f32) * 0.62;
    let cy = (h as f32) * 0.46;
    for y in 0..h {
        for x in 0..w {
            let dx = (x as f32 - cx) / (w as f32);
            let dy = (y as f32 - cy) / (h as f32);
            let r2 = dx * dx + dy * dy;
            let bump = (-r2 * 28.0).exp();
            let ripple = (dx * 14.0).sin() * (dy * 10.0).cos() * 0.25;
            out[y * w + x] = (bump + ripple) * 2.0 - 1.0;
        }
    }
    out
}

fn make_statistics_groups(groups_n: usize, points_per_group: usize) -> Vec<Vec<f32>> {
    let groups_n = groups_n.max(1);
    let points_per_group = points_per_group.max(8);
    let mut out = Vec::with_capacity(groups_n);
    for g in 0..groups_n {
        let mut vals = Vec::with_capacity(points_per_group);
        let shift = (g as f32) * 0.15;
        let spread = 0.4 + (g as f32 * 0.03).sin().abs() * 0.25;
        for i in 0..points_per_group {
            let t = i as f32 * 0.07;
            let v = (t + shift).sin() * spread + (t * 0.21 + shift).cos() * 0.15 + shift * 0.6;
            vals.push(v);
        }
        out.push(vals);
    }
    out
}

fn make_hierarchy_tree() -> HierarchyNode {
    HierarchyNode::node(
        "root",
        vec![
            HierarchyNode::node(
                "A",
                vec![
                    HierarchyNode::leaf("A-1", 6.0),
                    HierarchyNode::leaf("A-2", 2.0),
                    HierarchyNode::leaf("A-3", 4.0),
                ],
            ),
            HierarchyNode::node(
                "B",
                vec![
                    HierarchyNode::leaf("B-1", 3.0),
                    HierarchyNode::leaf("B-2", 7.0),
                    HierarchyNode::leaf("B-3", 1.5),
                    HierarchyNode::leaf("B-4", 2.2),
                ],
            ),
            HierarchyNode::node(
                "C",
                vec![
                    HierarchyNode::leaf("C-1", 4.5),
                    HierarchyNode::leaf("C-2", 1.2),
                    HierarchyNode::leaf("C-3", 3.4),
                ],
            ),
        ],
    )
}

fn make_graph_labels(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("N{i}")).collect()
}

fn make_graph_edges(n: usize) -> Vec<(usize, usize)> {
    let n = n.max(2);
    let mut out = Vec::new();
    // Ring
    for i in 0..n {
        out.push((i, (i + 1) % n));
    }
    // Chords
    for i in 0..n {
        if i % 3 == 0 {
            out.push((i, (i + 7) % n));
        }
        if i % 5 == 0 {
            out.push((i, (i + 13) % n));
        }
    }
    out
}

fn make_radar_dimensions() -> Vec<String> {
    vec![
        "Quality".into(),
        "Speed".into(),
        "Cost".into(),
        "Reliability".into(),
        "Scale".into(),
        "Latency".into(),
    ]
}

fn make_radar_series() -> Vec<Vec<f32>> {
    vec![
        vec![0.72, 0.81, 0.42, 0.66, 0.58, 0.76],
        vec![0.55, 0.62, 0.73, 0.51, 0.77, 0.48],
        vec![0.83, 0.44, 0.61, 0.74, 0.46, 0.69],
    ]
}

fn make_funnel_stages() -> Vec<(String, f32)> {
    vec![
        ("Visits".into(), 12_500.0),
        ("Signups".into(), 3_400.0),
        ("Trials".into(), 1_250.0),
        ("Paid".into(), 480.0),
        ("Renew".into(), 320.0),
    ]
}

fn make_geo_shapes() -> Vec<Vec<Point>> {
    // A couple of simple polygons/lines in arbitrary "geo" coords.
    let mut shapes = Vec::new();

    // Coast-like polyline
    let mut coast = Vec::new();
    for i in 0..220 {
        let t = i as f32 / 219.0;
        let x = t * 10.0;
        let y = (t * std::f32::consts::TAU * 1.7).sin() * 0.9
            + (t * std::f32::consts::TAU * 4.3).sin() * 0.25;
        coast.push(Point::new(x, y));
    }
    shapes.push(coast);

    // Closed island polygon
    let mut island = Vec::new();
    let cx = 6.8;
    let cy = -1.6;
    for i in 0..=64 {
        let a = i as f32 / 64.0 * std::f32::consts::TAU;
        island.push(Point::new(
            cx + a.cos() * 1.2,
            cy + a.sin() * 0.7 + (a * 3.0).sin() * 0.08,
        ));
    }
    shapes.push(island);

    shapes
}
