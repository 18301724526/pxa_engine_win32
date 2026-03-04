use crate::core::color::Color;
use std::collections::HashMap;

fn get_luminance(c: &Color) -> f32 {
    0.2126 * (c.r as f32) + 0.7152 * (c.g as f32) + 0.0722 * (c.b as f32)
}

/// 细节保留降采样：在众数的基础上，强制保护高反差的深色细节（如眼睛、线稿）
pub fn mode_downsample(img: &image::DynamicImage, target_w: u32, target_h: u32) -> Vec<Color> {
    let rgba_img = img.to_rgba8();
    let (src_w, src_h) = rgba_img.dimensions();
    
    let mut result = Vec::with_capacity((target_w * target_h) as usize);
    let block_w = src_w as f32 / target_w as f32;
    let block_h = src_h as f32 / target_h as f32;

    for y in 0..target_h {
        for x in 0..target_w {
            let start_x = (x as f32 * block_w) as u32;
            let start_y = (y as f32 * block_h) as u32;
            let end_x = ((x + 1) as f32 * block_w).ceil() as u32;
            let end_y = ((y + 1) as f32 * block_h).ceil() as u32;

            let mut color_counts = HashMap::new();
            let mut total_pixels = 0;
            
            for src_y in start_y..end_y.min(src_h) {
                for src_x in start_x..end_x.min(src_w) {
                    let pixel = rgba_img.get_pixel(src_x, src_y);
                    let color = Color::new(pixel[0], pixel[1], pixel[2], pixel[3]);
                    if color.a > 10 {
                        *color_counts.entry(color).or_insert(0) += 1;
                        total_pixels += 1;
                    }
                }
            }

            if total_pixels == 0 {
                result.push(Color::transparent());
                continue;
            }

            // 1. 找出区块内的绝对“众数”颜色（通常是皮肤或衣服的底色）
            let dominant_color = color_counts.iter()
                .max_by_key(|&(_, count)| count)
                .map(|(&color, _)| color)
                .unwrap_or_else(Color::transparent);

            let dominant_lum = get_luminance(&dominant_color);
            
            // 2. 细节寻回：寻找区块内是否有高反差的“少数派深色”（通常是线稿或五官）
            let mut final_color = dominant_color;
            let min_votes_required = (total_pixels as f32 * 0.05).ceil() as u32; // 至少占 5% 的像素才算数，避免纯噪点
            
            for (&color, &count) in &color_counts {
                if count >= min_votes_required {
                    let lum = get_luminance(&color);
                    // 如果这个颜色比背景色暗得多（反差大于 60），强制保留它！
                    if dominant_lum - lum > 60.0 {
                        final_color = color;
                        break; // 找到最明显的暗部特征就退出
                    }
                }
            }

            result.push(final_color);
        }
    }
    result
}