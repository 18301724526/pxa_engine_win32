use std::collections::HashMap;
use pixels::wgpu;
use crate::core::layer::Layer;

pub struct GpuTexture {
    pub texture: wgpu::Texture,
    pub bind_group: wgpu::BindGroup,
    pub version: u64,
}

pub struct TextureManager {
    cache: HashMap<String, GpuTexture>,
    pub layout: wgpu::BindGroupLayout,
}

impl TextureManager {
    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("anim_texture_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        Self { cache: HashMap::new(), layout }
    }

    pub fn sync_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sampler: &wgpu::Sampler,
        layer: &Layer,
    ) {
        if let Some(gpu_tex) = self.cache.get(&layer.id) {
            if gpu_tex.version == layer.version {
                return; 
            }
        }

        let rgba = layer.get_rect_data(0, 0, layer.width, layer.height);
        
        let size = wgpu::Extent3d {
            width: layer.width,
            height: layer.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("layer_tex_{}", layer.id)),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * layer.width),
                rows_per_image: Some(layer.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("anim_bind_group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler) },
            ],
        });

        self.cache.insert(layer.id.clone(), GpuTexture {
            texture,
            bind_group,
            version: layer.version,
        });
    }

    pub fn get_bind_group(&self, layer_id: &str) -> Option<&wgpu::BindGroup> {
        self.cache.get(layer_id).map(|t| &t.bind_group)
    }
}