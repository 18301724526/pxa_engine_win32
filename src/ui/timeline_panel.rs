use egui::{Ui, Color32, Sense, Stroke, Pos2, vec2, Align2, FontId};
use crate::app::state::AppState;
use crate::core::animation::timeline::{TimelineProperty, KeyframeValue, CurveType};
use crate::app::commands::AppCommand;
use rust_i18n::t;

pub struct TimelinePanel;

impl TimelinePanel {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        let mut drag_action: Option<(String, f32, f32)> = None;
        let mut new_seek_time = None;

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
            
            if let Some(bone_id) = app.ui.selected_bone_id.clone() {
                if ui.button("手动 K 帧 (所有)").clicked() {
                    let old_auto = app.animation.auto_key_enabled;
                    app.animation.auto_key_enabled = true; // 强制打帧
                    app.animation.auto_key_bone(&bone_id, TimelineProperty::Rotation);
                    app.animation.auto_key_bone(&bone_id, TimelineProperty::Translation);
                    app.animation.auto_key_bone(&bone_id, TimelineProperty::Scale);
                    app.animation.auto_key_enabled = old_auto;
                }
            } else {
                ui.label(t!("anim.select_bone_to_keyframe").to_string());
            }
        });

        ui.separator();

        let row_height = 20.0;
        let fps = 30.0;
        let frame_width = 10.0;
        let max_frames = 300;
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            if active_id.is_some() && app.animation.project.animations.contains_key(active_id.as_ref().unwrap()) {
                let left_width = 150.0;
                let right_width = (ui.available_width() - left_width).max(0.0);
                let mut track_min_x = 0.0;
                let mut header_rect = egui::Rect::NOTHING;

                ui.horizontal(|ui| {
                    ui.allocate_exact_size(vec2(left_width, 30.0), Sense::hover());
                    let (rect, response) = ui.allocate_exact_size(vec2(right_width, 30.0), Sense::click_and_drag());
                    header_rect = rect;
                    track_min_x = rect.min.x;
                    
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 0.0, Color32::from_gray(40));
                    
                    for i in 0..=max_frames {
                        let x = rect.min.x + i as f32 * frame_width;
                        if i % 30 == 0 {
                            painter.line_segment([Pos2::new(x, rect.min.y + 10.0), Pos2::new(x, rect.max.y)], Stroke::new(1.5, Color32::GRAY));
                            painter.text(Pos2::new(x + 2.0, rect.min.y), Align2::LEFT_TOP, format!("{}s", i / 30), FontId::monospace(10.0), Color32::GRAY);
                        } else if i % 5 == 0 {
                            painter.line_segment([Pos2::new(x, rect.min.y + 18.0), Pos2::new(x, rect.max.y)], Stroke::new(1.0, Color32::from_gray(100)));
                            painter.text(Pos2::new(x + 2.0, rect.min.y + 10.0), Align2::LEFT_TOP, format!("{}", i), FontId::proportional(8.0), Color32::from_gray(100));
                        } else {
                            painter.line_segment([Pos2::new(x, rect.min.y + 24.0), Pos2::new(x, rect.max.y)], Stroke::new(1.0, Color32::from_gray(60)));
                        }
                    }

                    if response.dragged() || response.clicked() {
                        if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                            let new_frame = ((pointer_pos.x - rect.min.x) / frame_width).round().max(0.0);
                            new_seek_time = Some(new_frame / fps);
                        }
                    }
                });
                let bones_list: Vec<(String, String)> = app.animation.project.skeleton.bones.iter()
                    .map(|b| (b.data.id.clone(), b.data.name.clone()))
                    .collect();

                let mut bone_keyframes: std::collections::HashMap<String, Vec<f32>> = std::collections::HashMap::new();
                if let Some(anim) = app.animation.project.animations.get(active_id.as_ref().unwrap()) {
                    for timeline in &anim.timelines {
                        let kfs = bone_keyframes.entry(timeline.target_id.clone()).or_default();
                        for kf in &timeline.keyframes { kfs.push(kf.time); }
                    }
                }

                for (bone_id, bone_name) in bones_list {
                    ui.horizontal(|ui| {
                        let (label_rect, label_resp) = ui.allocate_exact_size(vec2(left_width, row_height), Sense::click());
                        let is_selected = app.ui.selected_bone_id.as_ref() == Some(&bone_id);
                        let bg_color = if is_selected { Color32::from_rgb(60, 60, 80) } else { Color32::TRANSPARENT };
                        ui.painter().rect_filled(label_rect, 0.0, bg_color);
                        ui.painter().text(label_rect.left_center() + vec2(5.0, 0.0), Align2::LEFT_CENTER, &bone_name, FontId::proportional(14.0), Color32::WHITE);
                        
                        if label_resp.clicked() { app.ui.selected_bone_id = Some(bone_id.clone()); }

                        let (track_rect, _) = ui.allocate_exact_size(vec2(right_width, row_height), Sense::hover());
                        ui.painter().rect_filled(track_rect, 0.0, Color32::from_gray(30));
                        ui.painter().hline(track_rect.x_range(), track_rect.center().y, Stroke::new(1.0, Color32::from_gray(50)));
                        
                        if let Some(times) = bone_keyframes.get(&bone_id) {
                            let mut unique_times = times.clone();
                            unique_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
                            unique_times.dedup();

                            for &t in &unique_times {
                                let kx = track_rect.min.x + (t * fps) * frame_width;
                                let cy = track_rect.center().y;
                                let size = 5.0;

                                let kf_rect = egui::Rect::from_center_size(Pos2::new(kx, cy), vec2(size * 2.5, size * 2.5));
                                let resp = ui.interact(kf_rect, ui.id().with(format!("kf_{}_{}", bone_id, t)), Sense::click_and_drag());
                                
                                if resp.clicked() || resp.drag_started() {
                                    app.ui.selected_keyframe = Some((bone_id.clone(), t));
                                }

                                if resp.dragged() {
                                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                                        let new_frame = ((pos.x - track_rect.min.x) / frame_width).round().max(0.0);
                                        let new_time = new_frame / fps;
                                        if (new_time - t).abs() > 0.001 {
                                            drag_action = Some((bone_id.clone(), t, new_time));
                                            app.ui.selected_keyframe = Some((bone_id.clone(), new_time));
                                        }
                                    }
                                }

                                let is_selected = app.ui.selected_keyframe.as_ref() == Some(&(bone_id.clone(), t));
                                let fill_color = if is_selected { Color32::WHITE } else { Color32::from_rgb(0, 200, 255) };
                                
                                let points = vec![
                                    Pos2::new(kx, cy - size),
                                    Pos2::new(kx + size, cy),
                                    Pos2::new(kx, cy + size),
                                    Pos2::new(kx - size, cy),
                                ];
                                ui.painter().add(egui::Shape::convex_polygon(points, fill_color, Stroke::new(1.0, Color32::WHITE)));
                            }
                        }
                    });
                }

                let playhead_x = track_min_x + (app.animation.current_time * fps) * frame_width;
                let playhead_start = Pos2::new(playhead_x, header_rect.min.y);
                let playhead_end = Pos2::new(playhead_x, ui.min_rect().max.y);
                ui.painter().line_segment([playhead_start, playhead_end], Stroke::new(2.0, Color32::RED));
            } else {
                ui.label("No active animation.");
            }
        });

        if let Some((bone_id, old_time, new_time)) = drag_action {
            if let Some(active_id) = &app.animation.project.active_animation_id {
                if let Some(anim) = app.animation.project.animations.get_mut(active_id) {
                    for tl in &mut anim.timelines {
                        if tl.target_id == bone_id {
                            if let Some(kf) = tl.keyframes.iter_mut().find(|k| (k.time - old_time).abs() < 0.001) {
                                kf.time = new_time;
                            }
                            tl.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
                        }
                    }
                }
            }
            app.is_dirty = true;
            app.animation.current_time = new_time; 
            crate::animation::controller::AnimationController::apply_current_pose(&mut app.animation);
        }

        if let Some(t) = new_seek_time {
            app.animation.current_time = t;
            crate::animation::controller::AnimationController::apply_current_pose(&mut app.animation);
        }

        if app.animation.is_playing && app.animation.is_looping {
            if let Some(active_id) = &app.animation.project.active_animation_id {
                if let Some(anim) = app.animation.project.animations.get(active_id) {
                    if anim.duration > 0.0 && app.animation.current_time > anim.duration {
                        app.animation.current_time %= anim.duration;
                    }
                }
            }
        }

        if app.ui.show_new_anim_modal {
            egui::Window::new("新建动画")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
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