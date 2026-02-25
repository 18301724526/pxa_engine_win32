use egui::{Ui, Slider};
use crate::app::state::AppState;
use crate::core::symmetry::SymmetryMode;
use rust_i18n::t;

pub struct SymmetryPanel;

impl SymmetryPanel {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        ui.add_space(5.0);
        ui.heading(t!("symmetry.title").to_string());
        ui.separator();

        ui.horizontal(|ui| {
            ui.label(t!("symmetry.mode").to_string());
            egui::ComboBox::from_id_source("sym_mode")
                .selected_text(match app.engine.symmetry().mode {
                    SymmetryMode::None => t!("symmetry.none").to_string(),
                    SymmetryMode::Horizontal => t!("symmetry.mirror_x").to_string(),
                    SymmetryMode::Vertical => t!("symmetry.mirror_y").to_string(),
                    SymmetryMode::Quad => t!("symmetry.quad").to_string(),
                    SymmetryMode::Translational => t!("symmetry.translate").to_string(),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut app.engine.symmetry_mut().mode, SymmetryMode::None, t!("symmetry.none").to_string());
                    ui.selectable_value(&mut app.engine.symmetry_mut().mode, SymmetryMode::Horizontal, t!("symmetry.mirror_x").to_string());
                    ui.selectable_value(&mut app.engine.symmetry_mut().mode, SymmetryMode::Vertical, t!("symmetry.mirror_y").to_string());
                    ui.selectable_value(&mut app.engine.symmetry_mut().mode, SymmetryMode::Quad, t!("symmetry.quad").to_string());
                    ui.selectable_value(&mut app.engine.symmetry_mut().mode, SymmetryMode::Translational, t!("symmetry.translate").to_string());
                });
        });

        if app.engine.symmetry().mode != SymmetryMode::None {
            ui.checkbox(&mut app.engine.symmetry_mut().visible_guides, t!("symmetry.show_guides").to_string());
            
            match app.engine.symmetry().mode {
                SymmetryMode::Horizontal | SymmetryMode::Quad => {
                    let max_x = app.engine.store().canvas_width as f32;
                    ui.label(t!("symmetry.axis_x").to_string());
                    ui.add(Slider::new(&mut app.engine.symmetry_mut().axis_x, 0.0..=max_x).step_by(0.5));
                    if ui.button(t!("symmetry.center_x").to_string()).clicked() {
                        app.engine.symmetry_mut().axis_x = max_x / 2.0;
                    }
                },
                _ => {}
            }

            match app.engine.symmetry().mode {
                SymmetryMode::Vertical | SymmetryMode::Quad => {
                    let max_y = app.engine.store().canvas_height as f32;
                    ui.label(t!("symmetry.axis_y").to_string());
                    ui.add(Slider::new(&mut app.engine.symmetry_mut().axis_y, 0.0..=max_y).step_by(0.5));
                    if ui.button(t!("symmetry.center_y").to_string()).clicked() {
                        app.engine.symmetry_mut().axis_y = max_y / 2.0;
                    }
                },
                _ => {}
            }

            if app.engine.symmetry().mode == SymmetryMode::Translational {
                ui.label(t!("symmetry.offset_x").to_string());
                ui.add(Slider::new(&mut app.engine.symmetry_mut().translation_dx, -100..=100));
                ui.label(t!("symmetry.offset_y").to_string());
                ui.add(Slider::new(&mut app.engine.symmetry_mut().translation_dy, -100..=100));
            }
        }
    }
}