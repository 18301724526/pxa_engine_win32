use egui::Ui;
use crate::app::state::{AppState, ToolType};

pub struct ToolbarAnim;

impl ToolbarAnim {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        ui.horizontal(|ui| {
            let mut is_rotate = app.engine.tool_manager().active_type == ToolType::BoneRotate;
            if ui.toggle_value(&mut is_rotate, "⟳ 旋转").clicked() && is_rotate {
                app.set_tool(ToolType::BoneRotate);
            }

            let mut is_move = app.engine.tool_manager().active_type == ToolType::BoneTranslate;
            if ui.toggle_value(&mut is_move, "✥ 移动").clicked() && is_move {
                app.set_tool(ToolType::BoneTranslate);
            }
        });
    }
}