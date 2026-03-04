use crate::core::color::Color;

fn get_luminance(c: &Color) -> f32 {
    0.2126 * (c.r as f32) + 0.7152 * (c.g as f32) + 0.0722 * (c.b as f32)
}
pub fn apply_selout(pixels: &mut [Color], width: u32, height: u32) {
    let mut edge_magnitudes = vec![0.0; pixels.len()];
    let w = width as i32;
    let h = height as i32;

    let get_pixel = |x: i32, y: i32| -> Option<Color> {
        if x >= 0 && x < w && y >= 0 && y < h {
            Some(pixels[(y * w + x) as usize])
        } else {
            None
        }
    };

    // 1. 使用 Sobel 算子计算梯度幅值（精准寻找亮度突变的边界）
    for y in 0..h {
        for x in 0..w {
            let center = get_pixel(x, y).unwrap_or(Color::transparent());
            if center.a == 0 { continue; }

            let tl = get_pixel(x - 1, y - 1).map(|c| get_luminance(&c)).unwrap_or(0.0);
            let tc = get_pixel(x, y - 1).map(|c| get_luminance(&c)).unwrap_or(0.0);
            let tr = get_pixel(x + 1, y - 1).map(|c| get_luminance(&c)).unwrap_or(0.0);
            let l  = get_pixel(x - 1, y).map(|c| get_luminance(&c)).unwrap_or(0.0);
            let r  = get_pixel(x + 1, y).map(|c| get_luminance(&c)).unwrap_or(0.0);
            let bl = get_pixel(x - 1, y + 1).map(|c| get_luminance(&c)).unwrap_or(0.0);
            let bc = get_pixel(x, y + 1).map(|c| get_luminance(&c)).unwrap_or(0.0);
            let br = get_pixel(x + 1, y + 1).map(|c| get_luminance(&c)).unwrap_or(0.0);

            // Sobel X & Y 核计算
            let gx = (tr + 2.0 * r + br) - (tl + 2.0 * l + bl);
            let gy = (bl + 2.0 * bc + br) - (tl + 2.0 * tc + tr);
            
            let magnitude = (gx * gx + gy * gy).sqrt();
            edge_magnitudes[(y * w + x) as usize] = magnitude;
        }
    }

    let mut final_edges = vec![false; pixels.len()];
    
    let threshold = 180.0; // 提高阈值，无视微弱的阴影变化，只抓硬轮廓
    for y in 1..(h - 1) {
        for x in 1..(w - 1) {
            let idx = (y * w + x) as usize;
            let mag = edge_magnitudes[idx];
            
            if mag > threshold {
                // 如果比周围的梯度都大，或者至少是一个强边界
                let is_local_max = mag >= edge_magnitudes[idx - 1] && mag >= edge_magnitudes[idx + 1] ||
                                   mag >= edge_magnitudes[idx - w as usize] && mag >= edge_magnitudes[idx + w as usize];
                if is_local_max {
                    final_edges[idx] = true;
                }
            }
        }
    }

    let mut clean_edges = final_edges.clone();
    for y in 1..(h - 1) {
        for x in 1..(w - 1) {
            let idx = (y * w + x) as usize;
            if clean_edges[idx] {
                let mut neighbor_edges = 0;
                if final_edges[idx - 1] { neighbor_edges += 1; }
                if final_edges[idx + 1] { neighbor_edges += 1; }
                if final_edges[idx - w as usize] { neighbor_edges += 1; }
                if final_edges[idx + w as usize] { neighbor_edges += 1; }
                
                if neighbor_edges >= 3 {
                    clean_edges[idx] = false; // 去除粘连像素
                }
            }
        }
    }

    // 3. Selout 上色：找到边缘内侧的基底颜色，进行平滑加深
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if final_edges[idx] {
                // 尝试找一个非边缘的相邻色块作为“基底色”
                let neighbors = [
                    (x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)
                ];
                let mut base_color = pixels[idx]; // 默认用自己
                
                for (nx, ny) in neighbors {
                    if nx >= 0 && nx < w && ny >= 0 && ny < h {
                        let n_idx = (ny * w + nx) as usize;
                        if !clean_edges[n_idx] && edge_magnitudes[n_idx] < threshold && pixels[n_idx].a > 0 {
                            base_color = pixels[n_idx];
                            break;
                        }
                    }
                }
                
                // 稍微温柔一点的加深比例，40%，避免黑色过重
                pixels[idx] = base_color.darken(0.4);
            }
        }
    }
}