use crate::core::store::PixelStore;
use crate::history::patch::ActionPatch;
use super::tool_trait::Tool;
use super::geometry::Geometry;
use crate::core::id_gen;
use std::cmp::{min, max};
use std::collections::HashMap;
use crate::core::color::Color;
use crate::core::error::CoreError;
use crate::core::symmetry::SymmetryConfig;

pub struct PencilTool {
    pub original_pixels: HashMap<(u32, u32), Color>,
    pub last_pos: Option<(i32, i32)>,
    pub is_eraser: bool,
    pub dirty_rect: Option<(u32, u32, u32, u32)>,
    pub active_layer_id: Option<String>,
    rng_state: u32,
}

impl PencilTool {
    pub fn new(is_eraser: bool) -> Self {
        Self {
            original_pixels: HashMap::new(),
            last_pos: None,
            is_eraser,
            dirty_rect: None,
            active_layer_id: None,
            rng_state: 1337,
        }
    }

    fn update_dirty_rect_internal(&mut self, canvas_x: i32, canvas_y: i32, size: u32) {
        let offset = (size / 2) as i32;
        let x1 = max(0, canvas_x - offset) as u32;
        let y1 = max(0, canvas_y - offset) as u32;
        let x2 = max(0, canvas_x - offset + size as i32) as u32;
        let y2 = max(0, canvas_y - offset + size as i32) as u32;

        if x1 >= x2 || y1 >= y2 { return; }

        if let Some((rx1, ry1, rx2, ry2)) = self.dirty_rect {
            self.dirty_rect = Some((min(rx1, x1), min(ry1, y1), max(rx2, x2), max(ry2, y2)));
        } else {
            self.dirty_rect = Some((x1, y1, x2, y2));
        }
    }

    fn paint_brush_at(&mut self, cx: i32, cy: i32, store: &mut PixelStore, symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        let layer_id = match &self.active_layer_id {
            Some(id) => id.clone(),
            None => return Ok(()),
        };
        
        let brush_size = store.brush_size;
        let color = if self.is_eraser { Color::transparent() } else { store.primary_color };
        let (offset_x, offset_y, l_width, l_height) = match store.get_layer(&layer_id) {
            Some(l) => (l.offset_x, l.offset_y, l.width, l.height),
            None => return Ok(()),
        };
        
        let mut res = Ok(());
        let layer_id_inner = layer_id.clone();

        let size = brush_size as i32;
        let offset = size / 2;
        let shape = store.brush_shape;
        let jitter = store.brush_jitter;
        let mut rel_points = Vec::with_capacity((size * size) as usize);

        for i in 0..size {
            for j in 0..size {
                if shape == crate::core::store::BrushShape::Circle && size > 2 {
                    let dx = (i as f32 - offset as f32) + 0.5;
                    let dy = (j as f32 - offset as f32) + 0.5;
                    let radius = size as f32 / 2.0;
                    if dx * dx + dy * dy > radius * radius { continue; }
                }
                
                let mut rel_x = i - offset;
                let mut rel_y = j - offset;
                
                if jitter > 0 {
                    let bound = jitter * 2 + 1;
                    self.rng_state = self.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
                    rel_x += ((self.rng_state >> 16) % bound) as i32 - jitter as i32;
                    self.rng_state = self.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
                    rel_y += ((self.rng_state >> 16) % bound) as i32 - jitter as i32;
                }
                rel_points.push((rel_x, rel_y));
            }
        }
        
        symmetry.apply_symmetry(cx, cy, |tx, ty| {
            if res.is_err() { return; }
            self.update_dirty_rect_internal(tx, ty, brush_size + store.brush_jitter * 2);
            for (rx, ry) in &rel_points {
                let px = tx + rx;
                let py = ty + ry;
                    
                    if px >= 0 && py >= 0 {
                        let px_u = px as u32;
                        let py_u = py as u32;
                        let lx = px - offset_x;
                        let ly = py - offset_y;

                        if lx >= 0 && ly >= 0 && lx < l_width as i32 && ly < l_height as i32 {
                            let current_color = store.get_pixel(&layer_id_inner, px_u, py_u).unwrap_or(Color::transparent());
                            
                            if current_color == color {
                                continue;
                            }

                            let lx_u = lx as u32;
                            let ly_u = ly as u32;
                            
                            if !self.original_pixels.contains_key(&(lx_u, ly_u)) {
                                self.original_pixels.insert((lx_u, ly_u), current_color);
                            }

                            if let Err(e) = store.mut_set_pixel(&layer_id_inner, px_u, py_u, color) {
                                res = Err(e);
                            }
                        }
                    }
                }
            
        });
        res
    }
}

impl Tool for PencilTool {
    fn on_pointer_down(&mut self, x: u32, y: u32, store: &mut PixelStore, symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        if let Some(id) = &store.active_layer_id {
            self.active_layer_id = Some(id.clone());
            self.original_pixels.clear();
            self.dirty_rect = None;
            self.last_pos = Some((x as i32, y as i32));
            self.paint_brush_at(x as i32, y as i32, store, symmetry)?;
        }
        Ok(())
    }

    fn on_pointer_move(&mut self, x: u32, y: u32, store: &mut PixelStore, symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        if self.active_layer_id.is_none() { return Ok(()); }

        if let Some((last_x, last_y)) = self.last_pos {
            let cur_x = x as i32;
            let cur_y = y as i32;
            if last_x == cur_x && last_y == cur_y { return Ok(()); }

            let mut is_first_point = true;
            let mut res = Ok(());
            Geometry::bresenham_line(last_x, last_y, cur_x, cur_y, |tx, ty| {
                if res.is_err() { return; }
                if is_first_point { is_first_point = false; return; }
                res = self.paint_brush_at(tx, ty, store, symmetry);
            });
            res?;
            self.last_pos = Some((cur_x, cur_y));
        }
        Ok(())
    }

    fn on_pointer_up(&mut self, store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError> {
        let layer_id = match self.active_layer_id.take() { 
            Some(id) => id, 
            None => return Ok(None) 
        };
        self.last_pos = None;
        
        let layer = match store.get_layer(&layer_id) { Some(l) => l, None => return Ok(None) };
        
        let mut patch = ActionPatch::new_pixel_diff(id_gen::gen_id(), layer_id.clone());
        
        for (&(lx, ly), &old_color) in &self.original_pixels {
            let new_color = layer.get_pixel(lx, ly).unwrap_or(Color::transparent());
            if old_color != new_color {
                patch.add_pixel_diff(lx, ly, old_color, new_color);
            }
        }
        
        self.original_pixels.clear();
        if patch.is_empty() { return Ok(None); }
        Ok(Some(patch))
    }

    fn on_commit(&mut self, store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError> {
        self.on_pointer_up(store)
    }

    fn on_cancel(&mut self, _store: &mut PixelStore) {
        self.active_layer_id = None;
        self.original_pixels.clear();
        self.last_pos = None;
    }
    
    fn take_dirty_rect(&mut self) -> Option<(u32, u32, u32, u32)> {
        self.dirty_rect.take().map(|(x1, y1, x2, y2)| {
            (x1, y1, x2 - x1, y2 - y1)
        })
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}