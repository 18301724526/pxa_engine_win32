use egui::Ui;
use crate::app::state::AppState;
use crate::app::commands::*; // 确保导入了所有具体的命令结构体
use rust_i18n::t;

pub struct MenuFile;

impl MenuFile {
    pub fn show(ui: &mut Ui, app: &mut AppState, _ui_ctx: &mut crate::app::ui_context::UiContext) {
        ui.menu_button(t!("menu.file"), |ui| {
            ui.set_min_width(150.0);
            
            // 1. 加载工程
            if ui.button(t!("menu.open_project")).clicked() {
                app.enqueue_command(Box::new(LoadProjectCmd));
                ui.close_menu();
            }
            
            // 2. 保存工程
            if ui.button(t!("menu.save_project")).clicked() {
                app.enqueue_command(Box::new(SaveProjectCmd));
                ui.close_menu();
            }
            
            ui.separator();
            
            // 3. 导入图片
            if ui.button(t!("menu.import_image")).clicked() {
                app.enqueue_command(Box::new(ImportImageCmd));
                ui.close_menu();
            }
            
            // 4. 导出 PNG
            if ui.button(t!("menu.export_png")).clicked() {
                app.enqueue_command(Box::new(ExportPngCmd));
                ui.close_menu();
            }

            // 5. 导出 GIF
            if ui.button("🎞 导出 GIF 动画").clicked() {
                app.enqueue_command(Box::new(ExportGifCmd));
                ui.close_menu();
            }
            
            // 6. 导出序列帧
            if ui.button("📁 导出 PNG 序列帧").clicked() {
                app.enqueue_command(Box::new(ExportSequenceCmd));
                ui.close_menu();
            }

            ui.separator();

            // 7. 退出请求
            if ui.button(t!("menu.exit")).clicked() {
                app.enqueue_command(Box::new(RequestExitCmd));
                ui.close_menu();
            }
        });
    }
}