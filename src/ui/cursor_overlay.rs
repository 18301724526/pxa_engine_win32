use crate::app::state::{AppState, ToolType};
use crate::core::store::PixelStore;
use crate::core::store::BrushShape;
use egui::{Painter, Pos2, Rect, Stroke, Color32, Context};
use crate::core::symmetry::SymmetryMode;
use crate::tools::pen::PenTool;

pub struct CursorOverlay;

impl CursorOverlay {
    pub fn draw(ctx: &Context, app: &AppState) {
        let pointer_pos = match ctx.input(|i| i.pointer.hover_pos()) {
            Some(pos) => pos,
            None => return,
        };
        let scale_factor = ctx.pixels_per_point();
        let physical_x = pointer_pos.x * scale_factor;
        let physical_y = pointer_pos.y * scale_factor;

        let (cx, cy) = match app.view.screen_to_canvas(app.engine.store(), physical_x, physical_y) {
            Some(coords) => coords,
            None => return,
        };
        if app.engine.tool_manager().active_type == ToolType::Eyedropper {
            let color = app.engine.store().get_composite_pixel(cx, cy);
            Self::draw_eyedropper_preview(ctx, color);
        }
        if app.engine.tool_manager().active_type == ToolType::Pencil || app.engine.tool_manager().active_type == ToolType::Eraser {
            let (rect_x, rect_y, rect_w, rect_h) = Self::calculate_brush_rect(cx, cy, app.engine.store().brush_size);
            let screen_rect = Self::canvas_rect_to_screen_rect(
                rect_x, rect_y, rect_w, rect_h,
                app,
                app.view.width,
                app.view.height,
                scale_factor
            );

            let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("cursor_overlay")));
            Self::paint_cursor(&painter, screen_rect, app.engine.store());
        }
        if app.engine.symmetry().visible_guides && app.engine.symmetry().mode != SymmetryMode::None {
            let guide_painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("symmetry_guides")));
            Self::draw_symmetry_guides(&guide_painter, app, scale_factor);
        }
        if app.view.zoom_level >= 8.0 {
            let grid_painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Background, egui::Id::new("pixel_grid")));
            Self::draw_pixel_grid(&grid_painter, app, scale_factor);
        }

        if app.engine.tool_manager().active_type == ToolType::Transform {
            let transform_painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("transform_overlay")));
            Self::draw_transform_overlay(&transform_painter, app, scale_factor);
        }
        if app.engine.tool_manager().active_type == ToolType::Pen {
            let pen_painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("pen_overlay")));
            Self::draw_pen_overlay(&pen_painter, app, scale_factor);
        }
    }

    fn draw_pen_overlay(painter: &Painter, app: &AppState, scale_factor: f32) {
        let tool = match app.engine.tool_manager().tools.get(&ToolType::Pen) {
            Some(t) => t,
            None => return,
        };
        
        if let Some(pen_tool) = tool.as_any().downcast_ref::<PenTool>() {
            let path = &app.engine.store().active_path;

            let zoom = app.view.zoom_level as f32;
            let screen_cx = app.view.width / 2.0;
            let screen_cy = app.view.height / 2.0;
            let canvas_cx = app.engine.store().canvas_width as f32 / 2.0;
            let canvas_cy = app.engine.store().canvas_height as f32 / 2.0;

            let to_screen = |cx: f32, cy: f32| -> Pos2 {
                let phys_x = (cx - canvas_cx + app.view.pan_x) * zoom + screen_cx;
                let phys_y = (cy - canvas_cy + app.view.pan_y) * zoom + screen_cy;
                Pos2::new(phys_x / scale_factor, phys_y / scale_factor)
            };

            let stroke_path = Stroke::new(1.5, Color32::from_rgb(0, 160, 255));
            let stroke_handle = Stroke::new(1.0, Color32::GRAY);
            
            let count = path.nodes.len();
            let segments = if path.is_closed { count } else { count.saturating_sub(1) };

            for i in 0..segments {
                let n1 = &path.nodes[i];
                let n2 = &path.nodes[(i + 1) % count];

                let p0 = to_screen(n1.anchor.x, n1.anchor.y);
                let p3 = to_screen(n2.anchor.x, n2.anchor.y);
                
                let cp1_abs = n1.abs_out(); 
                let cp2_abs = n2.abs_in();
                
                let p1 = to_screen(cp1_abs.x, cp1_abs.y);
                let p2 = to_screen(cp2_abs.x, cp2_abs.y);

                let shape = egui::epaint::CubicBezierShape::from_points_stroke(
                    [p0, p1, p2, p3],
                    false,
                    Color32::TRANSPARENT,
                    stroke_path,
                );
                painter.add(shape);
            }

            for (i, node) in path.nodes.iter().enumerate() {
                let anchor_pos = to_screen(node.anchor.x, node.anchor.y);
                
                let show_handles = Some(i) == pen_tool.selected_node_idx;

                if show_handles {
                    if node.handle_in.x != 0.0 || node.handle_in.y != 0.0 {
                        let h_in = to_screen(node.abs_in().x, node.abs_in().y);
                        painter.line_segment([anchor_pos, h_in], stroke_handle);
                        painter.circle_filled(h_in, 3.0, Color32::WHITE);
                        painter.circle_stroke(h_in, 3.0, Stroke::new(1.0, Color32::BLACK));
                    }
                    if node.handle_out.x != 0.0 || node.handle_out.y != 0.0 {
                        let h_out = to_screen(node.abs_out().x, node.abs_out().y);
                        painter.line_segment([anchor_pos, h_out], stroke_handle);
                        painter.circle_filled(h_out, 3.0, Color32::WHITE);
                        painter.circle_stroke(h_out, 3.0, Stroke::new(1.0, Color32::BLACK));
                    }
                }
                let is_hovered = Some(i) == pen_tool.hover_node_idx;
                let is_selected = Some(i) == pen_tool.selected_node_idx;

                let rect = Rect::from_center_size(anchor_pos, egui::vec2(5.0, 5.0));
                let fill_color = if is_selected { Color32::BLACK } else { Color32::WHITE };
                let stroke_color = if is_hovered { Color32::RED } else { Color32::BLACK };
                
                painter.rect_filled(rect, 1.0, fill_color);
                painter.rect_stroke(rect, 1.0, Stroke::new(1.0, stroke_color));
            }
            if pen_tool.hover_start_point && !path.nodes.is_empty() {
                let start_pos = to_screen(path.nodes[0].anchor.x, path.nodes[0].anchor.y);
                painter.circle_stroke(start_pos, 8.0, Stroke::new(2.0, Color32::from_rgb(255, 255, 0)));
            }
        }
    }

    fn draw_transform_overlay(painter: &Painter, app: &AppState, scale_factor: f32) {
        if let Some(params) = app.engine.tool_manager().get_transform_params() {
            let (min_x, min_y, w, h, piv_x, piv_y, off_x, off_y, s_x, s_y, rot): (f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32) = params;

            let zoom = app.view.zoom_level as f32;
            let screen_cx = app.view.width / 2.0;
            let screen_cy = app.view.height / 2.0;
            let canvas_cx = app.engine.store().canvas_width as f32 / 2.0;
            let canvas_cy = app.engine.store().canvas_height as f32 / 2.0;

            let to_screen = |cx: f32, cy: f32| -> Pos2 {
                let phys_x = (cx - canvas_cx + app.view.pan_x) * zoom + screen_cx;
                let phys_y = (cy - canvas_cy + app.view.pan_y) * zoom + screen_cy;
                Pos2::new(phys_x / scale_factor, phys_y / scale_factor)
            };

            let base_corners = [
                (min_x, min_y),
                (min_x + w, min_y),
                (min_x + w, min_y + h),
                (min_x, min_y + h),
            ];

            let cos_t = rot.cos();
            let sin_t = rot.sin();

            let mut screen_corners = [Pos2::ZERO; 4];

            for i in 0..4 {
                let (cx, cy) = base_corners[i];
                let mut x = cx - piv_x;
                let mut y = cy - piv_y;
                x *= s_x;
                y *= s_y;
                let rx = x * cos_t - y * sin_t;
                let ry = x * sin_t + y * cos_t;
                let final_x = rx + piv_x + off_x;
                let final_y = ry + piv_y + off_y;

                screen_corners[i] = to_screen(final_x, final_y);
            }
            let stroke = Stroke::new(1.5, Color32::WHITE);
            for i in 0..4 {
                let p1 = screen_corners[i];
                let p2 = screen_corners[(i + 1) % 4];
                painter.line_segment([p1, p2], stroke);
            }

            let mut handles = Vec::new();
            handles.extend_from_slice(&screen_corners);
            for i in 0..4 {
                let p1 = screen_corners[i];
                let p2 = screen_corners[(i + 1) % 4];
                handles.push(Pos2::new((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0));
            }

            let handle_radius = 4.0;
            for p in handles {
                let rect = Rect::from_center_size(p, egui::vec2(handle_radius * 2.0, handle_radius * 2.0));
                painter.rect_filled(rect, 0.0, Color32::WHITE);
                painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::BLACK));
            }

            let piv_screen = to_screen(piv_x + off_x, piv_y + off_y);
            painter.circle_stroke(piv_screen, 3.0, Stroke::new(1.5, Color32::WHITE));
            painter.circle_stroke(piv_screen, 4.0, Stroke::new(1.0, Color32::BLACK));
        }
    }
    fn draw_eyedropper_preview(ctx: &Context, color: crate::core::color::Color) {
        let egui_color = Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a);
        egui::show_tooltip_at_pointer(ctx, egui::Id::new("eye_preview"), |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 4.0, egui_color);
                ui.painter().rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::GRAY));
                
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)).strong());
                    ui.label(format!("R:{} G:{} B:{}", color.r, color.g, color.b));
                });
            });
        });
    }

    fn calculate_brush_rect(center_x: u32, center_y: u32, size: u32) -> (i32, i32, u32, u32) {
        let size_i32 = size as i32;
        let offset = size_i32 / 2;
        let x = center_x as i32 - offset;
        let y = center_y as i32 - offset;
        (x, y, size, size)
    }

    fn canvas_rect_to_screen_rect(
        cx: i32, cy: i32, w: u32, h: u32,
        app: &AppState,
        viewport_w: f32, viewport_h: f32,
        scale_factor: f32
    ) -> Rect {
        let zoom = app.view.zoom_level as f32;
        let screen_cx = viewport_w / 2.0;
        let screen_cy = viewport_h / 2.0;
        let canvas_cx = app.engine.store().canvas_width as f32 / 2.0;
        let canvas_cy = app.engine.store().canvas_height as f32 / 2.0;

        let phys_x = (cx as f32 - canvas_cx + app.view.pan_x) * zoom + screen_cx;
        let phys_y = (cy as f32 - canvas_cy + app.view.pan_y) * zoom + screen_cy;

        let logical_x = phys_x / scale_factor;
        let logical_y = phys_y / scale_factor;
        
        let logical_w = (w as f32 * zoom) / scale_factor;
        let logical_h = (h as f32 * zoom) / scale_factor;

        Rect::from_min_size(
            Pos2::new(logical_x, logical_y),
            egui::vec2(logical_w, logical_h)
        )
    }

    fn paint_cursor(painter: &Painter, rect: Rect, store: &PixelStore) {
        if store.brush_shape == BrushShape::Circle {
            let r = rect.width() / 2.0;
            painter.circle_stroke(rect.center(), r, Stroke::new(1.0, Color32::from_black_alpha(180)));
            painter.circle_stroke(rect.center(), r - 1.0, Stroke::new(1.0, Color32::from_white_alpha(200)));
        } else {
            painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_black_alpha(180)));
            painter.rect_stroke(rect.shrink(1.0), 0.0, Stroke::new(1.0, Color32::from_white_alpha(200)));
        }
    }

    fn draw_symmetry_guides(painter: &Painter, app: &AppState, scale_factor: f32) {
        let zoom = app.view.zoom_level as f32;
        let screen_cx = app.view.width / 2.0;
        let screen_cy = app.view.height / 2.0;
        let canvas_cx = app.engine.store().canvas_width as f32 / 2.0;
        let canvas_cy = app.engine.store().canvas_height as f32 / 2.0;

        let to_logical_x = |val: f32| {
            let phys = (val - canvas_cx + app.view.pan_x) * zoom + screen_cx;
            phys / scale_factor
        };
        let to_logical_y = |val: f32| {
            let phys = (val - canvas_cy + app.view.pan_y) * zoom + screen_cy;
            phys / scale_factor
        };

        let stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 255, 255, 128));
        let log_vw = app.view.width / scale_factor;
        let log_vh = app.view.height / scale_factor;

        if app.engine.symmetry().mode == SymmetryMode::Horizontal || app.engine.symmetry().mode == SymmetryMode::Quad {
            let x = to_logical_x(app.engine.symmetry().axis_x);
            painter.line_segment([Pos2::new(x, 0.0), Pos2::new(x, log_vh)], stroke);
        }
        if app.engine.symmetry().mode == SymmetryMode::Vertical || app.engine.symmetry().mode == SymmetryMode::Quad {
            let y = to_logical_y(app.engine.symmetry().axis_y);
            painter.line_segment([Pos2::new(0.0, y), Pos2::new(log_vw, y)], stroke);
        }
    }
    fn draw_pixel_grid(painter: &Painter, app: &AppState, scale_factor: f32) {
        let zoom = app.view.zoom_level as f32;
        let screen_cx = app.view.width / 2.0;
        let screen_cy = app.view.height / 2.0;
        let canvas_cx = app.engine.store().canvas_width as f32 / 2.0;
        let canvas_cy = app.engine.store().canvas_height as f32 / 2.0;

        let to_logical_x = |cx: f32| { ((cx - canvas_cx + app.view.pan_x) * zoom + screen_cx) / scale_factor };
        let to_logical_y = |cy: f32| { ((cy - canvas_cy + app.view.pan_y) * zoom + screen_cy) / scale_factor };

        let start_cx = (((0.0 - screen_cx) / zoom + canvas_cx - app.view.pan_x).floor() as i32).max(0);
        let start_cy = (((0.0 - screen_cy) / zoom + canvas_cy - app.view.pan_y).floor() as i32).max(0);
        let end_cx = (((app.view.width - screen_cx) / zoom + canvas_cx - app.view.pan_x).ceil() as i32).min(app.engine.store().canvas_width as i32);
        let end_cy = (((app.view.height - screen_cy) / zoom + canvas_cy - app.view.pan_y).ceil() as i32).min(app.engine.store().canvas_height as i32);

        let stroke = Stroke::new(1.0, Color32::from_white_alpha(20));

        for x in start_cx..=end_cx {
            let lx = to_logical_x(x as f32);
            let start_ly = to_logical_y(start_cy as f32);
            let end_ly = to_logical_y(end_cy as f32);
            painter.line_segment([Pos2::new(lx, start_ly), Pos2::new(lx, end_ly)], stroke);
        }

        for y in start_cy..=end_cy {
            let ly = to_logical_y(y as f32);
            let start_lx = to_logical_x(start_cx as f32);
            let end_lx = to_logical_x(end_cx as f32);
            painter.line_segment([Pos2::new(start_lx, ly), Pos2::new(end_lx, ly)], stroke);
        }
    }
}