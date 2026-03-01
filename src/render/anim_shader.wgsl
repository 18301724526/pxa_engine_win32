struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
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
    @location(6) instance_color: vec4<f32>,
) -> VertexOutput {
    var model_matrix = mat4x4<f32>(m0, m1, m2, m3);
    
    // 【像素生产工具核心魔法】：
    // 将世界坐标平移分量强制吸附到最近的整数像素网格上。
    // 这保留了旋转矩阵的形态，但杜绝了 Sub-pixel 模糊！
    model_matrix[3].x = floor(model_matrix[3].x + 0.5);
    model_matrix[3].y = floor(model_matrix[3].y + 0.5);

    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * model_matrix * vec4<f32>(model.position, 0.0, 1.0);
    out.tex_coords = model.tex_coords;
    out.color = instance_color; 
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 配合外部 Nearest 采样器，这里采出的就是完美的像素色块
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let final_color = tex_color * in.color;
    
    if (final_color.a < 0.001) {
        discard;
    }
    return final_color;
}