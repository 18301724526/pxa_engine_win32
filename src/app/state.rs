use crate::app::io_service::IoService;
use crate::app::engine::PxaEngine;
use crate::history::patch::ActionPatch;
use crate::app::events::InputEvent;
use crate::app::command_handler::{CommandBus, Command};
use crate::app::shortcut_manager::ShortcutManager;
use crate::core::error::CoreError;
use rust_i18n::t;
use crate::core::store::PixelStore;
use crate::core::layer::Layer;
use crate::core::color::Color;
use crate::core::symmetry::SymmetryConfig;
use crate::app::session::{PixelEditSession, AnimationSession};

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
    pub pixel: PixelEditSession,
    pub anim: AnimationSession,
    pub is_space_pressed: bool,
    pub last_mouse_pos: Option<(u32, u32)>,
    pub is_dirty: bool,
    pub mode: AppMode,
    pub shortcuts: ShortcutManager,
    pub command_bus: CommandBus,
}

impl AppState {
    pub fn new() -> Self {
        let mut raw_store = PixelStore::new(128, 128);
        raw_store.add_layer(Layer::new("L1".to_string(), t!("layer.default_name", num = 1).to_string(), 128, 128));
        raw_store.primary_color = Color::new(255, 80, 80, 255);
        
        let engine = PxaEngine::new(
            Box::new(raw_store),
            Box::new(SymmetryConfig::new(128, 128)),
            Box::new(crate::core::id::AtomicIdGenerator::new(1))
        );
        let mut state = Self {
            pixel: PixelEditSession::new(engine),
            anim: AnimationSession::new(),
            is_space_pressed: false,
            last_mouse_pos: None,
            is_dirty: false,
            mode: AppMode::PixelEdit,
            shortcuts: ShortcutManager::new(),
            command_bus: CommandBus::new(),
        };
        let w = state.pixel.engine.store().canvas_width as f32;
        let h = state.pixel.engine.store().canvas_height as f32;
        state.pixel.view.update_viewport(w, h);

        let cx = state.pixel.engine.store().canvas_width as f32 / 2.0;
        let cy = state.pixel.engine.store().canvas_height as f32 / 2.0;

        if let Some(idx) = state.anim.state.project.skeleton.bone_id_to_index("root") {
            let root = &mut state.anim.state.project.skeleton.bones[idx];
            root.local_transform.x = cx;
            root.local_transform.y = cy;
            state.anim.state.project.skeleton.update();
        }
        
        if let Some(id) = state.pixel.engine.store().active_layer_id.clone() {
            if let Some(layer) = state.pixel.engine.store().get_layer(&id) {
                if state.anim.state.project.skeleton.bone_id_to_index("root").is_some() {
                    let mut slot = crate::core::animation::slot::SlotData::new(id.clone(), layer.name.clone(), "root".to_string());
                    slot.attachment = Some(id.clone());
                    state.anim.state.project.skeleton.slots.push(crate::core::animation::slot::RuntimeSlot::new(slot));
                }
            }
        }
        
        state
    }
    pub fn enqueue_command(&mut self, cmd: Box<dyn Command>) {
        self.command_bus.dispatch(cmd);
    }

    pub fn process_commands(&mut self) {
        let mut bus = std::mem::replace(&mut self.command_bus, CommandBus::new());
        bus.process_all(self);
        self.command_bus = bus;
    }

    pub fn set_tool(&mut self, tool_type: ToolType) {
        if self.pixel.engine.tool_manager().active_type == tool_type { return; }
        self.commit_current_tool();
        self.pixel.engine.tool_manager_mut().is_drawing = false;
        self.pixel.engine.tool_manager_mut().set_tool(tool_type);
    }

    pub fn commit_current_tool(&mut self) {
        let effect = self.pixel.engine.handle_input(InputEvent::CommitTool);
        crate::app::input_handler::InputHandler::handle_engine_effect(self, effect);
    }

    pub fn cancel_current_tool(&mut self) {
        let effect = self.pixel.engine.handle_input(InputEvent::CancelTool);
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
        let _ = self.enqueue_command(Box::new(crate::app::commands::UndoCmd));
    }
    pub fn redo(&mut self) { 
        let _ = self.enqueue_command(Box::new(crate::app::commands::RedoCmd));
    }
    
    pub fn add_new_layer(&mut self) { 
        if self.mode == AppMode::Animation {
            self.command_bus.events.push_back(crate::app::command_handler::AppEvent::ShowError("动画模式下禁止修改图层结构".into()));
            return;
        }
        let old_count = self.pixel.engine.store().layers.len();
        if let Err(e) = self.pixel.engine.add_new_layer() {
            self.command_bus.events.push_back(crate::app::command_handler::AppEvent::ShowError(e.to_string()));
        } else {
            if self.pixel.engine.store().layers.len() > old_count {
                if let Some(id) = &self.pixel.engine.store().active_layer_id {
                    let name = self.pixel.engine.store().get_layer(id).unwrap().name.clone();
                    let mut slot = crate::core::animation::slot::SlotData::new(id.clone(), name, "root".to_string());
                    slot.attachment = Some(id.clone());
                    let old_skel = self.anim.state.project.skeleton.clone();
                    self.anim.state.project.skeleton.slots.push(crate::core::animation::slot::RuntimeSlot::new(slot));
                    self.anim.state.history.commit(crate::animation::history::AnimPatch::Skeleton { old: old_skel, new: self.anim.state.project.skeleton.clone() });
                }
            }
            self.is_dirty = true;
            self.pixel.view.needs_full_redraw = true;
        }
    }

    pub fn delete_active_layer(&mut self) { 
        if self.mode == AppMode::Animation {
            self.command_bus.events.push_back(crate::app::command_handler::AppEvent::ShowError("动画模式下禁止修改图层结构".into()));
            return;
        }
        let id_to_delete = self.pixel.engine.store().active_layer_id.clone();
        if let Err(e) = self.pixel.engine.delete_active_layer() {
            self.command_bus.events.push_back(crate::app::command_handler::AppEvent::ShowError(e.to_string()));
        } else {
            if let Some(id) = id_to_delete {
                let old_skel = self.anim.state.project.skeleton.clone();
                self.anim.state.project.skeleton.slots.retain(|s| s.data.id != id);
                self.anim.state.history.commit(crate::animation::history::AnimPatch::Skeleton { old: old_skel, new: self.anim.state.project.skeleton.clone() });
            }
            self.is_dirty = true;
            self.pixel.view.needs_full_redraw = true;
        }
    }
    pub fn toggle_layer_visibility(&mut self, layer_id: &str) { 
        if let Err(e) = self.pixel.engine.toggle_layer_visibility(layer_id) {
            self.command_bus.events.push_back(crate::app::command_handler::AppEvent::ShowError(e.to_string()));
        } else {
            self.is_dirty = true;
            self.pixel.view.needs_full_redraw = true;
        }
    }

    pub fn import_image(&mut self) {
        if let Some(path) = IoService::pick_import_path() {
            let id = format!("layer_imp_{}", self.pixel.engine.id_gen().generate());
            let name = t!("layer.import_name", num = self.pixel.engine.store().layers.len() + 1).to_string();
            let w = self.pixel.engine.store().canvas_width;
            let h = self.pixel.engine.store().canvas_height;
            let old_active_id = self.pixel.engine.store().active_layer_id.clone();
            
            match IoService::load_as_layer(path, w, h, id.clone(), name) {
                Ok(layer) => {
                    let index = self.pixel.engine.store().layers.len();
                    let patch = ActionPatch::new_layer_add(format!("patch_{}", id), id.clone(), layer, index, old_active_id);
                    if let Err(e) = self.pixel.engine.commit_patch(patch) {
                        self.command_bus.events.push_back(crate::app::command_handler::AppEvent::ShowError(e.to_string()));
                    } else {
                        self.pixel.engine.set_active_layer(id);
                        self.is_dirty = true;
                        self.pixel.view.needs_full_redraw = true;
                    }
                }
                Err(e) => self.command_bus.events.push_back(crate::app::command_handler::AppEvent::ShowError(t!("error.import_image_failed", err = e.to_string()).to_string())),
            }
        }
    }

    pub fn export_to_png(&mut self) {
        if let Some(path) = IoService::pick_export_path() {
            if let Err(e) = IoService::save_png(path, self.pixel.engine.store()) {
                self.command_bus.events.push_back(crate::app::command_handler::AppEvent::ShowError(t!("error.export_failed", err = e.to_string()).to_string()));
            }
        }
    }

    pub fn import_palette(&mut self) {
        if let Some(path) = IoService::pick_palette_import_path() {
            match crate::format::hex_palette::load_from_hex(&path) {
                Ok(palette) => self.enqueue_command(Box::new(crate::app::commands::SetPaletteCmd(palette))),
                Err(e) => self.command_bus.events.push_back(crate::app::command_handler::AppEvent::ShowError(t!("error.load_palette_failed", err = e.to_string()).to_string())),
            }
        }
    }

    pub fn export_palette(&mut self) {
        if let Some(path) = IoService::pick_palette_export_path() {
            if let Err(e) = crate::format::hex_palette::save_to_hex(&path, &self.pixel.engine.store().palette) {
                self.command_bus.events.push_back(crate::app::command_handler::AppEvent::ShowError(t!("error.export_palette_failed", err = e.to_string()).to_string()));
            }
        }
    }
    pub fn save_project_to_pxad(&mut self) {
        if let Some(path) = IoService::pick_project_save_path() {
            if let Err(e) = IoService::save_project(path, self.pixel.engine.store(), self.pixel.engine.symmetry(), &self.pixel.view) {
                self.command_bus.events.push_back(crate::app::command_handler::AppEvent::ShowError(t!("error.save_project_failed", err = e.to_string()).to_string()));
            } else {
                self.is_dirty = false;
            }
        }
    }

    pub fn load_project_from_pxad(&mut self) {
        if let Some(path) = IoService::pick_project_load_path() {
            match IoService::load_project(path) {
                Ok((new_store, new_sym, px, py, zl)) => {
                    self.pixel.engine.replace_store_and_symmetry(new_store, new_sym);
                    self.pixel.view.pan_x = px;
                    self.pixel.view.pan_y = py;
                    self.pixel.view.zoom_level = zl;
                    self.is_dirty = false;
                    self.pixel.view.needs_full_redraw = true;
                }
                Err(e) => self.command_bus.events.push_back(crate::app::command_handler::AppEvent::ShowError(t!("error.load_project_failed", err = e.to_string()).to_string())),
            }
        }
    }

    pub fn export_animation(&mut self, as_gif: bool) {
        let active_id = match &self.anim.state.project.active_animation_id {
            Some(id) => id.clone(),
            None => { self.command_bus.events.push_back(crate::app::command_handler::AppEvent::ShowError("没有正在编辑的动画！".to_string())); return; }
        };
        let anim = match self.anim.state.project.animations.get(&active_id) {
            Some(a) => a.clone(),
            None => return,
        };
        let duration = anim.duration;
        if duration <= 0.0 { self.command_bus.events.push_back(crate::app::command_handler::AppEvent::ShowError("动画时长为 0，无法导出！".to_string())); return; }

        let path_opt = if as_gif { IoService::pick_gif_export_path() } else { IoService::pick_sequence_export_dir() };
        let path = match path_opt { Some(p) => p, None => return };

        let orig_time = self.anim.state.current_time;
        let orig_mode = self.mode;
        self.mode = AppMode::Animation;
        
        let fps = 30;
        let total_frames = (duration * fps as f32).ceil() as u32;
        let mut extracted_frames = Vec::with_capacity((total_frames + 1) as usize);
        
        for i in 0..=total_frames {
            self.anim.state.current_time = i as f32 / fps as f32;
            crate::animation::controller::AnimationController::apply_current_pose(&mut self.anim.state);
            self.sync_animation_to_layers();
            
            // 强制 CPU 渲染这一帧的所有像素
            self.pixel.engine.update_render_cache(None);
            extracted_frames.push(self.pixel.engine.store().composite_cache.clone());
        }
        
        // 恢复导出前的状态
        self.anim.state.current_time = orig_time;
        crate::animation::controller::AnimationController::apply_current_pose(&mut self.anim.state);
        self.sync_animation_to_layers();
        self.mode = orig_mode;
        self.pixel.engine.update_render_cache(None);
        self.pixel.view.needs_full_redraw = true;

        let w = self.pixel.engine.store().canvas_width;
        let h = self.pixel.engine.store().canvas_height;

        let result = if as_gif { IoService::save_gif(path, w, h, extracted_frames, fps) } 
                     else { IoService::save_sequence(path, w, h, extracted_frames) };

        match result {
            Err(e) => self.command_bus.events.push_back(crate::app::command_handler::AppEvent::ShowError(e.to_string())),
            _ => {}
        }
    }
    pub fn sync_animation_to_layers(&mut self) {
        let mut changes = false;
        let is_anim_mode = self.mode == AppMode::Animation;
        let skeleton = &self.anim.state.project.skeleton;
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
        
        let (store, _, _, _) = self.pixel.engine.parts_mut();
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
        
        if changes { self.pixel.engine.update_render_cache(None); }
    }
}