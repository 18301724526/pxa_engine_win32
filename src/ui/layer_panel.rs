use egui::{Ui, Color32, RichText};
use crate::app::state::{AppState, AppMode};
use crate::app::commands::*;
use crate::ui::symmetry_panel::SymmetryPanel;
use crate::core::blend_mode::BlendMode;
use rust_i18n::t;

const ICON_EYE_OPEN: &str  = "\u{ecb4}"; 
const ICON_EYE_CLOSE: &str = "\u{ecb6}"; 
const ICON_ADD: &str       = "\u{ea13}"; 
const ICON_DELETE: &str    = "\u{ec29}"; 
const ICON_MERGE: &str     = "\u{f180}"; 
const ICON_LOCK: &str      = "\u{eecd}"; 
const ICON_UNLOCK: &str    = "\u{eed2}"; 

pub struct LayerPanel;

impl LayerPanel {
    fn draw_layer_node(
        ui: &mut Ui,
        app: &mut AppState,
        ui_ctx: &mut crate::app::ui_context::UiContext,
        id: &str,
        meta: &(usize, String, bool, bool),
        depth: usize
    ) {
        let (idx, name, visible, locked) = meta;
        let is_selected = ui_ctx.selected_layer_ids.contains(&id.to_string());
        let is_active = Some(id.to_string()) == app.pixel.engine.store().active_layer_id; 
        let is_dragging = ui_ctx.dragging_layer_id.as_deref() == Some(id);
        
        let bg_color = if is_dragging { Color32::from_rgb(40, 60, 100) } 
                       else if is_active { Color32::from_rgb(60, 60, 70) } 
                       else if is_selected { Color32::from_rgb(45, 45, 50) } 
                       else { Color32::TRANSPARENT };

        let indent = depth as f32 * 16.0;
        
        egui::Frame::none().fill(bg_color).show(ui, |ui| {
            let row_height = 36.0;
            let (rect, _resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), row_height), egui::Sense::hover());
            let painter = ui.painter().clone();

            let content_min_x = rect.min.x + indent;

            let eye_rect = egui::Rect::from_min_size(egui::pos2(content_min_x, rect.min.y), egui::vec2(28.0, row_height));
            let eye_resp = ui.interact(eye_rect, ui.id().with(format!("eye_{}", id)), egui::Sense::click());
            if eye_resp.clicked() { app.toggle_layer_visibility(id); }
            let eye_icon = if *visible { ICON_EYE_OPEN } else { ICON_EYE_CLOSE };
            let eye_col = if *visible { Color32::LIGHT_GRAY } else { Color32::from_gray(80) };
            painter.text(eye_rect.center(), egui::Align2::CENTER_CENTER, eye_icon, egui::FontId::proportional(14.0), eye_col);

            let thumb_size = 32.0;
            let thumb_rect = egui::Rect::from_center_size(
                egui::pos2(content_min_x + 28.0 + thumb_size / 2.0, rect.center().y),
                egui::vec2(thumb_size, thumb_size)
            );
            
            let cs = 4.0; 
            for ty in 0..8 {
                for tx in 0..8 {
                    let color = if (tx + ty) % 2 == 0 { Color32::from_gray(100) } else { Color32::from_gray(150) };
                    painter.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(thumb_rect.min.x + tx as f32 * cs, thumb_rect.min.y + ty as f32 * cs), egui::vec2(cs, cs)),
                        0.0, color
                    );
                }
            }
            if let Some(layer) = app.pixel.engine.store().get_layer(id) {
                let step_x = (layer.width as f32 / 32.0).max(1.0);
                let step_y = (layer.height as f32 / 32.0).max(1.0);
                let sample_w = (layer.width as f32 / step_x).ceil() as u32;
                let sample_h = (layer.height as f32 / step_y).ceil() as u32;
                let px_w = 32.0 / sample_w.max(1) as f32;
                let px_h = 32.0 / sample_h.max(1) as f32;
                
                for sy in 0..sample_h.min(32) {
                    for sx in 0..sample_w.min(32) {
                        if let Some(c) = layer.get_pixel((sx as f32 * step_x) as u32, (sy as f32 * step_y) as u32) {
                            if c.a > 0 {
                                painter.rect_filled(
                                    egui::Rect::from_min_size(egui::pos2(thumb_rect.min.x + sx as f32 * px_w, thumb_rect.min.y + sy as f32 * px_h), egui::vec2(px_w, px_h)),
                                    0.0, Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
                                );
                            }
                        }
                    }
                }
            }
            painter.rect_stroke(thumb_rect, 0.0, egui::Stroke::new(1.0, Color32::from_gray(80)));

            let lock_rect = egui::Rect::from_min_max(egui::pos2(rect.max.x - 24.0, rect.min.y), rect.max);
            let lock_resp = ui.interact(lock_rect, ui.id().with(format!("lock_{}", id)), egui::Sense::click());
            if lock_resp.clicked() { app.enqueue_command(Box::new(ToggleLayerLockCmd(id.to_string()))); }
            
            if *locked { painter.text(lock_rect.center(), egui::Align2::CENTER_CENTER, ICON_LOCK, egui::FontId::proportional(14.0), Color32::WHITE); } 
            else if lock_resp.hovered() { painter.text(lock_rect.center(), egui::Align2::CENTER_CENTER, ICON_UNLOCK, egui::FontId::proportional(14.0), Color32::from_gray(120)); }

            let name_rect = egui::Rect::from_min_max(egui::pos2(thumb_rect.max.x + 8.0, rect.min.y), egui::pos2(lock_rect.min.x - 4.0, rect.max.y));
            
            if ui_ctx.renaming_layer_id.as_deref() == Some(id) {
                ui.allocate_ui_at_rect(name_rect, |ui| {
                    ui.centered_and_justified(|ui| {
                        let response = ui.add(egui::TextEdit::singleline(&mut ui_ctx.renaming_buffer));
                        response.request_focus();
                        if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            app.enqueue_command(Box::new(RenameLayerCmd(id.to_string(), ui_ctx.renaming_buffer.clone())));
                            ui_ctx.renaming_layer_id = None;
                        }
                    });
                });
            } else {
                let name_resp = ui.interact(name_rect, ui.id().with(format!("name_{}", id)), egui::Sense::click_and_drag());
                painter.text(name_rect.left_center(), egui::Align2::LEFT_CENTER, name, egui::FontId::proportional(14.0), Color32::WHITE);
                
                if name_resp.clicked() { 
                    let modifiers = ui.input(|inp| inp.modifiers);
                    if modifiers.ctrl || modifiers.command {
                        if is_selected { ui_ctx.selected_layer_ids.retain(|x| x != id); } 
                        else { ui_ctx.selected_layer_ids.push(id.to_string()); }
                    } else {
                        ui_ctx.selected_bone_id = None;
                        app.anim.selected_bone_id = None;
                        ui_ctx.selected_layer_ids = vec![id.to_string()];
                    }
                    app.pixel.engine.set_active_layer(id.to_string());
                    ui_ctx.last_clicked_layer_id = Some(id.to_string());
                }
                
                if name_resp.drag_started() { ui_ctx.dragging_layer_id = Some(id.to_string()); }
                if name_resp.double_clicked() { ui_ctx.renaming_layer_id = Some(id.to_string()); ui_ctx.renaming_buffer = name.to_string(); }

                if app.mode == AppMode::PixelEdit {
                    name_resp.context_menu(|ui| {
                        if ui.button(t!("layer.copy_layer")).clicked() { app.enqueue_command(Box::new(DuplicateLayerCmd(id.to_string()))); ui.close_menu(); }
                        if ui.button(t!("layer.merge_selected")).clicked() { app.enqueue_command(Box::new(MergeSelectedCmd(ui_ctx.selected_layer_ids.clone()))); ui.close_menu(); }
                        ui.separator();
                        if ui.button(t!("layer.delete_layer")).clicked() { app.pixel.engine.set_active_layer(id.to_string()); app.delete_active_layer(); ui.close_menu(); }
                    });
                }
            }

            if let Some(drag_id) = &ui_ctx.dragging_layer_id {
                if drag_id != id {
                    if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                        if rect.contains(pos) && ui_ctx.drag_target_bone_id.is_none() {
                            let is_top = pos.y < rect.center().y;
                            let line_y = if is_top { rect.top() } else { rect.bottom() };
                            painter.hline(rect.left()..=rect.right(), line_y, egui::Stroke::new(2.0, Color32::LIGHT_BLUE));
                            ui_ctx.drop_target_idx = Some(if is_top { *idx + 1 } else { *idx });
                        }
                    }
                } else {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
                }
            }
        });
    }

    fn draw_bone_tree(
        ui: &mut Ui,
        app: &mut AppState,
        ui_ctx: &mut crate::app::ui_context::UiContext,
        bone_id: &str,
        depth: usize,
        bone_children: &std::collections::HashMap<String, Vec<String>>,
        bone_slots: &std::collections::HashMap<String, Vec<String>>,
        layer_metas: &std::collections::HashMap<String, (usize, String, bool, bool)>
    ) {
        let bone_name = app.anim.state.project.skeleton.bones.iter().find(|b| b.data.id == bone_id).unwrap().data.name.clone();
        let indent = depth as f32 * 16.0;
        let row_height = 26.0;

        let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), row_height), egui::Sense::click());
        
        let mut is_drop_target = false;
        if ui_ctx.dragging_layer_id.is_some() {
            if ui.rect_contains_pointer(rect) {
                is_drop_target = true;
                ui_ctx.drag_target_bone_id = Some(bone_id.to_string());
            }
        };

        let bg_color = if is_drop_target { Color32::from_rgb(80, 80, 40) }
                       else if ui_ctx.selected_bone_id.as_deref() == Some(bone_id) { Color32::from_rgb(60, 60, 80) }
                       else { Color32::TRANSPARENT };
        
        ui.painter().rect_filled(rect, 0.0, bg_color);
        
        let is_expanded = ui_ctx.expanded_bones.contains(bone_id);
        let icon = if is_expanded { "▼" } else { "▶" };
        let text_pos = rect.min + egui::vec2(indent + 4.0, 6.0);
        ui.painter().text(text_pos, egui::Align2::LEFT_TOP, format!("{} 🦴 {}", icon, bone_name), egui::FontId::proportional(14.0), Color32::WHITE);
        
        if resp.clicked() {
            ui_ctx.selected_bone_id = Some(bone_id.to_string());
            app.anim.selected_bone_id = Some(bone_id.to_string());
            ui_ctx.selected_layer_ids.clear();
            if is_expanded { ui_ctx.expanded_bones.remove(bone_id); }
            else { ui_ctx.expanded_bones.insert(bone_id.to_string()); }
        }
        
        if app.mode == AppMode::PixelEdit {
            resp.context_menu(|ui| {
                if ui.button("➕ 新建子骨骼").clicked() {
                    app.set_tool(crate::app::state::ToolType::CreateBone);
                    ui_ctx.selected_bone_id = Some(bone_id.to_string());
                    ui.close_menu();
                }
                if ui.add_enabled(bone_id != "root", egui::Button::new("🗑 删除骨骼")).clicked() {
                    app.enqueue_command(Box::new(DeleteBoneCmd(bone_id.to_string())));
                    ui.close_menu();
                }
            });
        }

        if is_expanded {
            if let Some(layers) = bone_slots.get(bone_id) {
                let mut sorted = layers.clone();
                sorted.sort_by_key(|lid| std::cmp::Reverse(layer_metas.get(lid).map(|m| m.0).unwrap_or(0)));
                for lid in sorted {
                    if let Some(meta) = layer_metas.get(&lid) {
                        Self::draw_layer_node(ui, app, ui_ctx, &lid, meta, depth + 1);
                    }
                }

            }
            if let Some(children) = bone_children.get(bone_id) {
                for child_id in children {
                    Self::draw_bone_tree(ui, app, ui_ctx, child_id, depth + 1, bone_children, bone_slots, layer_metas);
                }
            }
        }
    }
    pub fn show(ui: &mut Ui, app: &mut AppState, ui_ctx: &mut crate::app::ui_context::UiContext) {
        
            egui::TopBottomPanel::bottom("layer_bottom_panel")
                .resizable(false)
                .frame(egui::Frame::none())
                .show_inside(ui, |ui| {
                    SymmetryPanel::show(ui, app, ui_ctx);
                    
                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        let can_setup = app.mode == AppMode::PixelEdit;
                        if ui.add_enabled(can_setup, egui::Button::new(RichText::new(ICON_ADD).size(16.0))).on_hover_text(t!("layer.new")).clicked() { app.add_new_layer(); }
                        let (del_hint, is_bone) = if let Some(bone_id) = &ui_ctx.selected_bone_id {
                            let name = app.anim.state.project.skeleton.bones.iter()
                                .find(|b| b.data.id == *bone_id)
                                .map(|b| b.data.name.as_str())
                                .unwrap_or("未知骨骼");
                            (format!("{} ({})", t!("layer.delete"), name), true)
                        } else {
                            (t!("layer.delete").to_string(), false)
                        };

                        let is_root_selected = ui_ctx.selected_bone_id.as_deref() == Some("root");
                        if ui.add_enabled(can_setup && !is_root_selected, egui::Button::new(RichText::new(ICON_DELETE).size(16.0))).on_hover_text(del_hint).clicked() {
                            if is_bone {
                                let bone_id = ui_ctx.selected_bone_id.clone().unwrap();
                                app.enqueue_command(Box::new(DeleteBoneCmd(bone_id)));
                            } else {
                                app.delete_active_layer();
                            }
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add_enabled(can_setup, egui::Button::new(RichText::new(ICON_MERGE).size(16.0))).on_hover_text(t!("layer.merge")).clicked() {
                                app.enqueue_command(Box::new(MergeSelectedCmd(ui_ctx.selected_layer_ids.clone()))); 
                            }
                        });
                    });
                    ui.add_space(5.0);
                });

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.heading(t!("layer.title").to_string());
                });
                ui.separator();

                let active_id = app.pixel.engine.store().active_layer_id.clone();
                let mut active_opacity = 255;
                let mut active_blend = BlendMode::Normal;
                let has_active = if let Some(id) = &active_id {
                    if let Some(layer) = app.pixel.engine.store().get_layer(id) {
                        active_opacity = layer.opacity;
                        active_blend = layer.blend_mode;
                        true
                    } else { false }
                } else { false };

                ui.add_enabled_ui(has_active, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", t!("layer.mode")));
                        let mut new_blend = active_blend;
                        egui::ComboBox::from_id_source("top_blend_mode")
                            .width(ui.available_width()) 
                            .selected_text(new_blend.name())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut new_blend, BlendMode::Normal, BlendMode::Normal.name());
                                ui.selectable_value(&mut new_blend, BlendMode::Multiply, BlendMode::Multiply.name());
                                ui.selectable_value(&mut new_blend, BlendMode::Screen, BlendMode::Screen.name());
                                ui.selectable_value(&mut new_blend, BlendMode::Add, BlendMode::Add.name());
                            });
                        if new_blend != active_blend { 
                            if let Some(id) = &active_id { app.enqueue_command(Box::new(SetLayerBlendModeCmd(id.clone(), new_blend))); } 
                        }
                    });
                    
                    ui.add_space(4.0);
                    
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", t!("layer.opacity")));
                        let mut op_percent = (active_opacity as f32 / 255.0 * 100.0).round() as u32;

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label("%");
                            let drag_resp = ui.add(egui::DragValue::new(&mut op_percent).clamp_range(0..=100));
                            
                            ui.spacing_mut().slider_width = (ui.available_width() - 5.0).max(10.0);
                            let slider_resp = ui.add(egui::Slider::new(&mut op_percent, 0..=100).show_value(false).trailing_fill(true));
                            
                            if drag_resp.changed() || slider_resp.changed() {
                                let new_op = ((op_percent as f32 / 100.0) * 255.0) as u8;
                                if let Some(id) = &active_id { app.enqueue_command(Box::new(SetLayerOpacityCmd(id.clone(), new_op))); }
                            }
                        });
                    });
                }); 
                
                ui.add_space(4.0);
                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui_ctx.drop_target_idx = None;
                        ui_ctx.drag_target_bone_id = None;

                        let mut layer_metas = std::collections::HashMap::new();
                        for (i, l) in app.pixel.engine.store().layers.iter().enumerate() {
                            layer_metas.insert(l.id.clone(), (i, l.name.clone(), l.visible, l.locked));
                        }

                        let current_active = active_id.clone();
                        if let Some(id) = &current_active { if !ui_ctx.selected_layer_ids.contains(id) { ui_ctx.selected_layer_ids.push(id.clone()); } }

                        let mut bone_children: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
                        let mut root_bones = Vec::new();
                        for bone in &app.anim.state.project.skeleton.bones {
                            if let Some(pid) = &bone.data.parent_id { bone_children.entry(pid.to_string()).or_default().push(bone.data.id.clone()); }
                            else { root_bones.push(bone.data.id.clone()); }
                        }

                        let mut bone_slots: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
                        let mut bound_layers = std::collections::HashSet::new();
                        for slot in &app.anim.state.project.skeleton.slots {
                            bone_slots.entry(slot.data.bone_id.clone()).or_default().push(slot.data.id.clone());
                            bound_layers.insert(slot.data.id.clone());
                        }
                        
                        let root_exists = root_bones.iter().any(|id| id == "root");
                        for lid in layer_metas.keys() {
                            if !bound_layers.contains(lid) && root_exists {
                                bone_slots.entry("root".to_string()).or_default().push(lid.clone());
                            }
                        }

                        for root_id in &root_bones {
                            Self::draw_bone_tree(ui, app, ui_ctx, root_id, 0, &bone_children, &bone_slots, &layer_metas);
                        }
                        
                        if ui.input(|i| i.pointer.any_released()) {
                            if let Some(drag_id) = ui_ctx.dragging_layer_id.take() {
                                if let Some(target_bone) = ui_ctx.drag_target_bone_id.take() {
                                    if app.mode == AppMode::PixelEdit {
                                        if ui_ctx.selected_layer_ids.contains(&drag_id) {
                                            let selected = ui_ctx.selected_layer_ids.clone();
                                            for sel_id in selected {
                                                app.enqueue_command(Box::new(BindLayerToBoneCmd(sel_id, target_bone.clone())));
                                            }
                                        } else { 
                                            app.enqueue_command(Box::new(BindLayerToBoneCmd(drag_id, target_bone))); 
                                        }
                                    } else {
                                        // 动画模式下释放，虽然不执行，但可以产生一个错误提示
                                        // 注意：BindLayerToBoneCmd 内部已有 AppMode 检查，
                                        // 这里可以选择不发送命令或发送后让 Handler 报错。
                                        // 为符合需求“不应触发绑定操作”，我们直接跳过命令发送。
                                    }
                               } else if let Some(target_idx) = ui_ctx.drop_target_idx.take() {
                                    app.enqueue_command(Box::new(MoveLayerToIndexCmd(drag_id, target_idx)));
                                }
                            }
                        }
                    });
            });
    }
}