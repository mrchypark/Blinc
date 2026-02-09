# Changelog

All notable changes to `blinc_gpu` will be documented in this file.

## [Unreleased]

### Added

#### 3D SDF Raymarching Pipeline

- Per-element 3D shapes via `shape-3d: box | sphere | cylinder | torus | capsule | group`
- `depth` property for 3D extrusion depth
- `perspective` property for camera distance
- `rotate-x`, `rotate-y` for 3D axis rotation
- `translate-z` for Z-axis translation (closer/farther positioning)
- 32-step raymarching with analytical ray-AABB intersection
- Edge anti-aliasing via closest-approach distance tracking
- Blinn-Phong lighting with configurable `ambient`, `specular`, `light-direction`, `light-intensity`

#### 3D Boolean Operations

- `ShapeDesc` struct for per-shape descriptors in group composition (64 bytes, 4 vec4s)
- `MAX_GROUP_SHAPES` constant (16 shapes per group)
- Boolean SDF operations: `union`, `subtract`, `intersect`
- Smooth boolean operations: `smooth-union`, `smooth-subtract`, `smooth-intersect` with configurable blend radius
- `shape-3d: group` for collecting children into compound SDF via storage buffer

#### UV Mapping

- Automatic UV mapping of background (solid/gradient) onto 3D surface hit points
- Box: face-based projection (front/back, top/bottom, left/right)
- Sphere: spherical coordinate mapping
- Cylinder/torus/capsule: cylindrical coordinate mapping

#### GpuPrimitive Extensions

- `perspective[4]` field: `(sin_rx, cos_rx, persp_d, shape_type)`
- `sdf_3d[4]` field: `(depth, ambient, specular_power, translate_z)`
- `light[4]` field: `(dir_x, dir_y, dir_z, intensity)`

#### Paint Context

- `set_3d_shape()` for configuring per-element 3D shape parameters
- `set_3d_translate_z()` for Z-axis offset
- `set_3d_group_raw()` for compound shape composition from raw float arrays
- 3D transient state management with `clear_3d()` reset

### Fixed

- Clippy warnings across image.rs, particles.rs, path.rs, primitives.rs, renderer.rs, text.rs

## [0.1.1] - Initial Release

- Initial public release with GPU-accelerated 2D rendering pipeline
