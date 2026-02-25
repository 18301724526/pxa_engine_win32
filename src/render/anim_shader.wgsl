struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>, // 传递颜色给 Fragment Shader
};

struct Uniforms {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

@vertex
fn vs_main(
    model: VertexInput,
    @location(2) m0: vec4<f32>,
    @location(3) m1: vec4<f32>,
    @location(4) m2: vec4<f32>,
    @location(5) m3: vec4<f32>,
    @location(6) instance_color: vec4<f32>, // 新增：插槽颜色属性
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(m0, m1, m2, m3);
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * model_matrix * vec4<f32>(model.position, 0.0, 1.0);
    out.tex_coords = model.tex_coords;
    out.color = instance_color; 
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    // 关键：将贴图颜色与动画插槽颜色相乘
    let final_color = tex_color * in.color;
    if (final_color.a < 0.001) {
        discard;
    }
    return final_color;
}