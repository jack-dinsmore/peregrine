struct CameraUniform {
    view_proj: mat4x4<f32>,
    light_pos: vec4<f32>,
};
struct ModelUniform {
    world: mat4x4<f32>,
    rot_mat: mat4x4<f32>,
};

// Vertex shader
@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(1) @binding(0)
var<uniform> model: ModelUniform;

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
    out.clip_position = camera.view_proj * model.world * vec4<f32>(in.position, 1.0);
    return out;
}


/// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(1., 1., 1., 1.);
}