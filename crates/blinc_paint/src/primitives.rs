//! Geometric primitives

use crate::path::Point;

/// A rectangle
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_points(p1: Point, p2: Point) -> Self {
        let x = p1.x.min(p2.x);
        let y = p1.y.min(p2.y);
        let width = (p2.x - p1.x).abs();
        let height = (p2.y - p1.y).abs();
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn origin(&self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }
}

/// A rounded rectangle
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct RoundedRect {
    pub rect: Rect,
    pub corner_radius: CornerRadius,
}

/// Corner radius for rounded rectangles
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct CornerRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadius {
    pub const fn uniform(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }
}

/// A circle
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct Circle {
    pub center: Point,
    pub radius: f32,
}

impl Circle {
    pub const fn new(center: Point, radius: f32) -> Self {
        Self { center, radius }
    }

    pub fn contains(&self, point: Point) -> bool {
        let dx = point.x - self.center.x;
        let dy = point.y - self.center.y;
        (dx * dx + dy * dy) <= (self.radius * self.radius)
    }
}

/// An ellipse
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct Ellipse {
    pub center: Point,
    pub radius_x: f32,
    pub radius_y: f32,
}

/// Shadow parameters
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Shadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub spread_radius: f32,
    pub color: crate::color::Color,
}

impl Shadow {
    pub const fn none() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            blur_radius: 0.0,
            spread_radius: 0.0,
            color: crate::color::Color::TRANSPARENT,
        }
    }

    pub fn sm() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 1.0,
            blur_radius: 2.0,
            spread_radius: 0.0,
            color: crate::color::Color::new(0.0, 0.0, 0.0, 0.1),
        }
    }

    pub fn md() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 4.0,
            blur_radius: 6.0,
            spread_radius: -1.0,
            color: crate::color::Color::new(0.0, 0.0, 0.0, 0.1),
        }
    }

    pub fn lg() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 10.0,
            blur_radius: 15.0,
            spread_radius: -3.0,
            color: crate::color::Color::new(0.0, 0.0, 0.0, 0.1),
        }
    }
}
