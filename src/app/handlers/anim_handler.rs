use crate::app::state::AppState;
use crate::animation::history::AnimPatch;
use crate::core::animation::timeline::{TimelineProperty, CurveType};

pub fn create_animation(app_state: &mut AppState, name: &str) -> Result<(), String> {
    let id = app_state.pixel.engine.id_gen().generate();
    let mut anim = crate::core::animation::timeline::Animation::new(name.to_string(), 2.0);
    anim.initialize_tracks(&app_state.anim.state.project.skeleton);
    app_state.anim.state.project.animations.insert(id.clone(), anim);
    app_state.anim.state.project.active_animation_id = Some(id);
    app_state.anim.state.current_time = 0.0;
    app_state.is_dirty = true;
    Ok(())
}

pub fn select_animation(app_state: &mut AppState, id: &str) -> Result<(), String> {
    if app_state.anim.state.project.animations.contains_key(id) {
        app_state.anim.state.project.active_animation_id = Some(id.to_string());
        app_state.anim.state.current_time = 0.0;
        crate::animation::controller::AnimationController::apply_current_pose(&mut app_state.anim.state);
        app_state.pixel.view.needs_full_redraw = true;
    }
    Ok(())
}

pub fn delete_keyframe(app_state: &mut AppState, bone_id: &str, prop_opt: Option<TimelineProperty>, time: f32) -> Result<(), String> {
    if let Some(active_id) = app_state.anim.state.project.active_animation_id.clone() {
        let mut old_tls = Vec::new();
        if let Some(anim) = app_state.anim.state.project.animations.get(&active_id) {
            for tl in &anim.timelines {
                if tl.target_id == bone_id {
                    if let Some(ref prop) = prop_opt { if &tl.property != prop { continue; } }
                    old_tls.push(tl.clone());
                }
            }
        }
        if let Some(anim) = app_state.anim.state.project.animations.get_mut(&active_id) {
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
            app_state.pixel.view.needs_full_redraw = true;
            crate::animation::controller::AnimationController::apply_current_pose(&mut app_state.anim.state);
        }
        let mut patches = Vec::new();
        for old_tl in old_tls {
            let new_tl = app_state.anim.state.project.animations.get(&active_id)
                .and_then(|a| a.timelines.iter().find(|t| t.target_id == old_tl.target_id && t.property == old_tl.property))
                .cloned();
            patches.push(AnimPatch::Timeline { anim_id: active_id.clone(), bone_id: old_tl.target_id.clone(), prop: old_tl.property.clone(), old: Some(old_tl), new: new_tl });
        }
        if !patches.is_empty() { app_state.anim.state.history.commit(AnimPatch::Composite(patches)); }
    }
    Ok(())
}

pub fn update_keyframe_curve(app_state: &mut AppState, bone_id: &str, prop: TimelineProperty, time: f32, curve: CurveType) -> Result<(), String> {
    if let Some(active_id) = app_state.anim.state.project.active_animation_id.clone() {
        let old_tl = app_state.anim.state.project.animations.get(&active_id)
            .and_then(|a| a.timelines.iter().find(|t| t.target_id == bone_id && t.property == prop)).cloned();
        if let Some(anim) = app_state.anim.state.project.animations.get_mut(&active_id) {
            for tl in &mut anim.timelines {
                if tl.target_id == bone_id && tl.property == prop {
                    if let Some(kf) = tl.keyframes.iter_mut().find(|k| (k.time - time).abs() < 0.001) {
                        kf.curve = curve;
                    }
                }
            }
            app_state.is_dirty = true;
            app_state.pixel.view.needs_full_redraw = true;
        }
        let new_tl = app_state.anim.state.project.animations.get(&active_id)
            .and_then(|a| a.timelines.iter().find(|t| t.target_id == bone_id && t.property == prop)).cloned();
        if let (Some(old), Some(new)) = (old_tl, new_tl) {
            app_state.anim.state.history.commit(AnimPatch::Timeline { anim_id: active_id, bone_id: bone_id.to_string(), prop, old: Some(old), new: Some(new) });
        }
    }
    Ok(())
}

pub fn move_selected_keyframes(app_state: &mut AppState, dt: f32) -> Result<(), String> {
    if let Some(active_id) = app_state.anim.state.project.active_animation_id.clone() {
        let mut old_tls = Vec::new();
        let mut min_t = f32::MAX;
        for (_, _, t) in &app_state.anim.selected_keyframes {
            min_t = min_t.min(*t);
        }
        let actual_dt = if min_t + dt < 0.0 { -min_t } else { dt };
        if actual_dt.abs() < 0.0001 { return Ok(()); }
        if let Some(anim) = app_state.anim.state.project.animations.get(&active_id) {
            for (bone_id, prop_opt, _) in &app_state.anim.selected_keyframes {
                if let Some(tl) = anim.timelines.iter().find(|t| &t.target_id == bone_id && prop_opt.as_ref().map_or(true, |p| &t.property == p)) {
                    if !old_tls.iter().any(|existing: &crate::core::animation::timeline::Timeline| existing.target_id == tl.target_id && existing.property == tl.property) {
                        old_tls.push(tl.clone());
                    }
                }
            }
        }
        if let Some(anim) = app_state.anim.state.project.animations.get_mut(&active_id) {
            let mut new_selection = Vec::new();
            for (bone_id, prop_opt, t) in &app_state.anim.selected_keyframes {
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
            app_state.anim.selected_keyframes = new_selection;
            app_state.is_dirty = true;
        }
        let mut patches = Vec::new();
        for old_tl in old_tls {
            let new_tl = app_state.anim.state.project.animations.get(&active_id)
                .and_then(|a| a.timelines.iter().find(|t| t.target_id == old_tl.target_id && t.property == old_tl.property))
                .cloned();
            patches.push(AnimPatch::Timeline { anim_id: active_id.clone(), bone_id: old_tl.target_id.clone(), prop: old_tl.property.clone(), old: Some(old_tl), new: new_tl });
        }
        if !patches.is_empty() { app_state.anim.state.history.commit(AnimPatch::Composite(patches)); }
    }
    Ok(())
}

pub fn insert_manual_keyframe(app_state: &mut AppState, bone_id: &str) -> Result<(), String> {
    if let Some(active_id) = app_state.anim.state.project.active_animation_id.clone() {
        let mut patches = Vec::new();
        let props = [
            crate::core::animation::timeline::TimelineProperty::Translation,
            crate::core::animation::timeline::TimelineProperty::Rotation,
            crate::core::animation::timeline::TimelineProperty::Scale,
        ];

        for prop in props {
            let old_tl = app_state.anim.state.project.animations.get(&active_id)
                .and_then(|a| a.timelines.iter().find(|t| t.target_id == bone_id && t.property == prop))
                .cloned();

            app_state.anim.state.auto_key_bone(bone_id, prop.clone());

            let new_tl = app_state.anim.state.project.animations.get(&active_id)
                .and_then(|a| a.timelines.iter().find(|t| t.target_id == bone_id && t.property == prop))
                .cloned();

            patches.push(AnimPatch::Timeline {
                anim_id: active_id.clone(),
                bone_id: bone_id.to_string(),
                prop,
                old: old_tl,
                new: new_tl,
            });
        }
        app_state.anim.state.history.commit(AnimPatch::Composite(patches));
        app_state.is_dirty = true;
        app_state.pixel.view.needs_full_redraw = true;
    }
    Ok(())
}

pub fn step_frame(app_state: &mut AppState, frames: i32) -> Result<(), String> {
    let dt = frames as f32 / app_state.anim.fps; // 使用动态 FPS
    app_state.anim.state.current_time = (app_state.anim.state.current_time + dt).max(0.0);
    crate::animation::controller::AnimationController::apply_current_pose(&mut app_state.anim.state);
    app_state.sync_animation_to_layers();
    app_state.is_dirty = true;
    Ok(())
}

pub fn set_time(app_state: &mut AppState, time: f32) -> Result<(), String> {
    app_state.anim.state.current_time = time.max(0.0);
    crate::animation::controller::AnimationController::apply_current_pose(&mut app_state.anim.state);
    app_state.sync_animation_to_layers();
    app_state.is_dirty = true;
    Ok(())
}

pub fn toggle_loop(app_state: &mut AppState) -> Result<(), String> {
    app_state.anim.state.is_looping = !app_state.anim.state.is_looping;
    Ok(())
}

pub fn toggle_timeline_filter(app_state: &mut AppState, prop: TimelineProperty) -> Result<(), String> {
    if let Some(pos) = app_state.anim.timeline_filter.iter().position(|p| p == &prop) {
        app_state.anim.timeline_filter.remove(pos);
    } else {
        app_state.anim.timeline_filter.push(prop);
    }
    Ok(())
}

// === 追加在 anim_handler.rs 底部 ===

pub fn offset_selected_keyframes(app_state: &mut AppState, dt: f32) -> Result<(), String> {
    // 快捷拖拽统一应用相同的偏移量 dt
    let mut offsets = std::collections::HashMap::new();
    for (bone_id, _, _) in &app_state.anim.selected_keyframes {
        offsets.insert(bone_id.clone(), dt);
    }
    execute_cyclic_offset(app_state, offsets)
}

pub fn apply_spine_offset(app_state: &mut AppState, mode: usize, fixed_frames: i32, step_frames: i32) -> Result<(), String> {
    let fps = app_state.anim.fps; // 【变动】获取全局 FPS
    let mut offsets = std::collections::HashMap::new();
    
    let mut unique_bones: Vec<String> = app_state.anim.selected_keyframes.iter().map(|k| k.0.clone()).collect();
    unique_bones.sort();
    unique_bones.dedup();

    for (index, bone_id) in unique_bones.into_iter().enumerate() {
        let frames = match mode {
            0 => fixed_frames,                                
            1 => fixed_frames + index as i32,                 
            2 => fixed_frames + (index as i32 * step_frames), 
            _ => fixed_frames,
        };
        let dt = frames as f32 / fps; // 数值逻辑自动映射为秒
        offsets.insert(bone_id, dt);
    }
    
    execute_cyclic_offset(app_state, offsets)
}

/// 核心：实现需求文档中的循环偏移、自动补帧与采样逻辑
fn execute_cyclic_offset(app_state: &mut AppState, bone_offsets: std::collections::HashMap<String, f32>) -> Result<(), String> {
    let active_id = app_state.anim.state.project.active_animation_id.clone().ok_or("无活动动画")?;
    
    // 获取需要偏移的关键帧的快照
    let selections = app_state.anim.selected_keyframes.clone();
    if selections.len() < 3 {
        // 根据你的需求文档，选中的帧至少需要3个
        return Ok(()); 
    }

    let anim = app_state.anim.state.project.animations.get_mut(&active_id).unwrap();
    let duration = anim.duration;
    if duration <= 0.0 { return Ok(()); }

    let old_timelines = anim.timelines.clone();
    let mut patches = Vec::new();
    let mut new_selection = Vec::new();

    for tl in &mut anim.timelines {
        let bone_id_cloned = tl.target_id.clone();
        let dt = match bone_offsets.get(&bone_id_cloned) {
            Some(&val) => val,
            None => continue,
        };

        // 检查该时间轴是否有选中的帧
        let selected_kfs: Vec<_> = selections.iter()
            .filter(|(b, p_opt, _)| *b == bone_id_cloned && p_opt.as_ref().map_or(true, |p| p == &tl.property))
            .collect();

        if selected_kfs.is_empty() { continue; }

        let old_tl = old_timelines.iter().find(|t| t.target_id == bone_id_cloned && t.property == tl.property).unwrap();
        
        let mut unselected_kfs = Vec::new();
        let mut shifted_kfs = Vec::new();

        // 步骤 1: 计算新时间 t' = (t + Δ) mod T
        for kf in &tl.keyframes {
            let is_selected = selected_kfs.iter().any(|s| (s.2 - kf.time).abs() < 0.001);
            if is_selected {
                let mut new_time = (kf.time + dt) % duration;
                if new_time < 0.0 { new_time += duration; } // 处理负数取模
                
                let mut new_kf = kf.clone();
                new_kf.time = new_time;
                shifted_kfs.push(new_kf);
                new_selection.push((bone_id_cloned.clone(), Some(tl.property.clone()), new_time));
            } else {
                unselected_kfs.push(kf.clone());
            }
        }

        // 步骤 3 & 4: 自动补帧与冲突处理
        let mut sample_time = (0.0 - dt) % duration;
        if sample_time < 0.0 { sample_time += duration; }
        
        if let Some(sampled_val) = old_tl.sample(sample_time) {
            // 移除因移动导致落在 0 或 T 的帧 (冲突处理：自动补帧优先)
            shifted_kfs.retain(|k| k.time > 0.001 && (duration - k.time) > 0.001);
            
            // 插入 0 处关键帧
            let mut start_kf = shifted_kfs.first().unwrap_or(&tl.keyframes[0]).clone();
            start_kf.time = 0.0;
            start_kf.value = sampled_val.clone();
            shifted_kfs.push(start_kf);

            // 插入 T 处关键帧，保持循环
            let mut end_kf = shifted_kfs.first().unwrap_or(&tl.keyframes[0]).clone();
            end_kf.time = duration;
            end_kf.value = sampled_val;
            shifted_kfs.push(end_kf);
        }

        // 步骤 2 & 5: 合并与排序 (timeline.add_keyframe自带覆盖重合时间点帧的功能)
        tl.keyframes = unselected_kfs;
        for kf in shifted_kfs {
            tl.add_keyframe(kf.time, kf.value, kf.curve);
        }
        
        let new_tl = tl.clone();
        patches.push(AnimPatch::Timeline {
            anim_id: active_id.clone(),
            bone_id: bone_id_cloned,
            prop: tl.property.clone(),
            old: Some(old_tl.clone()),
            new: Some(new_tl),
        });
    }

    if !patches.is_empty() {
        app_state.anim.state.history.commit(AnimPatch::Composite(patches));
        app_state.anim.selected_keyframes = new_selection;
        app_state.is_dirty = true;
        app_state.pixel.view.needs_full_redraw = true;
    }

    Ok(())
}