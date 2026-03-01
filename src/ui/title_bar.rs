use egui::{Ui, Sense, Color32};
use crate::app::state::{AppState, ToolType};
use crate::app::commands::AppCommand;
use crate::ui::menu_file::MenuFile;
use crate::ui::menu_image::MenuImage;
use crate::app::state::AppMode;
use crate::ui::window_controls::WindowControls;
use rust_i18n::t;

pub struct TitleBar;

impl TitleBar {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        let bar_height = 32.0;
        
        let full_rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(full_rect, 0.0, Color32::from_rgb(25, 25, 25));

        ui.horizontal(|ui| {
            ui.set_height(bar_height);
            
            ui.add_space(8.0);
            ui.label(egui::RichText::new("PXA PRO").strong().color(egui::Color32::from_rgb(0, 160, 255)));
            ui.separator();
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                if ui.selectable_label(app.mode == AppMode::PixelEdit, "🎨").on_hover_text("Paint Mode").clicked() {
                    app.mode = AppMode::PixelEdit;
                    app.sync_animation_to_layers();
                }
                
                if ui.selectable_label(app.mode == AppMode::Animation, "🎞").on_hover_text("Animate Mode").clicked() {
                    app.cancel_current_tool();
                    app.mode = AppMode::Animation;
                    app.set_tool(ToolType::Move);
                    app.sync_animation_to_layers();
                }
            });
            ui.separator();
            
            MenuFile::show(ui, app);
            MenuImage::show(ui, app);

            ui.menu_button(t!("menu.language"), |ui| {
                let langs = [
                    ("zh-CN", "简体中文"),
                    ("zh-TW", "繁體中文"),
                    ("en", "English"),
                    ("ja", "日本語"),
                    ("ko", "한국어"),
                    ("es", "Español"),
                    ("fr", "Français"),
                    ("de", "Deutsch"),
                    ("ru", "Русский"),
                ];
                for (code, name) in langs {
                    if ui.radio(app.ui.language == code, name).clicked() {
                        app.enqueue_command(AppCommand::SetLanguage(code.into()));
                        ui.close_menu();
                    }
                }
            });
            
            let rect = ui.available_rect_before_wrap();
            let response = ui.interact(rect, ui.id().with("drag_area"), Sense::drag());
            
            if response.drag_started() {
                app.enqueue_command(AppCommand::WindowDrag);
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                WindowControls::show(ui, app);
            });
        });
    }
}