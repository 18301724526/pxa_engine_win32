use crate::core::store::PixelStore;
use crate::history::patch::{ActionPatch, PixelDiffPatch, PixelDiff};
use super::tool_trait::Tool;
use crate::core::color::Color;
use crate::core::layer::{Layer, Chunk, CHUNK_SIZE};
use crate::core::id_gen;
use std::collections::HashMap;
use crate::core::error::CoreError;
use crate::core::symmetry::SymmetryConfig;

pub struct BucketTool {
    backup_chunks: HashMap<(u32, u32), Chunk>,
    active_layer_id: Option<String>,
    dirty_rect: Option<(u32, u32, u32, u32)>,
}

impl BucketTool {
    pub fn new() -> Self {
        Self { 
            backup_chunks: HashMap::new(),
            active_layer_id: None,
            dirty_rect: None,
        }
    }
}

impl Tool for BucketTool {
    fn on_pointer_down(&mut self, x: u32, y: u32, store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        self.backup_chunks.clear();
        self.active_layer_id = None;
        self.dirty_rect = None;

        let layer_id = match &store.active_layer_id {
            Some(id) => id.clone(), None => return Ok(()),
        };
        self.active_layer_id = Some(layer_id.clone());
        if !store.selection.contains(x, y) { return Ok(()); }

        let target_color = store.get_pixel(&layer_id, x, y).unwrap_or(Color::transparent());
        let fill_color = store.primary_color;
        if target_color == fill_color { return Ok(()); }

        let layer = match store.layers.iter_mut().find(|l| l.id == layer_id) {
            Some(l) => l, None => return Ok(()),
        };
        if layer.locked { return Err(CoreError::LayerLocked); }

        let width = layer.width as i32;
        let height = layer.height as i32;
        let offset_x = layer.offset_x;
        let offset_y = layer.offset_y;
        let start_x = x as i32 - offset_x;
        let start_y = y as i32 - offset_y;

        if start_x < 0 || start_x >= width || start_y < 0 || start_y >= height { return Ok(()); }

        let target_u32 = u32::from_le_bytes([target_color.r, target_color.g, target_color.b, target_color.a]);
        let fill_u32 = u32::from_le_bytes([fill_color.r, fill_color.g, fill_color.b, fill_color.a]);

        let mut stack: Vec<(i32, i32)> = Vec::with_capacity(4096);
        stack.push((start_x, start_y));
        
        let mut ctx = SafeFillContext::new(layer, &mut self.backup_chunks, &store.selection);
        let has_selection = store.selection.is_active;

        while let Some((px, py)) = stack.pop() {
            if !ctx.is_fillable(px, py, target_u32, has_selection) {
                continue;
            }

            let mut x1 = px;
            while x1 > 0 && ctx.is_fillable(x1 - 1, py, target_u32, has_selection) {
                x1 -= 1;
            }

            let mut x2 = px;
            while x2 < width - 1 && ctx.is_fillable(x2 + 1, py, target_u32, has_selection) {
                x2 += 1;
            }

            for x in x1..=x2 {
                ctx.set_pixel_u32(x, py, fill_u32);
            }
            
            if x1 < ctx.min_x { ctx.min_x = x1; }
            if x2 > ctx.max_x { ctx.max_x = x2; }
            if py < ctx.min_y { ctx.min_y = py; }
            if py > ctx.max_y { ctx.max_y = py; }

            if py > 0 {
                scan_line(&mut ctx, x1, x2, py - 1, target_u32, &mut stack, has_selection);
            }
            if py < height - 1 {
                scan_line(&mut ctx, x1, x2, py + 1, target_u32, &mut stack, has_selection);
            }
        }

        if ctx.min_x <= ctx.max_x {
            self.dirty_rect = Some((
                (ctx.min_x + offset_x).max(0) as u32,
                (ctx.min_y + offset_y).max(0) as u32,
                (ctx.max_x - ctx.min_x + 1) as u32,
                (ctx.max_y - ctx.min_y + 1) as u32
            ));
        }
        Ok(())
    }

    fn on_pointer_move(&mut self, _x: u32, _y: u32, _store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> { Ok(()) }

    fn on_pointer_up(&mut self, store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError> {
        let layer_id = match self.active_layer_id.take() { Some(id) => id, None => return Ok(None) };
        if self.backup_chunks.is_empty() { return Ok(None) }

        let layer = match store.get_layer(&layer_id) { Some(l) => l, None => return Ok(None) };
        let mut diffs = Vec::with_capacity(self.backup_chunks.len() * 4096);

        let mut backup_keys: Vec<_> = self.backup_chunks.keys().collect();
        backup_keys.sort();

        for key in backup_keys {
            let old_chunk = &self.backup_chunks[key];
            if let Some(new_chunk) = layer.chunks.get(key) {
                if new_chunk.data.as_ref() != old_chunk.data.as_ref() {
                    let base_x = key.0 * CHUNK_SIZE;
                    let base_y = key.1 * CHUNK_SIZE;
                    for i in 0..4096 {
                        let idx = i * 4;
                        let o_slice = &old_chunk.data[idx..idx+4];
                        let n_slice = &new_chunk.data[idx..idx+4];
                        if o_slice != n_slice {
                            diffs.push(PixelDiff {
                                x: base_x + ((i as u32) & 63),
                                y: base_y + ((i as u32) >> 6),
                                old_color: Color::new(o_slice[0], o_slice[1], o_slice[2], o_slice[3]),
                                new_color: Color::new(n_slice[0], n_slice[1], n_slice[2], n_slice[3]),
                            });
                        }
                    }
                }
            }
        }

        let mut layer_keys: Vec<_> = layer.chunks.keys().collect();
        layer_keys.sort();

        for key in layer_keys {
            let new_chunk = &layer.chunks[key];
            if !self.backup_chunks.contains_key(key) {
                let base_x = key.0 * CHUNK_SIZE;
                let base_y = key.1 * CHUNK_SIZE;
                for i in 0..4096 {
                    let idx = i * 4;
                    let n_slice = &new_chunk.data[idx..idx+4];
                    if n_slice != [0, 0, 0, 0] {
                        diffs.push(PixelDiff {
                            x: base_x + ((i as u32) & 63),
                            y: base_y + ((i as u32) >> 6),
                            old_color: Color::transparent(),
                            new_color: Color::new(n_slice[0], n_slice[1], n_slice[2], n_slice[3]),
                        });
                    }
                }
            }
        }

        self.backup_chunks.clear();

        if diffs.is_empty() { return Ok(None); }
        
        let mut patch = ActionPatch::new_pixel_diff(id_gen::gen_id(), layer_id);
        if let Some(p) = patch.action.as_any_mut().downcast_mut::<PixelDiffPatch>() {
            p.diffs = diffs;
        }    
        Ok(Some(patch))
    }
    
    fn take_dirty_rect(&mut self) -> Option<(u32, u32, u32, u32)> {
        self.dirty_rect.take()
    }

    fn on_commit(&mut self, store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError> {
        self.on_pointer_up(store)
    }

    fn on_cancel(&mut self, _store: &mut PixelStore) {
        self.backup_chunks.clear();
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

#[inline(always)]
fn scan_line(ctx: &mut SafeFillContext, x1: i32, x2: i32, y: i32, target_u32: u32, stack: &mut Vec<(i32, i32)>, has_selection: bool) {
    let mut in_span = false;
    for x in x1..=x2 {
        if ctx.is_fillable(x, y, target_u32, has_selection) {
            if !in_span {
                stack.push((x, y));
                in_span = true;
            }
        } else {
            in_span = false;
        }
    }
}

struct SafeFillContext<'a> {
    layer: &'a mut Layer,
    backups: &'a mut HashMap<(u32, u32), Chunk>,
    selection: &'a crate::core::selection::SelectionData,
    min_x: i32, min_y: i32, max_x: i32, max_y: i32,
}

impl<'a> SafeFillContext<'a> {
    fn new(layer: &'a mut Layer, backups: &'a mut HashMap<(u32, u32), Chunk>, selection: &'a crate::core::selection::SelectionData) -> Self {
        Self {
            layer,
            backups,
            selection,
            min_x: i32::MAX, 
            min_y: i32::MAX, 
            max_x: i32::MIN, 
            max_y: i32::MIN,
        }
    }

    #[inline(always)]
    fn get_pixel_u32(&self, x: i32, y: i32) -> u32 {
        let cx = (x as u32) / CHUNK_SIZE;
        let cy = (y as u32) / CHUNK_SIZE;
        
        if let Some(chunk) = self.layer.chunks.get(&(cx, cy)) {
            let lx = (x as u32) % CHUNK_SIZE;
            let ly = (y as u32) % CHUNK_SIZE;
            let idx = ((ly * CHUNK_SIZE + lx) * 4) as usize;
            let d = &chunk.data;
            u32::from_le_bytes([d[idx], d[idx+1], d[idx+2], d[idx+3]])
        } else {
            0
        }
    }

    #[inline(always)]
    fn is_fillable(&self, x: i32, y: i32, target_u32: u32, has_selection: bool) -> bool {
        let canvas_x = x + self.layer.offset_x;
        let canvas_y = y + self.layer.offset_y;

        if canvas_x < 0 || canvas_y < 0 || (canvas_x as u32) >= self.selection.width || (canvas_y as u32) >= self.selection.height { 
            return false; 
        }

        if has_selection && unsafe { !*self.selection.mask.get_unchecked((canvas_y as u32 * self.selection.width + canvas_x as u32) as usize) } {
            return false;
        }
        
        self.get_pixel_u32(x, y) == target_u32
    }

    #[inline(always)]
    fn set_pixel_u32(&mut self, x: i32, y: i32, fill_u32: u32) {
        let cx = (x as u32) / CHUNK_SIZE;
        let cy = (y as u32) / CHUNK_SIZE;
        let k = (cx, cy);

        let alpha = (fill_u32 >> 24) & 0xFF;
        if alpha == 0 && !self.layer.chunks.contains_key(&k) {
             return;
        }

        if !self.backups.contains_key(&k) {
            if let Some(existing) = self.layer.chunks.get(&k) {
                self.backups.insert(k, existing.clone());
            }
        }

        let chunk = self.layer.chunks.entry(k).or_insert_with(Chunk::new);
        
        let lx = (x as u32) % CHUNK_SIZE;
        let ly = (y as u32) % CHUNK_SIZE;
        
        let idx = ((ly * CHUNK_SIZE + lx) * 4) as usize;
        let bytes = fill_u32.to_le_bytes();
        chunk.data_mut()[idx..idx+4].copy_from_slice(&bytes);
    }
}