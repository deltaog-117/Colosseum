struct Camera {
    view_projection: mat4x4<f32>,
};

// Packed by `PsxSettings::pack`.
struct Psx {
    // xyz: fog color.
    fog_color: vec4<f32>,
    // x: fog start, y: one over the fog span, z: fog on, w: dither strength.
    fog: vec4<f32>,
    // xy: snap grid size in cells, z: snap on.
    snap: vec4<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> psx: Psx;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec3<f32>,
    // View depth, which perspective projection stores in clip.w.
    @location(1) depth: f32,
};

// The original hardware has 5 bits per color channel.
const LEVELS: f32 = 31.0;

// Vertices behind or on the camera plane have no screen position to snap.
const MIN_SNAP_W: f32 = 0.0001;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var clip = camera.view_projection * vec4<f32>(in.position, 1.0);
    let depth = clip.w;
    if psx.snap.z > 0.5 && depth > MIN_SNAP_W {
        // Round the screen position to the grid, then undo the divide.
        let screen = (clip.xy / depth * 0.5 + 0.5) * psx.snap.xy;
        let snapped = floor(screen + 0.5) / psx.snap.xy;
        clip = vec4<f32>((snapped - 0.5) * 2.0 * depth, clip.z, depth);
    }
    var out: VertexOutput;
    out.clip = clip;
    out.color = in.color;
    out.depth = depth;
    return out;
}

// 4x4 ordered-dither thresholds, 0..15.
fn bayer(pixel: vec2<u32>) -> f32 {
    var matrix = array<f32, 16>(
        0.0, 8.0, 2.0, 10.0,
        12.0, 4.0, 14.0, 6.0,
        3.0, 11.0, 1.0, 9.0,
        15.0, 7.0, 13.0, 5.0,
    );
    return matrix[(pixel.y % 4u) * 4u + (pixel.x % 4u)];
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color;
    if psx.fog.z > 0.5 {
        let amount = clamp((in.depth - psx.fog.x) * psx.fog.y, 0.0, 1.0);
        color = mix(color, psx.fog_color.xyz, amount);
    }
    let strength = psx.fog.w;
    if strength > 0.0 {
        // Threshold in -0.5..0.5 of one color step, so the average is unchanged.
        let threshold = (bayer(vec2<u32>(in.clip.xy)) + 0.5) / 16.0 - 0.5;
        color = floor(color * LEVELS + 0.5 + threshold * strength) / LEVELS;
    }
    return vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
