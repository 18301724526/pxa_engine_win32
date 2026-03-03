use crate::app::state::{AppState, AppMode};
use crate::history::patch::ActionPatch;
use crate::core::blend_mode::BlendMode;

pub fn toggle_layer_lock(app_state: &mut AppState, id: &str) -> Result<(), String> {
    if let Some(layer) = app_state.pixel.engine.store().get_layer(id) {
        let old_lock = layer.locked;
        let patch = ActionPatch::new_layer_lock(app_state.pixel.engine.id_gen().generate(), id.to_string(), old_lock, !old_lock);
        app_state.pixel.engine.commit_patch(patch).map_err(|e| e.to_string())?;
        app_state.is_dirty = true;
    }
    Ok(())
}

pub fn set_layer_opacity(app_state: &mut AppState, id: &str, opacity: u8) -> Result<(), String> {
    if let Some(layer) = app_state.pixel.engine.store().get_layer(id) {
        let old_opacity = layer.opacity;
        if old_opacity != opacity {
            let patch = ActionPatch::new_layer_opacity(app_state.pixel.engine.id_gen().generate(), id.to_string(), old_opacity, opacity);
            app_state.pixel.engine.commit_patch(patch).map_err(|e| e.to_string())?;
            app_state.is_dirty = true;
            app_state.pixel.view.needs_full_redraw = true;
        }
    }
    Ok(())
}

pub fn set_layer_blend_mode(app_state: &mut AppState, id: &str, mode: BlendMode) -> Result<(), String> {
    if let Some(layer) = app_state.pixel.engine.store().get_layer(id) {
        let old_mode = layer.blend_mode;
        if old_mode != mode {
            let patch = ActionPatch::new_layer_blend_mode(app_state.pixel.engine.id_gen().generate(), id.to_string(), old_mode, mode);
            app_state.pixel.engine.commit_patch(patch).map_err(|e| e.to_string())?;
            app_state.is_dirty = true;
            app_state.pixel.view.needs_full_redraw = true;
        }
    }
    Ok(())
}

pub fn move_layer_up(app_state: &mut AppState, id: &str) -> Result<(), String> {
    if app_state.mode == AppMode::Animation {
        return Err("动画模式下禁止调整图层顺序".into());
    }
    if let Some(idx) = app_state.pixel.engine.store().layers.iter().position(|l| l.id == id) {
        if idx + 1 < app_state.pixel.engine.store().layers.len() {
            let patch = ActionPatch::new_layer_move(app_state.pixel.engine.id_gen().generate(), id.to_string(), idx, idx + 1);
            app_state.pixel.engine.commit_patch(patch).map_err(|e| e.to_string())?;
            app_state.is_dirty = true;
        }
    }
    Ok(())
}

pub fn move_layer_down(app_state: &mut AppState, id: &str) -> Result<(), String> {
    if app_state.mode == AppMode::Animation {
        return Err("动画模式下禁止调整图层顺序".into());
    }
    if let Some(idx) = app_state.pixel.engine.store().layers.iter().position(|l| l.id == id) {
        if idx > 0 {
            let patch = ActionPatch::new_layer_move(app_state.pixel.engine.id_gen().generate(), id.to_string(), idx, idx - 1);
            app_state.pixel.engine.commit_patch(patch).map_err(|e| e.to_string())?;
            app_state.is_dirty = true;
            app_state.pixel.view.needs_full_redraw = true;
        }
    }
    Ok(())
}

pub fn move_layer_to_index(app_state: &mut AppState, id: &str, new_idx: usize) -> Result<(), String> {
    if app_state.mode == AppMode::Animation {
        return Err("动画模式下禁止调整图层顺序".into());
    }
    if let Some(old_idx) = app_state.pixel.engine.store().layers.iter().position(|l| l.id == id) {
        if old_idx != new_idx {
            let target_idx = new_idx.min(app_state.pixel.engine.store().layers.len().saturating_sub(1));
            let patch = ActionPatch::new_layer_move(app_state.pixel.engine.id_gen().generate(), id.to_string(), old_idx, target_idx);
            app_state.pixel.engine.commit_patch(patch).map_err(|e| e.to_string())?;
            app_state.is_dirty = true;
            app_state.pixel.view.needs_full_redraw = true;
        }
    }
    Ok(())
}

pub fn rename_layer(app_state: &mut AppState, id: &str, new_name: &str) -> Result<(), String> {
    if app_state.mode == AppMode::Animation {
        return Err("动画模式下禁止重命名图层".into());
    }
    if let Some(layer) = app_state.pixel.engine.store().get_layer(id) {
        let trimmed_name = new_name.trim().to_string();
        if !trimmed_name.is_empty() && layer.name != trimmed_name {
            let mut final_name = trimmed_name.clone();
            let mut counter = 1;
            while app_state.pixel.engine.store().layers.iter().any(|l| l.id != id && l.name == final_name) {
                counter += 1;
                final_name = format!("{} ({})", trimmed_name, counter);
            }
            let patch = ActionPatch::new_layer_rename(app_state.pixel.engine.id_gen().generate(), id.to_string(), layer.name.clone(), final_name);
            app_state.pixel.engine.commit_patch(patch).map_err(|e| e.to_string())?;
            app_state.is_dirty = true;
        }
    }
    Ok(())
}

pub fn duplicate_layer(app_state: &mut AppState, id: &str) -> Result<(), String> {
    if app_state.mode == AppMode::Animation {
        return Err("动画模式下禁止复制图层".into());
    }
    app_state.pixel.engine.duplicate_layer(id).map_err(|e| e.to_string())?;
    app_state.is_dirty = true;
    app_state.pixel.view.needs_full_redraw = true;
    Ok(())
}

pub fn merge_selected(app_state: &mut AppState, ids: Vec<String>) -> Result<(), String> {
    if app_state.mode == AppMode::Animation {
        return Err("动画模式下禁止合并图层".into());
    }
    app_state.pixel.engine.merge_selected_layers(ids).map_err(|e| e.to_string())?;
    app_state.is_dirty = true;
    app_state.pixel.view.needs_full_redraw = true;
    Ok(())
}