struct SceneViewUniforms {
    mvp: mat4x4<f32>,
    size: vec2<f32>,
    opacity: f32,
    corner_radius: f32,
    background: vec4<f32>,
    border_color: vec4<f32>,
    border_width: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: SceneViewUniforms;

@group(1) @binding(0)
var scene_texture: texture_2d<f32>;

@group(1) @binding(1)
var scene_sampler: sampler;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
) -> VertexOut {
    var out: VertexOut;
    out.position = uniforms.mvp * vec4<f32>(position, 1.0);
    out.uv = uv;
    return out;
}

fn rounded_rect_distance(point: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let bounded_radius = min(max(radius, 0.0), min(half_size.x, half_size.y));
    let q = abs(point) - half_size + vec2<f32>(bounded_radius);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - bounded_radius;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let local_point = (in.uv - vec2<f32>(0.5)) * uniforms.size;
    let distance = rounded_rect_distance(local_point, uniforms.size * 0.5, uniforms.corner_radius);
    let smoothing = max(fwidth(distance), 0.0001);
    let mask = 1.0 - smoothstep(-smoothing, smoothing, distance);

    var child = textureSample(scene_texture, scene_sampler, in.uv);
    // The offscreen pass accumulates premultiplied-looking RGB into a transparent target.
    // Convert it back before this pass applies normal alpha blending.
    if child.a > 0.0001 {
        child = vec4<f32>(child.rgb / child.a, child.a);
    }

    let background = uniforms.background;
    let combined_alpha = child.a + background.a * (1.0 - child.a);
    var combined_rgb = child.rgb;
    if combined_alpha > 0.0001 {
        combined_rgb = (
            child.rgb * child.a
            + background.rgb * background.a * (1.0 - child.a)
        ) / combined_alpha;
    }
    var color = vec4<f32>(combined_rgb, combined_alpha);

    if uniforms.border_width > 0.0 && distance > -uniforms.border_width {
        let border_mix = smoothstep(-uniforms.border_width - smoothing, -uniforms.border_width + smoothing, distance);
        color = mix(color, uniforms.border_color, border_mix);
    }

    color.a *= mask * uniforms.opacity;
    return color;
}
