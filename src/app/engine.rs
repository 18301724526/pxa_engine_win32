use crate::core::store::PixelStore;
use crate::history::manager::HistoryManager;
use crate::app::tool_manager::ToolManager;
use crate::core::symmetry::SymmetryConfig;
use crate::core::layer::Layer;
use crate::core::color::Color;
use crate::app::context::CanvasContext;
use crate::app::events::{InputEvent, EngineEffect};
use crate::render::compositor::Compositor;
use crate::history::patch::ActionPatch;
use crate::app::layer_service::LayerService;
use rust_i18n::t;

pub struct PxaEngine {
    store: PixelStore,
    history: HistoryManager,
    tool_manager: ToolManager,
    symmetry: SymmetryConfig,
}

impl PxaEngine {
    pub fn new() -> Self {
        let mut store = PixelStore::new(128, 128);
        store.add_layer(Layer::new("L1".to_string(), t!("layer.default_name", num = 1).to_string(), 128, 128));
        store.primary_color = Color::new(255, 80, 80, 255);
        
        let symmetry = SymmetryConfig::new(128, 128);
        Compositor::update_composite_cache(&mut store, None);

        Self {
            store,
            history: HistoryManager::new(50),
            tool_manager: ToolManager::new(),
            symmetry,
        }
    }

    pub fn store(&self) -> &PixelStore { &self.store }
    pub fn symmetry(&self) -> &SymmetryConfig { &self.symmetry }
    pub fn tool_manager(&self) -> &ToolManager { &self.tool_manager }
    pub fn history(&self) -> &HistoryManager { &self.history }

    pub fn tool_manager_mut(&mut self) -> &mut ToolManager { &mut self.tool_manager }
    pub fn symmetry_mut(&mut self) -> &mut SymmetryConfig { &mut self.symmetry }
    pub fn parts_mut(&mut self) -> (&mut PixelStore, &SymmetryConfig, &mut ToolManager) {
        (&mut self.store, &self.symmetry, &mut self.tool_manager)
    }

    pub fn brush_settings_mut(&mut self) -> (&mut u32, &mut crate::core::store::BrushShape, &mut u32) {
        (&mut self.store.brush_size, &mut self.store.brush_shape, &mut self.store.brush_jitter)
    }

    fn context(&mut self) -> CanvasContext<'_> {
        CanvasContext {
            store: &mut self.store,
            history: &mut self.history,
        }
    }

    fn refresh_cache(&mut self) {
        Compositor::update_composite_cache(&mut self.store, None);
    }

    pub fn update_render_cache(&mut self, rect: Option<(u32, u32, u32, u32)>) {
        Compositor::update_composite_cache(&mut self.store, rect);
    }

    pub fn set_primary_color(&mut self, color: Color) {
        self.store.primary_color = color;
    }

    pub fn set_palette(&mut self, palette: crate::core::palette::Palette) {
        self.store.palette = palette;
    }

    pub fn add_color_to_palette(&mut self, color: Color) {
        self.store.palette.add_color(color);
    }

    pub fn remove_palette_color(&mut self, index: usize) {
        self.store.palette.remove_color(index);
    }

    pub fn set_active_layer(&mut self, id: String) {
        if self.store.get_layer(&id).is_some() {
            self.store.active_layer_id = Some(id);
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
        self.history.commit(patch, &mut self.store)?;
        for layer in &mut self.store.layers {
            layer.prune_empty_chunks();
        }
        self.refresh_cache(); 
    Ok(())
    }

    pub fn undo(&mut self) -> crate::core::error::Result<()> {
        self.history.undo(&mut self.store)?;
        for layer in &mut self.store.layers {
            layer.prune_empty_chunks();
        }
        self.refresh_cache();
    Ok(())
    }

    pub fn redo(&mut self) -> crate::core::error::Result<()> {
        self.history.redo(&mut self.store)?;
        for layer in &mut self.store.layers {
            layer.prune_empty_chunks();
        }
        self.refresh_cache();
        Ok(())
    }

    pub fn replace_store_and_symmetry(&mut self, store: PixelStore, symmetry: SymmetryConfig) {
        self.store = store;
        self.symmetry = symmetry;
        self.history.undo_stack.clear();
        self.history.redo_stack.clear();
        self.refresh_cache();
    }

    pub fn handle_input(&mut self, event: InputEvent) -> EngineEffect {
        let result = match event {
            InputEvent::PointerDown { x, y } => {
                self.tool_manager.handle_pointer_down(x, y, &mut self.store, &self.symmetry)
                    .map(|_| self.process_dirty_rect())
            }
            InputEvent::PointerMove { x, y } => {
                if self.tool_manager.is_drawing {
                    self.tool_manager.handle_pointer_move(x, y, &mut self.store, &self.symmetry)
                        .map(|_| self.process_dirty_rect())
                } else {
                    Ok(EngineEffect::None)
                }
            }
            InputEvent::PointerUp => {
                match self.tool_manager.handle_pointer_up(&mut self.store) {
                    Ok(Some(patch)) => {
                        match self.history.commit(patch, &mut self.store) {
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
                    tool.on_cancel(&mut self.store);
                }
                self.refresh_cache();
                Ok(EngineEffect::RedrawCanvas)
            }
            InputEvent::CommitTool => {
                if let Some(tool) = self.tool_manager.tools.get_mut(&self.tool_manager.active_type) {
                    match tool.on_commit(&mut self.store) {
                        Ok(Some(patch)) => match self.history.commit(patch, &mut self.store) {
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
                Compositor::update_composite_cache(&mut self.store, Some(rect));
                EngineEffect::RedrawRect(rect.0, rect.1, rect.2, rect.3)
            }
        } else {
            EngineEffect::None
        }
    }
}