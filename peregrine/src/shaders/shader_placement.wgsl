struct CameraUniform {
    view_proj: mat4x4<f32>,
    light_pos: vec4<f32>,
};
struct ModelUniform {
    world: mat4x4<f32>,
    rot_mat: mat4x4<f32>,
};
struct MaterialUniform {
    light_info: vec4<f32>
};

// Vertex shader
@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(1) @binding(0)
var<uniform> model: ModelUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) normal_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) normal_coords: vec2<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = in.tex_coords;
    out.normal_coords = in.normal_coords;
    out.normal = (model.rot_mat * vec4<f32>(in.normal, 1.)).xyz;
    out.world_position = model.world * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * out.world_position;
    return out;
}


/// Fragment shader
@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}