use egui::{Ui, Color32};
use crate::app::state::AppState;
use crate::app::commands::*;
use crate::core::animation::timeline::TimelineProperty;
use rust_i18n::t;

pub struct Toolbar;

impl Toolbar {
    pub fn show(ui: &mut Ui, app: &mut AppState, ui_ctx: &mut crate::app::ui_context::UiContext) {
        let active_id = app.anim.state.project.active_animation_id.clone();
        let current_name = if let Some(ref id) = active_id {
            app.anim.state.project.animations.get(id).map(|a| a.name.clone()).unwrap_or_else(|| "Unknown".to_string())
        } else {
            "No Animation".to_string()
        };
        let anim_list: Vec<(String, String)> = app.anim.state.project.animations.iter()
            .map(|(k, v): (&String, &crate::core::animation::timeline::Animation)| (k.clone(), v.name.clone())).collect();
        let anim_count = app.anim.state.project.animations.len();

        ui.horizontal(|ui| {
            ui.label("动画:");
            egui::ComboBox::from_id_source("anim_selector")
                .selected_text(current_name)
                .width(150.0)
                .show_ui(ui, |ui| {
                    for (id, name) in anim_list {
                        if ui.selectable_label(Some(&id) == active_id.as_ref(), &name).clicked() {
                            app.enqueue_command(Box::new(SelectAnimationCmd(id)));
                        }
                    }
                });

            if ui.button("➕ 新建动画").clicked() {
                ui_ctx.new_anim_name = format!("anim_{}", anim_count + 1);
                ui_ctx.show_new_anim_modal = true;
            }
        });
        
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("当前:");
            let mut current_frame = (app.anim.state.current_time * app.anim.fps).round() as i32;
            if ui.add(egui::DragValue::new(&mut current_frame).speed(1.0).clamp_range(0..=300).prefix("    ")).changed() {
                app.anim.state.current_time = (current_frame as f32 / app.anim.fps).max(0.0);
                crate::animation::controller::AnimationController::apply_current_pose(&mut app.anim.state);
            }
            
            ui.separator();

            ui.label("FPS:");
            ui.add(egui::DragValue::new(&mut app.anim.fps).speed(1.0).clamp_range(12.0..=120.0));

            ui.separator();

            let auto_key_color = if ui_ctx.auto_keyframe { Color32::from_rgb(255, 60, 60) } else { Color32::GRAY };
            ui.toggle_value(&mut ui_ctx.auto_keyframe, egui::RichText::new("🔑 自动关键帧").color(auto_key_color));
             
            ui.separator();
            
            let duration = active_id.as_ref().and_then(|id| app.anim.state.project.animations.get(id)).map(|a| a.duration).unwrap_or(0.0);

            if ui.button("⏮").on_hover_text("回到首帧").clicked() { app.enqueue_command(Box::new(SetTimeCmd(0.0))); }
            if ui.button("⏪").on_hover_text("上一帧").clicked() { app.enqueue_command(Box::new(StepFrameCmd(-1))); }
            if ui.button(if app.anim.state.is_playing { "⏸ 暂停" } else { "▶ 播放" }).clicked() { app.enqueue_command(Box::new(TogglePlaybackCmd)); }
            if ui.button("⏩").on_hover_text("下一帧").clicked() { app.enqueue_command(Box::new(StepFrameCmd(1))); }
            if ui.button("⏭").on_hover_text("末帧").clicked() { app.enqueue_command(Box::new(SetTimeCmd(duration))); }
            
            let mut looping = app.anim.state.is_looping;
            if ui.toggle_value(&mut looping, "🔁").on_hover_text("循环播放").clicked() { app.enqueue_command(Box::new(ToggleLoopCmd)); }

            let mut speed = app.anim.state.playback_speed;
            if ui.add(egui::Slider::new(&mut speed, 0.1..=5.0).text("x Speed")).changed() {
                app.enqueue_command(Box::new(SetPlaybackSpeedCmd(speed)));
            }

            ui.separator();
            
            ui.menu_button("🔽 显示筛选", |ui| {
                let props = vec![
                    (TimelineProperty::Translation, "✥ 移动"),
                    (TimelineProperty::Rotation, "⟳ 旋转"),
                    (TimelineProperty::Scale, "◱ 缩放")
                ];
                for (prop, label) in props {
                    let mut is_active = ui_ctx.timeline_filter.contains(&prop);
                    if ui.checkbox(&mut is_active, label).clicked() {
                        app.enqueue_command(Box::new(ToggleTimelineFilterCmd(prop)));
                    }
                }
            });
            
            ui.toggle_value(&mut ui_ctx.show_curve_editor, "📈 曲线");
            ui.toggle_value(&mut ui_ctx.is_offset_mode_active, "➡️ 偏移模式")
                .on_hover_text("激活后拖拽帧进行循环偏移 (或使用快捷键: Ctrl+Alt+拖拽)");
                
            if ui.button("⚙ 偏移参数").clicked() {
                ui_ctx.show_offset_modal = true;
            }

            ui.separator();
            
            if let Some(bone_id) = ui_ctx.selected_bone_id.clone() {
                if ui.button("手动 K 帧 (所有)").clicked() {
                    app.anim.state.auto_key_bone(&bone_id, TimelineProperty::Rotation);
                    app.anim.state.auto_key_bone(&bone_id, TimelineProperty::Translation);
                    app.anim.state.auto_key_bone(&bone_id, TimelineProperty::Scale);
                }
            } else {
                ui.label(t!("anim.select_bone_to_keyframe").to_string());
            }
        });

        if ui_ctx.show_new_anim_modal {
            egui::Window::new("新建动画")
                .collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("名称:");
                        ui.text_edit_singleline(&mut ui_ctx.new_anim_name);
                    });
                    ui.horizontal(|ui| {
                        if ui.button(t!("dialog.confirm")).clicked() {
                            app.enqueue_command(Box::new(CreateAnimationCmd(ui_ctx.new_anim_name.clone())));
                            ui_ctx.show_new_anim_modal = false;
                        }
                        if ui.button(t!("dialog.cancel")).clicked() { ui_ctx.show_new_anim_modal = false; }
                    });
                });
        }
    }
}