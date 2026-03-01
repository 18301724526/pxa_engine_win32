use egui::{Ui, Color32, RichText};
use crate::app::state::AppState;
use crate::app::commands::AppCommand;
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
        id: &str,
        meta: &(usize, String, bool, bool),
        depth: usize
    ) {
        let (idx, name, visible, locked) = meta;
        let is_selected = app.ui.selected_layer_ids.contains(&id.to_string());
        let is_active = Some(id.to_string()) == app.engine.store().active_layer_id; 
        let is_dragging = app.ui.dragging_layer_id.as_deref() == Some(id);
        
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
            if let Some(layer) = app.engine.store().get_layer(id) {
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
            if lock_resp.clicked() { app.enqueue_command(AppCommand::ToggleLayerLock(id.to_string())); }
            
            if *locked { painter.text(lock_rect.center(), egui::Align2::CENTER_CENTER, ICON_LOCK, egui::FontId::proportional(14.0), Color32::WHITE); } 
            else if lock_resp.hovered() { painter.text(lock_rect.center(), egui::Align2::CENTER_CENTER, ICON_UNLOCK, egui::FontId::proportional(14.0), Color32::from_gray(120)); }

            let name_rect = egui::Rect::from_min_max(egui::pos2(thumb_rect.max.x + 8.0, rect.min.y), egui::pos2(lock_rect.min.x - 4.0, rect.max.y));
            
            if app.ui.renaming_layer_id.as_deref() == Some(id) {
                ui.allocate_ui_at_rect(name_rect, |ui| {
                    ui.centered_and_justified(|ui| {
                        let response = ui.add(egui::TextEdit::singleline(&mut app.ui.renaming_buffer));
                        response.request_focus();
                        if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            app.enqueue_command(AppCommand::RenameLayer(id.to_string(), app.ui.renaming_buffer.clone()));
                            app.ui.renaming_layer_id = None;
                        }
                    });
                });
            } else {
                let name_resp = ui.interact(name_rect, ui.id().with(format!("name_{}", id)), egui::Sense::click_and_drag());
                painter.text(name_rect.left_center(), egui::Align2::LEFT_CENTER, name, egui::FontId::proportional(14.0), Color32::WHITE);
                
                if name_resp.clicked() { 
                    let modifiers = ui.input(|inp| inp.modifiers);
                    if modifiers.ctrl || modifiers.command {
                        if is_selected { app.ui.selected_layer_ids.retain(|x| x != id); } 
                        else { app.ui.selected_layer_ids.push(id.to_string()); }
                    } else {
                        app.ui.selected_layer_ids = vec![id.to_string()];
                    }
                    app.engine.set_active_layer(id.to_string());
                    app.ui.last_clicked_layer_id = Some(id.to_string());
                }
                
                if name_resp.drag_started() { app.ui.dragging_layer_id = Some(id.to_string()); }
                if name_resp.double_clicked() { app.ui.renaming_layer_id = Some(id.to_string()); app.ui.renaming_buffer = name.to_string(); }

                name_resp.context_menu(|ui| {
                    if ui.button(t!("layer.copy_layer")).clicked() { app.enqueue_command(AppCommand::DuplicateLayer(id.to_string())); ui.close_menu(); }
                    if ui.button(t!("layer.merge_selected")).clicked() { app.enqueue_command(AppCommand::MergeSelected(app.ui.selected_layer_ids.clone())); ui.close_menu(); }
                    ui.separator();
                    if ui.button(t!("layer.delete_layer")).clicked() { app.engine.set_active_layer(id.to_string()); app.delete_active_layer(); ui.close_menu(); }
                });
            }

            if let Some(drag_id) = &app.ui.dragging_layer_id {
                if drag_id != id {
                    if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                        if rect.contains(pos) && app.ui.drag_target_bone_id.is_none() {
                            let is_top = pos.y < rect.center().y;
                            let line_y = if is_top { rect.top() } else { rect.bottom() };
                            painter.hline(rect.left()..=rect.right(), line_y, egui::Stroke::new(2.0, Color32::LIGHT_BLUE));
                            app.ui.drop_target_idx = Some(if is_top { *idx + 1 } else { *idx });
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
        bone_id: &str,
        depth: usize,
        bone_children: &std::collections::HashMap<String, Vec<String>>,
        bone_slots: &std::collections::HashMap<String, Vec<String>>,
        layer_metas: &std::collections::HashMap<String, (usize, String, bool, bool)>
    ) {
        let bone_name = app.animation.project.skeleton.bones.iter().find(|b| b.data.id == bone_id).unwrap().data.name.clone();
        let indent = depth as f32 * 16.0;
        let row_height = 26.0;

        let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), row_height), egui::Sense::click());
        
        let mut is_drop_target = false;
        if app.ui.dragging_layer_id.is_some() {
            if ui.rect_contains_pointer(rect) {
                is_drop_target = true;
                app.ui.drag_target_bone_id = Some(bone_id.to_string());
            }
        };

        let bg_color = if is_drop_target { Color32::from_rgb(80, 80, 40) }
                       else if app.ui.selected_bone_id.as_deref() == Some(bone_id) { Color32::from_rgb(60, 60, 80) }
                       else { Color32::TRANSPARENT };
        
        ui.painter().rect_filled(rect, 0.0, bg_color);
        
        let is_expanded = app.ui.expanded_bones.contains(bone_id);
        let icon = if is_expanded { "▼" } else { "▶" };
        let text_pos = rect.min + egui::vec2(indent + 4.0, 6.0);
        ui.painter().text(text_pos, egui::Align2::LEFT_TOP, format!("{} 🦴 {}", icon, bone_name), egui::FontId::proportional(14.0), Color32::WHITE);
        
        if resp.clicked() {
            app.ui.selected_bone_id = Some(bone_id.to_string());
            if is_expanded { app.ui.expanded_bones.remove(bone_id); }
            else { app.ui.expanded_bones.insert(bone_id.to_string()); }
        }
        
        resp.context_menu(|ui| {
            if ui.button("➕ 新建子骨骼").clicked() {
                app.set_tool(crate::app::state::ToolType::CreateBone);
                app.ui.selected_bone_id = Some(bone_id.to_string());
                ui.close_menu();
            }
            if ui.button("🗑 删除骨骼").clicked() {
                app.enqueue_command(AppCommand::DeleteBone(bone_id.to_string()));
                ui.close_menu();
            }
        });

        if is_expanded {
            if let Some(layers) = bone_slots.get(bone_id) {
                let mut sorted = layers.clone();
                sorted.sort_by_key(|lid| std::cmp::Reverse(layer_metas.get(lid).map(|m| m.0).unwrap_or(0)));
                for lid in sorted {
                    if let Some(meta) = layer_metas.get(&lid) {
                        Self::draw_layer_node(ui, app, &lid, meta, depth + 1);
                    }
                }

            }
            if let Some(children) = bone_children.get(bone_id) {
                for child_id in children {
                    Self::draw_bone_tree(ui, app, child_id, depth + 1, bone_children, bone_slots, layer_metas);
                }
            }
        }
    }
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        
            egui::TopBottomPanel::bottom("layer_bottom_panel")
                .resizable(false)
                .frame(egui::Frame::none())
                .show_inside(ui, |ui| {
                    SymmetryPanel::show(ui, app);
                    
                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        if ui.button(RichText::new(ICON_ADD).size(16.0)).on_hover_text(t!("layer.new")).clicked() { app.add_new_layer(); }
                        if ui.button(RichText::new(ICON_DELETE).size(16.0)).on_hover_text(t!("layer.delete")).clicked() { app.delete_active_layer(); }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(RichText::new(ICON_MERGE).size(16.0)).on_hover_text(t!("layer.merge")).clicked() {
                                app.enqueue_command(AppCommand::MergeSelected(app.ui.selected_layer_ids.clone())); 
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

                let active_id: Option<String> = app.engine.store().active_layer_id.clone();
                let mut active_opacity = 255;
                let mut active_blend = BlendMode::Normal;
                let has_active = if let Some(id) = &active_id {
                    if let Some(layer) = app.engine.store().get_layer(id) {
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
                            if let Some(id) = &active_id { app.enqueue_command(AppCommand::SetLayerBlendMode(id.clone(), new_blend)); } 
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
                                if let Some(id) = &active_id { app.enqueue_command(AppCommand::SetLayerOpacity(id.clone(), new_op)); }
                            }
                        });
                    });
                }); 
                
                ui.add_space(4.0);
                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        app.ui.drop_target_idx = None;
                        app.ui.drag_target_bone_id = None;

                        let mut layer_metas = std::collections::HashMap::new();
                        for (i, l) in app.engine.store().layers.iter().enumerate() {
                            layer_metas.insert(l.id.clone(), (i, l.name.clone(), l.visible, l.locked));
                        }

                        let current_active = active_id.clone();
                        if !app.ui.selected_layer_ids.contains(&current_active.clone().unwrap_or_default()) {
                            if let Some(id) = &current_active { app.ui.selected_layer_ids.push(id.clone()); }
                        }

                        let mut bone_children: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
                        let mut root_bones = Vec::new();
                        for bone in &app.animation.project.skeleton.bones {
                            if let Some(pid) = &bone.data.parent_id { bone_children.entry(pid.clone()).or_default().push(bone.data.id.clone()); } 
                            else { root_bones.push(bone.data.id.clone()); }
                        }

                        let mut bone_slots: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
                        let mut bound_layers = std::collections::HashSet::new();
                        for slot in &app.animation.project.skeleton.slots {
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
                            Self::draw_bone_tree(ui, app, root_id, 0, &bone_children, &bone_slots, &layer_metas);
                        }
                        
                        if ui.input(|i| i.pointer.any_released()) {
                            if let Some(drag_id) = app.ui.dragging_layer_id.take() {
                                if let Some(target_bone) = app.ui.drag_target_bone_id.take() {
                                    if app.ui.selected_layer_ids.contains(&drag_id) {
                                        let selected = app.ui.selected_layer_ids.clone();
                                        for sel_id in selected {
                                            app.enqueue_command(AppCommand::BindLayerToBone(sel_id, target_bone.clone()));
                                        }
                                    } else { app.enqueue_command(AppCommand::BindLayerToBone(drag_id, target_bone)); }
                                } else if let Some(target_idx) = app.ui.drop_target_idx.take() {
                                    app.enqueue_command(AppCommand::MoveLayerToIndex(drag_id, target_idx));
                                }
                            }
                        }
                    });
            });
    }
}