use crate::core::store::PixelStore;
use crate::history::manager::HistoryManager;
use crate::app::tool_manager::ToolManager;
use crate::core::symmetry::SymmetryConfig;
use crate::core::storage::PixelStorage;
use crate::core::symmetry::SymmetryProvider;
use crate::core::id::IdGenerator;
use crate::core::color::Color;
use crate::app::context::CanvasContext;
use crate::app::events::{InputEvent, EngineEffect};
use crate::render::compositor::Compositor;
use crate::history::patch::ActionPatch;
use crate::app::layer_service::LayerService;

pub struct PxaEngine {
    store: Box<dyn PixelStorage>,
    history: HistoryManager,
    tool_manager: ToolManager,
    symmetry: Box<dyn SymmetryProvider>,
    pub id_gen: Box<dyn IdGenerator>,
}

impl PxaEngine {
    pub fn new(
        mut store: Box<dyn PixelStorage>,
        symmetry: Box<dyn SymmetryProvider>,
        id_gen: Box<dyn IdGenerator>,
    ) -> Self {
        if let Some(s) = store.as_any_mut().downcast_mut::<PixelStore>() {
            Compositor::update_composite_cache(s, None);
        }

        Self {
            store,
            history: HistoryManager::new(50),
            tool_manager: ToolManager::new(),
            symmetry,
            id_gen,
        }
    }

    pub fn storage(&self) -> &dyn PixelStorage { self.store.as_ref() }
    pub fn id_gen(&self) -> &dyn IdGenerator { self.id_gen.as_ref() }

    pub fn store(&self) -> &PixelStore { self.store.as_any().downcast_ref::<PixelStore>().expect("Expected PixelStore implementation") }
    pub fn symmetry(&self) -> &SymmetryConfig { self.symmetry.as_any().downcast_ref::<SymmetryConfig>().expect("Expected SymmetryConfig") }
    pub fn tool_manager(&self) -> &ToolManager { &self.tool_manager }
    pub fn history(&self) -> &HistoryManager { &self.history }

    pub fn tool_manager_mut(&mut self) -> &mut ToolManager { &mut self.tool_manager }
    pub fn symmetry_mut(&mut self) -> &mut SymmetryConfig { self.symmetry.as_any_mut().downcast_mut::<SymmetryConfig>().unwrap() }
    pub fn parts_mut(&mut self) -> (&mut PixelStore, &SymmetryConfig, &mut ToolManager, &dyn IdGenerator) {
        (
            self.store.as_any_mut().downcast_mut::<PixelStore>().unwrap(),
            self.symmetry.as_any().downcast_ref::<SymmetryConfig>().unwrap(),
            &mut self.tool_manager,
            self.id_gen.as_ref()
        )
    }

    pub fn brush_settings_mut(&mut self) -> (&mut u32, &mut crate::core::store::BrushShape, &mut u32) {
        let s = self.store.as_any_mut().downcast_mut::<PixelStore>().unwrap();
        (&mut s.brush_size, &mut s.brush_shape, &mut s.brush_jitter)
    }

    fn context(&mut self) -> CanvasContext<'_> {
        CanvasContext {
            store: self.store.as_any_mut().downcast_mut::<PixelStore>().unwrap(),
            history: &mut self.history,
            id_gen: self.id_gen.as_ref(),
        }
    }

    fn refresh_cache(&mut self) {
        let s = self.store.as_any_mut().downcast_mut::<PixelStore>().unwrap();
        Compositor::update_composite_cache(s, None);
    }

    pub fn update_render_cache(&mut self, rect: Option<(u32, u32, u32, u32)>) {
        let s = self.store.as_any_mut().downcast_mut::<PixelStore>().unwrap();
        Compositor::update_composite_cache(s, rect);
    }

    pub fn set_primary_color(&mut self, color: Color) {
        let s = self.store.as_any_mut().downcast_mut::<PixelStore>().unwrap();
        s.primary_color = color;
    }

    pub fn set_palette(&mut self, palette: crate::core::palette::Palette) {
        let s = self.store.as_any_mut().downcast_mut::<PixelStore>().unwrap();
        s.palette = palette;
    }

    pub fn add_color_to_palette(&mut self, color: Color) {
        let s = self.store.as_any_mut().downcast_mut::<PixelStore>().unwrap();
        s.palette.add_color(color);
    }

    pub fn remove_palette_color(&mut self, index: usize) {
        let s = self.store.as_any_mut().downcast_mut::<PixelStore>().unwrap();
        s.palette.remove_color(index);
    }

    pub fn set_active_layer(&mut self, id: String) {
        let s = self.store.as_any_mut().downcast_mut::<PixelStore>().unwrap();
        if s.get_layer(&id).is_some() {
            s.active_layer_id = Some(id);
        }
    }

    pub fn add_new_layer(&mut self) -> crate::core::error::Result<()> {
        LayerService::add_new_layer(self.context())?;
        self.refresh_cache();
    Ok(())
    }

    pub fn delete_active_layer(&mut self) -> crate::core::error::Result<()> {
        LayerService::delete_active_layer(self.context())?;
        self.refresh_cache();
    Ok(())
    }

    pub fn toggle_layer_visibility(&mut self, layer_id: &str) -> crate::core::error::Result<()> {
        LayerService::toggle_visibility(self.context(), layer_id)?;
        self.refresh_cache();
    Ok(())
    }

    pub fn duplicate_layer(&mut self, layer_id: &str) -> crate::core::error::Result<()> {
        LayerService::duplicate_layer(self.context(), layer_id)?;
        self.refresh_cache();
    Ok(())
    }

    pub fn merge_selected_layers(&mut self, ids: Vec<String>) -> crate::core::error::Result<()> {
        LayerService::merge_selected_layers(self.context(), ids)?;
        self.refresh_cache();
        Ok(())
    }

    pub fn commit_patch(&mut self, patch: ActionPatch) -> crate::core::error::Result<()> {
        let s = self.store.as_any_mut().downcast_mut::<PixelStore>().unwrap();
        self.history.commit(patch, s)?;
        for layer in &mut s.layers {
            layer.prune_empty_chunks();
        }
        self.refresh_cache(); 
    Ok(())
    }

    pub fn undo(&mut self) -> crate::core::error::Result<()> {
        let s = self.store.as_any_mut().downcast_mut::<PixelStore>().unwrap();
        self.history.undo(s)?;
        for layer in &mut s.layers {
            layer.prune_empty_chunks();
        }
        self.refresh_cache();
    Ok(())
    }

    pub fn redo(&mut self) -> crate::core::error::Result<()> {
        let s = self.store.as_any_mut().downcast_mut::<PixelStore>().unwrap();
        self.history.redo(s)?;
        for layer in &mut s.layers {
            layer.prune_empty_chunks();
        }
        self.refresh_cache();
        Ok(())
    }

    pub fn replace_store_and_symmetry(&mut self, store: PixelStore, symmetry: SymmetryConfig) {
        self.store = Box::new(store);
        self.symmetry = Box::new(symmetry);
        self.history.undo_stack.clear();
        self.history.redo_stack.clear();
        self.refresh_cache();
    }

    pub fn handle_input(&mut self, event: InputEvent) -> EngineEffect {
        let s = self.store.as_any_mut().downcast_mut::<PixelStore>().unwrap();
        let sym = self.symmetry.as_any().downcast_ref::<SymmetryConfig>().unwrap();
        let result = match event {
            InputEvent::PointerDown { x, y } => {
                self.tool_manager.handle_pointer_down(x, y, s, sym)
                    .map(|_| self.process_dirty_rect())
            }
            InputEvent::PointerMove { x, y } => {
                if self.tool_manager.is_drawing {
                    self.tool_manager.handle_pointer_move(x, y, s, sym)
                        .map(|_| self.process_dirty_rect())
                } else {
                    Ok(EngineEffect::None)
                }
            }
            InputEvent::PointerUp => {
                match self.tool_manager.handle_pointer_up(s, self.id_gen.as_ref()) {
                    Ok(Some(patch)) => {
                        match self.history.commit(patch, s) {
                            Ok(_) => Ok(EngineEffect::merge(
                                self.process_dirty_rect(),
                                EngineEffect::RedrawCanvas
                            )),
                            Err(e) => Ok(EngineEffect::Error(e)),
                        }
                    }
                    Ok(None) => Ok(self.process_dirty_rect()),
                    Err(e) => Err(e),
                }
            }
            InputEvent::CancelTool => {
                if let Some(tool) = self.tool_manager.tools.get_mut(&self.tool_manager.active_type) {
                    tool.on_cancel(s);
                }
                self.refresh_cache();
                Ok(EngineEffect::RedrawCanvas)
            }
            InputEvent::CommitTool => {
                if let Some(tool) = self.tool_manager.tools.get_mut(&self.tool_manager.active_type) {
                    match tool.on_commit(s, self.id_gen.as_ref()) {
                        Ok(Some(patch)) => match self.history.commit(patch, s) {
                            Ok(_) => {
                                self.refresh_cache();
                                Ok(EngineEffect::merge(EngineEffect::ToolCommitted, EngineEffect::RedrawCanvas))
                            }
                            Err(e) => Ok(EngineEffect::Error(e)),
                        }
                        Ok(None) => Ok(EngineEffect::None),
                        Err(e) => Err(e),
                    }
                } else {
                    Ok(EngineEffect::None)
                }
            }
        };

        match result {
            Ok(effect) => effect,
            Err(e) => EngineEffect::Error(e),
        }
    }

    fn process_dirty_rect(&mut self) -> EngineEffect {
        let active_type = self.tool_manager.active_type;
        if let Some(rect) = self.tool_manager.tools.get_mut(&active_type).and_then(|t| t.take_dirty_rect()) {
            if rect.2 == u32::MAX && rect.3 == u32::MAX {
                self.refresh_cache();
                EngineEffect::RedrawCanvas
            } else {
                let s = self.store.as_any_mut().downcast_mut::<PixelStore>().unwrap();
                Compositor::update_composite_cache(s, Some(rect));
                EngineEffect::RedrawRect(rect.0, rect.1, rect.2, rect.3)
            }
        } else {
            EngineEffect::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage::mocks::MockPixelStorage;
    use crate::core::id::mocks::MockIdGenerator;
    use crate::core::symmetry::SymmetryConfig;
    
    #[test]
    fn test_engine_di() {
        let store = Box::new(MockPixelStorage::new(800, 600));
        let symmetry = Box::new(SymmetryConfig::new(800, 600));
        let id_gen = Box::new(MockIdGenerator::new("di_test"));
        
        let engine = PxaEngine::new(store, symmetry, id_gen);
        
        assert_eq!(engine.id_gen().generate(), "di_test_1");
        assert_eq!(engine.storage().canvas_width(), 800);
    }
}