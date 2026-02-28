use crate::app::state::AppState;
use crate::app::commands::{AppCommand, ResizeAnchor};
use crate::core::id_gen;
use crate::history::patch::ActionPatch;
use crate::tools::pen::PenTool;
use rust_i18n::t;
use crate::app::state::{AnimPatch, AppMode};

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
                if app_state.mode == AppMode::Animation {
                    if app_state.animation.history.undo(&mut app_state.animation.project) {
                        app_state.is_dirty = true; app_state.view.needs_full_redraw = true;
                        crate::animation::controller::AnimationController::apply_current_pose(&mut app_state.animation);
                    }
                } else {
                    if let Err(e) = app_state.engine.undo() { app_state.ui.error_message = Some(e.to_string()); }
                    else { app_state.is_dirty = true; app_state.view.needs_full_redraw = true; }
                }
            }
            AppCommand::Redo => {
                if app_state.mode == AppMode::Animation {
                    if app_state.animation.history.redo(&mut app_state.animation.project) {
                        app_state.is_dirty = true; app_state.view.needs_full_redraw = true;
                        crate::animation::controller::AnimationController::apply_current_pose(&mut app_state.animation);
                    }
                } else {
                    if let Err(e) = app_state.engine.redo() { app_state.ui.error_message = Some(e.to_string()); }
                    else { app_state.is_dirty = true; app_state.view.needs_full_redraw = true; }
                }
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
            AppCommand::CreateAnimation(name) => {
                let id = crate::core::id_gen::gen_id();
                let mut anim = crate::core::animation::timeline::Animation::new(name.clone(), 2.0);
                anim.initialize_tracks(&app_state.animation.project.skeleton);
                app_state.animation.project.animations.insert(id.clone(), anim);
                app_state.animation.project.active_animation_id = Some(id);
                app_state.animation.current_time = 0.0;
                app_state.is_dirty = true;
            }
            AppCommand::SelectAnimation(id) => {
                if app_state.animation.project.animations.contains_key(&id) {
                    app_state.animation.project.active_animation_id = Some(id);
                    app_state.animation.current_time = 0.0;
                    crate::animation::controller::AnimationController::apply_current_pose(&mut app_state.animation);
                    app_state.view.needs_full_redraw = true;
                }
            }

            AppCommand::DeleteKeyframe(bone_id, prop_opt, time) => {
                if let Some(active_id) = app_state.animation.project.active_animation_id.clone() {
                    let mut old_tls = Vec::new();
                    if let Some(anim) = app_state.animation.project.animations.get(&active_id) {
                        for tl in &anim.timelines {
                            if tl.target_id == bone_id {
                                if let Some(ref prop) = prop_opt { if &tl.property != prop { continue; } }
                                old_tls.push(tl.clone());
                            }
                        }
                    }
                    if let Some(anim) = app_state.animation.project.animations.get_mut(&active_id) {
                        for tl in &mut anim.timelines {
                            if tl.target_id == bone_id {
                                if let Some(prop) = &prop_opt {
                                    if &tl.property != prop { continue; }
                                }
                                tl.keyframes.retain(|k| (k.time - time).abs() > 0.001);
                            }
                        }
                        anim.recalculate_duration();
                        app_state.is_dirty = true;
                        app_state.view.needs_full_redraw = true;
                        crate::animation::controller::AnimationController::apply_current_pose(&mut app_state.animation);
                    }
                    let mut patches = Vec::new();
                    for old_tl in old_tls {
                        let new_tl = app_state.animation.project.animations.get(&active_id)
                            .and_then(|a| a.timelines.iter().find(|t| t.target_id == old_tl.target_id && t.property == old_tl.property))
                            .cloned();
                        patches.push(AnimPatch::Timeline { anim_id: active_id.clone(), bone_id: old_tl.target_id.clone(), prop: old_tl.property.clone(), old: Some(old_tl), new: new_tl });
                    }
                    if !patches.is_empty() { app_state.animation.history.commit(AnimPatch::Composite(patches)); }
                }
            }

            AppCommand::UpdateKeyframeCurve(bone_id, prop, time, curve) => {
                if let Some(active_id) = app_state.animation.project.active_animation_id.clone() {
                    let old_tl = app_state.animation.project.animations.get(&active_id)
                        .and_then(|a| a.timelines.iter().find(|t| t.target_id == bone_id && t.property == prop)).cloned();
                    if let Some(anim) = app_state.animation.project.animations.get_mut(&active_id) {
                        for tl in &mut anim.timelines {
                            if tl.target_id == bone_id && tl.property == prop {
                                if let Some(kf) = tl.keyframes.iter_mut().find(|k| (k.time - time).abs() < 0.001) {
                                    kf.curve = curve;
                                }
                            }
                        }
                        app_state.is_dirty = true;
                        app_state.view.needs_full_redraw = true;
                    }
                    let new_tl = app_state.animation.project.animations.get(&active_id)
                        .and_then(|a| a.timelines.iter().find(|t| t.target_id == bone_id && t.property == prop)).cloned();
                    if let (Some(old), Some(new)) = (old_tl, new_tl) {
                        app_state.animation.history.commit(AnimPatch::Timeline { anim_id: active_id, bone_id, prop, old: Some(old), new: Some(new) });
                    }
                }
            }
            AppCommand::MoveSelectedKeyframes(dt) => {
                if let Some(active_id) = app_state.animation.project.active_animation_id.clone() {
                    let mut old_tls = Vec::new();
                    if let Some(anim) = app_state.animation.project.animations.get(&active_id) {
                        for (bone_id, prop_opt, _) in &app_state.ui.selected_keyframes {
                            if let Some(tl) = anim.timelines.iter().find(|t| &t.target_id == bone_id && prop_opt.as_ref().map_or(true, |p| &t.property == p)) {
                                if !old_tls.iter().any(|existing: &crate::core::animation::timeline::Timeline| existing.target_id == tl.target_id && existing.property == tl.property) {
                                    old_tls.push(tl.clone());
                                }
                            }
                        }
                    }
                    if let Some(anim) = app_state.animation.project.animations.get_mut(&active_id) {
                        let mut new_selection = Vec::new();
                        for (bone_id, prop_opt, t) in &app_state.ui.selected_keyframes {
                            let new_time = (*t + dt).max(0.0);
                            new_selection.push((bone_id.clone(), prop_opt.clone(), new_time));
                            for tl in &mut anim.timelines {
                                if &tl.target_id == bone_id {
                                    if let Some(prop) = prop_opt { if &tl.property != prop { continue; } }
                                    if let Some(kf) = tl.keyframes.iter_mut().find(|k| (k.time - *t).abs() < 0.001) { kf.time = new_time; }
                                }
                            }
                        }
                        for tl in &mut anim.timelines {
                            tl.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
                        }
                        anim.recalculate_duration();
                        app_state.ui.selected_keyframes = new_selection;
                        app_state.is_dirty = true;
                    }
                    let mut patches = Vec::new();
                    for old_tl in old_tls {
                        let new_tl = app_state.animation.project.animations.get(&active_id)
                            .and_then(|a| a.timelines.iter().find(|t| t.target_id == old_tl.target_id && t.property == old_tl.property))
                            .cloned();
                        patches.push(AnimPatch::Timeline { anim_id: active_id.clone(), bone_id: old_tl.target_id.clone(), prop: old_tl.property.clone(), old: Some(old_tl), new: new_tl });
                    }
                    if !patches.is_empty() { app_state.animation.history.commit(AnimPatch::Composite(patches)); }
                }
            }
            AppCommand::ApplySpineOffset { mode, fixed_frames, step_frames } => {
                if let Some(active_id) = app_state.animation.project.active_animation_id.clone() {
                    let mut old_tls = Vec::new();
                    if let Some(anim) = app_state.animation.project.animations.get(&active_id) {
                        for (bone_id, prop_opt, _) in &app_state.ui.selected_keyframes {
                            if let Some(tl) = anim.timelines.iter().find(|t| &t.target_id == bone_id && prop_opt.as_ref().map_or(true, |p| &t.property == p)) {
                                if !old_tls.iter().any(|existing: &crate::core::animation::timeline::Timeline| existing.target_id == tl.target_id && existing.property == tl.property) { 
                                    old_tls.push(tl.clone()); 
                                }
                            }
                        }
                    }

                    let fps = 30.0;
                    let n_sec = fixed_frames as f32 / fps;

                    if let Some(anim) = app_state.animation.project.animations.get_mut(&active_id) {
                        let duration = anim.duration;
                        if duration <= 0.0 { return; }

                        let mut bone_offsets = std::collections::HashMap::new();
                        let mut bone_order = Vec::new();
                        for (b_id, _, _) in &app_state.ui.selected_keyframes {
                            if !bone_order.contains(b_id) { bone_order.push(b_id.clone()); }
                        }
                        for (idx, b_id) in bone_order.iter().enumerate() {
                            let step_sec = match mode {
                                1 => 1.0 / fps,
                                2 => step_frames as f32 / fps,
                                _ => 0.0,
                            };
                            bone_offsets.insert(b_id.clone(), n_sec + (idx as f32 * step_sec));
                        }

                        let mut new_selection = Vec::new();
                        let selected_copy = app_state.ui.selected_keyframes.clone();
                        
                        for tl in &mut anim.timelines {
                            if let Some(&dt) = bone_offsets.get(&tl.target_id) {
                                let affected_times: Vec<f32> = selected_copy.iter()
                                    .filter(|(b, p, _)| b == &tl.target_id && p.as_ref().map_or(true, |prop| prop == &tl.property))
                                    .map(|(_, _, t)| *t).collect();
                                
                                if affected_times.is_empty() { continue; }

                                let wrap_source_time = (0.0 - dt).rem_euclid(duration);
                                let wrap_value = tl.sample(wrap_source_time);

                                for kf in &mut tl.keyframes {
                                    if affected_times.iter().any(|&at| (at - kf.time).abs() < 0.001) {
                                        kf.time = (kf.time + dt).rem_euclid(duration);
                                        new_selection.push((tl.target_id.clone(), Some(tl.property.clone()), kf.time));
                                    }
                                }

                                if let Some(val) = wrap_value {
                                    tl.add_keyframe(0.0, val.clone(), crate::core::animation::timeline::CurveType::Linear);
                                    tl.add_keyframe(duration, val, crate::core::animation::timeline::CurveType::Linear);
                                }
                            }
                        }
                        for tl in &mut anim.timelines {
                            tl.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
                        }
                        app_state.ui.selected_keyframes = new_selection;
                        app_state.is_dirty = true;
                        app_state.view.needs_full_redraw = true;
                    }
                    let mut patches = Vec::new();
                    for old_tl in old_tls {
                        let new_tl = app_state.animation.project.animations.get(&active_id)
                            .and_then(|a| a.timelines.iter().find(|t| t.target_id == old_tl.target_id && t.property == old_tl.property))
                            .cloned();
                        patches.push(AnimPatch::Timeline { anim_id: active_id.clone(), bone_id: old_tl.target_id.clone(), prop: old_tl.property.clone(), old: Some(old_tl), new: new_tl });
                    }
                    if !patches.is_empty() { app_state.animation.history.commit(AnimPatch::Composite(patches)); }
                }
            }
            AppCommand::ChangeBrushSize(delta) => {
                let (size, _, _) = app_state.engine.brush_settings_mut();
                *size = (*size as i32 + delta).clamp(1, 20) as u32;
            }
            AppCommand::SelectTool(tool_type) => {
                app_state.ui.active_select_tool = tool_type;
                app_state.set_tool(tool_type);
            }
            AppCommand::TogglePathNodeType(idx) => {
                let old_path = app_state.engine.store().active_path.clone();
                let mut new_path = old_path.clone();
                if let Some(node) = new_path.nodes.get_mut(idx) {
                    node.kind = match node.kind {
                        crate::core::path::NodeType::Corner => crate::core::path::NodeType::Smooth,
                        crate::core::path::NodeType::Smooth => crate::core::path::NodeType::Corner,
                    };
                    if node.kind == crate::core::path::NodeType::Smooth {
                        node.handle_in = node.handle_out * -1.0;
                    }
                    let patch = ActionPatch::new_path_change(id_gen::gen_id(), old_path, new_path);
                    if let Err(e) = app_state.engine.commit_patch(patch) { app_state.ui.error_message = Some(e.to_string()); }
                    else { app_state.is_dirty = true; app_state.view.needs_full_redraw = true; }
                }
            }
            AppCommand::DeletePathNode(idx) => {
                let old_path = app_state.engine.store().active_path.clone();
                let mut new_path = old_path.clone();
                if idx < new_path.nodes.len() {
                    new_path.nodes.remove(idx);
                    let patch = ActionPatch::new_path_change(id_gen::gen_id(), old_path, new_path);
                    if let Err(e) = app_state.engine.commit_patch(patch) { app_state.ui.error_message = Some(e.to_string()); }
                    else { app_state.is_dirty = true; app_state.view.needs_full_redraw = true; }
                }
            }
            AppCommand::ToggleTransformCoordinateSystem => {
                app_state.ui.show_world_transform = !app_state.ui.show_world_transform;
            }
            AppCommand::InsertManualKeyframe(bone_id) => {
                if let Some(active_id) = app_state.animation.project.active_animation_id.clone() {
                    let mut patches = Vec::new();
                    let props = [
                        crate::core::animation::timeline::TimelineProperty::Translation,
                        crate::core::animation::timeline::TimelineProperty::Rotation,
                        crate::core::animation::timeline::TimelineProperty::Scale,
                    ];

                    for prop in props {
                        let old_tl = app_state.animation.project.animations.get(&active_id)
                            .and_then(|a| a.timelines.iter().find(|t| t.target_id == bone_id && t.property == prop))
                            .cloned();

                        app_state.animation.auto_key_bone(&bone_id, prop.clone());

                        let new_tl = app_state.animation.project.animations.get(&active_id)
                            .and_then(|a| a.timelines.iter().find(|t| t.target_id == bone_id && t.property == prop))
                            .cloned();

                        patches.push(AnimPatch::Timeline {
                            anim_id: active_id.clone(),
                            bone_id: bone_id.clone(),
                            prop,
                            old: old_tl,
                            new: new_tl,
                        });
                    }
                    app_state.animation.history.commit(AnimPatch::Composite(patches));
                    app_state.is_dirty = true;
                    app_state.view.needs_full_redraw = true;
                }
            }
            AppCommand::TogglePlayback => app_state.animation.is_playing = !app_state.animation.is_playing,
            AppCommand::StepFrame(frames) => {
                app_state.animation.current_time = (app_state.animation.current_time + frames as f32 / 30.0).max(0.0);
                crate::animation::controller::AnimationController::apply_current_pose(&mut app_state.animation);
                app_state.is_dirty = true;
            }
            AppCommand::SetTime(time) => {
                app_state.animation.current_time = time.max(0.0);
                crate::animation::controller::AnimationController::apply_current_pose(&mut app_state.animation);
                app_state.is_dirty = true;
            }
            AppCommand::SetPlaybackSpeed(speed) => app_state.animation.playback_speed = speed.max(0.1),
            AppCommand::ToggleLoop => app_state.animation.is_looping = !app_state.animation.is_looping,
            AppCommand::ToggleTimelineFilter(prop) => {
                if let Some(pos) = app_state.ui.timeline_filter.iter().position(|p| p == &prop) {
                    app_state.ui.timeline_filter.remove(pos);
                } else {
                    app_state.ui.timeline_filter.push(prop);
                }
            }
            _ => {}
        }
    }
}