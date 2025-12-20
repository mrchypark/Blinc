//! Layer Model for BLINC Canvas Architecture
//!
//! All visual content is represented as composable layers rendered to a unified canvas.
//! This module provides the core types for representing layers, scene graphs, and
//! dimension bridging between 2D UI, 2D canvas drawing, and 3D scenes.
//!
//! # Layer Types
//!
//! - **Ui**: 2D primitives rendered with SDF shaders
//! - **Canvas2D**: Vector drawing with paths and brushes
//! - **Scene3D**: 3D scene with meshes, materials, and lighting
//! - **Composition**: Stack, Transform, Clip, Opacity layers
//! - **Bridging**: Billboard (2D in 3D), Viewport3D (3D in 2D)

use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// Core Geometry Types
// ─────────────────────────────────────────────────────────────────────────────

/// 2D point
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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

/// 2D size
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Size = Size {
        width: 0.0,
        height: 0.0,
    };

    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Convert to a Rect at the origin (0, 0)
    pub const fn to_rect(self) -> Rect {
        Rect {
            origin: Point::ZERO,
            size: self,
        }
    }
}

impl From<Size> for Rect {
    /// Convert Size to Rect at origin (0, 0)
    fn from(size: Size) -> Self {
        Rect {
            origin: Point::ZERO,
            size,
        }
    }
}

/// 2D rectangle
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub const ZERO: Rect = Rect {
        origin: Point::ZERO,
        size: Size::ZERO,
    };

    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    pub fn from_origin_size(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    pub fn x(&self) -> f32 {
        self.origin.x
    }

    pub fn y(&self) -> f32 {
        self.origin.y
    }

    pub fn width(&self) -> f32 {
        self.size.width
    }

    pub fn height(&self) -> f32 {
        self.size.height
    }

    pub fn center(&self) -> Point {
        Point::new(
            self.origin.x + self.size.width / 2.0,
            self.origin.y + self.size.height / 2.0,
        )
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.origin.x
            && point.x <= self.origin.x + self.size.width
            && point.y >= self.origin.y
            && point.y <= self.origin.y + self.size.height
    }

    /// Get the size of this rect
    pub fn size(&self) -> Size {
        self.size
    }

    /// Offset the rect by a delta
    pub fn offset(&self, dx: f32, dy: f32) -> Self {
        Rect {
            origin: Point::new(self.origin.x + dx, self.origin.y + dy),
            size: self.size,
        }
    }

    /// Inset the rect by a delta (shrink from all sides)
    pub fn inset(&self, dx: f32, dy: f32) -> Self {
        Rect {
            origin: Point::new(self.origin.x + dx, self.origin.y + dy),
            size: Size::new(
                (self.size.width - 2.0 * dx).max(0.0),
                (self.size.height - 2.0 * dy).max(0.0),
            ),
        }
    }
}

/// 2D vector
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    pub const ONE: Vec2 = Vec2 { x: 1.0, y: 1.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self::new(self.x / len, self.y / len)
        } else {
            Self::ZERO
        }
    }
}

/// 3D vector
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: Vec3 = Vec3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    pub const UP: Vec3 = Vec3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    pub const FORWARD: Vec3 = Vec3 {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    };

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self::new(self.x / len, self.y / len, self.z / len)
        } else {
            Self::ZERO
        }
    }

    pub fn dot(&self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
}

/// 4x4 transformation matrix (column-major)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    pub cols: [[f32; 4]; 4],
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4 {
        cols: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        Self {
            cols: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [x, y, z, 1.0],
            ],
        }
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        Self {
            cols: [
                [x, 0.0, 0.0, 0.0],
                [0.0, y, 0.0, 0.0],
                [0.0, 0.0, z, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_y(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            cols: [
                [c, 0.0, -s, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [s, 0.0, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Multiply two matrices
    pub fn mul(&self, other: &Mat4) -> Mat4 {
        let mut result = [[0.0f32; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i][j] += self.cols[k][j] * other.cols[i][k];
                }
            }
        }
        Mat4 { cols: result }
    }
}

/// 2D affine transformation
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine2D {
    /// Matrix elements [a, b, c, d, tx, ty]
    /// | a  c  tx |
    /// | b  d  ty |
    /// | 0  0   1 |
    pub elements: [f32; 6],
}

impl Default for Affine2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Affine2D {
    pub const IDENTITY: Affine2D = Affine2D {
        elements: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    };

    pub fn translation(x: f32, y: f32) -> Self {
        Self {
            elements: [1.0, 0.0, 0.0, 1.0, x, y],
        }
    }

    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            elements: [sx, 0.0, 0.0, sy, 0.0, 0.0],
        }
    }

    pub fn rotation(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            elements: [c, s, -s, c, 0.0, 0.0],
        }
    }

    pub fn transform_point(&self, point: Point) -> Point {
        let [a, b, c, d, tx, ty] = self.elements;
        Point::new(a * point.x + c * point.y + tx, b * point.x + d * point.y + ty)
    }

    /// Concatenate this transform with another (self * other)
    /// The resulting transform first applies `other`, then `self`.
    pub fn then(&self, other: &Affine2D) -> Affine2D {
        let [a1, b1, c1, d1, tx1, ty1] = self.elements;
        let [a2, b2, c2, d2, tx2, ty2] = other.elements;

        // Matrix multiplication for 2D affine transforms:
        // [a1 c1 tx1]   [a2 c2 tx2]
        // [b1 d1 ty1] * [b2 d2 ty2]
        // [0  0  1  ]   [0  0  1  ]
        Affine2D {
            elements: [
                a1 * a2 + c1 * b2,       // a
                b1 * a2 + d1 * b2,       // b
                a1 * c2 + c1 * d2,       // c
                b1 * c2 + d1 * d2,       // d
                a1 * tx2 + c1 * ty2 + tx1, // tx
                b1 * tx2 + d1 * ty2 + ty1, // ty
            ],
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Color and Visual Types
// ─────────────────────────────────────────────────────────────────────────────

/// RGBA color (linear space)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        Self::rgb(r, g, b)
    }

    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.a = alpha;
        self
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

/// Gradient stop
#[derive(Clone, Copy, Debug)]
pub struct GradientStop {
    pub offset: f32,
    pub color: Color,
}

/// Gradient type
#[derive(Clone, Debug)]
pub enum Gradient {
    Linear {
        start: Point,
        end: Point,
        stops: Vec<GradientStop>,
    },
    Radial {
        center: Point,
        radius: f32,
        stops: Vec<GradientStop>,
    },
}

/// Brush for filling shapes
#[derive(Clone, Debug)]
pub enum Brush {
    Solid(Color),
    Gradient(Gradient),
    // Future: Image, Pattern
}

impl From<Color> for Brush {
    fn from(color: Color) -> Self {
        Brush::Solid(color)
    }
}

/// Blend mode for layer composition
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
}

/// Corner radii for rounded rectangles
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CornerRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadius {
    pub const ZERO: CornerRadius = CornerRadius {
        top_left: 0.0,
        top_right: 0.0,
        bottom_right: 0.0,
        bottom_left: 0.0,
    };

    pub fn uniform(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ]
    }
}

impl From<f32> for CornerRadius {
    fn from(radius: f32) -> Self {
        Self::uniform(radius)
    }
}

/// Shadow configuration
#[derive(Clone, Copy, Debug, Default)]
pub struct Shadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

impl Shadow {
    pub fn new(offset_x: f32, offset_y: f32, blur: f32, color: Color) -> Self {
        Self {
            offset_x,
            offset_y,
            blur,
            spread: 0.0,
            color,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Layer Identifiers
// ─────────────────────────────────────────────────────────────────────────────

/// Unique identifier for a layer
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LayerId(pub u64);

impl LayerId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Generator for unique layer IDs
#[derive(Debug, Default)]
pub struct LayerIdGenerator {
    next: u64,
}

impl LayerIdGenerator {
    pub fn new() -> Self {
        Self { next: 1 }
    }

    pub fn next(&mut self) -> LayerId {
        let id = LayerId(self.next);
        self.next += 1;
        id
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Layer Properties
// ─────────────────────────────────────────────────────────────────────────────

/// Pointer event behavior
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PointerEvents {
    /// Normal hit testing
    #[default]
    Auto,
    /// Transparent to input
    None,
    /// Receive events but don't block
    PassThrough,
}

/// Billboard facing mode for 2D content in 3D space
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BillboardFacing {
    /// Always faces camera
    #[default]
    Camera,
    /// Faces camera but stays upright
    CameraY,
    /// Uses transform rotation
    Fixed,
}

/// Layer cache policy
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CachePolicy {
    /// Always re-render
    #[default]
    None,
    /// Cache until content changes
    Content,
    /// Cache with explicit invalidation
    Manual,
}

/// Post-processing effect
#[derive(Clone, Debug)]
pub enum PostEffect {
    Blur { radius: f32 },
    Saturation { factor: f32 },
    Brightness { factor: f32 },
    Contrast { factor: f32 },
    GlassBlur { radius: f32, tint: Color },
}

/// Texture format for offscreen layers
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextureFormat {
    #[default]
    Bgra8Unorm,
    Rgba8Unorm,
    Rgba16Float,
    Rgba32Float,
}

/// Properties common to all layers
#[derive(Clone, Debug, Default)]
pub struct LayerProperties {
    /// Unique identifier for referencing
    pub id: Option<LayerId>,

    /// Visibility (skips render entirely when false)
    pub visible: bool,

    /// Pointer event behavior
    pub pointer_events: PointerEvents,

    /// Render order hint (within same Z-level)
    pub order: i32,

    /// Optional name for debugging
    pub name: Option<String>,
}

impl LayerProperties {
    pub fn new() -> Self {
        Self {
            visible: true,
            ..Default::default()
        }
    }

    pub fn with_id(mut self, id: LayerId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Clip Shape
// ─────────────────────────────────────────────────────────────────────────────

/// Shape used for clipping
#[derive(Clone, Debug)]
pub enum ClipShape {
    Rect(Rect),
    RoundedRect {
        rect: Rect,
        corner_radius: CornerRadius,
    },
    Circle {
        center: Point,
        radius: f32,
    },
    Ellipse {
        center: Point,
        radii: Vec2,
    },
    // Future: Path
}

// ─────────────────────────────────────────────────────────────────────────────
// 3D Scene Types
// ─────────────────────────────────────────────────────────────────────────────

/// Camera projection type
#[derive(Clone, Copy, Debug)]
pub enum CameraProjection {
    Perspective {
        fov_y: f32,
        aspect: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
}

impl Default for CameraProjection {
    fn default() -> Self {
        CameraProjection::Perspective {
            fov_y: std::f32::consts::FRAC_PI_4,
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

/// Camera for 3D scenes
#[derive(Clone, Debug, Default)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub projection: CameraProjection,
}

impl Camera {
    pub fn perspective(position: Vec3, target: Vec3, fov_y: f32) -> Self {
        Self {
            position,
            target,
            up: Vec3::UP,
            projection: CameraProjection::Perspective {
                fov_y,
                aspect: 16.0 / 9.0,
                near: 0.1,
                far: 1000.0,
            },
        }
    }

    pub fn orthographic(position: Vec3, target: Vec3, scale: f32) -> Self {
        Self {
            position,
            target,
            up: Vec3::UP,
            projection: CameraProjection::Orthographic {
                left: -scale,
                right: scale,
                bottom: -scale,
                top: scale,
                near: 0.1,
                far: 1000.0,
            },
        }
    }
}

/// Light type for 3D scenes
#[derive(Clone, Debug)]
pub enum Light {
    Directional {
        direction: Vec3,
        color: Color,
        intensity: f32,
        cast_shadows: bool,
    },
    Point {
        position: Vec3,
        color: Color,
        intensity: f32,
        range: f32,
    },
    Spot {
        position: Vec3,
        direction: Vec3,
        color: Color,
        intensity: f32,
        range: f32,
        inner_angle: f32,
        outer_angle: f32,
    },
    Ambient {
        color: Color,
        intensity: f32,
    },
}

/// Environment settings for 3D scenes (skybox, IBL)
#[derive(Clone, Debug, Default)]
pub struct Environment {
    /// HDRI texture path (if any)
    pub hdri: Option<String>,
    /// Environment intensity
    pub intensity: f32,
    /// Background blur amount
    pub blur: f32,
    /// Solid background color (used if no HDRI)
    pub background_color: Option<Color>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Layer Command Types (for Canvas2D and Scene3D)
// ─────────────────────────────────────────────────────────────────────────────

/// Commands for 2D canvas drawing
/// These are recorded and then executed by the Canvas2D renderer
#[derive(Clone, Debug, Default)]
pub struct Canvas2DCommands {
    commands: Vec<Canvas2DCommand>,
}

#[derive(Clone, Debug)]
pub enum Canvas2DCommand {
    // Future: Path operations, fills, strokes, etc.
    // For now, placeholder
    Clear(Color),
}

impl Canvas2DCommands {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn push(&mut self, command: Canvas2DCommand) {
        self.commands.push(command);
    }

    pub fn commands(&self) -> &[Canvas2DCommand] {
        &self.commands
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

/// Commands for 3D scene
#[derive(Clone, Debug, Default)]
pub struct Scene3DCommands {
    commands: Vec<Scene3DCommand>,
}

#[derive(Clone, Debug)]
pub enum Scene3DCommand {
    // Future: DrawMesh, AddLight, SetEnvironment, etc.
    // For now, placeholder
    Clear(Color),
}

impl Scene3DCommands {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn push(&mut self, command: Scene3DCommand) {
        self.commands.push(command);
    }

    pub fn commands(&self) -> &[Scene3DCommand] {
        &self.commands
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// UI Node (placeholder for full UI system)
// ─────────────────────────────────────────────────────────────────────────────

/// Reference to a UI node in the layout tree
#[derive(Clone, Copy, Debug)]
pub struct UiNode {
    /// Node identifier
    pub id: u64,
}

impl UiNode {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Layer Enum - The Core Abstraction
// ─────────────────────────────────────────────────────────────────────────────

/// All visual content is represented as a `Layer`.
///
/// Layers can be 2D UI primitives, 2D canvas drawings, 3D scenes, or
/// composition/transformation of other layers.
#[derive(Clone, Debug)]
pub enum Layer {
    // ─────────────────────────────────────────────────────────────────────────
    // 2D Primitives (SDF Rendered)
    // ─────────────────────────────────────────────────────────────────────────
    /// UI node tree (SDF rendered)
    Ui {
        node: UiNode,
        props: LayerProperties,
    },

    // ─────────────────────────────────────────────────────────────────────────
    // 2D Vector Drawing
    // ─────────────────────────────────────────────────────────────────────────
    /// 2D canvas with vector drawing commands
    Canvas2D {
        size: Size,
        commands: Canvas2DCommands,
        cache_policy: CachePolicy,
        props: LayerProperties,
    },

    // ─────────────────────────────────────────────────────────────────────────
    // 3D Scene
    // ─────────────────────────────────────────────────────────────────────────
    /// 3D scene with meshes, materials, and lighting
    Scene3D {
        viewport: Rect,
        commands: Scene3DCommands,
        camera: Camera,
        environment: Option<Environment>,
        props: LayerProperties,
    },

    // ─────────────────────────────────────────────────────────────────────────
    // Composition
    // ─────────────────────────────────────────────────────────────────────────
    /// Stack of layers composited together
    Stack {
        layers: Vec<Layer>,
        blend_mode: BlendMode,
        props: LayerProperties,
    },

    /// 2D transform applied to a layer
    Transform2D {
        transform: Affine2D,
        layer: Box<Layer>,
        props: LayerProperties,
    },

    /// 3D transform applied to a layer
    Transform3D {
        transform: Mat4,
        layer: Box<Layer>,
        props: LayerProperties,
    },

    /// Clip mask applied to a layer
    Clip {
        shape: ClipShape,
        layer: Box<Layer>,
        props: LayerProperties,
    },

    /// Opacity applied to a layer
    Opacity {
        value: f32,
        layer: Box<Layer>,
        props: LayerProperties,
    },

    // ─────────────────────────────────────────────────────────────────────────
    // Render Target Indirection
    // ─────────────────────────────────────────────────────────────────────────
    /// Layer rendered to an offscreen texture with optional effects
    Offscreen {
        size: Size,
        format: TextureFormat,
        layer: Box<Layer>,
        effects: Vec<PostEffect>,
        props: LayerProperties,
    },

    // ─────────────────────────────────────────────────────────────────────────
    // Dimension Bridging
    // ─────────────────────────────────────────────────────────────────────────
    /// 2D layer placed in 3D space
    Billboard {
        layer: Box<Layer>,
        transform: Mat4,
        facing: BillboardFacing,
        props: LayerProperties,
    },

    /// 3D scene embedded in 2D layout
    Viewport3D {
        rect: Rect,
        scene: Box<Layer>, // Must be Scene3D
        props: LayerProperties,
    },

    /// Reference to another layer's render output
    Portal {
        source: LayerId,
        sample_rect: Rect,
        dest_rect: Rect,
        props: LayerProperties,
    },

    /// Empty layer (useful as placeholder)
    Empty { props: LayerProperties },
}

impl Layer {
    /// Get the layer properties
    pub fn props(&self) -> &LayerProperties {
        match self {
            Layer::Ui { props, .. } => props,
            Layer::Canvas2D { props, .. } => props,
            Layer::Scene3D { props, .. } => props,
            Layer::Stack { props, .. } => props,
            Layer::Transform2D { props, .. } => props,
            Layer::Transform3D { props, .. } => props,
            Layer::Clip { props, .. } => props,
            Layer::Opacity { props, .. } => props,
            Layer::Offscreen { props, .. } => props,
            Layer::Billboard { props, .. } => props,
            Layer::Viewport3D { props, .. } => props,
            Layer::Portal { props, .. } => props,
            Layer::Empty { props } => props,
        }
    }

    /// Get mutable layer properties
    pub fn props_mut(&mut self) -> &mut LayerProperties {
        match self {
            Layer::Ui { props, .. } => props,
            Layer::Canvas2D { props, .. } => props,
            Layer::Scene3D { props, .. } => props,
            Layer::Stack { props, .. } => props,
            Layer::Transform2D { props, .. } => props,
            Layer::Transform3D { props, .. } => props,
            Layer::Clip { props, .. } => props,
            Layer::Opacity { props, .. } => props,
            Layer::Offscreen { props, .. } => props,
            Layer::Billboard { props, .. } => props,
            Layer::Viewport3D { props, .. } => props,
            Layer::Portal { props, .. } => props,
            Layer::Empty { props } => props,
        }
    }

    /// Get the layer ID if set
    pub fn id(&self) -> Option<LayerId> {
        self.props().id
    }

    /// Check if the layer is visible
    pub fn is_visible(&self) -> bool {
        self.props().visible
    }

    /// Create an empty layer
    pub fn empty() -> Self {
        Layer::Empty {
            props: LayerProperties::new(),
        }
    }

    /// Create a stack of layers
    pub fn stack(layers: Vec<Layer>) -> Self {
        Layer::Stack {
            layers,
            blend_mode: BlendMode::Normal,
            props: LayerProperties::new(),
        }
    }

    /// Wrap this layer with a 2D transform
    pub fn with_transform_2d(self, transform: Affine2D) -> Self {
        Layer::Transform2D {
            transform,
            layer: Box::new(self),
            props: LayerProperties::new(),
        }
    }

    /// Wrap this layer with a 3D transform
    pub fn with_transform_3d(self, transform: Mat4) -> Self {
        Layer::Transform3D {
            transform,
            layer: Box::new(self),
            props: LayerProperties::new(),
        }
    }

    /// Wrap this layer with a clip shape
    pub fn with_clip(self, shape: ClipShape) -> Self {
        Layer::Clip {
            shape,
            layer: Box::new(self),
            props: LayerProperties::new(),
        }
    }

    /// Wrap this layer with opacity
    pub fn with_opacity(self, value: f32) -> Self {
        Layer::Opacity {
            value,
            layer: Box::new(self),
            props: LayerProperties::new(),
        }
    }

    /// Check if this is a 3D layer
    pub fn is_3d(&self) -> bool {
        matches!(
            self,
            Layer::Scene3D { .. } | Layer::Billboard { .. } | Layer::Transform3D { .. }
        )
    }

    /// Check if this is a 2D layer
    pub fn is_2d(&self) -> bool {
        matches!(
            self,
            Layer::Ui { .. }
                | Layer::Canvas2D { .. }
                | Layer::Transform2D { .. }
                | Layer::Viewport3D { .. }
        )
    }

    /// Visit all child layers
    pub fn visit_children<F: FnMut(&Layer)>(&self, mut f: F) {
        match self {
            Layer::Stack { layers, .. } => {
                for layer in layers {
                    f(layer);
                }
            }
            Layer::Transform2D { layer, .. }
            | Layer::Transform3D { layer, .. }
            | Layer::Clip { layer, .. }
            | Layer::Opacity { layer, .. }
            | Layer::Offscreen { layer, .. }
            | Layer::Billboard { layer, .. }
            | Layer::Viewport3D { scene: layer, .. } => {
                f(layer);
            }
            _ => {}
        }
    }

    /// Visit all child layers mutably
    pub fn visit_children_mut<F: FnMut(&mut Layer)>(&mut self, mut f: F) {
        match self {
            Layer::Stack { layers, .. } => {
                for layer in layers {
                    f(layer);
                }
            }
            Layer::Transform2D { layer, .. }
            | Layer::Transform3D { layer, .. }
            | Layer::Clip { layer, .. }
            | Layer::Opacity { layer, .. }
            | Layer::Offscreen { layer, .. }
            | Layer::Billboard { layer, .. }
            | Layer::Viewport3D { scene: layer, .. } => {
                f(layer);
            }
            _ => {}
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Scene Graph
// ─────────────────────────────────────────────────────────────────────────────

/// Scene graph containing the root layer and layer index
#[derive(Debug, Default)]
pub struct SceneGraph {
    /// Root layer of the scene
    pub root: Option<Layer>,

    /// Index for fast layer lookup by ID
    layer_index: HashMap<LayerId, usize>,

    /// ID generator
    id_generator: LayerIdGenerator,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            root: None,
            layer_index: HashMap::new(),
            id_generator: LayerIdGenerator::new(),
        }
    }

    /// Set the root layer
    pub fn set_root(&mut self, layer: Layer) {
        self.root = Some(layer);
        self.rebuild_index();
    }

    /// Generate a new unique layer ID
    pub fn new_layer_id(&mut self) -> LayerId {
        self.id_generator.next()
    }

    /// Find a layer by ID (traverses the tree)
    pub fn find_layer(&self, id: LayerId) -> Option<&Layer> {
        fn find_in_layer(layer: &Layer, target_id: LayerId) -> Option<&Layer> {
            if layer.id() == Some(target_id) {
                return Some(layer);
            }

            match layer {
                Layer::Stack { layers, .. } => {
                    for child in layers {
                        if let Some(found) = find_in_layer(child, target_id) {
                            return Some(found);
                        }
                    }
                }
                Layer::Transform2D { layer: child, .. }
                | Layer::Transform3D { layer: child, .. }
                | Layer::Clip { layer: child, .. }
                | Layer::Opacity { layer: child, .. }
                | Layer::Offscreen { layer: child, .. }
                | Layer::Billboard { layer: child, .. }
                | Layer::Viewport3D { scene: child, .. } => {
                    if let Some(found) = find_in_layer(child, target_id) {
                        return Some(found);
                    }
                }
                _ => {}
            }

            None
        }

        self.root.as_ref().and_then(|root| find_in_layer(root, id))
    }

    /// Rebuild the layer index
    fn rebuild_index(&mut self) {
        self.layer_index.clear();
        // Future: implement full index rebuilding
    }

    /// Traverse all layers in depth-first order
    pub fn traverse<F: FnMut(&Layer, usize)>(&self, mut f: F) {
        fn traverse_layer<F: FnMut(&Layer, usize)>(layer: &Layer, depth: usize, f: &mut F) {
            f(layer, depth);
            layer.visit_children(|child| traverse_layer(child, depth + 1, f));
        }

        if let Some(root) = &self.root {
            traverse_layer(root, 0, &mut f);
        }
    }

    /// Count total number of layers
    pub fn layer_count(&self) -> usize {
        let mut count = 0;
        self.traverse(|_, _| count += 1);
        count
    }

    /// Check if the scene contains any 3D layers
    pub fn has_3d(&self) -> bool {
        let mut has_3d = false;
        self.traverse(|layer, _| {
            if layer.is_3d() {
                has_3d = true;
            }
        });
        has_3d
    }

    /// Count all visible layers
    pub fn visible_layer_count(&self) -> usize {
        let mut count = 0;
        self.traverse(|layer, _| {
            if layer.is_visible() {
                count += 1;
            }
        });
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_creation() {
        let layer = Layer::empty();
        assert!(layer.is_visible());
        assert!(layer.id().is_none());
    }

    #[test]
    fn test_layer_stack() {
        let stack = Layer::stack(vec![
            Layer::empty(),
            Layer::empty(),
            Layer::empty(),
        ]);

        let mut count = 0;
        stack.visit_children(|_| count += 1);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_layer_transforms() {
        let layer = Layer::empty()
            .with_transform_2d(Affine2D::translation(10.0, 20.0))
            .with_opacity(0.5);

        assert!(matches!(layer, Layer::Opacity { .. }));
    }

    #[test]
    fn test_scene_graph() {
        let mut scene = SceneGraph::new();

        let id1 = scene.new_layer_id();
        let id2 = scene.new_layer_id();

        assert_ne!(id1, id2);

        scene.set_root(Layer::stack(vec![
            Layer::Empty {
                props: LayerProperties::new().with_id(id1),
            },
            Layer::Empty {
                props: LayerProperties::new().with_id(id2),
            },
        ]));

        assert_eq!(scene.layer_count(), 3); // stack + 2 empty

        let found = scene.find_layer(id1);
        assert!(found.is_some());
    }

    #[test]
    fn test_geometry_types() {
        let p = Point::new(1.0, 2.0);
        let s = Size::new(100.0, 50.0);
        let r = Rect::from_origin_size(p, s);

        assert_eq!(r.center(), Point::new(51.0, 27.0));
        assert!(r.contains(Point::new(50.0, 25.0)));
        assert!(!r.contains(Point::new(200.0, 100.0)));

        // Test Size to Rect conversion
        let size = Size::new(200.0, 100.0);
        let rect: Rect = size.into();
        assert_eq!(rect.x(), 0.0);
        assert_eq!(rect.y(), 0.0);
        assert_eq!(rect.width(), 200.0);
        assert_eq!(rect.height(), 100.0);

        // Test to_rect() method
        let rect2 = size.to_rect();
        assert_eq!(rect, rect2);

        // Test offset and inset
        let offset_rect = rect.offset(10.0, 20.0);
        assert_eq!(offset_rect.x(), 10.0);
        assert_eq!(offset_rect.y(), 20.0);

        let inset_rect = rect.inset(5.0, 10.0);
        assert_eq!(inset_rect.x(), 5.0);
        assert_eq!(inset_rect.y(), 10.0);
        assert_eq!(inset_rect.width(), 190.0);
        assert_eq!(inset_rect.height(), 80.0);
    }

    #[test]
    fn test_color() {
        let c = Color::from_hex(0xFF5500);
        assert_eq!(c.r, 1.0);
        assert!((c.g - 85.0 / 255.0).abs() < 0.001);
        assert_eq!(c.b, 0.0);

        let c2 = c.with_alpha(0.5);
        assert_eq!(c2.a, 0.5);
    }

    #[test]
    fn test_mat4_operations() {
        let t = Mat4::translation(1.0, 2.0, 3.0);
        let s = Mat4::scale(2.0, 2.0, 2.0);
        let result = t.mul(&s);

        // Verify it's a valid combined transform
        assert_eq!(result.cols[3][0], 1.0); // translation preserved
    }
}
