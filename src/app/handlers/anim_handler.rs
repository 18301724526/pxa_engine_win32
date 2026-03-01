use crate::app::state::AppState;
use crate::app::commands::AppCommand;
use crate::animation::history::AnimPatch;

pub fn execute(app_state: &mut AppState, cmd: AppCommand) {
    match cmd {
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
        AppCommand::BindLayerToBone(layer_id, target_bone) => {
            let (bx, by) = app_state.animation.project.skeleton.get_bone_world_position(&target_bone).unwrap_or((0.0, 0.0));

            if let Some(slot) = app_state.animation.project.skeleton.slots.iter_mut().find(|s| s.data.id == layer_id) {
                if slot.data.bone_id != target_bone {
                    let old_bone = slot.data.bone_id.clone();
                    slot.data.bone_id = target_bone.clone();

                    if let Some(layer) = app_state.engine.parts_mut().0.get_layer_mut(&layer_id) {
                        layer.anim_offset_x = 0;
                        layer.anim_offset_y = 0;
                    }
                    app_state.animation.history.commit(AnimPatch::SlotBone { slot_id: layer_id, old_bone, new_bone: target_bone });
                    app_state.is_dirty = true;
                    app_state.view.needs_full_redraw = true;
                    app_state.sync_animation_to_layers();
                }
            }
        }
        AppCommand::DeleteBone(bone_id) => {
            let old_skel = app_state.animation.project.skeleton.clone();
            let parent_id = app_state.animation.project.skeleton.bones.iter()
                .find(|b| b.data.id == bone_id).and_then(|b| b.data.parent_id.clone());
            
            let bind_target = parent_id.unwrap_or_else(|| "root".to_string());
            for slot in &mut app_state.animation.project.skeleton.slots {
                if slot.data.bone_id == bone_id { slot.data.bone_id = bind_target.clone(); }
            }
            for bone in &mut app_state.animation.project.skeleton.bones {
                if bone.data.parent_id.as_deref() == Some(&bone_id) { bone.data.parent_id = Some(bind_target.clone()); }
            }
            if let Some(anim_id) = &app_state.animation.project.active_animation_id {
                if let Some(anim) = app_state.animation.project.animations.get_mut(anim_id) {
                    anim.timelines.retain(|tl| tl.target_id != bone_id);
                }
            }
            app_state.animation.project.skeleton.bones.retain(|b| b.data.id != bone_id);
            app_state.animation.history.commit(AnimPatch::Skeleton { old: old_skel, new: app_state.animation.project.skeleton.clone() });
            
            if app_state.ui.selected_bone_id.as_deref() == Some(&bone_id) {
                app_state.ui.selected_bone_id = None;
            }
            app_state.is_dirty = true;
            app_state.view.needs_full_redraw = true;
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
                let mut min_t = f32::MAX;
                for (_, _, t) in &app_state.ui.selected_keyframes {
                    min_t = min_t.min(*t);
                }
                let actual_dt = if min_t + dt < 0.0 { -min_t } else { dt };
                if actual_dt.abs() < 0.0001 { return; }
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
                        let new_time = *t + actual_dt;
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
        AppCommand::BeginOffsetSnapshot => {
            if let Some(active_id) = &app_state.animation.project.active_animation_id {
                if let Some(anim) = app_state.animation.project.animations.get(active_id) {
                    app_state.ui.offset_snapshot_anim = Some(anim.clone());
                    app_state.ui.offset_snapshot_selection = app_state.ui.selected_keyframes.clone();
                }
            }
        }
        AppCommand::CommitOffsetSnapshot => {
            if let Some(active_id) = &app_state.animation.project.active_animation_id {
                if let Some(old_anim) = app_state.ui.offset_snapshot_anim.take() {
                    if let Some(new_anim) = app_state.animation.project.animations.get(active_id) {
                        let mut patches = Vec::new();
                        for new_tl in &new_anim.timelines {
                            let old_tl = old_anim.timelines.iter().find(|t| t.target_id == new_tl.target_id && t.property == new_tl.property);
                            if old_tl != Some(new_tl) {
                                patches.push(AnimPatch::Timeline {
                                    anim_id: active_id.clone(),
                                    bone_id: new_tl.target_id.clone(),
                                    prop: new_tl.property.clone(),
                                    old: old_tl.cloned(),
                                    new: Some(new_tl.clone()),
                                });
                            }
                        }
                        if !patches.is_empty() { app_state.animation.history.commit(AnimPatch::Composite(patches)); }
                    }
                }
            }
            app_state.ui.offset_snapshot_selection.clear();
        }
        AppCommand::OffsetSelectedKeyframes(total_dt) => {
            let snapshot_anim = match &app_state.ui.offset_snapshot_anim {
                Some(anim) => anim.clone(),
                None => return,
            };
            let snapshot_selection = app_state.ui.offset_snapshot_selection.clone();
            let duration = snapshot_anim.duration;
            
            if duration <= 0.0 {
                app_state.ui.error_message = Some("无法使用偏移：动画时长必须大于0。".to_string());
                return;
            }

            if let Some(active_id) = app_state.animation.project.active_animation_id.clone() {
                let mut current_anim = snapshot_anim.clone();
                let mut new_selection = Vec::new();
                let mut error_msg = None;

                for tl in &mut current_anim.timelines {
                    let selected_times: Vec<f32> = snapshot_selection.iter()
                        .filter(|(b, p, _)| b == &tl.target_id && p.as_ref().map_or(true, |prop| prop == &tl.property))
                        .map(|(_, _, t)| *t).collect();
                    
                    if selected_times.is_empty() { continue; }
                    if selected_times.len() < 3 {
                        error_msg = Some("无法使用偏移：请选中至少三个关键帧以维持循环。".to_string());
                        break;
                    }
                    
                    let first_val = tl.sample(0.0);
                    let last_val = tl.sample(duration);
                    if first_val != last_val {
                        error_msg = Some("无法使用偏移：请确保该骨骼/属性的动画首尾关键帧值相同。".to_string());
                        break;
                    }

                    let sample_time = (0.0 - total_dt).rem_euclid(duration);
                    let sample_val = tl.sample(sample_time);

                    let mut unselected_kfs = Vec::new();
                    let mut moving_kfs = Vec::new();
                    for kf in &tl.keyframes {
                        if selected_times.iter().any(|&st| (st - kf.time).abs() < 0.001) { moving_kfs.push(kf.clone()); } 
                        else { unselected_kfs.push(kf.clone()); }
                    }

                    let mut new_kfs = unselected_kfs;
                    for mut kf in moving_kfs {
                        let new_time = (kf.time + total_dt).rem_euclid(duration);
                        kf.time = new_time;
                        new_selection.push((tl.target_id.clone(), Some(tl.property.clone()), new_time));
                        new_kfs.retain(|existing| (existing.time - new_time).abs() > 0.001);
                        new_kfs.push(kf);
                    }

                    if let Some(val) = sample_val {
                        new_kfs.retain(|k| k.time > 0.001 && (duration - k.time) > 0.001);
                        new_kfs.push(crate::core::animation::timeline::Keyframe { time: 0.0, value: val.clone(), curve: crate::core::animation::timeline::CurveType::Linear });
                        new_kfs.push(crate::core::animation::timeline::Keyframe { time: duration, value: val, curve: crate::core::animation::timeline::CurveType::Linear });
                    }

                    new_kfs.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
                    tl.keyframes = new_kfs;
                }
                if let Some(err) = error_msg { app_state.ui.error_message = Some(err); return; }
                app_state.animation.project.animations.insert(active_id, current_anim);
                app_state.ui.selected_keyframes = new_selection;
                app_state.is_dirty = true;
                app_state.view.needs_full_redraw = true;
            }
        }
        AppCommand::ApplySpineOffset { mode, fixed_frames, step_frames } => {
            if let Some(active_id) = app_state.animation.project.active_animation_id.clone() {
                let anim = match app_state.animation.project.animations.get(&active_id) {
                    Some(a) => a.clone(),
                    None => return,
                };
                let duration = anim.duration;
                if duration <= 0.0 {
                    app_state.ui.error_message = Some("无法使用偏移：动画时长必须大于0。".to_string());
                    return;
                }

                let fps = 30.0;
                let n_sec = fixed_frames as f32 / fps;

                let mut bone_order = Vec::new();
                for (b_id, _, _) in &app_state.ui.selected_keyframes {
                    if !bone_order.contains(b_id) { bone_order.push(b_id.clone()); }
                }

                let mut bone_offsets = std::collections::HashMap::new();
                for (idx, b_id) in bone_order.iter().enumerate() {
                    let step_sec = match mode {
                        1 => 1.0 / fps,
                        2 => step_frames as f32 / fps,
                        _ => 0.0,
                    };
                    bone_offsets.insert(b_id.clone(), n_sec + (idx as f32 * step_sec));
                }

                let mut current_anim = anim.clone();
                let mut new_selection = Vec::new();
                let mut error_msg = None;
                let mut old_tls = Vec::new();

                for tl in &mut current_anim.timelines {
                    let total_dt = match bone_offsets.get(&tl.target_id) {
                        Some(&dt) => dt,
                        None => continue,
                    };

                    if tl.keyframes.is_empty() {
                        continue;
                    }

                    let selected_times: Vec<f32> = app_state.ui.selected_keyframes.iter()
                        .filter(|(b, p, _)| b == &tl.target_id && p.as_ref().map_or(true, |prop| prop == &tl.property))
                        .map(|(_, _, t)| *t).collect();
                    
                    if selected_times.is_empty() { continue; }

                    if let Some(original_tl) = anim.timelines.iter().find(|t| t.target_id == tl.target_id && t.property == tl.property) {
                        old_tls.push(original_tl.clone());
                    }
                    if selected_times.len() < 3 {
                        error_msg = Some("无法使用偏移：请确保受影响的骨骼/属性选中至少三个关键帧以维持循环。".to_string());
                        break;
                    }
                    let first_val = tl.sample(0.0);
                    let last_val = tl.sample(duration);
                    if first_val != last_val {
                        error_msg = Some("无法使用偏移：请确保该骨骼/属性的动画首尾关键帧值相同。".to_string());
                        break;
                    }

                    let sample_time = (0.0 - total_dt).rem_euclid(duration);
                    let sample_val = tl.sample(sample_time);

                    let mut unselected_kfs = Vec::new();
                    let mut moving_kfs = Vec::new();
                    for kf in &tl.keyframes {
                        if selected_times.iter().any(|&st| (st - kf.time).abs() < 0.001) { moving_kfs.push(kf.clone()); } 
                        else { unselected_kfs.push(kf.clone()); }
                    }

                    let mut new_kfs = unselected_kfs;
                    for mut kf in moving_kfs {
                        let new_time = (kf.time + total_dt).rem_euclid(duration);
                        kf.time = new_time;
                        new_selection.push((tl.target_id.clone(), Some(tl.property.clone()), new_time));
                        new_kfs.retain(|existing| (existing.time - new_time).abs() > 0.001);
                        new_kfs.push(kf);
                    }

                    if let Some(val) = sample_val {
                        new_kfs.retain(|k| k.time > 0.001 && (duration - k.time) > 0.001);
                        new_kfs.push(crate::core::animation::timeline::Keyframe { time: 0.0, value: val.clone(), curve: crate::core::animation::timeline::CurveType::Linear });
                        new_kfs.push(crate::core::animation::timeline::Keyframe { time: duration, value: val, curve: crate::core::animation::timeline::CurveType::Linear });
                    }

                    new_kfs.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
                    tl.keyframes = new_kfs;
                }

                if let Some(err) = error_msg { app_state.ui.error_message = Some(err); return; }

                app_state.animation.project.animations.insert(active_id.clone(), current_anim.clone());
                app_state.ui.selected_keyframes = new_selection;
                app_state.is_dirty = true;
                app_state.view.needs_full_redraw = true;
                let mut patches = Vec::new();
                for old_tl in old_tls {
                    let new_tl = current_anim.timelines.iter().find(|t| t.target_id == old_tl.target_id && t.property == old_tl.property).cloned();
                    patches.push(AnimPatch::Timeline { anim_id: active_id.clone(), bone_id: old_tl.target_id.clone(), prop: old_tl.property.clone(), old: Some(old_tl), new: new_tl });
                }
                if !patches.is_empty() { app_state.animation.history.commit(AnimPatch::Composite(patches)); }
            }
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
            app_state.sync_animation_to_layers();
            app_state.is_dirty = true;
        }
        AppCommand::SetTime(time) => {
            app_state.animation.current_time = time.max(0.0);
            crate::animation::controller::AnimationController::apply_current_pose(&mut app_state.animation);
            app_state.sync_animation_to_layers();
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