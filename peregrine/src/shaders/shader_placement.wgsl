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
}

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = in.tex_coords;
    out.clip_position = camera.view_proj * model.world * vec4<f32>(in.position, 1.0);
    return out;
}


/// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if in.clip_position.z < 0.001 {
        return vec4(0., 0., 0., 0.);
    }
    let stretch = 1. / (in.clip_position.z * in.clip_position.z);
    let scaled_coords_ul = in.tex_coords * stretch;
    let scaled_coords_lr = (vec2(1., 1.) - in.tex_coords) * stretch;
    if (
        scaled_coords_ul.x < 0.1 |
        scaled_coords_ul.y < 0.1 |
        scaled_coords_lr.x < 0.1 |
        scaled_coords_lr.y < 0.1
    ) {
        return vec4(1., 1., 1., 1.);
    } else {
        return vec4(0., 0., 0., 0.);
    }
}