use crate::core::store::PixelStore;
use crate::history::patch::ActionPatch;
use super::tool_trait::Tool;
use crate::core::id_gen;
use crate::core::symmetry::SymmetryConfig;
use crate::core::selection::SelectionData;
use crate::core::error::CoreError;

pub struct EllipseSelectTool {
    start_pos: Option<(u32, u32)>,
    old_selection: Option<SelectionData>,
    needs_redraw: bool,
}

impl EllipseSelectTool {
    pub fn new() -> Self {
        Self { start_pos: None, old_selection: None, needs_redraw: false }
    }
}

impl Tool for EllipseSelectTool {
    fn on_pointer_down(&mut self, x: u32, y: u32, store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        self.old_selection = Some(store.selection.clone());
        self.start_pos = Some((x, y));
        store.selection.set_ellipse(x, y, 1, 1);
        self.needs_redraw = true;
        Ok(())
    }

    fn on_pointer_move(&mut self, x: u32, y: u32, store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        if let Some((sx, sy)) = self.start_pos {
            let min_x = sx.min(x);
            let min_y = sy.min(y);
            let w = sx.max(x) - min_x + 1;
            let h = sy.max(y) - min_y + 1;
            store.selection.set_ellipse(min_x, min_y, w, h);
            self.needs_redraw = true;
        }
        Ok(())
    }

    fn on_pointer_up(&mut self, store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError> {
        self.start_pos = None;
        let old = match self.old_selection.take() { 
            Some(s) => s, 
            None => return Ok(None)
        };

        let mask_count = store.selection.mask.iter().filter(|&&m| m).count();
        if mask_count == 0 {
            store.selection = old.clone();
            return Ok(None);
        }
        let new = store.selection.clone();
        
        if old == new { return Ok(None); }
        
        Ok(Some(ActionPatch::new_selection_change(id_gen::gen_id(), old, new)))
    }

    fn take_dirty_rect(&mut self) -> Option<(u32, u32, u32, u32)> {
        if self.needs_redraw {
            self.needs_redraw = false;
            Some((0, 0, u32::MAX, u32::MAX))
        } else {
            None
        }
    }
    
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}