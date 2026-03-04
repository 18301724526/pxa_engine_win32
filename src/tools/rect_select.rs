use crate::core::store::PixelStore;
use crate::history::patch::ActionPatch;
use super::tool_trait::Tool;
use crate::core::symmetry::SymmetryConfig;
use crate::core::selection::SelectionData;
use crate::core::error::CoreError;

pub struct RectSelectTool {
    start_pos: Option<(i32, i32)>,
    old_selection: Option<SelectionData>,
    needs_redraw: bool,
}

impl RectSelectTool {
    pub fn new() -> Self {
        Self { start_pos: None, old_selection: None, needs_redraw: false }
    }
}

impl Tool for RectSelectTool {
    fn on_pointer_down(&mut self, x: i32, y: i32, store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        self.old_selection = Some(store.selection.clone());
        self.start_pos = Some((x, y));
        store.selection.set_rect(x.max(0) as u32, y.max(0) as u32, 1, 1);
        self.needs_redraw = true;
        Ok(())
    }

    fn on_pointer_move(&mut self, x: i32, y: i32, store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        if let Some((sx, sy)) = self.start_pos {
            let min_x = sx.min(x).max(0) as u32;
            let min_y = sy.min(y).max(0) as u32;
            let max_x = sx.max(x).max(0) as u32;
            let max_y = sy.max(y).max(0) as u32;
            let w = max_x - min_x + 1;
            let h = max_y - min_y + 1;
            store.selection.set_rect(min_x, min_y, w, h);
            self.needs_redraw = true;
        }
        Ok(())
    }

    fn on_pointer_up(&mut self, store: &mut PixelStore, id_gen: &dyn crate::core::id::IdGenerator) -> Result<Option<ActionPatch>, CoreError> {
        let (_sx, _sy) = match self.start_pos.take() {
            Some(pos) => pos,
            None => return Ok(None),
        };
        let old = match self.old_selection.take() {
            Some(s) => s,
            None => return Ok(None),
        };

        let mask_count = store.selection.mask.iter().filter(|&&m| m).count();
        if mask_count <= 1 {
            store.selection.clear();
        }
        
        let new = store.selection.clone();
        if old == new { return Ok(None); }
        Ok(Some(ActionPatch::new_selection_change(id_gen.generate(), old, new)))
    }

fn on_commit(&mut self, store: &mut PixelStore, id_gen: &dyn crate::core::id::IdGenerator) -> Result<Option<ActionPatch>, CoreError> {
    self.on_pointer_up(store, id_gen)
}

fn on_cancel(&mut self, store: &mut PixelStore) {
    if let Some(old) = self.old_selection.take() {
        store.selection = old;
        self.needs_redraw = true;
    }
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