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
        let mut state = Self {
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
        };
        
        if let Some(id) = state.engine.store().active_layer_id.clone() {
            if let Some(layer) = state.engine.store().get_layer(&id) {
                let mut slot = crate::core::animation::slot::SlotData::new(id.clone(), layer.name.clone(), "root".to_string());
                slot.attachment = Some(id.clone());
                state.animation.project.skeleton.slots.push(crate::core::animation::slot::RuntimeSlot::new(slot));
            }
        }
        
        state
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
        let old_count = self.engine.store().layers.len();
        if let Err(e) = self.engine.add_new_layer() {
            self.ui.error_message = Some(e.to_string());
        } else {
            if self.engine.store().layers.len() > old_count {
                if let Some(id) = &self.engine.store().active_layer_id {
                    let name = self.engine.store().get_layer(id).unwrap().name.clone();
                    let mut slot = crate::core::animation::slot::SlotData::new(id.clone(), name, "root".to_string());
                    slot.attachment = Some(id.clone());
                    let old_skel = self.animation.project.skeleton.clone();
                    self.animation.project.skeleton.slots.push(crate::core::animation::slot::RuntimeSlot::new(slot));
                    self.animation.history.commit(crate::animation::history::AnimPatch::Skeleton { old: old_skel, new: self.animation.project.skeleton.clone() });
                }
            }
            self.is_dirty = true;
            self.view.needs_full_redraw = true;
        }
    }

    pub fn delete_active_layer(&mut self) { 
        let id_to_delete = self.engine.store().active_layer_id.clone();
        if let Err(e) = self.engine.delete_active_layer() {
            self.ui.error_message = Some(e.to_string());
        } else {
            if let Some(id) = id_to_delete {
                let old_skel = self.animation.project.skeleton.clone();
                self.animation.project.skeleton.slots.retain(|s| s.data.id != id);
                self.animation.history.commit(crate::animation::history::AnimPatch::Skeleton { old: old_skel, new: self.animation.project.skeleton.clone() });
            }
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
    pub fn sync_animation_to_layers(&mut self) {
        let mut changes = false;
        let is_anim_mode = self.mode == AppMode::Animation;
        let skeleton = &self.animation.project.skeleton;
        let mut new_transforms = std::collections::HashMap::new();
        let mut new_offsets = std::collections::HashMap::new();

        if is_anim_mode {
            // 1. 动态计算所有骨骼的“Setup/Bind Pose(装配姿态)”的世界矩阵
            let mut setup_matrices = vec![[1.0, 0.0, 0.0, 1.0, 0.0, 0.0]; skeleton.bones.len()];
            for i in 0..skeleton.bones.len() {
                let bone = &skeleton.bones[i];
                let local_matrix = bone.data.local_transform.to_matrix();
                let parent_matrix = bone.parent_index.map(|p_idx| setup_matrices[p_idx]);
                setup_matrices[i] = match parent_matrix {
                    None => local_matrix,
                    Some(pm) => {
                        let pa = pm[0]; let pb = pm[1]; let pc = pm[2]; let pd = pm[3]; let px = pm[4]; let py = pm[5];
                        let la = local_matrix[0]; let lb = local_matrix[1]; let lc = local_matrix[2]; let ld = local_matrix[3]; let lx = local_matrix[4]; let ly = local_matrix[5];
                        [
                            pa * la + pc * lb, pb * la + pd * lb,
                            pa * lc + pc * ld, pb * lc + pd * ld,
                            pa * lx + pc * ly + px, pb * lx + pd * ly + py
                        ]
                    }
                };
            }

            // 2. 根据 M_bind(绑定状态) 和 M_curr(当前状态) 计算逆矩阵
            for slot in &skeleton.slots {
                if let Some(layer_id) = &slot.current_attachment {
                    if let Some(bone_idx) = skeleton.bones.iter().position(|b| b.data.id == slot.data.bone_id) {
                        let m_bind = setup_matrices[bone_idx];
                        let m_curr = skeleton.bones[bone_idx].world_matrix;

                        // [核心修复] 提取纯位移差，保持严格的像素网格对齐
                        let dx = (m_curr[4] - m_bind[4]).round() as i32;
                        let dy = (m_curr[5] - m_bind[5]).round() as i32;
                        new_offsets.insert(layer_id.clone(), (dx, dy));

                        // [智能降级] 仅在含有旋转或缩放时，才启用浮点矩阵渲染
                        let has_rotation_or_scale = (m_curr[0] - m_bind[0]).abs() > 1e-4 ||
                                                    (m_curr[1] - m_bind[1]).abs() > 1e-4 ||
                                                    (m_curr[2] - m_bind[2]).abs() > 1e-4 ||
                                                    (m_curr[3] - m_bind[3]).abs() > 1e-4;

                        if has_rotation_or_scale {
                            let det = m_curr[0] * m_curr[3] - m_curr[1] * m_curr[2];
                            if det.abs() > 1e-6 {
                                let inv_det = 1.0 / det;
                                let i_a = m_curr[3] * inv_det;
                                let i_b = -m_curr[1] * inv_det;
                                let i_c = -m_curr[2] * inv_det;
                                let i_d = m_curr[0] * inv_det;
                                let i_tx = (m_curr[2]*m_curr[5] - m_curr[3]*m_curr[4]) * inv_det;
                                let i_ty = (m_curr[1]*m_curr[4] - m_curr[0]*m_curr[5]) * inv_det;

                                let f_a = m_bind[0]*i_a + m_bind[2]*i_b;
                                let f_b = m_bind[1]*i_a + m_bind[3]*i_b;
                                let f_c = m_bind[0]*i_c + m_bind[2]*i_d;
                                let f_d = m_bind[1]*i_c + m_bind[3]*i_d;
                                let f_tx = m_bind[0]*i_tx + m_bind[2]*i_ty + m_bind[4];
                                let f_ty = m_bind[1]*i_tx + m_bind[3]*i_ty + m_bind[5];

                                new_transforms.insert(layer_id.clone(), [f_a, f_b, f_c, f_d, f_tx, f_ty]);
                            }
                        }
                    }
                }
            }
        }
        
        let (store, _, _) = self.engine.parts_mut();
        if store.layer_anim_transforms != new_transforms {
            store.layer_anim_transforms = new_transforms;
            changes = true;
        }

        for layer in &mut store.layers {
            let (target_tx, target_ty) = new_offsets.get(&layer.id).copied().unwrap_or((0, 0));
            if layer.anim_offset_x != target_tx || layer.anim_offset_y != target_ty {
                layer.anim_offset_x = target_tx;
                layer.anim_offset_y = target_ty;
                changes = true;
            }
        }
        
        if changes { self.engine.update_render_cache(None); }
    }
}