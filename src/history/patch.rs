use crate::core::color::Color;
use crate::core::layer::Layer; 
use crate::core::blend_mode::BlendMode;
use crate::core::selection::SelectionData;
use crate::core::store::PixelStore;
use crate::core::path::BezierPath;
use crate::core::error::{CoreError, Result};
use std::fmt::Debug;
use std::any::Any;
use std::sync::Arc;

pub trait Patch: Debug + Send + Sync + Any {
    fn apply(&self, layer_id: &str, store: &mut PixelStore, forward: bool) -> Result<()>;
    fn is_empty(&self) -> bool { false }
    fn clone_box(&self) -> Box<dyn Patch>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl Clone for Box<dyn Patch> {
    fn clone(&self) -> Self { self.clone_box() }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PixelDiff {
    pub x: u32, pub y: u32, pub old_color: Color, pub new_color: Color,
}

#[derive(Debug, Clone)]
pub struct PixelDiffPatch { pub diffs: Vec<PixelDiff> }
impl Patch for PixelDiffPatch {
    fn apply(&self, layer_id: &str, store: &mut PixelStore, forward: bool) -> Result<()> {
        let layer = store.get_layer_mut(layer_id)
            .ok_or_else(|| CoreError::LayerNotFound(layer_id.to_string()))?;

            let iter: Box<dyn Iterator<Item = &PixelDiff>> = if forward {
            Box::new(self.diffs.iter())
        } else {
            Box::new(self.diffs.iter().rev())
        };

        for diff in iter {
            let target_color = if forward { diff.new_color } else { diff.old_color };
            let _ = layer.set_pixel_raw(diff.x, diff.y, target_color);
        }
        Ok(())
    }
    fn is_empty(&self) -> bool { self.diffs.is_empty() }
    fn clone_box(&self) -> Box<dyn Patch> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct RegionDiffPatch {
    pub x: u32, pub y: u32, pub width: u32, pub height: u32,
    pub old_data: Vec<u8>, pub new_data: Vec<u8>,
}
impl Patch for RegionDiffPatch {
    fn apply(&self, layer_id: &str, store: &mut PixelStore, forward: bool) -> Result<()> {
        let layer = store.get_layer_mut(layer_id)
            .ok_or_else(|| CoreError::LayerNotFound(layer_id.to_string()))?;
        let data = if forward { &self.new_data } else { &self.old_data };
        self.apply_raw(layer, data);
        Ok(())
    }
    fn is_empty(&self) -> bool { self.width == 0 || self.height == 0 }
    fn clone_box(&self) -> Box<dyn Patch> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
impl RegionDiffPatch {
    fn apply_raw(&self, layer: &mut Layer, data: &[u8]) {
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = ((row * self.width + col) * 4) as usize;
                let color = Color::new(data[idx], data[idx+1], data[idx+2], data[idx+3]);
                let _ = layer.set_pixel_raw(self.x + col, self.y + row, color);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayerAddPatch { 
    pub layer: Layer, 
    pub index: usize,
    pub old_active_id: Option<String>,
}
impl Patch for LayerAddPatch {
    fn apply(&self, _layer_id: &str, store: &mut PixelStore, forward: bool) -> Result<()> {
        if forward { 
            store.add_layer_at(self.layer.clone(), self.index);
            store.active_layer_id = Some(self.layer.id.clone());
        }
        else { 
            store.remove_layer_by_id(&self.layer.id);
            if let Some(ref old_id) = self.old_active_id {
                store.active_layer_id = Some(old_id.clone());
            }
        }
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn Patch> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct LayerRemovePatch { 
    pub layer: Layer, 
    pub index: usize,
    pub old_active_id: Option<String>,
}
impl Patch for LayerRemovePatch {
    fn apply(&self, _layer_id: &str, store: &mut PixelStore, forward: bool) -> Result<()> {
        if forward { store.remove_layer_by_id(&self.layer.id); } 
        else { 
            store.add_layer_at(self.layer.clone(), self.index);
            if let Some(ref old_id) = self.old_active_id {
                store.active_layer_id = Some(old_id.clone());
            }
        }
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn Patch> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct LayerVisibilityPatch { pub visible: bool }
impl Patch for LayerVisibilityPatch {
    fn apply(&self, layer_id: &str, store: &mut PixelStore, forward: bool) -> Result<()> {
        let target_visible = if forward { self.visible } else { !self.visible };
        store.set_layer_visibility(layer_id, target_visible);
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn Patch> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct LayerOpacityPatch { pub old_opacity: u8, pub new_opacity: u8 }
impl Patch for LayerOpacityPatch {
    fn apply(&self, layer_id: &str, store: &mut PixelStore, forward: bool) -> Result<()> {
        if let Some(layer) = store.get_layer_mut(layer_id) {
            layer.opacity = if forward { self.new_opacity } else { self.old_opacity };
        }
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn Patch> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct LayerLockPatch { pub old_locked: bool, pub new_locked: bool }
impl Patch for LayerLockPatch {
    fn apply(&self, layer_id: &str, store: &mut PixelStore, forward: bool) -> Result<()> {
        if let Some(layer) = store.get_layer_mut(layer_id) {
            layer.locked = if forward { self.new_locked } else { self.old_locked };
        }
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn Patch> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct LayerMovePatch { pub old_index: usize, pub new_index: usize }
impl Patch for LayerMovePatch {
    fn apply(&self, _layer_id: &str, store: &mut PixelStore, forward: bool) -> Result<()> {
        let from = if forward { self.old_index } else { self.new_index };
        let to = if forward { self.new_index } else { self.old_index };
        if from < store.layers.len() && to < store.layers.len() {
            let layer = store.layers.remove(from);
            store.layers.insert(to, layer);
        }
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn Patch> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct SelectionChangePatch { pub old_sel: SelectionData, pub new_sel: SelectionData }
impl Patch for SelectionChangePatch {
    fn apply(&self, _layer_id: &str, store: &mut PixelStore, forward: bool) -> Result<()> {
        store.selection = if forward { self.new_sel.clone() } else { self.old_sel.clone() };
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn Patch> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct LayerOffsetPatch { pub old_offset: (i32, i32), pub new_offset: (i32, i32) }
impl Patch for LayerOffsetPatch {
    fn apply(&self, layer_id: &str, store: &mut PixelStore, forward: bool) -> Result<()> {
        if let Some(layer) = store.get_layer_mut(layer_id) {
            let target = if forward { self.new_offset } else { self.old_offset };
            layer.offset_x = target.0;
            layer.offset_y = target.1;
        }
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn Patch> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct CompositePatch { pub patches: Vec<ActionPatch> }
impl Patch for CompositePatch {
    fn apply(&self, _layer_id: &str, store: &mut PixelStore, forward: bool) -> Result<()> {
        let iter: Box<dyn Iterator<Item = &ActionPatch>> = if forward {
            Box::new(self.patches.iter())
        } else {
            Box::new(self.patches.iter().rev())
        };
        for p in iter {
            p.action.apply(&p.layer_id, store, forward)?;
        }
        Ok(())
    }
    fn is_empty(&self) -> bool { self.patches.is_empty() }
    fn clone_box(&self) -> Box<dyn Patch> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct LayerRenamePatch { pub old_name: String, pub new_name: String }
impl Patch for LayerRenamePatch {
    fn apply(&self, layer_id: &str, store: &mut PixelStore, forward: bool) -> Result<()> {
        if let Some(layer) = store.get_layer_mut(layer_id) {
            layer.name = if forward { self.new_name.clone() } else { self.old_name.clone() };
        }
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn Patch> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct LayerBlendModePatch { pub old_mode: BlendMode, pub new_mode: BlendMode }
impl Patch for LayerBlendModePatch {
    fn apply(&self, layer_id: &str, store: &mut PixelStore, forward: bool) -> Result<()> {
        if let Some(layer) = store.get_layer_mut(layer_id) {
            layer.blend_mode = if forward { self.new_mode } else { self.old_mode };
        }
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn Patch> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct CanvasResizePatch {
    pub old_width: u32, pub old_height: u32,
    pub new_width: u32, pub new_height: u32,
    pub old_layers: Arc<Vec<Layer>>, pub new_layers: Arc<Vec<Layer>>,
    pub old_selection: Arc<SelectionData>, pub new_selection: Arc<SelectionData>,
}
impl Patch for CanvasResizePatch {
    fn apply(&self, _layer_id: &str, store: &mut PixelStore, forward: bool) -> Result<()> {
        let (w, h, layers, sel) = if forward {
            (self.new_width, self.new_height, &self.new_layers, &self.new_selection)
        } else {
            (self.old_width, self.old_height, &self.old_layers, &self.old_selection)
        };
        store.canvas_width = w; store.canvas_height = h;
        store.layers = layers.as_ref().clone(); store.selection = sel.as_ref().clone();
        store.composite_cache = vec![0u8; (w * h * 4) as usize];
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn Patch> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct PathChangePatch { pub old_path: BezierPath, pub new_path: BezierPath }
impl Patch for PathChangePatch {
    fn apply(&self, _layer_id: &str, store: &mut PixelStore, forward: bool) -> Result<()> {
        store.active_path = if forward { self.new_path.clone() } else { self.old_path.clone() };
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn Patch> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct ActionPatch {
    pub id: String,
    pub layer_id: String,
    pub action: Box<dyn Patch>,
}

impl ActionPatch {
    pub fn is_empty(&self) -> bool {
        self.action.is_empty()
    }

    pub fn pixel_diffs(&self) -> Option<&Vec<PixelDiff>> {
        self.action.as_any().downcast_ref::<PixelDiffPatch>().map(|p| &p.diffs)
    }

    pub fn new_composite(id: String, patches: Vec<ActionPatch>) -> Self {
        Self { id, layer_id: "global".into(), action: Box::new(CompositePatch { patches }) }
    }

    pub fn new_pixel_diff(id: String, layer_id: String) -> Self {
        Self { id, layer_id, action: Box::new(PixelDiffPatch { diffs: Vec::new() }) }
    }
    
    pub fn new_region_diff(
        id: String, layer_id: String, x: u32, y: u32, width: u32, height: u32,
        old_data: Vec<u8>, new_data: Vec<u8>,
    ) -> Self {
        Self { id, layer_id, action: Box::new(RegionDiffPatch { x, y, width, height, old_data, new_data }) }
    }

    pub fn new_layer_add(id: String, layer_id: String, layer: Layer, index: usize, old_active_id: Option<String>) -> Self {
        Self { 
            id, 
            layer_id, 
            action: Box::new(LayerAddPatch { layer, index, old_active_id }) 
        }
    }

    pub fn new_layer_remove(id: String, layer_id: String, layer: Layer, index: usize, old_active_id: Option<String>) -> Self {
        Self { id, layer_id, action: Box::new(LayerRemovePatch { layer, index, old_active_id }) }
    }

    pub fn new_layer_visibility(id: String, layer_id: String, visible: bool) -> Self {
        Self { id, layer_id, action: Box::new(LayerVisibilityPatch { visible }) }
    }

    pub fn new_layer_opacity(id: String, layer_id: String, old_opacity: u8, new_opacity: u8) -> Self {
        Self { id, layer_id, action: Box::new(LayerOpacityPatch { old_opacity, new_opacity }) }
    }

    pub fn new_layer_lock(id: String, layer_id: String, old_locked: bool, new_locked: bool) -> Self {
        Self { id, layer_id, action: Box::new(LayerLockPatch { old_locked, new_locked }) }
    }

    pub fn new_layer_move(id: String, layer_id: String, old_index: usize, new_index: usize) -> Self {
        Self { id, layer_id, action: Box::new(LayerMovePatch { old_index, new_index }) }
    }

    pub fn new_selection_change(id: String, old_sel: SelectionData, new_sel: SelectionData) -> Self {
        Self { id, layer_id: "global".into(), action: Box::new(SelectionChangePatch { old_sel, new_sel }) }
    }

    pub fn new_layer_offset(id: String, layer_id: String, old_offset: (i32, i32), new_offset: (i32, i32)) -> Self {
        Self { id, layer_id, action: Box::new(LayerOffsetPatch { old_offset, new_offset }) }
    }

    pub fn new_layer_rename(id: String, layer_id: String, old_name: String, new_name: String) -> Self {
        Self { id, layer_id, action: Box::new(LayerRenamePatch { old_name, new_name }) }
    }

    pub fn new_layer_blend_mode(id: String, layer_id: String, old_mode: BlendMode, new_mode: BlendMode) -> Self {
        Self { id, layer_id, action: Box::new(LayerBlendModePatch { old_mode, new_mode }) }
    }

    pub fn new_path_change(id: String, old_path: BezierPath, new_path: BezierPath) -> Self {
        Self { id, layer_id: "global".into(), action: Box::new(PathChangePatch { old_path, new_path }) }
    }

    pub fn new_canvas_resize(
        id: String, old_width: u32, old_height: u32, new_width: u32, new_height: u32,
        old_layers: Vec<Layer>, new_layers: Vec<Layer>, old_selection: SelectionData, new_selection: SelectionData
    ) -> Self {
        Self { 
            id, 
            layer_id: "global".into(), 
            action: Box::new(CanvasResizePatch { 
                old_width, old_height, new_width, new_height, 
                old_layers: Arc::new(old_layers), new_layers: Arc::new(new_layers), 
                old_selection: Arc::new(old_selection), new_selection: Arc::new(new_selection) 
            }) 
        }
    }

    pub fn add_pixel_diff(&mut self, x: u32, y: u32, old_color: Color, new_color: Color) {
        if let Some(patch) = self.action.as_any_mut().downcast_mut::<PixelDiffPatch>() {
            patch.diffs.push(PixelDiff { x, y, old_color, new_color });
        }
    }
}

#[cfg(test)] mod tests;