use crate::core::store::PixelStore;
use crate::core::layer::{Layer, Chunk, CHUNK_SIZE};
use crate::render::blend::blend_pixels;
use rayon::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub screen_width: u32,
    pub screen_height: u32,
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
}

pub struct Compositor;

struct LayerRenderCache<'a> {
    layer: &'a Layer,
    active_chunk: Option<&'a Chunk>,
    active_chunk_coords: (i32, i32),
}

impl Compositor {
    pub fn render_from_cache(store: &PixelStore, frame: &mut [u8], view: Viewport) {
        let zoom = view.zoom;
        let inv_zoom = 1.0 / zoom;
        let s_cx = view.screen_width as f32 * 0.5;
        let s_cy = view.screen_height as f32 * 0.5;
        let c_cx = store.canvas_width as f32 * 0.5;
        let c_cy = store.canvas_height as f32 * 0.5;
        let color_bg_empty = [20, 20, 20, 255];
        let canvas_w = store.canvas_width;
        let canvas_h = store.canvas_height;
        let stride = (view.screen_width * 4) as usize;

        frame.par_chunks_exact_mut(stride)
            .enumerate()
            .for_each(|(y_idx, row)| {
                let sy = y_idx as f32 + 0.5;
                let ly = (sy - s_cy) * inv_zoom + c_cy - view.pan_y;
                let ty = ly.floor() as i32;

                let mut current_lx = (0.5f32 - s_cx) * inv_zoom + c_cx - view.pan_x;
                let step_lx = inv_zoom;

                for x in 0..view.screen_width {
                    let tx = current_lx.floor() as i32;
                    current_lx += step_lx;
                    let idx = (x as usize) * 4;

                    if ty >= 0 && ty < canvas_h as i32 && tx >= 0 && tx < canvas_w as i32 {
                        let cache_idx = ((ty as u32 * canvas_w + tx as u32) * 4) as usize;
                        if cache_idx + 4 <= store.composite_cache.len() {
                            row[idx..idx+4].copy_from_slice(&store.composite_cache[cache_idx..cache_idx+4]);
                        }
                    } else {
                        row[idx..idx+4].copy_from_slice(&color_bg_empty);
                    }
                }
            });
    }

    pub fn update_composite_cache(store: &mut PixelStore, rect: Option<(u32, u32, u32, u32)>) {
        let canvas_w = store.canvas_width;
        let canvas_h = store.canvas_height;
        let color_grid_a = [35, 35, 35, 255];
        let color_grid_b = [30, 30, 30, 255];

        let (rx, ry, rw, rh) = rect.unwrap_or((0, 0, canvas_w, canvas_h));
        let x_start = rx.clamp(0, canvas_w);
        let x_end = (rx + rw).clamp(0, canvas_w);
        let y_start = ry.clamp(0, canvas_h);
        let y_end = (ry + rh).clamp(0, canvas_h);

        let layers_refs: Vec<&Layer> = store.layers.iter().filter(|l| l.visible).collect();
        let stride = (canvas_w * 4) as usize;
        let full_range_start = (y_start * canvas_w * 4) as usize;
        let full_range_end = (y_end * canvas_w * 4) as usize;

        if full_range_start >= full_range_end || full_range_end > store.composite_cache.len() { return; }

        store.composite_cache[full_range_start..full_range_end]
            .par_chunks_exact_mut(stride)
            .enumerate()
            .for_each(|(y_offset, row_data)| {
                let ty = y_start + y_offset as u32;
                
                let mut layer_caches: Vec<LayerRenderCache> = layers_refs.iter()
                    .map(|l| LayerRenderCache {
                        layer: l,
                        active_chunk: None,
                        active_chunk_coords: (-999, -999),
                    })
                    .collect();

                for tx in x_start..x_end {
                    let is_even = ((tx >> 3) + (ty >> 3)) % 2 == 0;
                    let mut fc = if is_even { color_grid_a } else { color_grid_b };

                    for cache in &mut layer_caches {
                        let lx = tx as i32 - cache.layer.offset_x;
                        let ly = ty as i32 - cache.layer.offset_y;

                        if lx >= 0 && lx < cache.layer.width as i32 && ly >= 0 && ly < cache.layer.height as i32 {
                            let cx = (lx as u32) / CHUNK_SIZE;
                            let cy = (ly as u32) / CHUNK_SIZE;

                            if (cx as i32, cy as i32) != cache.active_chunk_coords {
                                cache.active_chunk = cache.layer.chunks.get(&(cx, cy));
                                cache.active_chunk_coords = (cx as i32, cy as i32);
                            }

                            if let Some(chunk) = cache.active_chunk {
                                let slx = (lx as u32) % CHUNK_SIZE;
                                let sly = (ly as u32) % CHUNK_SIZE;
                                let chunk_idx = ((sly * CHUNK_SIZE + slx) * 4) as usize;
                                let src = &chunk.data[chunk_idx..chunk_idx+4];
                                if src[3] > 0 {
                                    fc = blend_pixels(fc, [src[0], src[1], src[2], src[3]], cache.layer.blend_mode, cache.layer.opacity);
                                }
                            }
                        }
                    }

                    if store.selection.is_active {
                        if store.selection.contains(tx, ty) {
                            let is_border = tx == 0 || ty == 0 || tx == canvas_w - 1 || ty == canvas_h - 1 ||
                                !store.selection.contains(tx.saturating_sub(1), ty) ||
                                !store.selection.contains(tx + 1, ty) ||
                                !store.selection.contains(tx, ty.saturating_sub(1)) ||
                                !store.selection.contains(tx, ty + 1);
                            if is_border {
                                fc = blend_pixels(fc, [0, 255, 255, 255], crate::core::blend_mode::BlendMode::Normal, 220);
                            }
                        } else {
                            fc = blend_pixels(fc, [0, 0, 0, 255], crate::core::blend_mode::BlendMode::Normal, 120);
                        }
                    }

                    let px_idx = (tx * 4) as usize;
                    if px_idx + 4 <= row_data.len() {
                        row_data[px_idx] = fc[0];
                        row_data[px_idx+1] = fc[1];
                        row_data[px_idx+2] = fc[2];
                        row_data[px_idx+3] = fc[3];
                    }
                }
            });
    }

    pub fn render(store: &PixelStore, frame: &mut [u8], view: Viewport) {
        Self::render_from_cache(store, frame, view);
    }
}