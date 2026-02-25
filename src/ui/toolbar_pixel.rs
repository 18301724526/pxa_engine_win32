use egui::{Ui, Color32};
use crate::app::state::{AppState, ToolType};
use crate::app::commands::AppCommand;
use crate::core::store::BrushShape;
use crate::ui::palette_panel::PalettePanel;
use rust_i18n::t;

const ICON_PENCIL: &str     = "\u{efdf}"; 
const ICON_ERASER: &str     = "\u{ec9e}"; 
const ICON_BUCKET: &str     = "\u{efc2}"; 
const ICON_DROPPER: &str    = "\u{f530}"; 
const ICON_RECT_SEL: &str   = "\u{ed4c}"; 
const ICON_ELLIPSE_SEL: &str= "\u{eb7d}";
const ICON_MOVE: &str       = "\u{ec61}"; 
const ICON_TRANSFORM: &str  = "\u{ea7c}"; 
const ICON_PEN: &str        = "\u{f049}";
const ICON_BONE: &str       = "\u{f5d7}";

pub struct ToolbarPixel;

impl ToolbarPixel {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        ui.label(egui::RichText::new(t!("anim.setup_mode")).size(10.0).color(egui::Color32::GRAY));
        
        egui::Grid::new("pixel_tools")
            .num_columns(2)
            .spacing([6.0, 6.0])
            .show(ui, |ui| {
                Self::tool_btn(ui, app, ToolType::Pencil, ICON_PENCIL, &t!("tool.pencil"));
                Self::tool_btn(ui, app, ToolType::Eraser, ICON_ERASER, &t!("tool.eraser"));
                ui.end_row();
                Self::tool_btn(ui, app, ToolType::Bucket, ICON_BUCKET, &t!("tool.bucket"));
                Self::tool_btn(ui, app, ToolType::Eyedropper, ICON_DROPPER, &t!("tool.dropper"));
                ui.end_row();

                let (sel_icon, sel_name) = if app.ui.active_select_tool == ToolType::EllipseSelect {
                    (ICON_ELLIPSE_SEL, t!("tool.ellipse_select").to_string())
                } else {
                    (ICON_RECT_SEL, t!("tool.rect_select").to_string())
                };
                let sel_resp = Self::tool_btn(ui, app, app.ui.active_select_tool, sel_icon, &sel_name);
                sel_resp.context_menu(|ui| {
                    if ui.selectable_label(app.ui.active_select_tool == ToolType::RectSelect, format!("{} {}", ICON_RECT_SEL, t!("tool.rect_select"))).clicked() {
                        app.ui.active_select_tool = ToolType::RectSelect;
                        app.set_tool(ToolType::RectSelect);
                        ui.close_menu();
                    }
                    if ui.selectable_label(app.ui.active_select_tool == ToolType::EllipseSelect, format!("{} {}", ICON_ELLIPSE_SEL, t!("tool.ellipse_select"))).clicked() {
                        app.ui.active_select_tool = ToolType::EllipseSelect;
                        app.set_tool(ToolType::EllipseSelect);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(t!("tool.deselect")).clicked() {
                        app.enqueue_command(AppCommand::ClearSelection);
                        ui.close_menu();
                    }
                });

                Self::tool_btn(ui, app, ToolType::Move, ICON_MOVE, &t!("tool.move"));
                ui.end_row();
                Self::tool_btn(ui, app, ToolType::Transform, ICON_TRANSFORM, &t!("tool.transform"));
                Self::tool_btn(ui, app, ToolType::Pen, ICON_PEN, &t!("tool.pen"));
                ui.end_row();

                Self::tool_btn(ui, app, ToolType::CreateBone, ICON_BONE, &t!("tool.bone"));
            });

        if app.engine.tool_manager().active_type == ToolType::Pen {
            ui.add_space(5.0);
            ui.separator();
            ui.label("Path Options");
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);
                if ui.small_button("Sel").clicked() { app.enqueue_command(AppCommand::CommitCurrentTool); }
                if ui.small_button("Fill").clicked() { app.enqueue_command(AppCommand::PenFill); }
                if ui.small_button("Strk").clicked() { app.enqueue_command(AppCommand::PenStroke); }
            });
        }

        ui.separator();
        ui.add_space(5.0);
        PalettePanel::show(ui, app);
        ui.add_space(10.0);

        let (brush_size, brush_shape, brush_jitter) = app.engine.brush_settings_mut();
        ui.label(format!("{}: {}px", t!("toolbar.size"), *brush_size));
        ui.add(egui::Slider::new(brush_size, 1..=20).show_value(false));
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.label(format!("{}:", t!("toolbar.shape")));
            egui::ComboBox::from_id_source("brush_shape")
                .selected_text(if *brush_shape == BrushShape::Square { t!("toolbar.square").to_string() } else { t!("toolbar.circle").to_string() })
                .show_ui(ui, |ui| {
                    ui.selectable_value(brush_shape, BrushShape::Square, t!("toolbar.square").to_string());
                    ui.selectable_value(brush_shape, BrushShape::Circle, t!("toolbar.circle").to_string());
                });
        });
        ui.add_space(5.0);
        ui.label(format!("{}: {}", t!("toolbar.jitter"), *brush_jitter));
        ui.add(egui::Slider::new(brush_jitter, 0..=15).show_value(false));
    }

    fn tool_btn(ui: &mut Ui, app: &mut AppState, tool: ToolType, icon: &str, name: &str) -> egui::Response {
        let is_active = app.engine.tool_manager().active_type == tool;
        let bg_color = if is_active { Color32::from_rgb(60, 60, 60) } else { Color32::TRANSPARENT };
        let fg_color = if is_active { Color32::LIGHT_BLUE } else { Color32::GRAY };
        
        let size = egui::vec2(44.0, 44.0);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

        if is_active || response.hovered() {
            let stroke = if is_active { egui::Stroke::new(1.0, Color32::LIGHT_BLUE) } else { egui::Stroke::NONE };
            ui.painter().rect_filled(rect, 6.0, bg_color);
            if is_active { ui.painter().rect_stroke(rect, 6.0, stroke); }
        }

        let text_icon = ui.painter().layout_no_wrap(icon.to_string(), egui::FontId::proportional(20.0), fg_color);
        let text_name = ui.painter().layout_no_wrap(name.to_string(), egui::FontId::proportional(11.0), fg_color);

        ui.painter().galley(egui::pos2(rect.center().x - text_icon.size().x / 2.0, rect.center().y - 11.0), text_icon);
        ui.painter().galley(egui::pos2(rect.center().x - text_name.size().x / 2.0, rect.center().y + 9.0), text_name);

        if response.clicked() { 
            app.set_tool(tool); 
        }
        response
    }
}