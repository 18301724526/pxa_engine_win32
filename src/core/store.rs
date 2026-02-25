use super::layer::Layer;
use super::color::Color;
use super::palette::Palette;
use super::selection::SelectionData;
use super::path::BezierPath;
use crate::core::error::{CoreError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushShape {
    Square,
    Circle,
}

pub struct PixelStore {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub layers: Vec<Layer>,
    pub active_layer_id: Option<String>,
    pub primary_color: Color,
    pub brush_size: u32,
    pub brush_shape: BrushShape,
    pub brush_jitter: u32,
    pub palette: Palette,
    pub selection: SelectionData,
    pub composite_cache: Vec<u8>,
    pub active_path: BezierPath,
}

impl PixelStore {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            canvas_width: width,
            canvas_height: height,
            layers: Vec::new(),
            active_layer_id: None,
            primary_color: Color::new(0, 0, 0, 255),
            brush_size: 1,
            brush_shape: BrushShape::Square,
            brush_jitter: 0,
            palette: Palette::default_pico8(),
            selection: SelectionData::new(width, height),
            composite_cache: vec![0u8; (width * height * 4) as usize],
            active_path: BezierPath::new(),
        }
    }

    pub fn add_layer(&mut self, layer: Layer) {
        if self.layers.is_empty() {
            self.active_layer_id = Some(layer.id.clone());
        }
        self.layers.push(layer);
    }

    pub fn add_layer_at(&mut self, layer: Layer, index: usize) {
        let idx = index.min(self.layers.len());
        self.layers.insert(idx, layer);
    }

    pub fn remove_layer_by_id(&mut self, id: &str) -> Option<(Layer, usize)> {
        if self.layers.len() <= 1 { return None; }
        
        if let Some(index) = self.layers.iter().position(|l| l.id == id) {
            let removed = self.layers.remove(index);
            if self.active_layer_id.as_deref() == Some(id) {
                let new_idx = index.saturating_sub(1);
                self.active_layer_id = Some(self.layers[new_idx].id.clone());
            }
            Some((removed, index))
        } else {
            None
        }
    }

    pub fn set_layer_visibility(&mut self, id: &str, visible: bool) {
        if let Some(layer) = self.get_layer_mut(id) {
            layer.visible = visible;
        }
    }

    pub fn get_layer(&self, id: &str) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }

    pub fn get_layer_mut(&mut self, id: &str) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    pub fn get_pixel(&self, layer_id: &str, canvas_x: u32, canvas_y: u32) -> Option<Color> {
        let layer = self.get_layer(layer_id)?;
        let local_x = canvas_x as i32 - layer.offset_x;
        let local_y = canvas_y as i32 - layer.offset_y;
        if local_x < 0 || local_y < 0 { return None; }
        layer.get_pixel(local_x as u32, local_y as u32)
    }

    pub fn mut_set_pixel(&mut self, layer_id: &str, canvas_x: u32, canvas_y: u32, color: Color) -> Result<()> {
        if !self.selection.contains(canvas_x, canvas_y) {
            return Ok(());
        }
        self.force_set_pixel(layer_id, canvas_x, canvas_y, color)
    }

    pub fn force_set_pixel(&mut self, layer_id: &str, canvas_x: u32, canvas_y: u32, color: Color) -> Result<()> {
        let layer = self.get_layer_mut(layer_id).ok_or_else(|| CoreError::LayerNotFound(layer_id.to_string()))?;
        let local_x = canvas_x as i32 - layer.offset_x;
        let local_y = canvas_y as i32 - layer.offset_y;
        if local_x < 0 || local_y < 0 || local_x >= layer.width as i32 || local_y >= layer.height as i32 {
            return Err(CoreError::OutOfBounds { x: canvas_x, y: canvas_y });
        }
        layer.set_pixel(local_x as u32, local_y as u32, color)
    }

    pub fn get_composite_pixel(&self, x: u32, y: u32) -> Color {
        if x >= self.canvas_width || y >= self.canvas_height {
            return Color::transparent();
        }
        let idx = ((y * self.canvas_width + x) * 4) as usize;
        if idx + 3 < self.composite_cache.len() {
            Color::new(
                self.composite_cache[idx],
                self.composite_cache[idx + 1],
                self.composite_cache[idx + 2],
                self.composite_cache[idx + 3],
            )
        } else {
            Color::transparent()
        }
    }
}

#[cfg(test)]
mod tests;