use egui::Ui;
use crate::app::state::AppState;
use rust_i18n::t;

pub struct ToolbarAnim;

impl ToolbarAnim {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        ui.label(egui::RichText::new(t!("anim.animate_mode")).strong().color(egui::Color32::LIGHT_BLUE));
        ui.add_space(5.0);

        ui.label(t!("tool.rect_select")); 
        ui.label("仅支持骨骼选择");

        ui.add_space(10.0);
        ui.separator();
        ui.label(egui::RichText::new(t!("anim.bone_properties")).size(11.0));
        
        if let Some(bone_id) = &app.ui.selected_bone_id {
            if let Some(bone) = app.animation.project.skeleton.bones.iter_mut().find(|b| b.data.id == *bone_id) {
                ui.label(egui::RichText::new(&bone.data.name).strong());
                
                ui.horizontal(|ui| {
                    ui.label(t!("anim.rotation").to_string());
                    ui.drag_angle(&mut bone.local_transform.rotation);
                });
                ui.horizontal(|ui| {
                    ui.label(t!("anim.position").to_string());
                    ui.add(egui::DragValue::new(&mut bone.local_transform.x).speed(0.1));
                    ui.add(egui::DragValue::new(&mut bone.local_transform.y).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label(t!("anim.scale").to_string());
                    ui.add(egui::DragValue::new(&mut bone.local_transform.scale_x).speed(0.01));
                    ui.add(egui::DragValue::new(&mut bone.local_transform.scale_y).speed(0.01));
                });
            }
        } else {
            ui.label(egui::RichText::new(t!("anim.no_bone_selected")).italics().color(egui::Color32::GRAY));
        }
    }
}