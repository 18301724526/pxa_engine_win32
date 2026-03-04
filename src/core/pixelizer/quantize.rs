use crate::core::color::Color;

pub fn quantize_colors(pixels: &mut [Color], k: usize, min_color_distance: u32) {
    if pixels.is_empty() { return; }

    let mut centroids = Vec::new();
    if let Some(&first) = pixels.iter().find(|p| p.a > 0) { centroids.push(first); } else { return; }

    let stride = (pixels.len() / 1000).max(1);
    while centroids.len() < k {
        let mut max_dist = 0; let mut best_color = centroids[0];
        for i in (0..pixels.len()).step_by(stride) {
            let p = &pixels[i]; if p.a == 0 { continue; }
            let mut min_dist_to_centroids = u32::MAX;
            for c in &centroids {
                let d = p.distance_squared(c);
                if d < min_dist_to_centroids { min_dist_to_centroids = d; }
            }
            if min_dist_to_centroids > max_dist {
                max_dist = min_dist_to_centroids; best_color = *p;
            }
        }
        if max_dist == 0 { break; }
        centroids.push(best_color);
    }
    while centroids.len() < k { centroids.push(centroids[0]); }

    for _ in 0..8 {
        let mut new_centroids = vec![(0u64, 0u64, 0u64, 0u64, 0u64); k];
        for p in pixels.iter() {
            if p.a == 0 { continue; }
            let mut best_idx = 0; let mut min_dist = u32::MAX;
            for (i, c) in centroids.iter().enumerate() {
                let dist = p.distance_squared(c);
                if dist < min_dist { min_dist = dist; best_idx = i; }
            }
            new_centroids[best_idx].0 += p.r as u64; new_centroids[best_idx].1 += p.g as u64;
            new_centroids[best_idx].2 += p.b as u64; new_centroids[best_idx].3 += p.a as u64;
            new_centroids[best_idx].4 += 1;
        }

        for i in 0..k {
            let count = new_centroids[i].4;
            if count > 0 {
                let mut r = (new_centroids[i].0 / count) as u8;
                let mut g = (new_centroids[i].1 / count) as u8;
                let mut b = (new_centroids[i].2 / count) as u8;
                
                if r > 245 && g > 245 && b > 240 { r = 255; g = 255; b = 255; }
                if r < 40 && g < 40 && b < 40 { r = 30; g = 30; b = 40; }
                centroids[i] = Color::new(r, g, b, (new_centroids[i].3 / count) as u8);
            }
        }
    }

    let mut distinct_centroids = Vec::new();
    for c in centroids {
        if !distinct_centroids.iter().any(|dc: &Color| c.distance_squared(dc) < min_color_distance) {
            distinct_centroids.push(c);
        }
    }

    for p in pixels.iter_mut() {
        if p.a == 0 { continue; }
        let mut best_color = distinct_centroids[0]; let mut min_dist = u32::MAX;
        for c in &distinct_centroids {
            let dist = p.distance_squared(c);
            if dist < min_dist { min_dist = dist; best_color = *c; }
        }
        *p = best_color;
    }
}