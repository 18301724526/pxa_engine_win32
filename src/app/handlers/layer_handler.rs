use crate::app::state::AppState;
use crate::app::commands::AppCommand;
use crate::core::id_gen;
use crate::history::patch::ActionPatch;

pub fn execute(app_state: &mut AppState, cmd: AppCommand) {
    match cmd {
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
        _ => {}
    }
}