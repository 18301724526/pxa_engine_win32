mod toolbar;
mod dopesheet;
mod curve_editor;
mod offset_modal;

use egui::Ui;
use crate::app::state::AppState;

pub struct TimelinePanel;

impl TimelinePanel {
    pub fn show(ui: &mut Ui, app: &mut AppState, ui_ctx: &mut crate::app::ui_context::UiContext) {
        toolbar::Toolbar::show(ui, app, ui_ctx);
        ui.separator();

        dopesheet::Dopesheet::show(ui, app, ui_ctx);

        curve_editor::CurveEditor::show(ui, app, ui_ctx);

        offset_modal::OffsetModal::show(ui, app, ui_ctx);
    }
}