use crate::core::store::PixelStore;
use crate::history::patch::ActionPatch;
use crate::core::symmetry::SymmetryConfig;
use std::any::Any;
use crate::core::error::CoreError;

pub trait Tool: Any {
    fn on_pointer_down(&mut self, x: u32, y: u32, store: &mut PixelStore, symmetry: &SymmetryConfig) -> Result<(), CoreError>;
    fn on_pointer_move(&mut self, x: u32, y: u32, store: &mut PixelStore, symmetry: &SymmetryConfig) -> Result<(), CoreError>;
    fn on_pointer_up(&mut self, store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError>;
    fn take_dirty_rect(&mut self) -> Option<(u32, u32, u32, u32)> {
        None
    }
    fn on_commit(&mut self, _store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError> { Ok(None) }
    fn on_cancel(&mut self, _store: &mut PixelStore) {}
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}