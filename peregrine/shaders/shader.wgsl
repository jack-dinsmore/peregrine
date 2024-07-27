struct CameraUniform {
    view_proj: mat4x4<f32>,
    light_pos: vec4<f32>,

};
struct ModelUniform {
    model_proj: mat4x4<f32>,
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
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec4<f32>,
    @location(2) normal: vec3<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = in.tex_coords;
    out.normal = in.normal;
    out.world_position = model.model_proj * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * out.world_position;
    return out;
}


// Fragment shader
@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;
@group(2) @binding(2)
var t_normal: texture_2d<f32>;
@group(2) @binding(3)
var s_normal: sampler;
@group(2) @binding(4)
var<uniform> material: MaterialUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Lighting
    let pos = in.world_position.xyz;
    let surface_to_light = normalize(camera.light_pos.xyz - pos);
    let bump_normal = textureSample(t_normal, s_normal, in.tex_coords*2).rgb - vec3(0.5, 0.5, 0.5);
    let normal = normalize(in.normal + bump_normal);
    let surface_to_light_dot_normal = dot(surface_to_light, normal);
    if surface_to_light_dot_normal < 0. {
        return vec4(0., 0., 0., 1.);
    }

    let surface_to_reflect = 2 * surface_to_light_dot_normal * normal - surface_to_light;
    let surface_to_camera = normalize(-pos);
    let diffuse_color = textureSample(t_diffuse, s_diffuse, in.tex_coords).rgb;
    let specular_color = vec3(1., 1., 1.);
    let color = (
        diffuse_color * material.light_info.x * surface_to_light_dot_normal// Diffuse
        + specular_color * material.light_info.y * pow(max(dot(surface_to_camera, surface_to_reflect), 0.), material.light_info.z)// Specular//
    );
    return vec4(color, 1.);
}



