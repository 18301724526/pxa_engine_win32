use egui::{Ui, Color32, Sense, Stroke, Pos2, vec2, Align2, FontId};
use crate::app::state::AppState;
use crate::core::animation::timeline::TimelineProperty;
use crate::app::commands::AppCommand;

pub struct Dopesheet;

impl Dopesheet {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        let active_id = app.animation.project.active_animation_id.clone();
        let mut new_seek_time = None;

        let row_height = 20.0;
        let fps = 30.0;
        let frame_width = 10.0 * app.ui.timeline_zoom;
        let current_anim_duration = app.animation.project.animations.get(active_id.as_ref().unwrap_or(&"".to_string())).map(|a| a.duration).unwrap_or(0.0);
        let max_frames = ((current_anim_duration * fps) as i32 + 60).max(300);
        
        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            if active_id.is_some() && app.animation.project.animations.contains_key(active_id.as_ref().unwrap()) {
                let left_width = 150.0;
                let right_width = (ui.available_width() - left_width).max(0.0);
                let mut track_min_x = 0.0;
                let mut header_rect = egui::Rect::NOTHING;
                let mut content_width = 0.0;
                let mut rendered_kfs: Vec<(String, Option<TimelineProperty>, f32, egui::Rect)> = Vec::new();

                ui.horizontal(|ui| {
                    ui.allocate_exact_size(vec2(left_width, 30.0), Sense::hover());
                    content_width = (max_frames as f32 * frame_width).max(right_width);
                    let (rect, response) = ui.allocate_exact_size(vec2(content_width, 30.0), Sense::click_and_drag());
                    header_rect = rect;
                    track_min_x = rect.min.x;

                    if response.hovered() {
                        let scroll = ui.input(|i| i.scroll_delta.y);
                        if scroll != 0.0 {
                            app.ui.timeline_zoom = (app.ui.timeline_zoom * (1.0 + scroll * 0.005)).clamp(0.2, 5.0);
                        }
                    }

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

                let bg_rect = egui::Rect::from_min_size(Pos2::new(track_min_x, header_rect.max.y), vec2(content_width, 1000.0));
                println!("[DEBUG-DOPESHEET] bg_rect allocated at: {:?}", bg_rect);
                let overlay_id = ui.id().with("box_select_overlay");
                let overlay_resp = ui.interact(bg_rect, overlay_id, Sense::drag());
                
                if overlay_resp.drag_started() {
                    println!("[DEBUG-DOPESHEET] overlay_resp.drag_started() = true!");
                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                        println!("[DEBUG-DOPESHEET] Captured drag start pos: {:?}", pos);
                        app.ui.box_select_start = Some(pos);
                        if !ui.input(|i| i.modifiers.shift || i.modifiers.ctrl) { 
                            app.ui.selected_keyframes.clear(); 
                        }
                    }
                }

                let bones_list: Vec<(String, String)> = app.animation.project.skeleton.bones.iter()
                    .map(|b| (b.data.id.clone(), b.data.name.clone())).collect();

                let mut bone_all_times: std::collections::HashMap<String, Vec<f32>> = std::collections::HashMap::new();
                let mut bone_prop_times: std::collections::HashMap<(String, TimelineProperty), Vec<f32>> = std::collections::HashMap::new();
                if let Some(anim) = app.animation.project.animations.get(active_id.as_ref().unwrap()) {
                    for timeline in &anim.timelines {
                        let kfs_all = bone_all_times.entry(timeline.target_id.clone()).or_default();
                        let kfs_prop = bone_prop_times.entry((timeline.target_id.clone(), timeline.property.clone())).or_default();
                        for kf in &timeline.keyframes { 
                            kfs_all.push(kf.time); kfs_prop.push(kf.time);
                        }
                    }
                }

                for (bone_id, bone_name) in bones_list {
                    let is_expanded = app.ui.expanded_timeline_bones.contains(&bone_id);
                    let mut tracks_to_draw = vec![
                        (None, bone_name.clone(), Color32::WHITE, 0.0, bone_all_times.get(&bone_id))
                    ];
                    
                    if is_expanded {
                        tracks_to_draw.push((Some(TimelineProperty::Rotation), "⟳ 旋转".to_string(), Color32::GREEN, 20.0, bone_prop_times.get(&(bone_id.clone(), TimelineProperty::Rotation))));
                        tracks_to_draw.push((Some(TimelineProperty::Translation), "✥ 移动".to_string(), Color32::from_rgb(0, 150, 255), 20.0, bone_prop_times.get(&(bone_id.clone(), TimelineProperty::Translation))));
                        tracks_to_draw.push((Some(TimelineProperty::Scale), "◱ 缩放".to_string(), Color32::RED, 20.0, bone_prop_times.get(&(bone_id.clone(), TimelineProperty::Scale))));
                    }

                    for (prop_opt, label_text, kf_color, indent, times) in tracks_to_draw {
                        ui.horizontal(|ui| {
                            let (label_rect, label_resp) = ui.allocate_exact_size(vec2(left_width, row_height), Sense::click());
                            let is_selected = app.ui.selected_bone_id.as_ref() == Some(&bone_id) && prop_opt.is_none();
                            let bg_color = if is_selected { Color32::from_rgb(60, 60, 80) } else { Color32::TRANSPARENT };
                            ui.painter().rect_filled(label_rect, 0.0, bg_color);

                            if prop_opt.is_none() {
                                let icon = if is_expanded { "▼" } else { "▶" };
                                let icon_rect = egui::Rect::from_min_size(label_rect.min, vec2(20.0, row_height));
                                let icon_resp = ui.interact(icon_rect, ui.id().with(format!("exp_{}", bone_id)), Sense::click());
                                if icon_resp.clicked() {
                                    if is_expanded { app.ui.expanded_timeline_bones.remove(&bone_id); }
                                    else { app.ui.expanded_timeline_bones.insert(bone_id.clone()); }
                                }
                                ui.painter().text(icon_rect.center(), Align2::CENTER_CENTER, icon, FontId::proportional(12.0), Color32::LIGHT_GRAY);
                                ui.painter().text(label_rect.left_center() + vec2(20.0, 0.0), Align2::LEFT_CENTER, &label_text, FontId::proportional(14.0), Color32::WHITE);
                                if label_resp.clicked() && !icon_resp.hovered() { app.ui.selected_bone_id = Some(bone_id.clone()); }
                            } else {
                                ui.painter().text(label_rect.left_center() + vec2(indent, 0.0), Align2::LEFT_CENTER, &label_text, FontId::proportional(12.0), Color32::LIGHT_GRAY);
                            }

                            let (track_rect, _) = ui.allocate_exact_size(vec2(right_width, row_height), Sense::hover());
                            ui.painter().rect_filled(track_rect, 0.0, Color32::from_gray(30));
                            
                            if let Some(times) = times {
                                let mut unique_times = times.clone();
                                unique_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
                                unique_times.dedup();

                                for i in 0..unique_times.len() {
                                    let t = unique_times[i];
                                    let kx = track_rect.min.x + (t * fps) * frame_width;
                                    let cy = track_rect.center().y;
                                    let size = 5.0;

                                    let kf_rect = egui::Rect::from_center_size(Pos2::new(kx, cy), vec2(size * 2.5, size * 2.5));
                                    let id_suffix = prop_opt.as_ref().map(|p| format!("{:?}", p)).unwrap_or_else(|| "main".to_string());
                                    let resp = ui.interact(kf_rect, ui.id().with(format!("kf_{}_{}_{}", bone_id, id_suffix, i)), Sense::click_and_drag());
                                    
                                    let is_selected = app.ui.selected_keyframes.iter().any(|k| k.0 == bone_id && k.1 == prop_opt && (k.2 - t).abs() < 0.001);
                                    rendered_kfs.push((bone_id.clone(), prop_opt.clone(), t, kf_rect));

                                    if resp.clicked() {
                                        let mods = ui.input(|i| i.modifiers);
                                        if mods.shift || mods.ctrl {
                                            if is_selected { app.ui.selected_keyframes.retain(|k| !(k.0 == bone_id && k.1 == prop_opt && (k.2 - t).abs() < 0.001)); } 
                                            else { app.ui.selected_keyframes.push((bone_id.clone(), prop_opt.clone(), t)); }
                                        } else {
                                            app.ui.selected_keyframes = vec![(bone_id.clone(), prop_opt.clone(), t)];
                                        }
                                    }
                                    if resp.drag_started() && !is_selected {
                                        app.ui.selected_keyframes = vec![(bone_id.clone(), prop_opt.clone(), t)];
                                    }

                                    if resp.dragged() {
                                        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                                            let new_frame = ((pos.x - track_rect.min.x) / frame_width).round().max(0.0);
                                            let new_time = new_frame / fps;
                                            let dt = new_time - t;
                                            if dt.abs() >= (1.0 / fps) * 0.5 {
                                                app.enqueue_command(AppCommand::MoveSelectedKeyframes(dt));
                                            }
                                        }
                                    }

                                    resp.context_menu(|ui| {
                                        if ui.button("🗑 删除关键帧").clicked() {
                                            app.enqueue_command(AppCommand::DeleteKeyframe(bone_id.clone(), prop_opt.clone(), t));
                                            ui.close_menu();
                                        }
                                    });

                                    let fill_color = if is_selected { Color32::WHITE } else { kf_color };
                                    let stroke_col = if is_selected { Color32::BLACK } else { Color32::WHITE };
                                    
                                    let points = vec![
                                        Pos2::new(kx, cy - size),
                                        Pos2::new(kx + size, cy),
                                        Pos2::new(kx, cy + size),
                                        Pos2::new(kx - size, cy),
                                    ];
                                    ui.painter().add(egui::Shape::convex_polygon(points, fill_color, Stroke::new(1.0, stroke_col)));
                                }
                            }
                        });
                    }
                }

                println!("[DEBUG-DOPESHEET] Total rendered_kfs this frame: {}", rendered_kfs.len());
                for (id, _, _, rect) in &rendered_kfs {
                    println!("[DEBUG-DOPESHEET] KF {} rendered at rect: {:?}", id, rect);
                }

                if overlay_resp.dragged() {
                    if let Some(start_pos) = app.ui.box_select_start {
                        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                            let box_rect = egui::Rect::from_two_pos(start_pos, pos);
                            println!("[DEBUG-DOPESHEET] Dragging... current box_rect: {:?}", box_rect);
                            ui.painter().rect(box_rect, 0.0, Color32::from_white_alpha(20), Stroke::new(1.0, Color32::WHITE));
                        }
                    }
                }

                if ui.input(|i| i.pointer.any_released()) {
                    println!("[DEBUG-DOPESHEET] Pointer released! box_select_start is: {:?}", app.ui.box_select_start);
                }

                if ui.input(|i| i.pointer.any_released()) && app.ui.box_select_start.is_some() {
                    if let Some(start_pos) = app.ui.box_select_start {
                        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                            let box_rect = egui::Rect::from_two_pos(start_pos, pos);
                            println!("[DEBUG-DOPESHEET] Releasing! Final box_rect: {:?}", box_rect);
                            for (b_id, p_opt, t, k_rect) in &rendered_kfs {
                                let intersects = box_rect.intersects(*k_rect);
                                println!("[DEBUG-DOPESHEET] Check intersection with KF {} at {:?} -> {}", b_id, k_rect, intersects);
                                if intersects {                                   
                                    if !app.ui.selected_keyframes.iter().any(|k| k.0 == *b_id && k.1 == *p_opt && (k.2 - t).abs() < 0.001) {
                                        app.ui.selected_keyframes.push((b_id.clone(), p_opt.clone(), *t));
                                    }
                                }
                            }
                        }
                    }
                    app.ui.box_select_start = None;
                }

                let playhead_x = track_min_x + (app.animation.current_time * fps) * frame_width;
                let playhead_start = Pos2::new(playhead_x, header_rect.min.y);
                let playhead_end = Pos2::new(playhead_x, ui.min_rect().max.y);
                ui.painter().line_segment([playhead_start, playhead_end], Stroke::new(2.0, Color32::RED));
            } else {
                ui.label("No active animation.");
            }
        });

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
    }
}