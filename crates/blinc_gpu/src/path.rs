//! Path tessellation for GPU rendering
//!
//! Converts vector paths into GPU-renderable triangle meshes using lyon.

use blinc_core::{Brush, Color, Path, PathCommand, Point, Stroke};
use lyon::lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};
use lyon::math::point;
use lyon::path::PathEvent;

/// A vertex for path rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PathVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

/// Tessellated path geometry ready for GPU rendering
#[derive(Default)]
pub struct TessellatedPath {
    pub vertices: Vec<PathVertex>,
    pub indices: Vec<u32>,
}

impl TessellatedPath {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() || self.indices.is_empty()
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
}

/// Convert blinc_core Path to lyon path events
fn path_to_lyon_events(path: &Path) -> Vec<PathEvent> {
    let mut events = Vec::new();
    let mut first_point: Option<Point> = None;
    let mut current_point = Point::new(0.0, 0.0);

    for cmd in path.commands() {
        match cmd {
            PathCommand::MoveTo(p) => {
                if first_point.is_some() {
                    // End previous subpath
                    events.push(PathEvent::End {
                        last: point(current_point.x, current_point.y),
                        first: point(first_point.unwrap().x, first_point.unwrap().y),
                        close: false,
                    });
                }
                events.push(PathEvent::Begin {
                    at: point(p.x, p.y),
                });
                first_point = Some(*p);
                current_point = *p;
            }
            PathCommand::LineTo(p) => {
                if first_point.is_none() {
                    // Implicit moveto at origin
                    events.push(PathEvent::Begin {
                        at: point(0.0, 0.0),
                    });
                    first_point = Some(Point::new(0.0, 0.0));
                }
                events.push(PathEvent::Line {
                    from: point(current_point.x, current_point.y),
                    to: point(p.x, p.y),
                });
                current_point = *p;
            }
            PathCommand::QuadTo { control, end } => {
                if first_point.is_none() {
                    events.push(PathEvent::Begin {
                        at: point(0.0, 0.0),
                    });
                    first_point = Some(Point::new(0.0, 0.0));
                }
                events.push(PathEvent::Quadratic {
                    from: point(current_point.x, current_point.y),
                    ctrl: point(control.x, control.y),
                    to: point(end.x, end.y),
                });
                current_point = *end;
            }
            PathCommand::CubicTo {
                control1,
                control2,
                end,
            } => {
                if first_point.is_none() {
                    events.push(PathEvent::Begin {
                        at: point(0.0, 0.0),
                    });
                    first_point = Some(Point::new(0.0, 0.0));
                }
                events.push(PathEvent::Cubic {
                    from: point(current_point.x, current_point.y),
                    ctrl1: point(control1.x, control1.y),
                    ctrl2: point(control2.x, control2.y),
                    to: point(end.x, end.y),
                });
                current_point = *end;
            }
            PathCommand::ArcTo {
                radii,
                rotation,
                large_arc,
                sweep,
                end,
            } => {
                if first_point.is_none() {
                    events.push(PathEvent::Begin {
                        at: point(0.0, 0.0),
                    });
                    first_point = Some(Point::new(0.0, 0.0));
                }
                // Lyon doesn't have direct arc support in PathEvent, so we approximate with cubic beziers
                // For a proper implementation, we'd use lyon's arc_to on a PathBuilder
                // For now, treat as a line to the endpoint
                events.push(PathEvent::Line {
                    from: point(current_point.x, current_point.y),
                    to: point(end.x, end.y),
                });
                let _ = (radii, rotation, large_arc, sweep); // Suppress warnings
                current_point = *end;
            }
            PathCommand::Close => {
                if let Some(first) = first_point {
                    events.push(PathEvent::End {
                        last: point(current_point.x, current_point.y),
                        first: point(first.x, first.y),
                        close: true,
                    });
                    first_point = None;
                }
            }
        }
    }

    // Close any remaining open subpath
    if let Some(first) = first_point {
        events.push(PathEvent::End {
            last: point(current_point.x, current_point.y),
            first: point(first.x, first.y),
            close: false,
        });
    }

    events
}

/// Tessellate a path for filling
pub fn tessellate_fill(path: &Path, brush: &Brush) -> TessellatedPath {
    let color = brush_to_color(brush);
    let events = path_to_lyon_events(path);

    if events.is_empty() {
        return TessellatedPath::new();
    }

    let mut geometry: VertexBuffers<PathVertex, u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    let options = FillOptions::default().with_tolerance(0.1);

    let result = tessellator.tessellate(
        events.iter().cloned(),
        &options,
        &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| PathVertex {
            position: vertex.position().to_array(),
            color: [color.r, color.g, color.b, color.a],
        }),
    );

    if result.is_err() {
        tracing::warn!("Path fill tessellation failed: {:?}", result.err());
        return TessellatedPath::new();
    }

    TessellatedPath {
        vertices: geometry.vertices,
        indices: geometry.indices,
    }
}

/// Tessellate a path for stroking
pub fn tessellate_stroke(path: &Path, stroke: &Stroke, brush: &Brush) -> TessellatedPath {
    let color = brush_to_color(brush);
    let events = path_to_lyon_events(path);

    if events.is_empty() {
        return TessellatedPath::new();
    }

    let mut geometry: VertexBuffers<PathVertex, u32> = VertexBuffers::new();
    let mut tessellator = StrokeTessellator::new();

    let mut options = StrokeOptions::default()
        .with_line_width(stroke.width)
        .with_tolerance(0.1);

    // Convert line cap
    options = options.with_line_cap(match stroke.cap {
        blinc_core::LineCap::Butt => lyon::lyon_tessellation::LineCap::Butt,
        blinc_core::LineCap::Round => lyon::lyon_tessellation::LineCap::Round,
        blinc_core::LineCap::Square => lyon::lyon_tessellation::LineCap::Square,
    });

    // Convert line join
    options = options.with_line_join(match stroke.join {
        blinc_core::LineJoin::Miter => lyon::lyon_tessellation::LineJoin::Miter,
        blinc_core::LineJoin::Round => lyon::lyon_tessellation::LineJoin::Round,
        blinc_core::LineJoin::Bevel => lyon::lyon_tessellation::LineJoin::Bevel,
    });

    options = options.with_miter_limit(stroke.miter_limit);

    let result = tessellator.tessellate(
        events.iter().cloned(),
        &options,
        &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| PathVertex {
            position: vertex.position().to_array(),
            color: [color.r, color.g, color.b, color.a],
        }),
    );

    if result.is_err() {
        tracing::warn!("Path stroke tessellation failed: {:?}", result.err());
        return TessellatedPath::new();
    }

    TessellatedPath {
        vertices: geometry.vertices,
        indices: geometry.indices,
    }
}

/// Extract solid color from brush (gradients not yet supported for paths)
fn brush_to_color(brush: &Brush) -> Color {
    match brush {
        Brush::Solid(color) => *color,
        Brush::Gradient(gradient) => {
            // Use first stop color as fallback
            gradient
                .stops()
                .first()
                .map(|s| s.color)
                .unwrap_or(Color::BLACK)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blinc_core::Rect;

    #[test]
    fn test_tessellate_rect() {
        let path = Path::rect(Rect::new(0.0, 0.0, 100.0, 100.0));
        let result = tessellate_fill(&path, &Color::RED.into());

        assert!(!result.is_empty());
        assert!(!result.vertices.is_empty());
        assert!(!result.indices.is_empty());
    }

    #[test]
    fn test_tessellate_circle() {
        let path = Path::circle(Point::new(50.0, 50.0), 25.0);
        let result = tessellate_fill(&path, &Color::BLUE.into());

        assert!(!result.is_empty());
    }

    #[test]
    fn test_tessellate_stroke() {
        let path = Path::line(Point::new(0.0, 0.0), Point::new(100.0, 100.0));
        let result = tessellate_stroke(&path, &Stroke::new(3.0), &Color::BLACK.into());

        assert!(!result.is_empty());
    }
}
