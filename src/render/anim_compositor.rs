use pixels::wgpu;
use pixels::wgpu::util::DeviceExt;
use crate::core::store::PixelStore;
use crate::core::animation::skeleton::Skeleton;
use crate::render::compositor::Viewport;
use crate::render::texture_manager::TextureManager;
use bytemuck;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
}

pub struct AnimCompositor {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    pub texture_manager: TextureManager,
}

impl AnimCompositor {
    pub fn new(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Self {
        let texture_manager = TextureManager::new(device);
        
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("anim_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("anim_shader.wgsl").into()),
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("anim_uniform_buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("anim_uniform_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("anim_uniform_bg"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("anim_pipeline_layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_manager.layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("anim_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 80, 
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 
                            6 => Float32x4 
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertices = [
            Vertex { position: [0.0, 0.0], tex_coords: [0.0, 0.0] },
            Vertex { position: [1.0, 0.0], tex_coords: [1.0, 0.0] },
            Vertex { position: [0.0, 1.0], tex_coords: [0.0, 1.0] },
            Vertex { position: [1.0, 1.0], tex_coords: [1.0, 1.0] },
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("anim_vbuf"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let indices: [u16; 6] = [0, 1, 2, 2, 1, 3];
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("anim_ibuf"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self { pipeline, vertex_buffer, index_buffer, uniform_buffer, uniform_bind_group, sampler, texture_manager }
    }

    pub fn prepare_textures(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, store: &PixelStore, skeleton: &Skeleton) {
        for slot in &skeleton.slots {
            if let Some(layer_id) = &slot.current_attachment {
                if let Some(layer) = store.get_layer(layer_id) {
                    self.texture_manager.sync_layer(device, queue, &self.sampler, layer);
                }
            }
        }
    }

    pub fn render_gpu<'a>(
        &'a self,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'a>,
        store: &PixelStore,
        skeleton: &Skeleton,
        view: Viewport, 
        _selected_id: Option<&String>,
        instance_buffers: &'a [wgpu::Buffer],
    ) {
        let world_to_clip = self.calculate_projection(view);
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[Uniforms { view_proj: world_to_clip }]));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        let mut instance_idx = 0;
        for slot in &skeleton.slots {
            if let Some(layer_id) = &slot.current_attachment {
                if store.get_layer(layer_id).is_some() {
                    if let Some(bind_group) = self.texture_manager.get_bind_group(layer_id) {
                        if skeleton.bones.iter().any(|b| b.data.id == slot.data.bone_id) {
                            if instance_idx < instance_buffers.len() {
                                render_pass.set_vertex_buffer(1, instance_buffers[instance_idx].slice(..));
                                render_pass.set_bind_group(1, bind_group, &[]);
                                render_pass.draw_indexed(0..6, 0, 0..1);
                                instance_idx += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    fn calculate_projection(&self, v: Viewport) -> [[f32; 4]; 4] {
        let zoom = v.zoom;
        let sw = v.screen_width as f32;
        let sh = v.screen_height as f32;

        let c_cx = (sw * 0.5) / zoom; 
        let c_cy = (sh * 0.5) / zoom;

        let scale_x = 2.0 * zoom / sw;
        let scale_y = -2.0 * zoom / sh;
        let tx = ((v.pan_x - c_cx) * zoom) * (2.0 / sw) + 1.0;
        let ty = ((v.pan_y - c_cy) * zoom) * (-2.0 / sh) - 1.0;
        [[scale_x, 0.0, 0.0, 0.0], [0.0, scale_y, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [tx, ty, 0.0, 1.0]]
    }

    pub fn render_cpu(
        store: &PixelStore,
        skeleton: &Skeleton,
        frame: &mut [u8],
        view: Viewport,
        selected_id: Option<&String> 
    ) {

        let zoom = view.zoom;
        let sw = view.screen_width as f32;
        let sh = view.screen_height as f32;
        let cw = store.canvas_width as f32;
        let ch = store.canvas_height as f32;

        let to_screen = |wx: f32, wy: f32| -> (i32, i32) {
            let canvas_cx = cw / 2.0;
            let canvas_cy = ch / 2.0;
            let screen_cx = sw / 2.0;
            let screen_cy = sh / 2.0;
            let sx = (wx - canvas_cx + view.pan_x) * zoom + screen_cx;
            let sy = (wy - canvas_cy + view.pan_y) * zoom + screen_cy;
            (sx as i32, sy as i32)
        };

        for bone in &skeleton.bones {
            let is_selected = selected_id == Some(&bone.data.id);
            let color = if bone.data.id.starts_with("preview") {
                [255, 255, 0, 255]
            } else if is_selected {
                [255, 50, 50, 255]
            } else {
                [200, 200, 200, 255]
            };
            
            let m = bone.world_matrix;
            let root_x = m[4].round();
            let root_y = m[5].round();
            let length = bone.data.length;

            if length < 1.0 {
                let p = to_screen(root_x, root_y);
                draw_cross(frame, view.screen_width, view.screen_height, p.0, p.1, 4, color);
                continue;
            }

            let tip_x = root_x + length * m[0];
            let tip_y = root_y + length * m[1];

            let width = (length * 0.15).clamp(3.0, 15.0);

            let dx = m[0]; 
            let dy = m[1];
            let perp_x = -dy * width;
            let perp_y = dx * width;

            
            let split_ratio = 0.2; 
            let mid_x = root_x + dx * length * split_ratio;
            let mid_y = root_y + dy * length * split_ratio;

            let p_root = to_screen(root_x, root_y);
            let p_tip = to_screen(tip_x, tip_y);
            let p_left = to_screen(mid_x + perp_x, mid_y + perp_y);
            let p_right = to_screen(mid_x - perp_x, mid_y - perp_y);

            draw_line(frame, view.screen_width, view.screen_height, p_root, p_left, color);
            draw_line(frame, view.screen_width, view.screen_height, p_left, p_tip, color);
            draw_line(frame, view.screen_width, view.screen_height, p_tip, p_right, color);
            draw_line(frame, view.screen_width, view.screen_height, p_right, p_root, color);
            
            let mid_color = [color[0], color[1], color[2], 128];
            draw_line(frame, view.screen_width, view.screen_height, p_root, p_tip, mid_color);

            draw_circle_filled(frame, view.screen_width, view.screen_height, p_root.0, p_root.1, 3, color);
        }
    }
}


fn draw_pixel_safe(frame: &mut [u8], w: u32, h: u32, x: i32, y: i32, color: [u8; 4]) {
    if x >= 0 && y >= 0 && x < w as i32 && y < h as i32 {
        let idx = ((y as u32 * w + x as u32) * 4) as usize;
        if idx + 4 <= frame.len() { 
            let bg = &frame[idx..idx+4];
            let alpha = color[3] as f32 / 255.0;
            let inv_alpha = 1.0 - alpha;
            
            let r = (color[0] as f32 * alpha + bg[0] as f32 * inv_alpha) as u8;
            let g = (color[1] as f32 * alpha + bg[1] as f32 * inv_alpha) as u8;
            let b = (color[2] as f32 * alpha + bg[2] as f32 * inv_alpha) as u8;
            
            frame[idx] = r;
            frame[idx+1] = g;
            frame[idx+2] = b;
            frame[idx+3] = 255; 
        }
    }
}

fn draw_line(frame: &mut [u8], w: u32, h: u32, p1: (i32, i32), p2: (i32, i32), color: [u8; 4]) {
    crate::tools::geometry::Geometry::bresenham_line(p1.0, p1.1, p2.0, p2.1, |x, y| {
        draw_pixel_safe(frame, w, h, x, y, color);
    });
}

fn draw_cross(frame: &mut [u8], w: u32, h: u32, x: i32, y: i32, size: i32, color: [u8; 4]) {
    draw_line(frame, w, h, (x - size, y), (x + size, y), color);
    draw_line(frame, w, h, (x, y - size), (x, y + size), color);
}

fn draw_circle_filled(frame: &mut [u8], w: u32, h: u32, cx: i32, cy: i32, r: i32, color: [u8; 4]) {
    let r2 = r * r;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx*dx + dy*dy <= r2 {
                draw_pixel_safe(frame, w, h, cx + dx, cy + dy, color);
            }
        }
    }
}