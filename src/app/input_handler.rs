use crate::app::state::{AppState, AppMode, ToolType};
use crate::app::events::{InputEvent, EngineEffect};
use crate::core::error::CoreError;
use crate::animation::history::AnimPatch;

pub struct InputHandler;

impl InputHandler {
    pub fn on_mouse_down(app: &mut AppState, x: u32, y: u32) -> Result<(), CoreError> {
        if app.mode == AppMode::Animation {
            app.engine.tool_manager_mut().is_drawing = true;
            app.last_mouse_pos = Some((x, y));

            app.animation.drag_start_skeleton = Some(app.animation.project.skeleton.clone());
            if let Some(id) = &app.animation.project.active_animation_id {
                if let Some(anim) = app.animation.project.animations.get(id) {
                    app.animation.drag_start_animation = Some(anim.clone());
                }
            }
            if let Some(tool) = app.engine.tool_manager_mut().tools.get_mut(&ToolType::CreateBone) {
                if let Some(bone_tool) = tool.as_any_mut().downcast_mut::<crate::tools::create_bone::CreateBoneTool>() {
                    bone_tool.parent_bone_id = app.ui.selected_bone_id.clone();
                }
            }
            if app.engine.tool_manager().active_type == ToolType::CreateBone {
                let (store, symmetry, tool_manager) = app.engine.parts_mut();
                let _ = tool_manager.handle_pointer_down(x, y, store, symmetry);
                return Ok(());
            } else {
                return Self::handle_animation_click(app, x, y);
            }
        }

        if app.engine.tool_manager().active_type == ToolType::CreateBone {
            if let Some(tool) = app.engine.tool_manager_mut().tools.get_mut(&ToolType::CreateBone) {
                if let Some(bone_tool) = tool.as_any_mut().downcast_mut::<crate::tools::create_bone::CreateBoneTool>() {
                    bone_tool.parent_bone_id = app.ui.selected_bone_id.clone();
                }
            }
            let (store, symmetry, tool_manager) = app.engine.parts_mut();
            return tool_manager.handle_pointer_down(x, y, store, symmetry);
        }
        let effect = app.engine.handle_input(InputEvent::PointerDown { x, y });
        app.last_mouse_pos = Some((x, y));
        let result = match &effect {
            EngineEffect::Error(e) => Err(e.clone()),
            _ => Ok(()),
        };

        Self::handle_engine_effect(app, effect);
        result
    }

    pub fn on_mouse_move(app: &mut AppState, x: u32, y: u32) -> Result<(), CoreError> {
        let last_pos = app.last_mouse_pos.unwrap_or((x, y));
        let dx = (x as i32).wrapping_sub(last_pos.0 as i32) as f32;
        let dy = (y as i32).wrapping_sub(last_pos.1 as i32) as f32;
        app.last_mouse_pos = Some((x, y));
        if app.mode == AppMode::Animation {
            if app.engine.tool_manager().active_type == ToolType::CreateBone {
                let (store, symmetry, tool_manager) = app.engine.parts_mut();
                return tool_manager.handle_pointer_move(x, y, store, symmetry);
            }

            if app.engine.tool_manager().is_drawing {
                let tool = app.engine.tool_manager().active_type;
                if let Some(bone_id) = &app.ui.selected_bone_id {
                    let skeleton = &mut app.animation.project.skeleton;
                    if let Some(bone_idx) = skeleton.bones.iter().position(|b| b.data.id == *bone_id) {
                        let mut changed = false;
                        match tool {
                            ToolType::BoneRotate => {
                                let bone = &mut skeleton.bones[bone_idx];
                                let base_sensitivity = 0.2;
                                let acceleration = 0.02;
                                let delta = dy * (base_sensitivity + dy.abs() * acceleration);
                                bone.local_transform.rotation -= delta;
                                changed = true;
                            }
                            ToolType::BoneTranslate => {
                                let current_world_x = skeleton.bones[bone_idx].world_matrix[4];
                                let current_world_y = skeleton.bones[bone_idx].world_matrix[5];
                                
                                let target_world_x = current_world_x + dx;
                                let target_world_y = current_world_y + dy;
                                let pm = skeleton.get_parent_world_matrix(bone_idx);
                                let (a, b, c, d, tx, ty) = (pm[0], pm[1], pm[2], pm[3], pm[4], pm[5]);
                                let det = a * d - b * c;

                                if det.abs() > 1e-6 {
                                    let inv_det = 1.0 / det;
                                    let dx_world = target_world_x - tx;
                                    let dy_world = target_world_y - ty;

                                    let bone = &mut skeleton.bones[bone_idx];
                                    bone.local_transform.x = (d * dx_world - c * dy_world) * inv_det;
                                    bone.local_transform.y = (-b * dx_world + a * dy_world) * inv_det;
                                    changed = true;
                                }
                            }
                            _ => {}
                        }

                        if changed {
                            app.is_dirty = true;
                            app.view.needs_full_redraw = true;
                            skeleton.update();

                            let prop = match tool {
                                ToolType::BoneRotate => Some(crate::core::animation::timeline::TimelineProperty::Rotation),
                                ToolType::BoneTranslate => Some(crate::core::animation::timeline::TimelineProperty::Translation),
                                _ => None,
                            };
                            if let Some(p) = prop {
                                app.animation.auto_key_bone(bone_id, p);
                            }
                        }
                    }
                }
            }
            return Ok(());
        }

        if app.engine.tool_manager().active_type == ToolType::CreateBone {
            let (store, symmetry, tool_manager) = app.engine.parts_mut();
            return tool_manager.handle_pointer_move(x, y, store, symmetry);
        }
        let effect = app.engine.handle_input(InputEvent::PointerMove { x, y });
        let result = match &effect {
            EngineEffect::Error(e) => Err(e.clone()),
            _ => Ok(()),
        };

        Self::handle_engine_effect(app, effect);
        result
    }

    pub fn on_mouse_up(app: &mut AppState) -> Result<(), CoreError> {
        let was_drawing = app.engine.tool_manager().is_drawing;
        app.last_mouse_pos = None;
        
        if app.mode == AppMode::Animation {
            app.engine.tool_manager_mut().is_drawing = false;
            if app.engine.tool_manager().active_type == ToolType::CreateBone {
                 if let Some(tool) = app.engine.tool_manager().tools.get(&ToolType::CreateBone) {
                     if let Some(bone_tool) = tool.as_any().downcast_ref::<crate::tools::create_bone::CreateBoneTool>() {
                         let new_bone_id = bone_tool.commit_to_skeleton(&mut app.animation.project.skeleton);
                         if let Some(id) = new_bone_id {
                             app.ui.selected_bone_id = Some(id);
                             app.animation.project.skeleton.update();
                             app.is_dirty = true;
                             app.view.needs_full_redraw = true;
                         }
                     }
                 }
                 let (store, _, tool_manager) = app.engine.parts_mut();
                 return tool_manager.handle_pointer_up(store).map(|_| ());
            }
            if let Some(old_skel) = app.animation.drag_start_skeleton.take() {
                let mut patches = Vec::new();
                patches.push(AnimPatch::Skeleton { old: old_skel, new: app.animation.project.skeleton.clone() });

                if let Some(old_anim) = app.animation.drag_start_animation.take() {
                    if let Some(id) = &app.animation.project.active_animation_id {
                        if let Some(new_anim) = app.animation.project.animations.get(id) {
                            for new_tl in &new_anim.timelines {
                                let old_tl = old_anim.timelines.iter().find(|t| t.target_id == new_tl.target_id && t.property == new_tl.property);
                                patches.push(AnimPatch::Timeline {
                                    anim_id: id.clone(),
                                    bone_id: new_tl.target_id.clone(),
                                    prop: new_tl.property.clone(),
                                    old: old_tl.cloned(),
                                    new: Some(new_tl.clone()),
                                });
                            }
                        }
                    }
                }
                app.animation.history.commit(AnimPatch::Composite(patches));
            }
            return Ok(());
        }

        if was_drawing && app.engine.tool_manager().active_type == ToolType::CreateBone {
             if let Some(tool) = app.engine.tool_manager().tools.get(&ToolType::CreateBone) {
                 if let Some(bone_tool) = tool.as_any().downcast_ref::<crate::tools::create_bone::CreateBoneTool>() {
                     let new_bone_id = bone_tool.commit_to_skeleton(&mut app.animation.project.skeleton);
                     
                     if let Some(id) = new_bone_id {
                         app.ui.selected_bone_id = Some(id);
                         app.animation.project.skeleton.update();
                         app.is_dirty = true;
                         app.view.needs_full_redraw = true;
                     }
                 }
             }
             let (store, _, tool_manager) = app.engine.parts_mut();
             return tool_manager.handle_pointer_up(store).map(|_| ());
        }

        let effect = app.engine.handle_input(InputEvent::PointerUp);
        let result = match &effect {
            EngineEffect::Error(e) => Err(e.clone()),
            _ => Ok(()),
        };

        Self::handle_engine_effect(app, effect);
        result
    }

    pub fn handle_animation_click(app: &mut AppState, x: u32, y: u32) -> Result<(), CoreError> {
        let mut clicked_bone_id = None;
        for bone in &app.animation.project.skeleton.bones {
            let bx = bone.world_matrix[4];
            let by = bone.world_matrix[5];
            let fx = x as i32 as f32;
            let fy = y as i32 as f32;
            
            if ((fx - bx).powi(2) + (fy - by).powi(2)).sqrt() < 10.0 {
                clicked_bone_id = Some(bone.data.id.clone());
                break;
            }
        }
        let is_transform_tool = matches!(app.engine.tool_manager().active_type, ToolType::BoneRotate | ToolType::BoneTranslate);
        if clicked_bone_id.is_some() || !is_transform_tool {
            app.ui.selected_bone_id = clicked_bone_id;
        }
        Ok(())
    }

    pub fn handle_engine_effect(app: &mut AppState, effect: EngineEffect) {
        match effect {
            EngineEffect::None => {},
            EngineEffect::RedrawCanvas => {
                app.is_dirty = true;
                app.view.needs_full_redraw = true;
            },
            EngineEffect::RedrawRect(x, y, w, h) => {
                app.view.mark_dirty_canvas_rect(app.engine.store(), x, y, w, h);
            },
            EngineEffect::ToolCommitted => {
                app.is_dirty = true;
            },
            EngineEffect::Error(e) => {
                app.ui.error_message = Some(e.to_string());
            }
        }
    }
}