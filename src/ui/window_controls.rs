use egui::{Ui, Rect, Sense, Color32, Stroke, vec2};
use crate::app::state::AppState;
use crate::app::commands::AppCommand;

pub struct WindowControls;

impl WindowControls {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        ui.spacing_mut().item_spacing.x = 0.0; 
        
        let btn_size = vec2(46.0, 32.0); 
        let (rect_close, resp_close) = ui.allocate_exact_size(btn_size, Sense::click());
        let close_hovered = resp_close.hovered();
        
        if close_hovered {
            ui.painter().rect_filled(rect_close, 0.0, Color32::from_rgb(232, 17, 35));
        }
        if resp_close.clicked() {
            app.enqueue_command(AppCommand::RequestExit);
        }
        
        let center = rect_close.center();
        let color_close = if close_hovered { Color32::WHITE } else { Color32::from_gray(200) };
        let stroke_close = Stroke::new(1.0, color_close);
        ui.painter().line_segment([center + vec2(-5.0, -5.0), center + vec2(5.0, 5.0)], stroke_close);
        ui.painter().line_segment([center + vec2(5.0, -5.0), center + vec2(-5.0, 5.0)], stroke_close);

        let (rect_max, resp_max) = ui.allocate_exact_size(btn_size, Sense::click());
        let max_hovered = resp_max.hovered();

        if max_hovered {
            ui.painter().rect_filled(rect_max, 0.0, Color32::from_white_alpha(25));
        }
        if resp_max.clicked() {
            app.enqueue_command(AppCommand::WindowMaximize);
        }

        let center = rect_max.center();
        let color_max = if max_hovered { Color32::WHITE } else { Color32::from_gray(200) };
        ui.painter().rect_stroke(
            Rect::from_center_size(center, vec2(10.0, 10.0)),
            0.0,
            Stroke::new(1.0, color_max)
        );

        let (rect_min, resp_min) = ui.allocate_exact_size(btn_size, Sense::click());
        let min_hovered = resp_min.hovered();
        
        if min_hovered {
            ui.painter().rect_filled(rect_min, 0.0, Color32::from_white_alpha(25));
        }
        if resp_min.clicked() {
            app.enqueue_command(AppCommand::WindowMinimize);
        }

        let center = rect_min.center();
        let color_min = if min_hovered { Color32::WHITE } else { Color32::from_gray(200) };
        ui.painter().line_segment([center + vec2(-5.0, 0.0), center + vec2(5.0, 0.0)], Stroke::new(1.0, color_min));
    }
}