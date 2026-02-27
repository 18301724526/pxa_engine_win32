mod toolbar;
mod dopesheet;
mod curve_editor;
mod offset_modal;

use egui::Ui;
use crate::app::state::AppState;

pub struct TimelinePanel;

impl TimelinePanel {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        toolbar::Toolbar::show(ui, app);
        ui.separator();

        dopesheet::Dopesheet::show(ui, app);

        curve_editor::CurveEditor::show(ui, app);

        offset_modal::OffsetModal::show(ui, app);
    }
}