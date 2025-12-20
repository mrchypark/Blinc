//! Path building and representation

use smallvec::SmallVec;

/// A 2D point
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Point = Point { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Path command
#[derive(Clone, Copy, Debug)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    QuadTo {
        control: Point,
        end: Point,
    },
    CubicTo {
        control1: Point,
        control2: Point,
        end: Point,
    },
    ArcTo {
        center: Point,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
    },
    Close,
}

/// A 2D path composed of commands
#[derive(Clone, Debug, Default)]
pub struct Path {
    commands: SmallVec<[PathCommand; 16]>,
}

impl Path {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn commands(&self) -> &[PathCommand] {
        &self.commands
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

/// Builder for constructing paths
pub struct PathBuilder {
    path: Path,
    current: Point,
}

impl PathBuilder {
    pub fn new() -> Self {
        Self {
            path: Path::new(),
            current: Point::ZERO,
        }
    }

    pub fn move_to(mut self, x: f32, y: f32) -> Self {
        let point = Point::new(x, y);
        self.path.commands.push(PathCommand::MoveTo(point));
        self.current = point;
        self
    }

    pub fn line_to(mut self, x: f32, y: f32) -> Self {
        let point = Point::new(x, y);
        self.path.commands.push(PathCommand::LineTo(point));
        self.current = point;
        self
    }

    pub fn quad_to(mut self, cx: f32, cy: f32, x: f32, y: f32) -> Self {
        let end = Point::new(x, y);
        self.path.commands.push(PathCommand::QuadTo {
            control: Point::new(cx, cy),
            end,
        });
        self.current = end;
        self
    }

    pub fn cubic_to(mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) -> Self {
        let end = Point::new(x, y);
        self.path.commands.push(PathCommand::CubicTo {
            control1: Point::new(c1x, c1y),
            control2: Point::new(c2x, c2y),
            end,
        });
        self.current = end;
        self
    }

    pub fn arc_to(mut self, cx: f32, cy: f32, radius: f32, start: f32, end: f32) -> Self {
        self.path.commands.push(PathCommand::ArcTo {
            center: Point::new(cx, cy),
            radius,
            start_angle: start,
            end_angle: end,
        });
        self
    }

    pub fn close(mut self) -> Self {
        self.path.commands.push(PathCommand::Close);
        self
    }

    pub fn build(self) -> Path {
        self.path
    }
}

impl Default for PathBuilder {
    fn default() -> Self {
        Self::new()
    }
}
