use crate::app::state::{AppState, AppMode};
use crate::animation::history::AnimPatch;
use crate::history::patch::ActionPatch;
use crate::core::animation::bone::BoneData;

pub fn bind_layer_to_bone(app_state: &mut AppState, layer_id: &str, target_bone: &str) -> Result<(), String> {
    if app_state.mode != AppMode::PixelEdit {
        return Err("只能在绘画模式下修改骨骼结构".into());
    }
    if let Some(slot) = app_state.anim.state.project.skeleton.slots.iter_mut().find(|s| s.data.id == layer_id) {
        if slot.data.bone_id != target_bone {
            let old_bone = slot.data.bone_id.clone();
            slot.data.bone_id = target_bone.to_string();
            if let Some(layer) = app_state.pixel.engine.parts_mut().0.get_layer_mut(layer_id) {
                layer.anim_offset_x = 0;
                layer.anim_offset_y = 0;
            }
            let patch = AnimPatch::SlotBone { slot_id: layer_id.to_string(), old_bone, new_bone: target_bone.to_string() };
            let action = ActionPatch::new_animation(app_state.pixel.engine.id_gen().generate(), patch);
            app_state.pixel.engine.commit_patch(action).map_err(|e| e.to_string())?;
            app_state.is_dirty = true;
            app_state.pixel.view.needs_full_redraw = true;
            app_state.sync_animation_to_layers();
        }
    }
    Ok(())
}

pub fn delete_bone(app_state: &mut AppState, bone_id: &str) -> Result<(), String> {
    if app_state.mode != AppMode::PixelEdit {
        return Err("只能在绘画模式下修改骨骼结构".into());
    }

    if bone_id == "root" {
        return Err("禁止删除根骨骼 (root)".into());
    }
    let old_skel = app_state.anim.state.project.skeleton.clone();

    let mut patches = Vec::new();

    let anim_ids: Vec<String> = app_state.anim.state.project.animations.keys().cloned().collect();
    for anim_id in anim_ids {
        if let Some(anim) = app_state.anim.state.project.animations.get_mut(&anim_id) {
            for tl in anim.timelines.iter().filter(|t| t.target_id == bone_id) {
                patches.push(AnimPatch::Timeline {
                    anim_id: anim_id.clone(),
                    bone_id: bone_id.to_string(),
                    prop: tl.property.clone(),
                    old: Some(tl.clone()),
                    new: None,
                });
            }
            anim.timelines.retain(|tl| tl.target_id != bone_id);
            anim.recalculate_duration();
        }
    }

    if let Err(e) = app_state.anim.state.project.skeleton.purge_bone_atomic(bone_id) {
        return Err(e.to_string());
    }

    let mut final_patches = patches;
    final_patches.push(AnimPatch::Skeleton { old: old_skel, new: app_state.anim.state.project.skeleton.clone() });

    let patch = AnimPatch::Composite(final_patches);
    let action = ActionPatch::new_animation(app_state.pixel.engine.id_gen().generate(), patch);
    app_state.pixel.engine.commit_patch(action).map_err(|e| e.to_string())?;
    
    if app_state.anim.selected_bone_id.as_deref() == Some(bone_id) {
        app_state.anim.selected_bone_id = None;
    }
    app_state.is_dirty = true;
    app_state.pixel.view.needs_full_redraw = true;
    Ok(())
}

pub fn create_bone(
    app_state: &mut AppState, 
    start: (f32, f32), 
    end: (f32, f32), 
    parent_id: Option<String>
) -> Result<Option<String>, String> {
    if app_state.mode != AppMode::PixelEdit {
        return Err("只能在绘画模式下修改骨骼结构".into());
    }

    let world_dx = end.0 - start.0;
    let world_dy = end.1 - start.1;
    let length = (world_dx * world_dx + world_dy * world_dy).sqrt();
    if length < 1.0 { return Ok(None); }

    let old_skel = app_state.anim.state.project.skeleton.clone();

    let id = format!("bone_{}", app_state.pixel.engine.id_gen().generate());
    let mut bone_data = BoneData::new(id.clone(), id.clone());
    bone_data.parent_id = parent_id;
    bone_data.length = length;

    let skeleton = &mut app_state.anim.state.project.skeleton;

    if let Some(parent_id) = &bone_data.parent_id {
        if let Some(parent_idx) = skeleton.bone_id_to_index(parent_id) {
            let pm = skeleton.bones[parent_idx].world_matrix;

            let (a, b, c, d, tx, ty) = (pm[0], pm[1], pm[2], pm[3], pm[4], pm[5]);
            let det = a * d - b * c;
            
            if det.abs() > 1e-6 {
                let inv_det = 1.0 / det;
                let dx = start.0 - tx;
                let dy = start.1 - ty;
                
                bone_data.local_transform.x = (d * dx - c * dy) * inv_det;
                bone_data.local_transform.y = (-b * dx + a * dy) * inv_det;

                let global_angle = world_dy.atan2(world_dx).to_degrees();
                let parent_angle = b.atan2(a).to_degrees();
                bone_data.local_transform.rotation = global_angle - parent_angle;
            } else {
                bone_data.local_transform.x = start.0;
                bone_data.local_transform.y = start.1;
                bone_data.local_transform.rotation = world_dy.atan2(world_dx).to_degrees();
            }
        }
    } else {
        bone_data.local_transform.x = start.0;
        bone_data.local_transform.y = start.1;
        bone_data.local_transform.rotation = world_dy.atan2(world_dx).to_degrees();
    }

    skeleton.add_bone(bone_data);
    skeleton.update();

    let new_skel = skeleton.clone();
    let patch = AnimPatch::Skeleton { old: old_skel, new: new_skel };
    let action = ActionPatch::new_animation(app_state.pixel.engine.id_gen().generate(), patch);
    app_state.pixel.engine.commit_patch(action).map_err(|e| e.to_string())?;
    app_state.is_dirty = true;
    app_state.pixel.view.needs_full_redraw = true;
    app_state.sync_animation_to_layers();

    Ok(Some(id))
}