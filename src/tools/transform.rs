use crate::core::store::PixelStore;
use crate::history::patch::ActionPatch;
use super::tool_trait::Tool;
use crate::core::symmetry::SymmetryConfig;
use crate::core::layer::{Layer, Chunk, CHUNK_SIZE};
use crate::core::color::Color;
use crate::core::selection::SelectionData;
use crate::core::error::CoreError;
use crate::core::id_gen;
use std::collections::{HashMap, HashSet};
use rust_i18n::t;

#[derive(Clone, Copy, PartialEq)]
pub enum DragMode {
    None,
    Move,
    Scale(f32, f32),
    Rotate,
}

#[derive(Clone)]
pub struct ExtractedImage {
    pub min_x: i32, pub min_y: i32, pub width: u32, pub height: u32,
    pub pixels: Vec<Color>, pub mask: Vec<bool>, 
}

pub struct TransformTool {
    pub is_active: bool,
    pub start_pos: Option<(i32, i32)>,
    pub extracted: Option<ExtractedImage>,
    pub layer_backup: Option<Layer>,
    pub sel_backup: Option<SelectionData>,
    pub erased_chunks: HashMap<(u32, u32), Chunk>,
    pub preview_chunks: HashSet<(u32, u32)>,

    pub pivot_x: f32, pub pivot_y: f32,
    pub offset_x: f32, pub offset_y: f32,
    pub scale_x: f32, pub scale_y: f32,
    pub rotation: f32,
    pub needs_redraw: bool,

    drag_mode: DragMode,
    base_scale_x: f32,
    base_scale_y: f32,
    base_rotation: f32,
    start_angle: f32,
}

impl TransformTool {
    pub fn new() -> Self {
        Self {
            is_active: false,
            start_pos: None, extracted: None, layer_backup: None, sel_backup: None,
            erased_chunks: HashMap::new(),
            preview_chunks: HashSet::new(),
            pivot_x: 0.0, pivot_y: 0.0, offset_x: 0.0, offset_y: 0.0,
            scale_x: 1.0, scale_y: 1.0, rotation: 0.0, needs_redraw: false,
            drag_mode: DragMode::None,
            base_scale_x: 1.0, base_scale_y: 1.0, base_rotation: 0.0, start_angle: 0.0,
        }
    }

    pub fn get_transform_params(&self) -> Option<(f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32)> {
        if !self.is_active { return None; }
        if let Some(img) = &self.extracted {
            Some((
                img.min_x as f32, img.min_y as f32, img.width as f32, img.height as f32,
                self.pivot_x, self.pivot_y, self.offset_x, self.offset_y,
                self.scale_x, self.scale_y, self.rotation
            ))
        } else { None }
    }

    fn extract_pixels(&mut self, store: &mut PixelStore, layer_id: &str) -> Result<(), CoreError> {
        let selection = store.selection.clone();
        self.sel_backup = Some(selection.clone());
        let layer = match store.get_layer_mut(layer_id) { Some(l) => l, None => return Ok(()) };
        if layer.locked { return Err(CoreError::LayerLocked); }
        self.layer_backup = Some(layer.clone());
        
        let mut min_x = i32::MAX; let mut min_y = i32::MAX;
        let mut max_x = i32::MIN; let mut max_y = i32::MIN;

        if selection.is_active {
            for cy in 0..selection.height {
                for cx in 0..selection.width {
                    if selection.contains(cx, cy) {
                        if (cx as i32) < min_x { min_x = cx as i32; }
                        if (cy as i32) < min_y { min_y = cy as i32; }
                        if (cx as i32) > max_x { max_x = cx as i32; }
                        if (cy as i32) > max_y { max_y = cy as i32; }
                    }
                }
            }
        } else {
            min_x = layer.offset_x; min_y = layer.offset_y;
            max_x = layer.offset_x + layer.width as i32 - 1;
            max_y = layer.offset_y + layer.height as i32 - 1;
        }
        
        if min_x > max_x || min_y > max_y { return Ok(()); }
        
        let w = (max_x - min_x + 1) as u32;
        let h = (max_y - min_y + 1) as u32;
        let mut pixels = vec![Color::transparent(); (w * h) as usize];
        let mut mask = vec![false; (w * h) as usize];
        let off_x = layer.offset_x; let off_y = layer.offset_y;
        
        for cy in 0..h {
            for cx in 0..w {
                let canvas_x = min_x + cx as i32;
                let canvas_y = min_y + cy as i32;

                if !selection.is_active || selection.contains(canvas_x as u32, canvas_y as u32) {
                    let idx = (cy * w + cx) as usize;
                    mask[idx] = true;

                    let lx = canvas_x - off_x;
                    let ly = canvas_y - off_y;
                    if lx >= 0 && ly >= 0 && lx < layer.width as i32 && ly < layer.height as i32 {
                        if let Some(color) = layer.get_pixel(lx as u32, ly as u32) {
                            pixels[idx] = color;
                        }
                    }
                }
            }
        }
        
        self.extracted = Some(ExtractedImage { min_x, min_y, width: w, height: h, pixels, mask });
        self.pivot_x = min_x as f32 + w as f32 / 2.0;
        self.pivot_y = min_y as f32 + h as f32 / 2.0;
        self.offset_x = 0.0; self.offset_y = 0.0;
        self.scale_x = 1.0; self.scale_y = 1.0;
        self.rotation = 0.0;
        self.is_active = true;

        self.erased_chunks.clear();
        self.preview_chunks.clear();

        if let Some(ExtractedImage { width: ew, height: eh, mask: emask, min_x: eminx, min_y: eminy, .. }) = &self.extracted {
            for cy in 0..*eh {
                for cx in 0..*ew {
                    let idx = (cy * ew + cx) as usize;
                    if emask[idx] {
                        let lx = eminx + cx as i32 - off_x;
                        let ly = eminy + cy as i32 - off_y;
                        if lx >= 0 && ly >= 0 && lx < layer.width as i32 && ly < layer.height as i32 {
                            let chunk_x = lx as u32 / CHUNK_SIZE;
                            let chunk_y = ly as u32 / CHUNK_SIZE;
                            let k = (chunk_x, chunk_y);

                            let chunk = self.erased_chunks.entry(k).or_insert_with(|| {
                                layer.chunks.get(&k).cloned().unwrap_or_else(Chunk::new)
                            });
                            let slx = lx as u32 % CHUNK_SIZE;
                            let sly = ly as u32 % CHUNK_SIZE;
                            let c_idx = ((sly * CHUNK_SIZE + slx) * 4) as usize;
                            let data = chunk.data_mut();
                            data[c_idx] = 0; data[c_idx+1] = 0;
                            data[c_idx+2] = 0; data[c_idx+3] = 0;
                        }
                    }
                }
            }
        }
        for (k, v) in &self.erased_chunks {
            layer.chunks.insert(*k, v.clone());
        }
        Ok(())
    }

    fn apply_preview(&mut self, store: &mut PixelStore) -> Result<(), CoreError> {
        let layer_id = match &store.active_layer_id { Some(id) => id.clone(), None => return Ok(()) };
        let img = match &self.extracted { Some(i) => i, None => return Ok(()) };
        let backup = match &self.layer_backup { Some(l) => l, None => return Ok(()) };
        let sel_backup = match &self.sel_backup { Some(s) => s, None => return Ok(()) };

        let mut new_selection = SelectionData::new(store.canvas_width, store.canvas_height);
        new_selection.is_active = sel_backup.is_active;

        let cos_t = self.rotation.cos();
        let sin_t = self.rotation.sin();

        let corners = [
            (0.0, 0.0), (img.width as f32, 0.0),
            (0.0, img.height as f32), (img.width as f32, img.height as f32),
        ];

        let mut out_min_x = f32::MAX; let mut out_min_y = f32::MAX;
        let mut out_max_x = f32::MIN; let mut out_max_y = f32::MIN;

        for (cx, cy) in corners {
            let x = img.min_x as f32 + cx - self.pivot_x;
            let y = img.min_y as f32 + cy - self.pivot_y;
            let sx = x * self.scale_x; let sy = y * self.scale_y;
            let rx = sx * cos_t - sy * sin_t; let ry = sx * sin_t + sy * cos_t;
            let fx = rx + self.pivot_x + self.offset_x;
            let fy = ry + self.pivot_y + self.offset_y;
            
            if fx < out_min_x { out_min_x = fx; } if fx > out_max_x { out_max_x = fx; }
            if fy < out_min_y { out_min_y = fy; } if fy > out_max_y { out_max_y = fy; }
        }

        let start_x = out_min_x.floor() as i32; let start_y = out_min_y.floor() as i32;
        let end_x = out_max_x.ceil() as i32; let end_y = out_max_y.ceil() as i32;
        let canvas_w = store.canvas_width; let canvas_h = store.canvas_height;

        if let Some(layer) = store.get_layer_mut(&layer_id) {
            for &k in &self.preview_chunks {
                if let Some(erased) = self.erased_chunks.get(&k) {
                    layer.chunks.insert(k, erased.clone());
                } else if let Some(backup_chunk) = backup.chunks.get(&k) {
                    layer.chunks.insert(k, backup_chunk.clone());
                } else {
                    layer.chunks.remove(&k);
                }
            }
            self.preview_chunks.clear();
            let off_x = layer.offset_x; let off_y = layer.offset_y;

            for dy in start_y..=end_y {
                for dx in start_x..=end_x {
                    let x1 = dx as f32 - self.pivot_x - self.offset_x;
                    let y1 = dy as f32 - self.pivot_y - self.offset_y;
                    
                    let x2 = x1 * cos_t + y1 * sin_t;
                    let y2 = -x1 * sin_t + y1 * cos_t;
                    
                    let x3 = if self.scale_x != 0.0 { x2 / self.scale_x } else { 0.0 };
                    let y3 = if self.scale_y != 0.0 { y2 / self.scale_y } else { 0.0 };
                    
                    let sx = (x3 + self.pivot_x - img.min_x as f32).round() as i32;
                    let sy = (y3 + self.pivot_y - img.min_y as f32).round() as i32;

                    if sx >= 0 && sy >= 0 && sx < img.width as i32 && sy < img.height as i32 {
                        let idx = (sy * img.width as i32 + sx) as usize;
                        if img.mask[idx] {
                            if dx >= 0 && dy >= 0 && dx < canvas_w as i32 && dy < canvas_h as i32 {
                                new_selection.mask[(dy as u32 * canvas_w + dx as u32) as usize] = true;
                            }

                            let color = img.pixels[idx];
                            if color.a > 0 {
                                let lx = dx - off_x; let ly = dy - off_y;
                                if lx >= 0 && ly >= 0 && lx < layer.width as i32 && ly < layer.height as i32 {
                                    let chunk_x = lx as u32 / CHUNK_SIZE;
                                    let chunk_y = ly as u32 / CHUNK_SIZE;
                                    let k = (chunk_x, chunk_y);

                                    if self.preview_chunks.insert(k) {
                                        let base_chunk = self.erased_chunks.get(&k)
                                            .or_else(|| backup.chunks.get(&k))
                                            .cloned()
                                            .unwrap_or_else(Chunk::new);
                                        layer.chunks.insert(k, base_chunk);
                                    }

                                    let chunk = layer.chunks.get_mut(&k).ok_or_else(|| {
                                        CoreError::LayerNotFound(t!("error.transform_preview_failed", chunk = format!("{:?}", k)).to_string())
                                    })?;
                                    let slx = lx as u32 % CHUNK_SIZE;
                                    let sly = ly as u32 % CHUNK_SIZE;
                                    let c_idx = ((sly * CHUNK_SIZE + slx) * 4) as usize;
                                    let data = chunk.data_mut();
                                    data[c_idx] = color.r;
                                    data[c_idx+1] = color.g;
                                    data[c_idx+2] = color.b;
                                    data[c_idx+3] = color.a;
                                }
                            }
                        }
                    }
                }
            }
        } 
        store.selection = new_selection;
        self.needs_redraw = true;
        Ok(())
    }
}

impl Tool for TransformTool {
    fn on_pointer_down(&mut self, x: u32, y: u32, store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        let layer_id = match &store.active_layer_id { Some(id) => id.clone(), None => return Ok(()) };
        if !self.is_active { self.extract_pixels(store, &layer_id)?; }
        
        if let Some(img) = &self.extracted {
            let cos_t = self.rotation.cos();
            let sin_t = self.rotation.sin();
            let hw = img.width as f32 / 2.0;
            let hh = img.height as f32 / 2.0;
            
            let mut hit_mode = DragMode::None;
            let hit_radius = 10.0;

            let handles = [
                (-1.0, -1.0), (0.0, -1.0), (1.0, -1.0),
                (-1.0,  0.0),              (1.0,  0.0),
                (-1.0,  1.0), (0.0,  1.0), (1.0,  1.0),
            ];

            for (dx, dy) in handles {
                let cx = img.min_x as f32 + hw + dx * hw;
                let cy = img.min_y as f32 + hh + dy * hh;
                let tx = (cx - self.pivot_x) * self.scale_x;
                let ty = (cy - self.pivot_y) * self.scale_y;
                let rx = tx * cos_t - ty * sin_t + self.pivot_x + self.offset_x;
                let ry = tx * sin_t + ty * cos_t + self.pivot_y + self.offset_y;

                let dist = ((x as f32 - rx).powi(2) + (y as f32 - ry).powi(2)).sqrt();
                if dist < hit_radius {
                    hit_mode = DragMode::Scale(dx, dy);
                    break;
                }
            }

            if hit_mode == DragMode::None {
                let lx = x as f32 - self.pivot_x - self.offset_x;
                let ly = y as f32 - self.pivot_y - self.offset_y;
                let u_x = lx * cos_t + ly * sin_t;
                let u_y = -lx * sin_t + ly * cos_t;
                let scaled_hw = hw * self.scale_x.abs();
                let scaled_hh = hh * self.scale_y.abs();
                
                if u_x.abs() <= scaled_hw && u_y.abs() <= scaled_hh {
                    hit_mode = DragMode::Move;
                } else {
                    hit_mode = DragMode::Rotate;
                }
            }

            self.drag_mode = hit_mode;
            self.start_pos = Some((x as i32, y as i32));

            self.base_scale_x = self.scale_x;
            self.base_scale_y = self.scale_y;
            self.base_rotation = self.rotation;
            self.start_angle = (y as f32 - (self.pivot_y + self.offset_y)).atan2(x as f32 - (self.pivot_x + self.offset_x));
        }
        Ok(())
    }

    fn on_pointer_move(&mut self, x: u32, y: u32, store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        if !self.is_active || self.drag_mode == DragMode::None { return Ok(()); }
        if let Some((sx, sy)) = self.start_pos {
            let cx = x as f32; let cy = y as f32;
            let st_x = sx as f32; let st_y = sy as f32;
            
            match self.drag_mode {
                DragMode::Move => {
                    self.offset_x += cx - st_x;
                    self.offset_y += cy - st_y;
                    self.start_pos = Some((x as i32, y as i32));
                }
                DragMode::Rotate => {
                    let current_angle = (cy - (self.pivot_y + self.offset_y)).atan2(cx - (self.pivot_x + self.offset_x));
                    let angle_diff = current_angle - self.start_angle;
                    self.rotation = self.base_rotation + angle_diff;
                }
                DragMode::Scale(dir_x, dir_y) => {
                    if let Some(img) = &self.extracted {
                        let hw = img.width as f32 / 2.0;
                        let hh = img.height as f32 / 2.0;
                        let dx = cx - st_x;
                        let dy = cy - st_y;
                        
                        let cos_t = self.base_rotation.cos();
                        let sin_t = self.base_rotation.sin();
                        let local_dx = dx * cos_t + dy * sin_t;
                        let local_dy = -dx * sin_t + dy * cos_t;
                        
                        if dir_x != 0.0 { self.scale_x = self.base_scale_x + (local_dx * dir_x) / hw; }
                        if dir_y != 0.0 { self.scale_y = self.base_scale_y + (local_dy * dir_y) / hh; }
                    }
                }
                _ => {}
            }
            
            self.apply_preview(store)?;
        }
        Ok(())
    }

    fn on_pointer_up(&mut self, _store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError> {
        self.start_pos = None;
        self.drag_mode = DragMode::None;
        Ok(None)
    }
    
    fn on_commit(&mut self, store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError> {
        if !self.is_active { return Ok(None); }
        self.is_active = false;
        
        let layer_id = match store.active_layer_id.clone() { Some(id) => id, None => return Ok(None) };
        let backup = match self.layer_backup.take() { Some(b) => b, None => return Ok(None) };
        let sel_backup = match self.sel_backup.take() { Some(s) => s, None => return Ok(None) };
        let current = match store.get_layer(&layer_id) { Some(l) => l, None => return Ok(None) };

        let mut sub_patches = Vec::new();
        let new_sel = store.selection.clone();
        
        if sel_backup != new_sel {
            sub_patches.push(ActionPatch::new_selection_change(id_gen::gen_id(), sel_backup.clone(), new_sel));
        }

        let mut pixel_patch = ActionPatch::new_pixel_diff(id_gen::gen_id(), layer_id.clone());
        for (key, old_chunk) in &backup.chunks {
            if let Some(new_chunk) = current.chunks.get(key) {
                if new_chunk.data.as_ref() != old_chunk.data.as_ref() {
                    for i in (0..new_chunk.data.len()).step_by(4) {
                        if new_chunk.data[i..i+4] != old_chunk.data[i..i+4] {
                            let ly = (i / 4) as u32 / CHUNK_SIZE;
                            let lx = (i / 4) as u32 % CHUNK_SIZE;
                            let px = key.0 * CHUNK_SIZE + lx;
                            let py = key.1 * CHUNK_SIZE + ly;
                            let old_c = Color::new(old_chunk.data[i], old_chunk.data[i+1], old_chunk.data[i+2], old_chunk.data[i+3]);
                            let new_c = Color::new(new_chunk.data[i], new_chunk.data[i+1], new_chunk.data[i+2], new_chunk.data[i+3]);
                            pixel_patch.add_pixel_diff(px, py, old_c, new_c);
                        }
                    }
                }
            }
        }
        for (key, new_chunk) in &current.chunks {
            if !backup.chunks.contains_key(key) {
                for i in (0..new_chunk.data.len()).step_by(4) {
                    if new_chunk.data[i+3] > 0 {
                        let ly = (i / 4) as u32 / CHUNK_SIZE;
                        let lx = (i / 4) as u32 % CHUNK_SIZE;
                        let px = key.0 * CHUNK_SIZE + lx;
                        let py = key.1 * CHUNK_SIZE + ly;
                        let new_c = Color::new(new_chunk.data[i], new_chunk.data[i+1], new_chunk.data[i+2], new_chunk.data[i+3]);
                        pixel_patch.add_pixel_diff(px, py, Color::transparent(), new_c);
                    }
                }
            }
        }

        if !pixel_patch.is_empty() { sub_patches.push(pixel_patch); }

        if !sub_patches.is_empty() {
            return Ok(Some(ActionPatch::new_composite(id_gen::gen_id(), sub_patches)));
        }
        Ok(None)
    }
    
    fn on_cancel(&mut self, store: &mut PixelStore) {
        if !self.is_active { return; }
        self.is_active = false;
        
        let active_id = store.active_layer_id.clone();
        if let Some(backup) = self.layer_backup.take() {
            if let Some(id) = active_id {
                if let Some(layer) = store.get_layer_mut(&id) { layer.chunks = backup.chunks; }
            }
        }
        if let Some(sel) = self.sel_backup.take() { store.selection = sel; }
        self.needs_redraw = true;
    }

    fn take_dirty_rect(&mut self) -> Option<(u32, u32, u32, u32)> {
        if self.needs_redraw {
            self.needs_redraw = false;
            Some((0, 0, u32::MAX, u32::MAX))
        } else { None }
    }
    
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}