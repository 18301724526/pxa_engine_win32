use egui::{Ui, Color32};
use crate::app::state::AppState;
use crate::app::commands::AppCommand;
use crate::core::animation::timeline::TimelineProperty;
use rust_i18n::t;

pub struct Toolbar;

impl Toolbar {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        let active_id = app.animation.project.active_animation_id.clone();
        let current_name = if let Some(ref id) = active_id {
            app.animation.project.animations.get(id).map(|a| a.name.clone()).unwrap_or_else(|| "Unknown".to_string())
        } else {
            "No Animation".to_string()
        };
        let anim_list: Vec<(String, String)> = app.animation.project.animations.iter()
            .map(|(k, v)| (k.clone(), v.name.clone())).collect();
        let anim_count = app.animation.project.animations.len();

        ui.horizontal(|ui| {
            ui.label("动画:");
            egui::ComboBox::from_id_source("anim_selector")
                .selected_text(current_name)
                .width(150.0)
                .show_ui(ui, |ui| {
                    for (id, name) in anim_list {
                        if ui.selectable_label(Some(&id) == active_id.as_ref(), &name).clicked() {
                            app.enqueue_command(AppCommand::SelectAnimation(id));
                        }
                    }
                });

            if ui.button("➕ 新建动画").clicked() {
                app.ui.new_anim_name = format!("anim_{}", anim_count + 1);
                app.ui.show_new_anim_modal = true;
            }
        });
        
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("当前:");
            let mut current_frame = (app.animation.current_time * 30.0).round() as i32;
            if ui.add(egui::DragValue::new(&mut current_frame).speed(1.0).clamp_range(0..=300).prefix("    ")).changed() {
                app.animation.current_time = (current_frame as f32 / 30.0).max(0.0);
                crate::animation::controller::AnimationController::apply_current_pose(&mut app.animation);
            }
            
            ui.separator();

            let mut auto_key = app.animation.auto_key_enabled;
            let auto_key_color = if auto_key { Color32::from_rgb(255, 60, 60) } else { Color32::GRAY };
            if ui.toggle_value(&mut auto_key, egui::RichText::new("🔑 自动关键帧").color(auto_key_color)).clicked() {
                app.animation.auto_key_enabled = auto_key;
            }
             
            ui.separator();
            
            if ui.button("⏮").on_hover_text("回到首帧").clicked() { 
                app.animation.current_time = 0.0; 
                crate::animation::controller::AnimationController::apply_current_pose(&mut app.animation); 
            }
            if ui.button("⏪").on_hover_text("上一帧").clicked() { 
                app.animation.current_time = (app.animation.current_time - 1.0/30.0).max(0.0); 
                crate::animation::controller::AnimationController::apply_current_pose(&mut app.animation); 
            }
            if ui.button(if app.animation.is_playing { "⏸ 暂停" } else { "▶ 播放" }).clicked() {
                app.animation.is_playing = !app.animation.is_playing;
            }
            if ui.button("⏩").on_hover_text("下一帧").clicked() { 
                app.animation.current_time += 1.0/30.0; 
                crate::animation::controller::AnimationController::apply_current_pose(&mut app.animation); 
            }
            
            let mut looping = app.animation.is_looping;
            if ui.toggle_value(&mut looping, "🔁").on_hover_text("循环播放").clicked() { 
                app.animation.is_looping = looping; 
            }

            ui.separator();
            
            ui.toggle_value(&mut app.ui.show_curve_editor, "📈 曲线");
            if ui.button("➡️ 自动偏移").clicked() {
                app.ui.show_offset_modal = true;
            }

            ui.separator();
            
            if let Some(bone_id) = app.ui.selected_bone_id.clone() {
                if ui.button("手动 K 帧 (所有)").clicked() {
                    let old_auto = app.animation.auto_key_enabled;
                    app.animation.auto_key_enabled = true;
                    app.animation.auto_key_bone(&bone_id, TimelineProperty::Rotation);
                    app.animation.auto_key_bone(&bone_id, TimelineProperty::Translation);
                    app.animation.auto_key_bone(&bone_id, TimelineProperty::Scale);
                    app.animation.auto_key_enabled = old_auto;
                }
            } else {
                ui.label(t!("anim.select_bone_to_keyframe").to_string());
            }
        });

        if app.ui.show_new_anim_modal {
            egui::Window::new("新建动画")
                .collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("名称:");
                        ui.text_edit_singleline(&mut app.ui.new_anim_name);
                    });
                    ui.horizontal(|ui| {
                        if ui.button(t!("dialog.confirm")).clicked() {
                            app.enqueue_command(AppCommand::CreateAnimation(app.ui.new_anim_name.clone()));
                            app.ui.show_new_anim_modal = false;
                        }
                        if ui.button(t!("dialog.cancel")).clicked() { app.ui.show_new_anim_modal = false; }
                    });
                });
        }
    }
}