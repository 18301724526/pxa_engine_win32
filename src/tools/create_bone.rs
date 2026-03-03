use crate::core::store::PixelStore;
use crate::history::patch::ActionPatch;
use crate::tools::tool_trait::Tool;
use crate::core::symmetry::SymmetryConfig;
use crate::core::error::CoreError;
use std::any::Any;

pub struct CreateBoneTool {
    pub start_pos: Option<(f32, f32)>,
    pub preview_end: Option<(f32, f32)>,
    pub parent_bone_id: Option<String>,
}

impl CreateBoneTool {
    pub fn new() -> Self {
        Self {
            start_pos: None,
            preview_end: None,
            parent_bone_id: None,
        }
    }
}

impl Tool for CreateBoneTool {
    fn on_pointer_down(&mut self, x: u32, y: u32, _store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        self.start_pos = Some((x as f32, y as f32));
        self.preview_end = Some((x as f32, y as f32));
        Ok(())
    }

    fn on_pointer_move(&mut self, x: u32, y: u32, _store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        if self.start_pos.is_some() {
            self.preview_end = Some((x as f32, y as f32));
        }
        Ok(())
    }

    fn on_pointer_up(&mut self, _store: &mut PixelStore, _id_gen: &dyn crate::core::id::IdGenerator) -> Result<Option<ActionPatch>, CoreError> {
        Ok(None)
    }

    fn take_dirty_rect(&mut self) -> Option<(u32, u32, u32, u32)> {
        Some((0, 0, u32::MAX, u32::MAX))
    }

    fn on_cancel(&mut self, _store: &mut PixelStore) {
        self.start_pos = None;
        self.preview_end = None;
    }

    fn on_commit(&mut self, _store: &mut PixelStore, _id_gen: &dyn crate::core::id::IdGenerator) -> Result<Option<ActionPatch>, CoreError> {
        self.on_cancel(_store);
        Ok(None)
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[cfg(test)]
mod tests;