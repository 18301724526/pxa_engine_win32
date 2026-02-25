use egui::{Ui, Color32, Sense, Stroke, Pos2, vec2, Align2, FontId};
use crate::app::state::AppState;
use crate::core::animation::timeline::{TimelineProperty, KeyframeValue, CurveType};
use rust_i18n::t;

pub struct TimelinePanel;

impl TimelinePanel {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        let anim_state = &mut app.animation;
        let mut new_seek_time = None;
        
        ui.horizontal(|ui| {
            if ui.button(if anim_state.is_playing { "⏸" } else { "▶" }).clicked() {
                anim_state.is_playing = !anim_state.is_playing;
            }
            ui.label(t!("anim.time").to_string());
            ui.add(egui::DragValue::new(&mut anim_state.current_time).speed(0.01).clamp_range(0.0..=10.0));
            
            ui.separator();
            
            if let Some(bone_id) = &app.ui.selected_bone_id {
                if ui.button(t!("anim.key_rotation")).clicked() {
                    let current_time = anim_state.current_time;
                    let project = &mut anim_state.project;
                    if let Some(active_anim_id) = &project.active_animation_id {
                        if let Some(anim) = project.animations.get_mut(active_anim_id) {
                            let timeline_exists = anim.timelines.iter().any(|t| t.target_id == *bone_id && t.property == TimelineProperty::Rotation);
                            if !timeline_exists {
                                anim.timelines.push(crate::core::animation::timeline::Timeline::new(bone_id.clone(), TimelineProperty::Rotation));
                            }
                            let current_rot = project.skeleton.bones.iter()
                                .find(|b| b.data.id == *bone_id)
                                .map(|b| b.local_transform.rotation)
                                .unwrap_or(0.0);

                            if let Some(timeline) = anim.timelines.iter_mut().find(|t| t.target_id == *bone_id && t.property == TimelineProperty::Rotation) {
                                timeline.add_keyframe(current_time, KeyframeValue::Rotate(current_rot), CurveType::Linear);
                            }
                        }
                    }
                }
            } else {
                ui.label(t!("anim.select_bone_to_keyframe").to_string());
            }
        });

        ui.separator();

        let row_height = 20.0;
        let pixels_per_second = 100.0;
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Some(active_anim_id) = &anim_state.project.active_animation_id {
                if let Some(_anim) = anim_state.project.animations.get(active_anim_id) {
                    let (rect, response) = ui.allocate_at_least(vec2(ui.available_width(), 30.0), Sense::click_and_drag());
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 0.0, Color32::from_gray(40));
                    
                    for i in 0..=10 {
                        let x = rect.min.x + i as f32 * pixels_per_second;
                        painter.line_segment([Pos2::new(x, rect.min.y + 15.0), Pos2::new(x, rect.max.y)], Stroke::new(1.0, Color32::GRAY));
                        painter.text(Pos2::new(x + 2.0, rect.min.y), Align2::LEFT_TOP, format!("{}s", i), FontId::monospace(10.0), Color32::GRAY);
                    }

                    if response.dragged() || response.clicked() {
                        if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                            let new_time = ((pointer_pos.x - rect.min.x) / pixels_per_second).max(0.0);
                            new_seek_time = Some(new_time);
                        }
                    }

                    let playhead_x = rect.min.x + anim_state.current_time * pixels_per_second;
                    painter.line_segment([Pos2::new(playhead_x, rect.min.y), Pos2::new(playhead_x, rect.max.y + 200.0)], Stroke::new(2.0, Color32::RED));

                    ui.separator();

                    for bone in &anim_state.project.skeleton.bones {
                        ui.horizontal(|ui| {
                            ui.set_min_height(row_height);
                            let is_selected = app.ui.selected_bone_id.as_ref() == Some(&bone.data.id);
                            if ui.selectable_label(is_selected, &bone.data.name).clicked() {
                                app.ui.selected_bone_id = Some(bone.data.id.clone());
                            }
                        });
                    }
                }
            } else {
                ui.label("No active animation.");
            }
        });
        if let Some(t) = new_seek_time {
            anim_state.current_time = t;
            crate::animation::controller::AnimationController::apply_current_pose(anim_state);
        }
    }
}