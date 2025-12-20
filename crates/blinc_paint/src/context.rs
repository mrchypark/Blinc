//! Paint context - the main drawing API

use crate::color::Color;
use crate::gradient::Gradient;
use crate::path::{Path, Point};
use crate::primitives::*;

/// Fill style for shapes
#[derive(Clone, Debug)]
pub enum FillStyle {
    Color(Color),
    Gradient(Gradient),
}

impl From<Color> for FillStyle {
    fn from(color: Color) -> Self {
        FillStyle::Color(color)
    }
}

impl From<Gradient> for FillStyle {
    fn from(gradient: Gradient) -> Self {
        FillStyle::Gradient(gradient)
    }
}

/// Stroke style
#[derive(Clone, Debug)]
pub struct StrokeStyle {
    pub color: Color,
    pub width: f32,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

/// A paint command for the renderer
#[derive(Clone, Debug)]
pub enum PaintCommand {
    FillRect {
        rect: Rect,
        style: FillStyle,
    },
    StrokeRect {
        rect: Rect,
        style: StrokeStyle,
    },
    FillRoundedRect {
        rect: RoundedRect,
        style: FillStyle,
    },
    StrokeRoundedRect {
        rect: RoundedRect,
        style: StrokeStyle,
    },
    FillCircle {
        circle: Circle,
        style: FillStyle,
    },
    StrokeCircle {
        circle: Circle,
        style: StrokeStyle,
    },
    FillPath {
        path: Path,
        style: FillStyle,
    },
    StrokePath {
        path: Path,
        style: StrokeStyle,
    },
    DrawShadow {
        shape: ShadowShape,
        shadow: Shadow,
    },
    DrawText {
        text: String,
        position: Point,
        size: f32,
        color: Color,
    },
    PushClip {
        rect: Rect,
    },
    PopClip,
    PushTransform {
        transform: Transform2D,
    },
    PopTransform,
}

#[derive(Clone, Debug)]
pub enum ShadowShape {
    Rect(Rect),
    RoundedRect(RoundedRect),
    Circle(Circle),
}

/// 2D affine transform
#[derive(Clone, Copy, Debug)]
pub struct Transform2D {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform2D {
    pub const fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: x,
            f: y,
        }
    }

    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: 0.0,
            f: 0.0,
        }
    }

    pub fn scale_uniform(s: f32) -> Self {
        Self::scale(s, s)
    }

    pub fn rotate(angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: 0.0,
            f: 0.0,
        }
    }
}

/// The paint context used for custom drawing
pub struct PaintContext {
    commands: Vec<PaintCommand>,
    current_path: Path,
    transform_stack: Vec<Transform2D>,
    clip_stack: Vec<Rect>,
}

impl PaintContext {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            current_path: Path::new(),
            transform_stack: Vec::new(),
            clip_stack: Vec::new(),
        }
    }

    /// Get all recorded commands
    pub fn commands(&self) -> &[PaintCommand] {
        &self.commands
    }

    /// Take ownership of recorded commands
    pub fn take_commands(&mut self) -> Vec<PaintCommand> {
        std::mem::take(&mut self.commands)
    }

    // === Shape drawing ===

    pub fn fill_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        style: impl Into<FillStyle>,
    ) {
        self.commands.push(PaintCommand::FillRect {
            rect: Rect::new(x, y, width, height),
            style: style.into(),
        });
    }

    pub fn stroke_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        width_: f32,
    ) {
        self.commands.push(PaintCommand::StrokeRect {
            rect: Rect::new(x, y, width, height),
            style: StrokeStyle {
                color,
                width: width_,
                ..Default::default()
            },
        });
    }

    pub fn fill_rounded_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
        style: impl Into<FillStyle>,
    ) {
        self.commands.push(PaintCommand::FillRoundedRect {
            rect: RoundedRect {
                rect: Rect::new(x, y, width, height),
                corner_radius: CornerRadius::uniform(radius),
            },
            style: style.into(),
        });
    }

    pub fn fill_circle(&mut self, cx: f32, cy: f32, radius: f32, style: impl Into<FillStyle>) {
        self.commands.push(PaintCommand::FillCircle {
            circle: Circle::new(Point::new(cx, cy), radius),
            style: style.into(),
        });
    }

    pub fn stroke_circle(&mut self, cx: f32, cy: f32, radius: f32, color: Color, width: f32) {
        self.commands.push(PaintCommand::StrokeCircle {
            circle: Circle::new(Point::new(cx, cy), radius),
            style: StrokeStyle {
                color,
                width,
                ..Default::default()
            },
        });
    }

    // === Path drawing ===

    pub fn fill_path(&mut self, path: Path, style: impl Into<FillStyle>) {
        self.commands.push(PaintCommand::FillPath {
            path,
            style: style.into(),
        });
    }

    pub fn stroke_path(&mut self, path: Path, color: Color, width: f32) {
        self.commands.push(PaintCommand::StrokePath {
            path,
            style: StrokeStyle {
                color,
                width,
                ..Default::default()
            },
        });
    }

    // === Text ===

    pub fn draw_text(&mut self, text: impl Into<String>, x: f32, y: f32, size: f32, color: Color) {
        self.commands.push(PaintCommand::DrawText {
            text: text.into(),
            position: Point::new(x, y),
            size,
            color,
        });
    }

    // === Shadows ===

    pub fn draw_shadow(&mut self, rect: Rect, shadow: Shadow) {
        self.commands.push(PaintCommand::DrawShadow {
            shape: ShadowShape::Rect(rect),
            shadow,
        });
    }

    // === Clipping ===

    pub fn push_clip(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let rect = Rect::new(x, y, width, height);
        self.clip_stack.push(rect);
        self.commands.push(PaintCommand::PushClip { rect });
    }

    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
        self.commands.push(PaintCommand::PopClip);
    }

    // === Transforms ===

    pub fn push_transform(&mut self, transform: Transform2D) {
        self.transform_stack.push(transform);
        self.commands
            .push(PaintCommand::PushTransform { transform });
    }

    pub fn pop_transform(&mut self) {
        self.transform_stack.pop();
        self.commands.push(PaintCommand::PopTransform);
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.push_transform(Transform2D::translate(x, y));
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.push_transform(Transform2D::scale(sx, sy));
    }

    pub fn rotate(&mut self, angle: f32) {
        self.push_transform(Transform2D::rotate(angle));
    }
}

impl Default for PaintContext {
    fn default() -> Self {
        Self::new()
    }
}
