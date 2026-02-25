use crate::core::store::PixelStore;
use super::patch::ActionPatch;
use crate::core::error::Result;

pub struct HistoryManager {
    pub undo_stack: Vec<ActionPatch>,
    pub redo_stack: Vec<ActionPatch>,
    pub max_steps: usize,
}

impl HistoryManager {
    pub fn new(max_steps: usize) -> Self {
        Self { undo_stack: Vec::new(), redo_stack: Vec::new(), max_steps }
    }

    pub fn commit(&mut self, patch: ActionPatch, store: &mut PixelStore) -> Result<()> {
        if patch.is_empty() { return Ok(()); }

        self.apply_patch(&patch, store, true)?;
        self.undo_stack.push(patch);
        if self.undo_stack.len() > self.max_steps { self.undo_stack.remove(0); }
        self.redo_stack.clear();
        Ok(())
    }

    pub fn undo(&mut self, store: &mut PixelStore) -> Result<()> {
        if let Some(patch) = self.undo_stack.pop() {
            self.apply_patch(&patch, store, false)?;
            self.redo_stack.push(patch);
        }
        Ok(())
    }

    pub fn redo(&mut self, store: &mut PixelStore) -> Result<()> {
        if let Some(patch) = self.redo_stack.pop() {
            self.apply_patch(&patch, store, true)?;
            self.undo_stack.push(patch);
        }
        Ok(())
    }

    fn apply_patch(&self, patch: &ActionPatch, store: &mut PixelStore, forward: bool) -> Result<()> {
        patch.action.apply(&patch.layer_id, store, forward)
    }
}

#[cfg(test)]
mod tests;