use egui::{Ui, Color32, RichText};
use crate::app::state::{AppState, ToolType};

pub struct BoneTransformPanel;

impl BoneTransformPanel {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        let store = app.engine.store();
        let center_x = store.canvas_width as f32 / 2.0;
        let center_y = store.canvas_height as f32 / 2.0;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 20.0;

            ui.horizontal(|ui| {
                let is_rotate = app.engine.tool_manager().active_type == ToolType::BoneRotate;
                let btn_color = if is_rotate { Color32::LIGHT_BLUE } else { Color32::GRAY };
                
                if ui.button(RichText::new("⟳").color(btn_color).strong()).clicked() {
                    app.set_tool(ToolType::BoneRotate);
                }
                
                ui.label("旋转");
                if let Some(bone_id) = &app.ui.selected_bone_id {
                    if let Some(bone) = app.animation.project.skeleton.bones.iter_mut().find(|b| b.data.id == *bone_id) {
                        if ui.add(egui::DragValue::new(&mut bone.local_transform.rotation).suffix("°").speed(0.1)).changed() {
                            app.is_dirty = true;
                            app.view.needs_full_redraw = true;
                            app.animation.project.skeleton.update();
                        }
                    }
                } else {
                    ui.label(RichText::new("0.0°").color(Color32::DARK_GRAY));
                }
            });

            ui.horizontal(|ui| {
                let is_trans = app.engine.tool_manager().active_type == ToolType::BoneTranslate;
                let btn_color = if is_trans { Color32::LIGHT_GREEN } else { Color32::GRAY };
                
                if ui.button(RichText::new("Move").color(btn_color).strong()).clicked() {
                    app.set_tool(ToolType::BoneTranslate);
                }

                ui.label("位移");
                if let Some(bone_id) = &app.ui.selected_bone_id {
                    if let Some(bone) = app.animation.project.skeleton.bones.iter_mut().find(|b| b.data.id == *bone_id) {
                        let mut disp_x = bone.local_transform.x - center_x;
                        let mut disp_y = center_y - bone.local_transform.y;

                        ui.label("X:");
                        if ui.add(egui::DragValue::new(&mut disp_x).speed(0.5)).changed() {
                            bone.local_transform.x = disp_x + center_x;
                            app.is_dirty = true;
                            app.view.needs_full_redraw = true;
                            app.animation.project.skeleton.update();
                        }
                        
                        ui.label("Y:");
                        if ui.add(egui::DragValue::new(&mut disp_y).speed(0.5)).changed() {
                            app.is_dirty = true;
                            app.view.needs_full_redraw = true;
                            app.animation.project.skeleton.update();
                        }
                    }
                } else {
                    ui.label(RichText::new("X: 0.0  Y: 0.0").color(Color32::DARK_GRAY));
                }
            });
        });
    }
}