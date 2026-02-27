use egui::{Ui, Color32, Sense, Stroke, Pos2, vec2, Rect, Align2, FontId};
use crate::app::state::AppState;
use crate::core::animation::timeline::{TimelineProperty, KeyframeValue, CurveType};
use crate::app::commands::AppCommand;

pub struct CurveEditor;

impl CurveEditor {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        let mut show_curve_editor = app.ui.show_curve_editor;
        if show_curve_editor {
            egui::Window::new("曲线编辑器 (Graph Editor)")
                .open(&mut show_curve_editor)
                .resizable(true)
                .default_size(vec2(800.0, 450.0))
                .show(ui.ctx(), |ui| {
                    let mut pending_commands = Vec::new();
                    let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
                    
                    if response.dragged() && !ui.input(|i| i.pointer.any_down() && i.modifiers.is_none()) {
                        app.ui.graph_pan += response.drag_delta(); 
                    }
                    if response.hovered() {
                        let scroll = ui.input(|i| i.scroll_delta.y);
                        if scroll != 0.0 {
                            app.ui.graph_zoom.x = (app.ui.graph_zoom.x * (1.0 + scroll * 0.005)).clamp(10.0, 1000.0);
                            app.ui.graph_zoom.y = (app.ui.graph_zoom.y * (1.0 + scroll * 0.005)).clamp(0.1, 50.0);
                        }
                    }

                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 0.0, Color32::from_rgb(45, 45, 48));

                    let origin_y = rect.center().y + app.ui.graph_pan.y;
                    let origin_x = rect.min.x + app.ui.graph_pan.x;

                    let val_step = 100.0 * app.ui.graph_zoom.y;
                    let mut y_offset = origin_y % val_step;
                    if y_offset < 0.0 { y_offset += val_step; }
                    let mut current_y = rect.min.y + y_offset;
                    while current_y <= rect.max.y {
                        painter.hline(rect.x_range(), current_y, Stroke::new(1.0, Color32::from_rgb(30, 30, 32)));
                        let val = (origin_y - current_y) / app.ui.graph_zoom.y;
                        painter.text(Pos2::new(rect.max.x - 30.0, current_y - 8.0), Align2::LEFT_TOP, format!("{:.0}", val.round()), FontId::proportional(10.0), Color32::from_gray(140));
                        current_y += val_step;
                    }

                    let fps = 30.0;
                    let frame_step_x = (10.0 / fps) * app.ui.graph_zoom.x;
                    let mut x_offset = origin_x % frame_step_x;
                    if x_offset < 0.0 { x_offset += frame_step_x; }
                    let mut current_x = rect.min.x + x_offset;
                    while current_x <= rect.max.x {
                        painter.vline(current_x, rect.y_range(), Stroke::new(1.0, Color32::from_rgb(35, 35, 38)));
                        let frame = ((current_x - origin_x) / app.ui.graph_zoom.x * fps).round() as i32;
                        painter.text(Pos2::new(current_x + 4.0, rect.min.y + 4.0), Align2::LEFT_TOP, format!("{}", frame), FontId::proportional(10.0), Color32::from_gray(150));
                        current_x += frame_step_x;
                    }

                    painter.hline(rect.x_range(), origin_y, Stroke::new(1.5, Color32::from_rgb(80, 80, 85)));

                    if let Some(anim_id) = &app.animation.project.active_animation_id {
                        if let Some(anim) = app.animation.project.animations.get(anim_id) {
                            if let Some(bone_id) = &app.ui.selected_bone_id {
                                for tl in &anim.timelines {
                                    if &tl.target_id == bone_id {
                                        let (col_x, col_y) = match tl.property {
                                            TimelineProperty::Rotation => (Color32::from_rgb(0, 255, 0), Color32::from_rgb(0, 255, 0)),
                                            TimelineProperty::Translation => (Color32::from_rgb(0, 153, 255), Color32::from_rgb(0, 102, 204)),
                                            TimelineProperty::Scale => (Color32::from_rgb(255, 51, 51), Color32::from_rgb(204, 0, 0)),
                                            _ => continue,
                                        };

                                        for i in 0..tl.keyframes.len() {
                                            let kf = &tl.keyframes[i];
                                            let screen_x = origin_x + kf.time * app.ui.graph_zoom.x;
                                            let (val_x, val_y) = match kf.value {
                                                KeyframeValue::Rotate(r) => (r, r),
                                                KeyframeValue::Translate(tx, ty) => (tx, ty),
                                                KeyframeValue::Scale(sx, sy) => (sx * 100.0, sy * 100.0),
                                                _ => (0.0, 0.0),
                                            };
                                            let p_x = Pos2::new(screen_x, origin_y - val_x * app.ui.graph_zoom.y);
                                            let p_y = Pos2::new(screen_x, origin_y - val_y * app.ui.graph_zoom.y);

                                            let kf_rect = Rect::from_center_size(p_x, vec2(6.0, 14.0));
                                            let resp = ui.interact(kf_rect, ui.id().with(format!("gkf_{}_{:?}_{}", bone_id, tl.property, kf.time)), Sense::click());
                                            
                                            resp.context_menu(|ui| {
                                                if ui.button("直线 (Linear)").clicked() { pending_commands.push(AppCommand::UpdateKeyframeCurve(bone_id.clone(), tl.property.clone(), kf.time, CurveType::Linear)); ui.close_menu(); }
                                                if ui.button("平滑 (Bezier)").clicked() { pending_commands.push(AppCommand::UpdateKeyframeCurve(bone_id.clone(), tl.property.clone(), kf.time, CurveType::Bezier(0.33, 0.0, 0.66, 1.0))); ui.close_menu(); }
                                                if ui.button("阶跃 (Stepped)").clicked() { pending_commands.push(AppCommand::UpdateKeyframeCurve(bone_id.clone(), tl.property.clone(), kf.time, CurveType::Stepped)); ui.close_menu(); }
                                            });

                                            if i + 1 < tl.keyframes.len() {
                                                let next_kf = &tl.keyframes[i+1];
                                                let n_screen_x = origin_x + next_kf.time * app.ui.graph_zoom.x;
                                                let (n_val_x, n_val_y) = match next_kf.value {
                                                    KeyframeValue::Rotate(r) => (r, r),
                                                    KeyframeValue::Translate(tx, ty) => (tx, ty),
                                                    KeyframeValue::Scale(sx, sy) => (sx * 100.0, sy * 100.0),
                                                    _ => (0.0, 0.0),
                                                };
                                                let n_p_x = Pos2::new(n_screen_x, origin_y - n_val_x * app.ui.graph_zoom.y);
                                                let n_p_y = Pos2::new(n_screen_x, origin_y - n_val_y * app.ui.graph_zoom.y);

                                                match kf.curve {
                                                    CurveType::Linear => {
                                                        painter.line_segment([p_x, n_p_x], Stroke::new(1.5, col_x));
                                                        if matches!(tl.property, TimelineProperty::Translation | TimelineProperty::Scale) { painter.line_segment([p_y, n_p_y], Stroke::new(1.5, col_y)); }
                                                    }
                                                    CurveType::Stepped => {
                                                        let mid_x = Pos2::new(n_p_x.x, p_x.y);
                                                        painter.line_segment([p_x, mid_x], Stroke::new(1.5, col_x));
                                                        painter.line_segment([mid_x, n_p_x], Stroke::new(1.5, col_x));
                                                    }
                                                    CurveType::Bezier(cx1, cy1, cx2, cy2) => {
                                                        let segments = 40;
                                                        let mut last_px = p_x;
                                                        let mut last_py = p_y;
                                                        for s in 1..=segments {
                                                            let t = s as f32 / segments as f32;
                                                            let eased_t = crate::core::animation::timeline::Timeline::solve_bezier_y(cx1, cy1, cx2, cy2, t);
                                                            let cur_sx = p_x.x + (n_p_x.x - p_x.x) * t;
                                                            let cur_sy_x = p_x.y + (n_p_x.y - p_x.y) * eased_t;
                                                            let cur_px = Pos2::new(cur_sx, cur_sy_x);
                                                            painter.line_segment([last_px, cur_px], Stroke::new(1.5, col_x));
                                                            last_px = cur_px;
                                                            
                                                            if matches!(tl.property, TimelineProperty::Translation | TimelineProperty::Scale) {
                                                                let cur_sy_y = p_y.y + (n_p_y.y - p_y.y) * eased_t;
                                                                let cur_py = Pos2::new(cur_sx, cur_sy_y);
                                                                painter.line_segment([last_py, cur_py], Stroke::new(2.0, col_y));
                                                                last_py = cur_py;
                                                            }
                                                        }

                                                        let handle_col = Color32::from_rgb(0, 255, 255);
                                                        let h1_pos = Pos2::new(p_x.x + cx1 * (n_p_x.x - p_x.x), p_x.y + cy1 * (n_p_x.y - p_x.y));
                                                        let h2_pos = Pos2::new(p_x.x + cx2 * (n_p_x.x - p_x.x), p_x.y + cy2 * (n_p_x.y - p_x.y));

                                                        painter.line_segment([p_x, h1_pos], Stroke::new(1.0, handle_col.linear_multiply(0.7)));
                                                        painter.line_segment([n_p_x, h2_pos], Stroke::new(1.0, handle_col.linear_multiply(0.7)));

                                                        let h1_rect = Rect::from_center_size(h1_pos, vec2(12.0, 12.0));
                                                        let h2_rect = Rect::from_center_size(h2_pos, vec2(12.0, 12.0));
                                                        let h1_resp = ui.interact(h1_rect, ui.id().with(format!("h1_{}_{:?}_{}", bone_id, tl.property, kf.time)), Sense::drag());
                                                        let h2_resp = ui.interact(h2_rect, ui.id().with(format!("h2_{}_{:?}_{}", bone_id, tl.property, kf.time)), Sense::drag());

                                                        painter.circle_stroke(h1_pos, 4.0, Stroke::new(1.5, handle_col));
                                                        painter.circle_stroke(h2_pos, 4.0, Stroke::new(1.5, handle_col));
                                                        painter.circle_filled(h1_pos, 2.0, Color32::from_rgb(25, 25, 25));
                                                        painter.circle_filled(h2_pos, 2.0, Color32::from_rgb(25, 25, 25));

                                                        if h1_resp.dragged() || h2_resp.dragged() {
                                                            let mut new_cx1 = cx1; let mut new_cy1 = cy1;
                                                            let mut new_cx2 = cx2; let mut new_cy2 = cy2;
                                                            let dx = n_p_x.x - p_x.x;
                                                            let dy = n_p_x.y - p_x.y;
                                                            let safe_dy = if dy.abs() < 0.1 { 100.0 } else { dy }; 
                                                            
                                                            if dx.abs() > 0.001 {
                                                                if let Some(pos) = h1_resp.interact_pointer_pos() { 
                                                                    new_cx1 = ((pos.x - p_x.x) / dx).clamp(0.0, 1.0); 
                                                                    new_cy1 = if dy.abs() > 0.1 { (pos.y - p_x.y) / safe_dy } else { 0.0 };
                                                                }
                                                                if let Some(pos) = h2_resp.interact_pointer_pos() { 
                                                                    new_cx2 = ((pos.x - p_x.x) / dx).clamp(0.0, 1.0); 
                                                                    new_cy2 = if dy.abs() > 0.1 { (pos.y - p_x.y) / safe_dy } else { 0.0 };
                                                                }
                                                                pending_commands.push(AppCommand::UpdateKeyframeCurve(bone_id.clone(), tl.property.clone(), kf.time, CurveType::Bezier(new_cx1, new_cy1, new_cx2, new_cy2)));
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            painter.rect_filled(Rect::from_center_size(p_x, vec2(3.0, 12.0)), 0.0, Color32::from_rgb(0, 255, 255));
                                            if matches!(tl.property, TimelineProperty::Translation | TimelineProperty::Scale) { 
                                                painter.circle_filled(p_y, 3.0, Color32::from_rgb(0, 255, 255)); 
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let playhead_x = origin_x + app.animation.current_time * app.ui.graph_zoom.x;
                    painter.vline(playhead_x, rect.y_range(), Stroke::new(1.5, Color32::from_rgb(0, 255, 255)));
                    painter.add(egui::Shape::convex_polygon(
                        vec![
                            Pos2::new(playhead_x - 6.0, rect.min.y),
                            Pos2::new(playhead_x + 6.0, rect.min.y),
                            Pos2::new(playhead_x, rect.min.y + 8.0),
                        ],
                        Color32::from_rgb(0, 255, 255),
                        Stroke::NONE,
                    ));

                    for cmd in pending_commands { app.enqueue_command(cmd); }
                });
        }
        app.ui.show_curve_editor = show_curve_editor;
    }
}