use crate::core::store::PixelStore;
use crate::history::patch::ActionPatch;
use super::tool_trait::Tool;
use crate::core::symmetry::SymmetryConfig;
use crate::core::error::CoreError;

pub struct EyedropperTool;

impl EyedropperTool {
    pub fn new() -> Self { Self }
}

impl Tool for EyedropperTool {
    fn on_pointer_down(&mut self, x: u32, y: u32, store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        let picked_color = store.get_composite_pixel(x, y);
        store.primary_color = picked_color;
        Ok(())
    }

    fn on_pointer_move(&mut self, x: u32, y: u32, store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        let picked_color = store.get_composite_pixel(x, y);
        store.primary_color = picked_color;
        Ok(())
    }

    fn on_pointer_up(&mut self, _store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError> { Ok(None) }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

#[cfg(test)]
mod tests;