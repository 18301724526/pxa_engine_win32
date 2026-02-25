use egui::Ui;
use crate::app::state::AppState;
use crate::app::commands::ResizeAnchor;
use rust_i18n::t;

pub struct MenuImage;

impl MenuImage {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        ui.menu_button(t!("menu.image"), |ui| {
            ui.set_min_width(120.0);
            
            if ui.button(t!("menu.canvas_size")).clicked() {
                app.ui.resize_new_width = app.engine.store().canvas_width.to_string();
                app.ui.resize_new_height = app.engine.store().canvas_height.to_string();
                app.ui.resize_anchor = ResizeAnchor::Center;
                app.ui.show_resize_modal = true;
                ui.close_menu();
            }
        });
    }
}