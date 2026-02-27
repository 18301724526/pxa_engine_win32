use egui::{Ui, vec2};
use crate::app::state::AppState;
use crate::app::commands::AppCommand;

pub struct OffsetModal;

impl OffsetModal {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        if app.ui.show_offset_modal {
            egui::Window::new("自动偏移 (Auto Offset)")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    ui.radio_value(&mut app.ui.offset_mode, 0, "固定偏移 (全体移动 N 帧)");
                    ui.radio_value(&mut app.ui.offset_mode, 1, "基础递增 (N + 1 帧)");
                    ui.radio_value(&mut app.ui.offset_mode, 2, "自定义递增 (N + M 帧)");

                    ui.horizontal(|ui| {
                        ui.label("初始偏移(N):");
                        ui.add(egui::DragValue::new(&mut app.ui.offset_fixed_frames).suffix(" 帧"));
                    });

                    if app.ui.offset_mode == 2 {
                        ui.horizontal(|ui| {
                            ui.label("递增步长(M):");
                            ui.add(egui::DragValue::new(&mut app.ui.offset_step_frames).suffix(" 帧"));
                        });
                    }

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("✅ 应用").clicked() {
                            app.enqueue_command(AppCommand::ApplySpineOffset {
                                mode: app.ui.offset_mode,
                                fixed_frames: app.ui.offset_fixed_frames,
                                step_frames: app.ui.offset_step_frames,
                            });
                            app.ui.show_offset_modal = false;
                        }
                        if ui.button("❌ 取消").clicked() { app.ui.show_offset_modal = false; }
                    });
                });
        }
    }
}