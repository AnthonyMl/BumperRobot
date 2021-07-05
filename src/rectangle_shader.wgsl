struct InstanceInput {
    [[location(3)]] color: vec3<f32>;
    [[location(4)]] transform_0: vec3<f32>;
    [[location(5)]] transform_1: vec3<f32>;
    [[location(6)]] transform_2: vec3<f32>;
    [[location(7)]] screen_size: vec2<f32>;
};

struct VertexInput {
    [[builtin(vertex_index)]] vertex_index: u32;
    [[location(0)]] position: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
    [[location(1)]] color: vec3<f32>;
    [[location(2)]] screen_size: vec2<f32>;
};

[[block]]
struct Uniforms {
    aspect_ratio: f32;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;



[[stage(vertex)]]
fn main(vi: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    let transform = mat3x3<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2
    );

    let p = transform * vec3<f32>(vi.position, 1.0);

    out.position = vec4<f32>(p.xy, 0.0, 1.0);

    out.color = instance.color;

    out.uv = vec2<f32>(
        f32( vi.vertex_index & 1u32),
        f32((vi.vertex_index & 2u32) >> 1u32)
    );
    out.screen_size = instance.screen_size;

    return out;
}

let WIDTH: f32 = 0.003;

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    if (in.uv.x > (WIDTH / in.screen_size.x) &&
        in.uv.x < (1.0 - WIDTH / in.screen_size.x) &&
        in.uv.y > (WIDTH / in.screen_size.y) &&
        in.uv.y < (1.0 - WIDTH / in.screen_size.y)) {
        discard;
    }

    return vec4<f32>(in.color, 1.0);
}
