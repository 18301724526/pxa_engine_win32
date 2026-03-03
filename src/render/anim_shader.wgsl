struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) inv_c0: vec4<f32>,
    @location(2) inv_c1: vec4<f32>,
    @location(3) inv_c2: vec4<f32>,
    @location(4) layer_size: vec2<f32>,
};

struct Uniforms {
    view_proj: mat4x4<f32>,
    viewport: vec4<f32>, // sw, sh, cw, ch
    view_params: vec4<f32>, // zoom, pan_x, pan_y, pad
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
    @location(7) inv_c0: vec4<f32>,
    @location(8) inv_c1: vec4<f32>,
    @location(9) inv_c2: vec4<f32>,
    @location(10) layer_size: vec2<f32>
) -> VertexOutput {
    var model_matrix = mat4x4<f32>(m0, m1, m2, m3);
    var out: VertexOutput;
    
    // 向外稍微扩展几何四边形，防止旋转时边缘像素被光栅化器提前裁剪
    let margin_x = 4.0 / layer_size.x;
    let margin_y = 4.0 / layer_size.y;
    let expanded_pos = vec2<f32>(
        model.position.x * (1.0 + 2.0 * margin_x) - margin_x,
        model.position.y * (1.0 + 2.0 * margin_y) - margin_y
    );

    out.clip_position = uniforms.view_proj * model_matrix * vec4<f32>(expanded_pos, 0.0, 1.0);
    out.color = instance_color; 
    out.inv_c0 = inv_c0;
    out.inv_c1 = inv_c1;
    out.inv_c2 = inv_c2;
    out.layer_size = layer_size;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sw = uniforms.viewport.x;
    let sh = uniforms.viewport.y;
    let cw = uniforms.viewport.z;
    let ch = uniforms.viewport.w;
    let zoom = uniforms.view_params.x;
    let pan_x = uniforms.view_params.y;
    let pan_y = uniforms.view_params.z;

    let screen_cx = sw * 0.5;
    let screen_cy = sh * 0.5;
    let canvas_cx = cw * 0.5;
    let canvas_cy = ch * 0.5;

    // 1. 将屏幕物理像素 (in.clip_position.xy) 严格吸附到画布的绝对整数像素坐标
    let cx = floor((in.clip_position.x - screen_cx) / zoom + canvas_cx - pan_x);
    let cy = floor((in.clip_position.y - screen_cy) / zoom + canvas_cy - pan_y);

    let canvas_x = cx + 0.5;
    let canvas_y = cy + 0.5;

    // 2. 利用传入的逆矩阵，将吸附后的画布坐标反求出图层的局部坐标
    let local_x = in.inv_c0.x * canvas_x + in.inv_c1.x * canvas_y + in.inv_c2.x;
    let local_y = in.inv_c0.y * canvas_x + in.inv_c1.y * canvas_y + in.inv_c2.y;

    let w = in.layer_size.x;
    let h = in.layer_size.y;

    // 4. 将反求坐标精准对齐到纹理 UV 中心，提取纯净的像素色块
    let u = (floor(local_x) + 0.5) / w;
    let v = (floor(local_y) + 0.5) / h;

    let tex_color = textureSample(t_diffuse, s_diffuse, vec2<f32>(u, v));
    let final_color = tex_color * in.color;
    
    if (local_x < 0.0 || local_x >= w || local_y < 0.0 || local_y >= h || final_color.a < 0.001) {
        discard;
    }
    return final_color;
}