use crate::app::state::AppState;
use crate::app::commands::{AppCommand, ResizeAnchor};
use crate::core::id_gen;
use crate::history::patch::ActionPatch;
use crate::tools::pen::PenTool;
use rust_i18n::t;

pub struct CommandHandler;

impl CommandHandler {
    pub fn execute(app_state: &mut AppState, cmd: AppCommand) {
        match cmd {
            AppCommand::RequestExit => {
                if app_state.is_dirty {
                    app_state.ui.show_exit_modal = true;
                } else {
                    app_state.enqueue_command(AppCommand::ConfirmExit);
                }
            }
            AppCommand::ConfirmExit => app_state.enqueue_command(AppCommand::WindowClose),
            AppCommand::CancelExit => app_state.ui.show_exit_modal = false,
            AppCommand::SaveProject => app_state.save_project_to_pxad(),
            AppCommand::LoadProject => app_state.load_project_from_pxad(),
            AppCommand::ImportImage => app_state.import_image(),
            AppCommand::ExportPng => app_state.export_to_png(),
            
            AppCommand::Undo => {
                if let Err(e) = app_state.engine.undo() { app_state.ui.error_message = Some(e.to_string()); }
                else { app_state.is_dirty = true; app_state.view.needs_full_redraw = true; }
            }
            AppCommand::Redo => {
                if let Err(e) = app_state.engine.redo() { app_state.ui.error_message = Some(e.to_string()); }
                else { app_state.is_dirty = true; app_state.view.needs_full_redraw = true; }
            }
            AppCommand::AddColorToPalette(color) => {
                app_state.engine.add_color_to_palette(color);
                app_state.is_dirty = true;
            }
            AppCommand::RemovePaletteColor(idx) => {
                app_state.engine.remove_palette_color(idx);
                app_state.is_dirty = true;
            }
            AppCommand::ImportPalette => app_state.import_palette(),
            AppCommand::ExportPalette => app_state.export_palette(),
            AppCommand::SetPalette(palette) => {
                app_state.engine.set_palette(palette);
                app_state.is_dirty = true;
            }
            AppCommand::SetPrimaryColor(color) => app_state.engine.set_primary_color(color),
            
            AppCommand::ToggleLayerLock(id) => {
                if let Some(layer) = app_state.engine.store().get_layer(&id) {
                    let old_lock = layer.locked;
                    let patch = ActionPatch::new_layer_lock(id_gen::gen_id(), id.clone(), old_lock, !old_lock);
                    if let Err(e) = app_state.engine.commit_patch(patch) { app_state.ui.error_message = Some(e.to_string()); }
                    else { app_state.is_dirty = true; }
                }
            }
            AppCommand::SetLayerOpacity(id, opacity) => {
                if let Some(layer) = app_state.engine.store().get_layer(&id) {
                    let old_opacity = layer.opacity;
                    if old_opacity != opacity {
                        let patch = ActionPatch::new_layer_opacity(id_gen::gen_id(), id.clone(), old_opacity, opacity);
                        if let Err(e) = app_state.engine.commit_patch(patch) { app_state.ui.error_message = Some(e.to_string()); }
                        else { app_state.is_dirty = true; app_state.view.needs_full_redraw = true; }
                    }
                }
            }
            AppCommand::SetLayerBlendMode(id, mode) => {
                if let Some(layer) = app_state.engine.store().get_layer(&id) {
                    let old_mode = layer.blend_mode;
                    if old_mode != mode {
                        let patch = ActionPatch::new_layer_blend_mode(id_gen::gen_id(), id.clone(), old_mode, mode);
                        if let Err(e) = app_state.engine.commit_patch(patch) { app_state.ui.error_message = Some(e.to_string()); }
                        else { app_state.is_dirty = true; app_state.view.needs_full_redraw = true; }
                    }
                }
            }
            AppCommand::MoveLayerUp(id) => {
                if let Some(idx) = app_state.engine.store().layers.iter().position(|l| l.id == id) {
                    if idx + 1 < app_state.engine.store().layers.len() {
                        let patch = ActionPatch::new_layer_move(id_gen::gen_id(), id.clone(), idx, idx + 1);
                        if let Err(e) = app_state.engine.commit_patch(patch) { app_state.ui.error_message = Some(e.to_string()); }
                        else { app_state.is_dirty = true; }
                    }
                }
            }
            AppCommand::MoveLayerDown(id) => {
                if let Some(idx) = app_state.engine.store().layers.iter().position(|l| l.id == id) {
                    if idx > 0 {
                        let patch = ActionPatch::new_layer_move(id_gen::gen_id(), id.clone(), idx, idx - 1);
                        if let Err(e) = app_state.engine.commit_patch(patch) { app_state.ui.error_message = Some(e.to_string()); }
                        else { app_state.is_dirty = true; app_state.view.needs_full_redraw = true; }
                    }
                }
            }

            AppCommand::MoveLayerToIndex(id, new_idx) => {
                if let Some(old_idx) = app_state.engine.store().layers.iter().position(|l| l.id == id) {
                    if old_idx != new_idx {
                        let target_idx = new_idx.min(app_state.engine.store().layers.len().saturating_sub(1));
                        let patch = ActionPatch::new_layer_move(id_gen::gen_id(), id.clone(), old_idx, target_idx);
                        if let Err(e) = app_state.engine.commit_patch(patch) { app_state.ui.error_message = Some(e.to_string()); }
                        else { app_state.is_dirty = true; app_state.view.needs_full_redraw = true; }
                    }
                }
            }

            AppCommand::RenameLayer(id, new_name) => {
                if let Some(layer) = app_state.engine.store().get_layer(&id) {
                    let trimmed_name = new_name.trim().to_string();
                    if !trimmed_name.is_empty() && layer.name != trimmed_name {
                        let mut final_name = trimmed_name.clone();
                        let mut counter = 1;

                        while app_state.engine.store().layers.iter().any(|l| l.id != id && l.name == final_name) {
                            counter += 1;
                            final_name = format!("{} ({})", trimmed_name, counter);
                        }

                        let patch = ActionPatch::new_layer_rename(id_gen::gen_id(), id.clone(), layer.name.clone(), final_name);
                        if let Err(e) = app_state.engine.commit_patch(patch) { app_state.ui.error_message = Some(e.to_string()); }
                        else { app_state.is_dirty = true; }
                    }
                }
            }

            AppCommand::DuplicateLayer(id) => {
                if let Err(e) = app_state.engine.duplicate_layer(&id) { app_state.ui.error_message = Some(e.to_string()); }
                else { app_state.is_dirty = true; app_state.view.needs_full_redraw = true; }
            }
            AppCommand::MergeSelected(ids) => {
                if let Err(e) = app_state.engine.merge_selected_layers(ids) { app_state.ui.error_message = Some(e.to_string()); }
                else { app_state.is_dirty = true; app_state.view.needs_full_redraw = true; }
            }
            AppCommand::ClearSelection => {
                if app_state.engine.store().selection.is_active {
                    let old = app_state.engine.store().selection.clone();
                    let mut new = old.clone();
                    new.clear();
                    let patch = ActionPatch::new_selection_change(id_gen::gen_id(), old, new);
                    if let Err(e) = app_state.engine.commit_patch(patch) { app_state.ui.error_message = Some(e.to_string()); }
                    else { app_state.is_dirty = true; app_state.view.needs_full_redraw = true; }
                }
            }

            AppCommand::InvertSelection => {
                let old = app_state.engine.store().selection.clone();
                let mut new = old.clone();
                new.invert();
                new.is_active = true;
                let patch = ActionPatch::new_selection_change(id_gen::gen_id(), old, new);
                if let Err(e) = app_state.engine.commit_patch(patch) { app_state.ui.error_message = Some(e.to_string()); }
                else { app_state.is_dirty = true; app_state.view.needs_full_redraw = true; }
            }

            AppCommand::StrokeSelection(width) => {
                let store = app_state.engine.store();
                let layer_id = match &store.active_layer_id { Some(id) => id, None => return };
                let layer = match store.get_layer(layer_id) { Some(l) => l, None => return };
                let mut patch = ActionPatch::new_pixel_diff(id_gen::gen_id(), layer_id.clone());
                let color = store.primary_color;
                
                let sel = &store.selection;
                let range = width as i32;
                let threshold_sq = range * range;
                for y in 0..sel.height {
                    for x in 0..sel.width {
                        if sel.contains(x, y) {
                            let mut is_edge = false;
                            'scan: for dy in -range..=range {
                                for dx in -range..=range {
                                    if dx * dx + dy * dy <= threshold_sq {
                                        let nx = x as i32 + dx;
                                        let ny = y as i32 + dy;
                                        if nx < 0 || ny < 0 || nx >= sel.width as i32 || ny >= sel.height as i32 || !sel.contains(nx as u32, ny as u32) {
                                            is_edge = true;
                                            break 'scan;
                                        }
                                    }
                                }
                            }
                            
                            if is_edge {
                                let lx = x as i32 - layer.offset_x;
                                let ly = y as i32 - layer.offset_y;
                                if lx >= 0 && ly >= 0 && lx < layer.width as i32 && ly < layer.height as i32 {
                                    let old_c = layer.get_pixel(lx as u32, ly as u32).unwrap_or(crate::core::color::Color::transparent());
                                    patch.add_pixel_diff(lx as u32, ly as u32, old_c, color);
                                }
                            }
                        }
                    }
                }
                if let Err(e) = app_state.engine.commit_patch(patch) { app_state.ui.error_message = Some(e.to_string()); }
                else { app_state.is_dirty = true; app_state.view.needs_full_redraw = true; }
            }

            AppCommand::ResizeCanvas(new_w, new_h, anchor) => {
                if new_w == 0 || new_h == 0 {
                    app_state.ui.error_message = Some(t!("error.canvas_size_zero").to_string());
                    return;
                }

                if new_w > 16384 || new_h > 16384 {
                    app_state.ui.error_message = Some(t!("error.canvas_size_limit", max = 16384).to_string());
                    return;
                }

                let old_w = app_state.engine.store().canvas_width;
                let old_h = app_state.engine.store().canvas_height;

                if new_w == old_w && new_h == old_h {
                    return;
                }
                
                let dx = match anchor {
                    ResizeAnchor::TopLeft | ResizeAnchor::MiddleLeft | ResizeAnchor::BottomLeft => 0,
                    ResizeAnchor::TopCenter | ResizeAnchor::Center | ResizeAnchor::BottomCenter => (new_w as i32 - old_w as i32) / 2,
                    ResizeAnchor::TopRight | ResizeAnchor::MiddleRight | ResizeAnchor::BottomRight => new_w as i32 - old_w as i32,
                };
                
                let dy = match anchor {
                    ResizeAnchor::TopLeft | ResizeAnchor::TopCenter | ResizeAnchor::TopRight => 0,
                    ResizeAnchor::MiddleLeft | ResizeAnchor::Center | ResizeAnchor::MiddleRight => (new_h as i32 - old_h as i32) / 2,
                    ResizeAnchor::BottomLeft | ResizeAnchor::BottomCenter | ResizeAnchor::BottomRight => new_h as i32 - old_h as i32,
                };

                let old_layers = app_state.engine.store().layers.clone();
                let old_selection = app_state.engine.store().selection.clone();

                let mut new_layers = old_layers.clone();
                for layer in &mut new_layers {
                    layer.shift_and_resize(dx, dy, new_w, new_h);
                }
                
                let mut new_selection = old_selection.clone();
                new_selection.shift_and_resize(dx, dy, new_w, new_h);

                let patch = ActionPatch::new_canvas_resize(
                    id_gen::gen_id(), old_w, old_h, new_w, new_h,
                    old_layers, new_layers, old_selection, new_selection
                );

                if let Err(e) = app_state.engine.commit_patch(patch) { app_state.ui.error_message = Some(e.to_string()); }
                else { app_state.is_dirty = true; app_state.view.needs_full_redraw = true; }
            }
            AppCommand::CommitCurrentTool => app_state.commit_current_tool(),
            AppCommand::CancelCurrentTool => app_state.cancel_current_tool(),
            AppCommand::SetLanguage(lang) => {
            rust_i18n::set_locale(&lang);
            app_state.ui.language = lang;
        }

        AppCommand::PenFill => {
                if let Some(tool) = app_state.engine.tool_manager().tools.get(&crate::app::state::ToolType::Pen) {
                    if let Some(pen) = tool.as_any().downcast_ref::<PenTool>() {
                        if let Some(patch) = pen.fill(app_state.engine.store()) {
                            if let Err(e) = app_state.engine.commit_patch(patch) { 
                                app_state.ui.error_message = Some(e.to_string()); 
                                } else {
                                app_state.is_dirty = true; 
                                app_state.view.needs_full_redraw = true;
                            }
                        }
                    }
                }
            }

            AppCommand::PenStroke => {
                if let Some(tool) = app_state.engine.tool_manager().tools.get(&crate::app::state::ToolType::Pen) {
                    if let Some(pen) = tool.as_any().downcast_ref::<PenTool>() {
                        if let Some(patch) = pen.stroke(app_state.engine.store()) {
                            if let Err(e) = app_state.engine.commit_patch(patch) { 
                                app_state.ui.error_message = Some(e.to_string()); 
                                } else {
                                app_state.is_dirty = true; 
                                app_state.view.needs_full_redraw = true;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}