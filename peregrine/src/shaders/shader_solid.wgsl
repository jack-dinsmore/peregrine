struct CameraUniform {
    view_proj: mat4x4<f32>,
    light_pos: vec4<f32>,
};
struct SolidUniform {
    world: mat4x4<f32>,
    color: vec4<f32>,
};

// Vertex shader
@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(1) @binding(0)
var<uniform> model: SolidUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex 
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let mvp = camera.view_proj * model.world;
    out.clip_position = mvp * vec4<f32>(in.position, 1.0);
    out.clip_position = mvp * vec4<f32>(in.position, 1./out.clip_position.w);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return model.color;
}