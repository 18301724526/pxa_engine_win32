use egui::Ui;
use crate::app::state::AppState;
use crate::app::commands::AppCommand;
use rust_i18n::t;

pub struct MenuFile;

impl MenuFile {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        ui.menu_button(t!("menu.file"), |ui| {
            ui.set_min_width(150.0);
            
            if ui.button(t!("menu.open_project")).clicked() {
                app.enqueue_command(AppCommand::LoadProject);
                ui.close_menu();
            }
            
            if ui.button(t!("menu.save_project")).clicked() {
                app.enqueue_command(AppCommand::SaveProject);
                ui.close_menu();
            }
            
            ui.separator();
            
            if ui.button(t!("menu.import_image")).clicked() {
                app.enqueue_command(AppCommand::ImportImage);
                ui.close_menu();
            }
            
            if ui.button(t!("menu.export_png")).clicked() {
                app.enqueue_command(AppCommand::ExportPng);
                ui.close_menu();
            }

            ui.separator();

            if ui.button(t!("menu.exit")).clicked() {
                app.enqueue_command(AppCommand::RequestExit);
                ui.close_menu();
            }
        });
    }
}