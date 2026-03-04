use crate::app::state::{AppState, AppMode, ToolType};
use crate::app::events::{InputEvent, EngineEffect};
use crate::core::error::CoreError;
use crate::animation::history::AnimPatch;

pub struct InputHandler;

impl InputHandler {
    pub fn on_mouse_down(app: &mut AppState, x: i32, y: i32) -> Result<(), CoreError> {
        let (fx, fy) = app.pixel.view.screen_to_canvas_raw(app.pixel.engine.store(), x as f32, y as f32);
        let cx = fx.round() as i32;
        let cy = fy.round() as i32;
        let zoom = app.pixel.view.zoom_level as f32;
        let world_x = (x as f32 - app.pixel.view.width / 2.0) / zoom + (app.pixel.engine.store().canvas_width as f32 / 2.0) - app.pixel.view.pan_x;
        let world_y = (y as f32 - app.pixel.view.height / 2.0) / zoom + (app.pixel.engine.store().canvas_height as f32 / 2.0) - app.pixel.view.pan_y;
        if app.is_space_pressed { 
            app.last_mouse_pos = Some((x as i32, y as i32));
            return Ok(()); 
        }
        if app.mode == AppMode::Animation {
            app.pixel.engine.tool_manager_mut().is_drawing = true;
            app.last_mouse_pos = Some((x as i32, y as i32));

            app.anim.state.drag_start_skeleton = Some(app.anim.state.project.skeleton.clone());
            if let Some(id) = &app.anim.state.project.active_animation_id {
                if let Some(anim) = app.anim.state.project.animations.get(id) {
                    app.anim.state.drag_start_animation = Some(anim.clone());
                }
            }
            let click_res = Self::handle_animation_click(app, world_x, world_y);
            if let Some(tool) = app.pixel.engine.tool_manager_mut().tools.get_mut(&ToolType::CreateBone) {
                if let Some(bone_tool) = tool.as_any_mut().downcast_mut::<crate::tools::create_bone::CreateBoneTool>() {
                    bone_tool.parent_bone_id = app.anim.selected_bone_id.clone();
                }
            }
            if app.pixel.engine.tool_manager().active_type == ToolType::CreateBone {
                let (store, symmetry, tool_manager, _) = app.pixel.engine.parts_mut();
                let _ = tool_manager.handle_pointer_down(cx, cy, store, symmetry);
                return Ok(());
            }
            return click_res;
        }

        if app.pixel.engine.tool_manager().active_type == ToolType::CreateBone {
            if let Some(tool) = app.pixel.engine.tool_manager_mut().tools.get_mut(&ToolType::CreateBone) {
                if let Some(bone_tool) = tool.as_any_mut().downcast_mut::<crate::tools::create_bone::CreateBoneTool>() {
                    bone_tool.parent_bone_id = app.anim.selected_bone_id.clone();
                }
            }
            let (store, symmetry, tool_manager, _) = app.pixel.engine.parts_mut();
            return tool_manager.handle_pointer_down(cx, cy, store, symmetry);
        }
        let effect = app.pixel.engine.handle_input(InputEvent::PointerDown { x: cx, y: cy });
        app.last_mouse_pos = Some((x as i32, y as i32));
        let result = match &effect {
            EngineEffect::Error(e) => Err(e.clone()),
            _ => Ok(()),
        };

        Self::handle_engine_effect(app, effect);
        result
    }

    pub fn on_mouse_move(app: &mut AppState, x: i32, y: i32) -> Result<(), CoreError> {
        let last_pos = app.last_mouse_pos.unwrap_or((x, y));
        let dx = (x - last_pos.0) as f32;
        let dy = (y - last_pos.1) as f32;
        app.last_mouse_pos = Some((x, y));
        let (fx, fy) = app.pixel.view.screen_to_canvas_raw(app.pixel.engine.store(), x as f32, y as f32);
        let cx = fx.round() as i32;
        let cy = fy.round() as i32;
        if app.is_space_pressed { return Ok(()); }
        if app.mode == AppMode::Animation {
            if app.pixel.engine.tool_manager().active_type == ToolType::CreateBone {
                let (store, symmetry, tool_manager, _) = app.pixel.engine.parts_mut();
                return tool_manager.handle_pointer_move(cx, cy, store, symmetry);
            }

            if app.pixel.engine.tool_manager().is_drawing {
                let tool = app.pixel.engine.tool_manager().active_type;
                if let Some(bone_id) = app.anim.selected_bone_id.clone() {
                    let skeleton = &mut app.anim.state.project.skeleton;
                    if let Some(bone_idx) = skeleton.bones.iter().position(|b| b.data.id == bone_id) {
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
                            app.pixel.view.needs_full_redraw = true;
                            skeleton.update();
                            app.sync_animation_to_layers();

                            let prop = match tool {
                                ToolType::BoneRotate => Some(crate::core::animation::timeline::TimelineProperty::Rotation),
                                ToolType::BoneTranslate => Some(crate::core::animation::timeline::TimelineProperty::Translation),
                                _ => None,
                            };
                            if let Some(p) = prop {
                                app.anim.state.auto_key_bone(&bone_id, p);
                            }
                        }
                    }
                }
            }
            return Ok(());
        }

        if app.pixel.engine.tool_manager().active_type == ToolType::CreateBone {
            let (store, symmetry, tool_manager, _) = app.pixel.engine.parts_mut();
            return tool_manager.handle_pointer_move(cx, cy, store, symmetry);
        }
        let effect = app.pixel.engine.handle_input(InputEvent::PointerMove { x: cx, y: cy });
        let result = match &effect {
            EngineEffect::Error(e) => Err(e.clone()),
            _ => Ok(()),
        };

        Self::handle_engine_effect(app, effect);
        result
    }

    pub fn on_mouse_up(app: &mut AppState) -> Result<(), CoreError> {
        let was_drawing = app.pixel.engine.tool_manager().is_drawing;
        app.last_mouse_pos = None;

        if was_drawing && app.pixel.engine.tool_manager().active_type == ToolType::CreateBone {
            let mut tool_data = None;
            if let Some(tool) = app.pixel.engine.tool_manager().tools.get(&ToolType::CreateBone) {
                if let Some(bt) = tool.as_any().downcast_ref::<crate::tools::create_bone::CreateBoneTool>() {
                    if let (Some(s), Some(e)) = (bt.start_pos, bt.preview_end) {
                        tool_data = Some((s, e, bt.parent_bone_id.clone()));
                    }
                }
            }
            if let Some((s, e, p)) = tool_data {
                app.enqueue_command(Box::new(crate::app::commands::CreateBoneCmd { start: s, end: e, parent_id: p }));
            }
            
            let (store, _, tool_manager, id_gen) = app.pixel.engine.parts_mut();
            return tool_manager.handle_pointer_up(store, id_gen).map(|_| ());
        }
        
        if app.mode == AppMode::Animation {
            app.pixel.engine.tool_manager_mut().is_drawing = false;
            if let Some(old_skel) = app.anim.state.drag_start_skeleton.take() {
                let mut patches = Vec::new();
                patches.push(AnimPatch::Skeleton { old: old_skel, new: app.anim.state.project.skeleton.clone() });

                if let Some(old_anim) = app.anim.state.drag_start_animation.take() {
                    if let Some(id) = &app.anim.state.project.active_animation_id {
                        if let Some(new_anim) = app.anim.state.project.animations.get(id) {
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
                app.anim.state.history.commit(AnimPatch::Composite(patches));
            }
            return Ok(());
        }

        let effect = app.pixel.engine.handle_input(InputEvent::PointerUp);
        let result = match &effect {
            EngineEffect::Error(e) => Err(e.clone()),
            _ => Ok(()),
        };

        Self::handle_engine_effect(app, effect);
        result
    }

    pub fn handle_animation_click(app: &mut AppState, world_x: f32, world_y: f32) -> Result<(), CoreError> {
        let mut clicked_bone_id = None;
        for bone in &app.anim.state.project.skeleton.bones {
            let bx = bone.world_matrix[4];
            let by = bone.world_matrix[5];
            let fx = world_x;
            let fy = world_y;
            
            if ((fx - bx).powi(2) + (fy - by).powi(2)).sqrt() < 10.0 {
                clicked_bone_id = Some(bone.data.id.clone());
                break;
            }
        }
        let is_transform_tool = matches!(app.pixel.engine.tool_manager().active_type, ToolType::BoneRotate | ToolType::BoneTranslate);
        if clicked_bone_id.is_some() || !is_transform_tool {
            app.anim.selected_bone_id = clicked_bone_id;
        }
        Ok(())
    }

    pub fn handle_engine_effect(app: &mut AppState, effect: EngineEffect) {
        match effect {
            EngineEffect::None => {},
            EngineEffect::RedrawCanvas => {
                app.is_dirty = true;
                app.pixel.view.needs_full_redraw = true;
            },
            EngineEffect::RedrawRect(x, y, w, h) => {
                app.pixel.view.mark_dirty_canvas_rect(app.pixel.engine.store(), x, y, w, h);
            },
            EngineEffect::ToolCommitted => {
                app.is_dirty = true;
            },
            EngineEffect::Error(e) => {
                app.command_bus.events.push_back(crate::app::command_handler::AppEvent::ShowError(e.to_string()));
            }
        }
    }
}