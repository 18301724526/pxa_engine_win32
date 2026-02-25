use crate::core::store::PixelStore;
use crate::history::patch::ActionPatch;
use super::tool_trait::Tool;
use crate::core::id_gen;
use crate::core::symmetry::SymmetryConfig;
use crate::core::layer::{Layer, CHUNK_SIZE};
use crate::core::selection::SelectionData;
use crate::core::color::Color;
use crate::core::error::CoreError;

pub struct MoveTool {
    start_pos: Option<(i32, i32)>,
    layer_backup: Option<Layer>,
    sel_backup: Option<SelectionData>,
    extracted_pixels: Vec<(u32, u32, Color)>, 
    needs_redraw: bool,
}

impl MoveTool {
    pub fn new() -> Self {
        Self { start_pos: None, layer_backup: None, sel_backup: None, extracted_pixels: Vec::new(), needs_redraw: false }
    }
}

impl Tool for MoveTool {
    fn on_pointer_down(&mut self, x: u32, y: u32, store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        let layer_id = match &store.active_layer_id { Some(id) => id.clone(), None => return Ok(()) };
        let layer = match store.get_layer(&layer_id) { Some(l) => l, None => return Ok(()) };
        if layer.locked { return Err(CoreError::LayerLocked); }

        self.layer_backup = Some(layer.clone());
        self.sel_backup = Some(store.selection.clone());
        self.start_pos = Some((x as i32, y as i32));
        self.extracted_pixels.clear();
        if store.selection.is_active {
            let (w, h) = (layer.width, layer.height);
            let (off_x, off_y) = (layer.offset_x, layer.offset_y);
            for cy in 0..h {
                for cx in 0..w {
                    let canvas_x = cx as i32 + off_x;
                    let canvas_y = cy as i32 + off_y;
                    if canvas_x >= 0 && canvas_y >= 0 && store.selection.contains(canvas_x as u32, canvas_y as u32) {
                        if let Some(color) = layer.get_pixel(cx, cy) {
                            if color.a > 0 { self.extracted_pixels.push((cx, cy, color)); }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn on_pointer_move(&mut self, x: u32, y: u32, store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        let (sx, sy) = match self.start_pos { Some(p) => p, None => return Ok(()) };
        let dx = x as i32 - sx;
        let dy = y as i32 - sy;
        if dx == 0 && dy == 0 { return Ok(()); }

        let layer_id = match store.active_layer_id.clone() { Some(id) => id, None => return Ok(()) };
        let sel_backup = match self.sel_backup.as_ref() { Some(s) => s, None => return Ok(()) };
        let initial_offset_x = self.layer_backup.as_ref().map(|l| l.offset_x).unwrap_or(0);
        let initial_offset_y = self.layer_backup.as_ref().map(|l| l.offset_y).unwrap_or(0);

        if !sel_backup.is_active {
            if let Some(layer) = store.get_layer_mut(&layer_id) {
                layer.offset_x = initial_offset_x + dx;
                layer.offset_y = initial_offset_y + dy;
                self.needs_redraw = true;
            }
        } else {
            if let Some(layer) = store.get_layer_mut(&layer_id) {
                if let Some(backup) = &self.layer_backup {
                    layer.chunks = backup.chunks.clone();
                }
                for &(cx, cy, _) in &self.extracted_pixels {
                    layer.set_pixel(cx, cy, Color::transparent())?;
                }
                for &(cx, cy, color) in &self.extracted_pixels {
                    let new_cx = cx as i32 + dx;
                    let new_cy = cy as i32 + dy;
                    if new_cx >= 0 && new_cx < layer.width as i32 && new_cy >= 0 && new_cy < layer.height as i32 {
                        layer.set_pixel(new_cx as u32, new_cy as u32, color)?;
                    }
                }
            }
            let mut new_mask = vec![false; (sel_backup.width * sel_backup.height) as usize];
            for sy in 0..sel_backup.height {
                for sx in 0..sel_backup.width {
                    if sel_backup.contains(sx, sy) {
                        let new_sx = sx as i32 + dx;
                        let new_sy = sy as i32 + dy;
                        if new_sx >= 0 && new_sx < sel_backup.width as i32 && new_sy >= 0 && new_sy < sel_backup.height as i32 {
                            new_mask[(new_sy as u32 * sel_backup.width + new_sx as u32) as usize] = true;
                        }
                    }
                }
            }
            store.selection.mask = new_mask;
            store.selection.is_active = true;
            self.needs_redraw = true;
        }
        Ok(())
    }

    fn on_pointer_up(&mut self, store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError> {
        self.start_pos = None;
        self.extracted_pixels.clear();
        let backup = match self.layer_backup.take() { Some(b) => b, None => return Ok(None) };
        let sel_backup = match self.sel_backup.take() { Some(s) => s, None => return Ok(None) };
        let layer_id = match store.active_layer_id.clone() { Some(id) => id, None => return Ok(None) };
        let current = match store.get_layer(&layer_id) { Some(l) => l, None => return Ok(None) };

        if !sel_backup.is_active {
            if backup.offset_x == current.offset_x && backup.offset_y == current.offset_y {
                return Ok(None);
            }
            if backup.offset_x != current.offset_x || backup.offset_y != current.offset_y {
                return Ok(Some(ActionPatch::new_layer_offset(id_gen::gen_id(), layer_id, (backup.offset_x, backup.offset_y), (current.offset_x, current.offset_y))));
            }
        } else {
            let mut sub_patches = Vec::new();
            let new_sel = store.selection.clone();
            
            if sel_backup.mask != new_sel.mask || sel_backup.is_active != new_sel.is_active {
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
                                let x = key.0 * CHUNK_SIZE + lx;
                                let y = key.1 * CHUNK_SIZE + ly;
                                let old_c = Color::new(old_chunk.data[i], old_chunk.data[i+1], old_chunk.data[i+2], old_chunk.data[i+3]);
                                let new_c = Color::new(new_chunk.data[i], new_chunk.data[i+1], new_chunk.data[i+2], new_chunk.data[i+3]);
                                pixel_patch.add_pixel_diff(x, y, old_c, new_c);
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
                            let x = key.0 * CHUNK_SIZE + lx;
                            let y = key.1 * CHUNK_SIZE + ly;
                            let new_c = Color::new(new_chunk.data[i], new_chunk.data[i+1], new_chunk.data[i+2], new_chunk.data[i+3]);
                            pixel_patch.add_pixel_diff(x, y, Color::transparent(), new_c);
                        }
                    }
                }
            }

            if !pixel_patch.is_empty() { sub_patches.push(pixel_patch); }

            if !sub_patches.is_empty() {
                return Ok(Some(ActionPatch::new_composite(id_gen::gen_id(), sub_patches)));
            }
        }
        Ok(None)
    }

fn take_dirty_rect(&mut self) -> Option<(u32, u32, u32, u32)> {
        if self.needs_redraw {
            self.needs_redraw = false;
            Some((0, 0, u32::MAX, u32::MAX))
        } else { None }
    }

    fn on_commit(&mut self, store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError> {
        self.on_pointer_up(store)
    }

    fn on_cancel(&mut self, store: &mut PixelStore) {
        if let Some(backup) = self.layer_backup.take() {
            if let Some(id) = store.active_layer_id.clone() {
                if let Some(layer) = store.get_layer_mut(&id) {
                    layer.chunks = backup.chunks;
                    layer.offset_x = backup.offset_x;
                    layer.offset_y = backup.offset_y;
                }
            }
        }
        if let Some(sel) = self.sel_backup.take() { store.selection = sel; }
        self.start_pos = None;
        self.extracted_pixels.clear();
        self.needs_redraw = true;
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}