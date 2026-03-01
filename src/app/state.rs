use crate::app::io_service::IoService;
use crate::app::engine::PxaEngine;
use crate::core::id_gen;
use crate::history::patch::ActionPatch;
use crate::app::events::InputEvent;
use crate::app::commands::AppCommand;
use crate::app::ui_state::UiState;
use crate::app::shortcut_manager::ShortcutManager;
use crate::app::view_state::ViewState;
use std::collections::VecDeque;
use crate::core::error::CoreError;
use rust_i18n::t;
use crate::animation::state::AnimationState;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ToolType { 
    Pencil, Eraser, Bucket, Eyedropper, RectSelect, EllipseSelect, 
    Move, Transform, Pen, CreateBone, BoneRotate, BoneTranslate 
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AppMode {
    PixelEdit,
    Animation,
}

pub struct AppState {
    pub engine: PxaEngine,
    pub command_queue: VecDeque<AppCommand>,
    pub is_space_pressed: bool,
    pub view: ViewState,
    pub ui: UiState,
    pub last_mouse_pos: Option<(u32, u32)>,
    pub is_dirty: bool,
    pub mode: AppMode,
    pub animation: AnimationState,
    pub shortcuts: ShortcutManager,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            engine: PxaEngine::new(),
            command_queue: VecDeque::new(),
            is_space_pressed: false,
            view: ViewState::new(),
            ui: UiState::new(),
            last_mouse_pos: None,
            is_dirty: false,
            mode: AppMode::PixelEdit,
            animation: AnimationState::new(),
            shortcuts: ShortcutManager::new(),
        }
    }
    pub fn enqueue_command(&mut self, cmd: AppCommand) {
        self.command_queue.push_back(cmd);
    }

    pub fn pop_command(&mut self) -> Option<AppCommand> {
        self.command_queue.pop_front()
    }

    pub fn set_tool(&mut self, tool_type: ToolType) {
        if self.engine.tool_manager().active_type == tool_type { return; }
        self.commit_current_tool();
        self.engine.tool_manager_mut().is_drawing = false;
        self.engine.tool_manager_mut().set_tool(tool_type);
    }

    pub fn commit_current_tool(&mut self) {
        let effect = self.engine.handle_input(InputEvent::CommitTool);
        crate::app::input_handler::InputHandler::handle_engine_effect(self, effect);
    }

    pub fn cancel_current_tool(&mut self) {
        let effect = self.engine.handle_input(InputEvent::CancelTool);
        crate::app::input_handler::InputHandler::handle_engine_effect(self, effect);
    }

    pub fn on_mouse_down(&mut self, x: u32, y: u32) -> Result<(), CoreError> {
        crate::app::input_handler::InputHandler::on_mouse_down(self, x, y)
    }

    pub fn on_mouse_move(&mut self, x: u32, y: u32) -> Result<(), CoreError> {
        crate::app::input_handler::InputHandler::on_mouse_move(self, x, y)
    }

    pub fn on_mouse_up(&mut self) -> Result<(), CoreError> {
        crate::app::input_handler::InputHandler::on_mouse_up(self)
    }

    pub fn undo(&mut self) { 
        if let Err(e) = self.engine.undo() {
            self.ui.error_message = Some(e.to_string());
        } else {
            self.is_dirty = true;
            self.view.needs_full_redraw = true;
        }
    }
    pub fn redo(&mut self) { 
        if let Err(e) = self.engine.redo() {
            self.ui.error_message = Some(e.to_string());
        } else {
            self.is_dirty = true;
            self.view.needs_full_redraw = true;
        }
    }
    
    pub fn add_new_layer(&mut self) { 
        if let Err(e) = self.engine.add_new_layer() {
            self.ui.error_message = Some(e.to_string());
        } else {
            self.is_dirty = true;
            self.view.needs_full_redraw = true;
        }
    }

    pub fn delete_active_layer(&mut self) { 
        if let Err(e) = self.engine.delete_active_layer() {
            self.ui.error_message = Some(e.to_string());
        } else {
            self.is_dirty = true;
            self.view.needs_full_redraw = true;
        }
    }
    pub fn toggle_layer_visibility(&mut self, layer_id: &str) { 
        if let Err(e) = self.engine.toggle_layer_visibility(layer_id) {
            self.ui.error_message = Some(e.to_string());
        } else {
            self.is_dirty = true;
            self.view.needs_full_redraw = true;
        }
    }

    pub fn import_image(&mut self) {
        if let Some(path) = IoService::pick_import_path() {
            let id = format!("layer_imp_{}", id_gen::gen_id());
            let name = t!("layer.import_name", num = self.engine.store().layers.len() + 1).to_string();
            let w = self.engine.store().canvas_width;
            let h = self.engine.store().canvas_height;
            let old_active_id = self.engine.store().active_layer_id.clone();
            
            match IoService::load_as_layer(path, w, h, id.clone(), name) {
                Ok(layer) => {
                    let index = self.engine.store().layers.len();
                    let patch = ActionPatch::new_layer_add(format!("patch_{}", id), id.clone(), layer, index, old_active_id);
                    if let Err(e) = self.engine.commit_patch(patch) {
                        self.ui.error_message = Some(e.to_string());
                    } else {
                        self.engine.set_active_layer(id);
                        self.is_dirty = true;
                        self.view.needs_full_redraw = true;
                    }
                }
                Err(e) => self.ui.error_message = Some(t!("error.import_image_failed", err = e.to_string()).to_string()),
            }
        }
    }

    pub fn export_to_png(&mut self) {
        if let Some(path) = IoService::pick_export_path() {
            if let Err(e) = IoService::save_png(path, self.engine.store()) {
                self.ui.error_message = Some(t!("error.export_failed", err = e.to_string()).to_string());
            }
        }
    }

    pub fn import_palette(&mut self) {
        if let Some(path) = IoService::pick_palette_import_path() {
            match crate::format::hex_palette::load_from_hex(&path) {
                Ok(palette) => self.enqueue_command(AppCommand::SetPalette(palette)),
                Err(e) => self.ui.error_message = Some(t!("error.load_palette_failed", err = e.to_string()).to_string()),
            }
        }
    }

    pub fn export_palette(&mut self) {
        if let Some(path) = IoService::pick_palette_export_path() {
            if let Err(e) = crate::format::hex_palette::save_to_hex(&path, &self.engine.store().palette) {
                self.ui.error_message = Some(t!("error.export_palette_failed", err = e.to_string()).to_string());
            }
        }
    }
    pub fn save_project_to_pxad(&mut self) {
        if let Some(path) = IoService::pick_project_save_path() {
            if let Err(e) = IoService::save_project(path, self.engine.store(), self.engine.symmetry(), &self.view) {
                self.ui.error_message = Some(t!("error.save_project_failed", err = e.to_string()).to_string());
            } else {
                self.is_dirty = false;
            }
        }
    }

    pub fn load_project_from_pxad(&mut self) {
        if let Some(path) = IoService::pick_project_load_path() {
            match IoService::load_project(path) {
                Ok((new_store, new_sym, px, py, zl)) => {
                    self.engine.replace_store_and_symmetry(new_store, new_sym);
                    self.view.pan_x = px;
                    self.view.pan_y = py;
                    self.view.zoom_level = zl;
                    self.is_dirty = false;
                    self.view.needs_full_redraw = true;
                }
                Err(e) => self.ui.error_message = Some(t!("error.load_project_failed", err = e.to_string()).to_string()),
            }
        }
    }
}