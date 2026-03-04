use crate::core::color::Color;
use super::quantize::quantize_colors;
use super::config::PixelizeConfig;
use super::edge_selout::apply_selout;
use image::imageops::FilterType;

pub struct PixelizerPipeline;

impl PixelizerPipeline {
    pub fn process_image(img: &image::DynamicImage, config: &PixelizeConfig) -> Vec<u8> {
        let resized = img.resize_exact(config.target_w, config.target_h, FilterType::Lanczos3);
        let rgba = resized.to_rgba8();
        
        let mut pixels = Vec::with_capacity((config.target_w * config.target_h) as usize);

        for p in rgba.pixels() {
            let mut r = p[0] as f32;
            let mut g = p[1] as f32;
            let mut b = p[2] as f32;
            
            r = ((r - 128.0) * config.contrast + 128.0 + config.brightness).clamp(0.0, 255.0);
            g = ((g - 128.0) * config.contrast + 128.0 + config.brightness).clamp(0.0, 255.0);
            b = ((b - 128.0) * config.contrast + 128.0 + config.brightness).clamp(0.0, 255.0);
            
            pixels.push(Color::new(r as u8, g as u8, b as u8, p[3]));
        }

        quantize_colors(&mut pixels, config.color_count, config.min_color_distance);

        if config.use_selout {
            apply_selout(&mut pixels, config.target_w, config.target_h);
        }

        let mut buffer = Vec::with_capacity((config.target_w * config.target_h * 4) as usize);
        for p in pixels {
            buffer.push(p.r); buffer.push(p.g); buffer.push(p.b); buffer.push(p.a);
        }
        buffer
    }
}