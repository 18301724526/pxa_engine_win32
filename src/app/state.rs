use crate::app::io_service::IoService;
use crate::app::engine::PxaEngine;
use crate::core::id_gen;
use crate::history::patch::ActionPatch;
use crate::app::events::{InputEvent, EngineEffect};
use crate::app::commands::AppCommand;
use crate::app::ui_state::UiState;
use crate::app::view_state::ViewState;
use std::collections::VecDeque;
use crate::core::error::CoreError;
use rust_i18n::t;
use crate::animation::project::AnimProject;

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

pub struct AnimationState {
    pub project: AnimProject,
    pub current_time: f32,
    pub is_playing: bool,
    pub playback_speed: f32,
    pub create_bone_tool: crate::tools::create_bone::CreateBoneTool,
}

impl AnimationState {
    pub fn new() -> Self {
        Self {
            project: AnimProject::new(),
            current_time: 0.0,
            is_playing: false,
            playback_speed: 1.0,
            create_bone_tool: crate::tools::create_bone::CreateBoneTool::new(),
        }
    }
    
    pub fn init_demo_data(&mut self) {
    }
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
        self.handle_engine_effect(effect);
    }

    pub fn cancel_current_tool(&mut self) {
        let effect = self.engine.handle_input(InputEvent::CancelTool);
        self.handle_engine_effect(effect);
    }

    pub fn on_mouse_down(&mut self, x: u32, y: u32) -> Result<(), CoreError> {
        if self.mode == AppMode::Animation {
            self.engine.tool_manager_mut().is_drawing = true;
            self.last_mouse_pos = Some((x, y));
            if let Some(tool) = self.engine.tool_manager_mut().tools.get_mut(&ToolType::CreateBone) {
                if let Some(bone_tool) = tool.as_any_mut().downcast_mut::<crate::tools::create_bone::CreateBoneTool>() {
                    bone_tool.parent_bone_id = self.ui.selected_bone_id.clone();
                }
            }
            return self.handle_animation_click(x, y);
        }

        if self.engine.tool_manager().active_type == ToolType::CreateBone {
            if let Some(tool) = self.engine.tool_manager_mut().tools.get_mut(&ToolType::CreateBone) {
                if let Some(bone_tool) = tool.as_any_mut().downcast_mut::<crate::tools::create_bone::CreateBoneTool>() {
                    bone_tool.parent_bone_id = self.ui.selected_bone_id.clone();
                }
            }
            let (store, symmetry, tool_manager) = self.engine.parts_mut();
            return tool_manager.handle_pointer_down(x, y, store, symmetry);
        }
        let effect = self.engine.handle_input(InputEvent::PointerDown { x, y });
        self.last_mouse_pos = Some((x, y));
        let result = match &effect {
            EngineEffect::Error(e) => Err(e.clone()),
            _ => Ok(()),
        };

        self.handle_engine_effect(effect);
        result
    }

    pub fn on_mouse_move(&mut self, x: u32, y: u32) -> Result<(), CoreError> {
        let last_pos = self.last_mouse_pos.unwrap_or((x, y));
        let dx = x as f32 - last_pos.0 as f32;
        let dy = y as f32 - last_pos.1 as f32; 
        self.last_mouse_pos = Some((x, y));
        if self.mode == AppMode::Animation {
            if self.engine.tool_manager().is_drawing {
                let tool = self.engine.tool_manager().active_type;
                if let Some(bone_id) = &self.ui.selected_bone_id {
                    let skeleton = &mut self.animation.project.skeleton;
                    if let Some(bone_idx) = skeleton.bones.iter().position(|b| b.data.id == *bone_id) {
                        let mut changed = false;
                        match tool {
                            ToolType::BoneRotate => {
                                let bone = &mut skeleton.bones[bone_idx];
                                let base_sensitivity = 0.2;
                                let acceleration = 0.02;
                                let delta = dy * (base_sensitivity + dy.abs() * acceleration);
                                bone.local_transform.rotation -= delta;
                                changed = true;
                            }
                            ToolType::BoneTranslate => {
                                let current_world_x = skeleton.bones[bone_idx].world_matrix[4];
                                let current_world_y = skeleton.bones[bone_idx].world_matrix[5];
                                
                                let target_world_x = current_world_x + dx;
                                let target_world_y = current_world_y + dy;
                                let pm = skeleton.get_parent_world_matrix(bone_idx);
                                let (a, b, c, d, tx, ty) = (pm[0], pm[1], pm[2], pm[3], pm[4], pm[5]);
                                let det = a * d - b * c;

                                if det.abs() > 1e-6 {
                                    let inv_det = 1.0 / det;
                                    let dx_world = target_world_x - tx;
                                    let dy_world = target_world_y - ty;

                                    let bone = &mut skeleton.bones[bone_idx];
                                    bone.local_transform.x = (d * dx_world - c * dy_world) * inv_det;
                                    bone.local_transform.y = (-b * dx_world + a * dy_world) * inv_det;
                                    changed = true;
                                }
                            }
                            _ => {}
                        }

                        if changed {
                            self.is_dirty = true;
                            self.view.needs_full_redraw = true;
                            skeleton.update();
                        }
                    }
                }
            }
            return Ok(());
        }

        if self.engine.tool_manager().active_type == ToolType::CreateBone {
            let (store, symmetry, tool_manager) = self.engine.parts_mut();
            return tool_manager.handle_pointer_move(x, y, store, symmetry);
        }
        let effect = self.engine.handle_input(InputEvent::PointerMove { x, y });
        let result = match &effect {
            EngineEffect::Error(e) => Err(e.clone()),
            _ => Ok(()),
        };

        self.handle_engine_effect(effect);
        result
    }

    pub fn on_mouse_up(&mut self) -> Result<(), CoreError> {
        let was_drawing = self.engine.tool_manager().is_drawing;
        self.last_mouse_pos = None;
        
        if self.mode == AppMode::Animation {
            self.engine.tool_manager_mut().is_drawing = false;
            return Ok(());
        }

        if was_drawing && self.engine.tool_manager().active_type == ToolType::CreateBone {
             if let Some(tool) = self.engine.tool_manager().tools.get(&ToolType::CreateBone) {
                 if let Some(bone_tool) = tool.as_any().downcast_ref::<crate::tools::create_bone::CreateBoneTool>() {
                     let new_bone_id = bone_tool.commit_to_skeleton(&mut self.animation.project.skeleton);
                     
                     if let Some(id) = new_bone_id {
                         self.ui.selected_bone_id = Some(id);
                         self.animation.project.skeleton.update();
                         self.is_dirty = true;
                         self.view.needs_full_redraw = true;
                     }
                 }
             }
             let (store, _, tool_manager) = self.engine.parts_mut();
             return tool_manager.handle_pointer_up(store).map(|_| ());
        }

        let effect = self.engine.handle_input(InputEvent::PointerUp);
        let result = match &effect {
            EngineEffect::Error(e) => Err(e.clone()),
            _ => Ok(()),
        };

        self.handle_engine_effect(effect);
        result
    }

    fn handle_animation_click(&mut self, x: u32, y: u32) -> Result<(), CoreError> {
        let mut clicked_bone_id = None;
        for bone in &self.animation.project.skeleton.bones {
            let bx = bone.world_matrix[4];
            let by = bone.world_matrix[5];
            if ((x as f32 - bx).powi(2) + (y as f32 - by).powi(2)).sqrt() < 10.0 {
                clicked_bone_id = Some(bone.data.id.clone());
                break;
            }
        }
        let is_transform_tool = matches!(self.engine.tool_manager().active_type, ToolType::BoneRotate | ToolType::BoneTranslate);
        if clicked_bone_id.is_some() || !is_transform_tool {
            self.ui.selected_bone_id = clicked_bone_id;
        }
        Ok(())
    }

    fn handle_engine_effect(&mut self, effect: EngineEffect) {
        match effect {
            EngineEffect::None => {},
            EngineEffect::RedrawCanvas => {
                self.is_dirty = true;
                self.view.needs_full_redraw = true;
            },
            EngineEffect::RedrawRect(x, y, w, h) => {
                self.view.mark_dirty_canvas_rect(self.engine.store(), x, y, w, h);
            },
            EngineEffect::ToolCommitted => {
                self.is_dirty = true;
            },
            EngineEffect::Error(e) => {
                self.ui.error_message = Some(e.to_string());
            }
        }
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