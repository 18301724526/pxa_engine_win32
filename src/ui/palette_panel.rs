use egui::{Ui, Color32, Sense, vec2, Stroke, RichText};
use crate::app::state::AppState;
use crate::app::commands::AppCommand;
use crate::core::color::Color;
use rust_i18n::t;

const ICON_SAVE: &str    = "\u{f0b2}"; 
const ICON_FOLDER: &str  = "\u{ed6f}"; 
const ICON_ADD_COL: &str = "\u{ea10}"; 
const ICON_TRASH: &str   = "\u{ec2a}"; 

pub struct PalettePanel;

impl PalettePanel {
    pub fn show(ui: &mut Ui, app: &mut AppState) {
        ui.vertical(|ui| {
            ui.horizontal_wrapped(|ui| {
                let raw_name = &app.engine.store().palette.name;
                let display_name = if raw_name.is_empty() {
                    t!("palette.title").to_string()
                } else if raw_name == "PICO-8 (默认)" || raw_name == "PICO-8 (Default)" {
                    t!("palette.default_pico8").to_string()
                } else if raw_name == "自定义" || raw_name == "Custom" {
                    t!("palette.default_custom").to_string()
                } else if raw_name == "工程调色板" || raw_name == "Project Palette" {
                    t!("palette.project_palette").to_string()
                } else {
                    raw_name.clone()
                };
                ui.label(RichText::new(display_name).strong().size(14.0));
            });
            
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(ICON_SAVE).on_hover_text(t!("palette.export_hex")).clicked() {
                        app.enqueue_command(AppCommand::ExportPalette);
                    }
                    if ui.button(ICON_FOLDER).on_hover_text(t!("palette.import_hex")).clicked() {
                        app.enqueue_command(AppCommand::ImportPalette);
                    }
                });
            });
            ui.add_space(10.0);

            let mut color_arr = [
                app.engine.store().primary_color.r,
                app.engine.store().primary_color.g,
                app.engine.store().primary_color.b,
            ];
            
            ui.horizontal(|ui| {
                if ui.color_edit_button_srgb(&mut color_arr).changed() {
                    let new_color = Color::new(color_arr[0], color_arr[1], color_arr[2], 255);
                    app.enqueue_command(AppCommand::SetPrimaryColor(new_color));
                }
                
                ui.label(RichText::new(t!("palette.main_color").to_string()).size(11.0).color(Color32::LIGHT_GRAY));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(ICON_ADD_COL).on_hover_text(t!("palette.add_color")).clicked() {
                        let current = Color::new(color_arr[0], color_arr[1], color_arr[2], 255);
                        app.enqueue_command(AppCommand::AddColorToPalette(current));
                    }
                });
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .id_source("palette_scroll")
                .max_height(200.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 4.0);

                    let cloned_colors = app.engine.store().palette.colors.clone();
                    
                    for (i, color) in cloned_colors.into_iter().enumerate() {
                        let egui_color = Color32::from_rgb(color.r, color.g, color.b);
                        let is_selected = app.engine.store().primary_color == color;
                        
                        let (rect, response) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click());
                        
                        let stroke = if is_selected {
                            Stroke::new(2.0, Color32::WHITE)
                        } else if response.hovered() {
                            Stroke::new(1.0, Color32::LIGHT_GRAY)
                        } else {
                            Stroke::new(1.0, Color32::DARK_GRAY)
                        };
                        
                        ui.painter().rect_filled(rect, 2.0, egui_color);
                        ui.painter().rect_stroke(rect, 2.0, stroke);

                        if response.clicked() {
                            app.enqueue_command(AppCommand::SetPrimaryColor(color));
                        }

                        let response = response.on_hover_text(format!("#{:02X}{:02X}{:02X}\n{}", color.r, color.g, color.b, t!("palette.delete_color")));

                        response.context_menu(|ui| {
                            if ui.button(format!("{} {}", ICON_TRASH, t!("palette.delete_btn"))).clicked() {
                                app.enqueue_command(AppCommand::RemovePaletteColor(i));
                                ui.close_menu();
                            }
                        });
                    }
                });
            });
        });
    }
}