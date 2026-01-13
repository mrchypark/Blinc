// Image rendering shader
// Supports: texture sampling, UV cropping, tinting, rounded corners, opacity, clipping

struct Uniforms {
    screen_size: vec2<f32>,
    _padding: vec2<f32>,
}

struct ImageInstance {
    // Destination rectangle (x, y, width, height) in screen pixels
    @location(0) dst_rect: vec4<f32>,
    // Source UV rectangle (u_min, v_min, u_max, v_max)
    @location(1) src_uv: vec4<f32>,
    // Tint color (RGBA)
    @location(2) tint: vec4<f32>,
    // Border radius and opacity
    @location(3) params: vec4<f32>, // (border_radius, opacity, padding, padding)
    // Clip bounds (x, y, width, height) - set to large values for no clip
    @location(4) clip_bounds: vec4<f32>,
    // Clip corner radii (top-left, top-right, bottom-right, bottom-left)
    @location(5) clip_radius: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
    @location(2) local_pos: vec2<f32>,
    @location(3) rect_size: vec2<f32>,
    @location(4) border_radius: f32,
    @location(5) opacity: f32,
    @location(6) world_pos: vec2<f32>,
    @location(7) clip_bounds: vec4<f32>,
    @location(8) clip_radius: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var image_texture: texture_2d<f32>;

@group(0) @binding(2)
var image_sampler: sampler;

// Vertex indices for a quad (two triangles)
var<private> QUAD_INDICES: array<u32, 6> = array<u32, 6>(0u, 1u, 2u, 2u, 3u, 0u);
var<private> QUAD_POSITIONS: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(0.0, 0.0), // Top-left
    vec2<f32>(1.0, 0.0), // Top-right
    vec2<f32>(1.0, 1.0), // Bottom-right
    vec2<f32>(0.0, 1.0), // Bottom-left
);

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
    instance: ImageInstance,
) -> VertexOutput {
    let quad_index = QUAD_INDICES[vertex_index];
    let local_pos = QUAD_POSITIONS[quad_index];

    // Calculate screen position
    let x = instance.dst_rect.x + local_pos.x * instance.dst_rect.z;
    let y = instance.dst_rect.y + local_pos.y * instance.dst_rect.w;

    // Convert to NDC
    let ndc_x = (x / uniforms.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (y / uniforms.screen_size.y) * 2.0;

    // Interpolate UV coordinates
    let uv = vec2<f32>(
        mix(instance.src_uv.x, instance.src_uv.z, local_pos.x),
        mix(instance.src_uv.y, instance.src_uv.w, local_pos.y),
    );

    var output: VertexOutput;
    output.position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    output.uv = uv;
    output.tint = instance.tint;
    output.local_pos = local_pos * vec2<f32>(instance.dst_rect.z, instance.dst_rect.w);
    output.rect_size = vec2<f32>(instance.dst_rect.z, instance.dst_rect.w);
    output.border_radius = instance.params.x;
    output.opacity = instance.params.y;
    output.world_pos = vec2<f32>(x, y);
    output.clip_bounds = instance.clip_bounds;
    output.clip_radius = instance.clip_radius;

    return output;
}

// SDF for rounded rectangle (uniform radius)
fn rounded_rect_sdf(pos: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let half_size = size * 0.5;
    let center_pos = pos - half_size;
    let r = min(radius, min(half_size.x, half_size.y));
    let q = abs(center_pos) - half_size + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

// SDF for rounded rectangle with per-corner radii
fn rounded_rect_sdf_corners(p: vec2<f32>, origin: vec2<f32>, size: vec2<f32>, radius: vec4<f32>) -> f32 {
    let half_size = size * 0.5;
    let center = origin + half_size;
    let rel = p - center;
    let q = abs(rel) - half_size;

    // Select corner radius based on quadrant
    // radius: (top-left, top-right, bottom-right, bottom-left)
    var r: f32;
    if rel.y < 0.0 {
        if rel.x > 0.0 {
            r = radius.y; // top-right
        } else {
            r = radius.x; // top-left
        }
    } else {
        if rel.x > 0.0 {
            r = radius.z; // bottom-right
        } else {
            r = radius.w; // bottom-left
        }
    }

    r = min(r, min(half_size.x, half_size.y));
    let q_adjusted = q + vec2<f32>(r);
    return length(max(q_adjusted, vec2<f32>(0.0))) + min(max(q_adjusted.x, q_adjusted.y), 0.0) - r;
}

// Calculate clip alpha (1.0 = inside clip, 0.0 = outside)
fn calculate_clip_alpha(p: vec2<f32>, clip_bounds: vec4<f32>, clip_radius: vec4<f32>) -> f32 {
    // Check if clip is effectively disabled (large bounds)
    if clip_bounds.x < -9000.0 {
        return 1.0;
    }

    let clip_d = rounded_rect_sdf_corners(p, clip_bounds.xy, clip_bounds.zw, clip_radius);

    // Anti-aliased clip edge
    let aa_width = fwidth(clip_d) * 0.5;
    return 1.0 - smoothstep(-aa_width, aa_width, clip_d);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Early clip test - discard if outside clip region
    let clip_alpha = calculate_clip_alpha(input.world_pos, input.clip_bounds, input.clip_radius);
    if clip_alpha < 0.001 {
        discard;
    }

    // Sample the texture
    var color = textureSample(image_texture, image_sampler, input.uv);

    // Apply tint
    color = color * input.tint;

    // Apply opacity
    color.a *= input.opacity;

    // Apply rounded corners if radius > 0
    if input.border_radius > 0.0 {
        let sdf = rounded_rect_sdf(input.local_pos, input.rect_size, input.border_radius);
        // Anti-aliased edge (1 pixel smooth)
        let alpha = 1.0 - smoothstep(-1.0, 1.0, sdf);
        color.a *= alpha;
    }

    // Apply clip alpha
    color.a *= clip_alpha;

    // Output premultiplied alpha for correct blending
    // (blend state uses src_factor: One, dst_factor: OneMinusSrcAlpha)
    color = vec4<f32>(color.rgb * color.a, color.a);

    return color;
}
