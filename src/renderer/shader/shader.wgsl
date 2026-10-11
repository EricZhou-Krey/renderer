struct Camera {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;

struct Material {
    base_color: vec4<f32>,
};

@group(1) @binding(0) var<uniform> material: Material;
@group(1) @binding(1) var base_color_texture: texture_2d<f32>;
@group(1) @binding(2) var normal_texture: texture_2d<f32>;
@group(1) @binding(3) var material_sampler: sampler;

struct Light {
    position_range: vec4<f32>,
    color_intensity: vec4<f32>,
}

struct LightUniform {
    lights: array<Light, 256>,
    count: u32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
}

@group(2) @binding(0) var<uniform> light_uniform: LightUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) model_0: vec4<f32>,
    @location(4) model_1: vec4<f32>,
    @location(5) model_2: vec4<f32>,
    @location(6) model_3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let model_matrix = mat4x4<f32>(
        input.model_0,
        input.model_1,
        input.model_2,
        input.model_3,
    );

    let world_position = model_matrix * vec4<f32>(input.position, 1.0);

    out.clip_position = camera.proj * camera.view * world_position;
    out.uv = input.uv;
    out.normal = normalize(
        (model_matrix * vec4<f32>(input.normal, 0.0)).xyz
    );
    out.world_position = world_position.xyz;

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sampled_color = textureSample(
        base_color_texture,
        material_sampler,
        input.uv,
    );
    let albedo = sampled_color * material.base_color;
    let normal = normalize(input.normal);

    var lighting = vec3<f32>(0.2);

    let light_count = min(light_uniform.count, 256u);

    for (var i = 0u; i < light_count; i = i + 1u) {
        let light = light_uniform.lights[i];
        let to_light = light.position_range.xyz - input.world_position;
        let distance_to_light = length(to_light);
        let light_direction = to_light / max(distance_to_light, 0.0001);

        let range = max(light.position_range.w, 0.0001);
        let range_factor = clamp(1.0 - distance_to_light / range, 0.0, 1.0);
        let attenuation = range_factor * range_factor;
        let diffuse = max(dot(normal, light_direction), 0.0);

        lighting += light.color_intensity.rgb
            * light.color_intensity.w
            * diffuse
            * attenuation;
    }

    return vec4<f32>(albedo.rgb * lighting, albedo.a);
}

