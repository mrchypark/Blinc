//! blinc_charts
//!
//! Canvas-first, GPU-accelerated, interactive charts for Blinc.
//!
//! Design goals (initial):
//! - Compose naturally with Blinc's layout tree (Stack overlays, Canvas rendering)
//! - Use Blinc's built-in event routing (mouse/touch/scroll/pinch/drag)
//! - Prioritize performance for large datasets via sampling/LOD and GPU pipelines

mod brush;
mod input;
mod link;
mod lod;
mod segments;
mod time_series;
mod view;

pub mod line;
pub mod multi_line;

pub use brush::BrushX;
pub use input::{ChartInputBindings, DragAction, DragBinding, ModifiersReq};
pub use link::{chart_link, ChartLink, ChartLinkHandle};
pub use lod::{downsample_min_max, DownsampleParams};
pub use segments::runs_by_gap;
pub use time_series::TimeSeriesF32;
pub use view::{ChartView, Domain1D, Domain2D};

/// Common imports for chart users.
pub mod prelude {
    pub use crate::input::{ChartInputBindings, DragAction, DragBinding, ModifiersReq};
    pub use crate::line::{
        line_chart, line_chart_with_bindings, linked_line_chart, linked_line_chart_with_bindings,
        LineChartHandle, LineChartModel, LineChartStyle,
    };
    pub use crate::link::{chart_link, ChartLink, ChartLinkHandle};
    pub use crate::multi_line::{
        linked_multi_line_chart, linked_multi_line_chart_with_bindings, multi_line_chart,
        multi_line_chart_with_bindings, MultiLineChartHandle, MultiLineChartModel,
        MultiLineChartStyle,
    };
    pub use crate::time_series::TimeSeriesF32;
    pub use crate::view::{ChartView, Domain1D, Domain2D};
}
