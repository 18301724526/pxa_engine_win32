use egui::{Ui, Color32, RichText};
use crate::app::state::AppState;
use crate::app::commands::AppCommand;

pub struct BoneTransformPanel;

impl BoneTransformPanel {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        let store = app.engine.store();
        let center_x = store.canvas_width as f32 / 2.0;
        let center_y = store.canvas_height as f32 / 2.0;
        let mut needs_update = false;

        let mut rot_changed = false;
        let mut pos_changed = false;
        let mut scale_changed = false;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            ui.label(RichText::new("Transform").strong().color(Color32::LIGHT_GRAY));

            let coord_text = if app.ui.show_world_transform { "World" } else { "Local" };
            if ui.button(coord_text).clicked() {
                app.enqueue_command(AppCommand::ToggleTransformCoordinateSystem);
            }
            ui.checkbox(&mut app.ui.auto_keyframe, "Auto-Key");

            if let Some(bone_id) = &app.ui.selected_bone_id {
                if let Some(bone) = app.animation.project.skeleton.bones.iter_mut().find(|b| b.data.id == *bone_id) {
                    
                    ui.separator();
                    ui.label("Rot:");
                    if ui.add(egui::DragValue::new(&mut bone.local_transform.rotation).suffix("°").speed(0.1)).changed() {
                        needs_update = true; rot_changed = true;
                    }

                    ui.separator();
                    let mut disp_x = bone.local_transform.x - center_x;
                    let mut disp_y = center_y - bone.local_transform.y;
                    
                    ui.label("Pos X:");
                    if ui.add(egui::DragValue::new(&mut disp_x).speed(0.5)).changed() {
                        bone.local_transform.x = disp_x + center_x;
                        needs_update = true; pos_changed = true;
                    }
                    
                    ui.label("Y:");
                    if ui.add(egui::DragValue::new(&mut disp_y).speed(0.5)).changed() {
                        bone.local_transform.y = center_y - disp_y;
                        needs_update = true; pos_changed = true;
                    }

                    ui.separator();
                    ui.label("Scale X:");
                    if ui.add(egui::DragValue::new(&mut bone.local_transform.scale_x).speed(0.01)).changed() {
                        needs_update = true; scale_changed = true;
                    }
                    
                    ui.label("Y:");
                    if ui.add(egui::DragValue::new(&mut bone.local_transform.scale_y).speed(0.01)).changed() {
                        needs_update = true; scale_changed = true;
                    }
                }
                if ui.button("Key Frame").clicked() {
                    app.enqueue_command(AppCommand::InsertManualKeyframe(bone_id.clone()));
                }
            } else {
                ui.label(RichText::new("未选中骨骼").color(Color32::DARK_GRAY));
            }
        });

        if needs_update {
            app.is_dirty = true;
            app.view.needs_full_redraw = true;
            app.animation.project.skeleton.update();

            if let Some(bone_id) = &app.ui.selected_bone_id {
                let id = bone_id.clone();
                if app.ui.auto_keyframe {
                    if rot_changed { app.animation.auto_key_bone(&id, crate::core::animation::timeline::TimelineProperty::Rotation); }
                    if pos_changed { app.animation.auto_key_bone(&id, crate::core::animation::timeline::TimelineProperty::Translation); }
                    if scale_changed { app.animation.auto_key_bone(&id, crate::core::animation::timeline::TimelineProperty::Scale); }
                }
            }
        }
    }
}