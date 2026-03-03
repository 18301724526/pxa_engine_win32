use crate::app::command_handler::{Command, AppEvent};
use crate::app::state::{AppState, ToolType};
use crate::app::commands::ResizeAnchor;
use std::collections::VecDeque;

pub struct ChangeBrushSizeCmd(pub i32);
impl Command for ChangeBrushSizeCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        let (size, _, _) = state.pixel.engine.brush_settings_mut();
        *size = (*size as i32 + self.0).clamp(1, 20) as u32;
        Ok(())
    }
}

pub struct SelectToolCmd(pub ToolType);
impl Command for SelectToolCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.pixel.active_select_tool = self.0;
        state.set_tool(self.0);
        Ok(())
    }
}

pub struct ClearSelectionCmd;
impl Command for ClearSelectionCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        if state.pixel.engine.store().selection.is_active {
            let old = state.pixel.engine.store().selection.clone();
            let mut new = old.clone();
            new.clear();
            let patch = crate::history::patch::ActionPatch::new_selection_change(state.pixel.engine.id_gen().generate(), old, new);
            state.pixel.engine.commit_patch(patch).map_err(|e| e.to_string())?;
            state.is_dirty = true; state.pixel.view.needs_full_redraw = true;
        }
        Ok(())
    }
}

pub struct InvertSelectionCmd;
impl Command for InvertSelectionCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        let old = state.pixel.engine.store().selection.clone();
        let mut new = old.clone();
        new.invert();
        new.is_active = true;
        let patch = crate::history::patch::ActionPatch::new_selection_change(state.pixel.engine.id_gen().generate(), old, new);
        state.pixel.engine.commit_patch(patch).map_err(|e| e.to_string())?;
        state.is_dirty = true; state.pixel.view.needs_full_redraw = true;
        Ok(())
    }
}

pub struct StrokeSelectionCmd(pub u32);
impl Command for StrokeSelectionCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        let thickness = self.0 as i32;
        if thickness <= 0 { return Ok(()); }

        let patch = {
            let (store, _, _, id_gen) = state.pixel.engine.parts_mut();
            if !store.selection.is_active { return Ok(()); }

            let layer_id = match store.active_layer_id.clone() {
                Some(id) => id,
                None => return Ok(()),
            };
            let color = store.primary_color;
            let w = store.canvas_width as i32;
            let h = store.canvas_height as i32;

            let mut patch = crate::history::patch::ActionPatch::new_pixel_diff(id_gen.generate(), layer_id.clone());
            let mut has_changes = false;
            let layer = store.get_layer(&layer_id).unwrap();

            for y in 0..h {
                for x in 0..w {
                    if store.selection.contains(x as u32, y as u32) {
                        let mut is_edge = false;
                        'outer: for dy in -thickness..=thickness {
                            for dx in -thickness..=thickness {
                                let nx = x + dx;
                                let ny = y + dy;
                                if nx < 0 || ny < 0 || nx >= w || ny >= h || !store.selection.contains(nx as u32, ny as u32) {
                                    is_edge = true;
                                    break 'outer;
                                }
                            }
                        }

                        if is_edge {
                            let lx = x - layer.offset_x;
                            let ly = y - layer.offset_y;
                            if lx >= 0 && ly >= 0 && lx < layer.width as i32 && ly < layer.height as i32 {
                                let old_color = layer.get_pixel(lx as u32, ly as u32).unwrap_or(crate::core::color::Color::transparent());
                                if old_color != color {
                                    patch.add_pixel_diff(lx as u32, ly as u32, old_color, color);
                                    has_changes = true;
                                }
                            }
                        }
                    }
                }
            }

            if has_changes { Some(patch) } else { None }
        };

        if let Some(p) = patch {
            state.pixel.engine.commit_patch(p).map_err(|e| e.to_string())?;
            state.is_dirty = true;
            state.pixel.view.needs_full_redraw = true;
        }

        Ok(())
    }
}

pub struct ResizeCanvasCmd(pub u32, pub u32, pub ResizeAnchor);
impl Command for ResizeCanvasCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        let new_w = self.0;
        let new_h = self.1;
        let anchor = self.2;

        let (store, _, _, _) = state.pixel.engine.parts_mut();
        
        let old_w = store.canvas_width;
        let old_h = store.canvas_height;

        if old_w == new_w && old_h == new_h {
            return Ok(());
        }

        let (dx, dy) = match anchor {
            ResizeAnchor::TopLeft => (0, 0),
            ResizeAnchor::TopCenter => (((new_w as i32) - (old_w as i32)) / 2, 0),
            ResizeAnchor::TopRight => ((new_w as i32) - (old_w as i32), 0),
            ResizeAnchor::MiddleLeft => (0, ((new_h as i32) - (old_h as i32)) / 2),
            ResizeAnchor::Center => (((new_w as i32) - (old_w as i32)) / 2, ((new_h as i32) - (old_h as i32)) / 2),
            ResizeAnchor::MiddleRight => ((new_w as i32) - (old_w as i32), ((new_h as i32) - (old_h as i32)) / 2),
            ResizeAnchor::BottomLeft => (0, (new_h as i32) - (old_h as i32)),
            ResizeAnchor::BottomCenter => (((new_w as i32) - (old_w as i32)) / 2, (new_h as i32) - (old_h as i32)),
            ResizeAnchor::BottomRight => ((new_w as i32) - (old_w as i32), (new_h as i32) - (old_h as i32)),
        };

        for layer in &mut store.layers {
            layer.shift_and_resize(dx, dy, new_w, new_h);
        }

        store.selection.shift_and_resize(dx, dy, new_w, new_h);
        store.canvas_width = new_w;
        store.canvas_height = new_h;
        store.composite_cache = vec![0u8; (new_w * new_h * 4) as usize];

        for bone in &mut state.anim.state.project.skeleton.bones {
            if bone.data.parent_id.is_none() {
                bone.local_transform.x += dx as f32;
                bone.local_transform.y += dy as f32;
            }
        }
        state.anim.state.project.skeleton.update();

        state.pixel.engine.update_render_cache(None);
        state.is_dirty = true;
        state.pixel.view.needs_full_redraw = true;

        Ok(())
    }
}

pub struct CommitCurrentToolCmd;
impl Command for CommitCurrentToolCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.commit_current_tool();
        Ok(())
    }
}

pub struct CancelCurrentToolCmd;
impl Command for CancelCurrentToolCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.cancel_current_tool();
        Ok(())
    }
}

pub struct TogglePathNodeTypeCmd(pub usize);
impl Command for TogglePathNodeTypeCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        let idx = self.0;
        let (store, _, _, id_gen) = state.pixel.engine.parts_mut();
        if idx < store.active_path.nodes.len() {
            let old_path = store.active_path.clone();
            let node = &mut store.active_path.nodes[idx];
            if node.kind == crate::core::path::NodeType::Smooth {
                node.kind = crate::core::path::NodeType::Corner;
            } else {
                node.kind = crate::core::path::NodeType::Smooth;
            }
            let patch = crate::history::patch::ActionPatch::new_path_change(id_gen.generate(), old_path, store.active_path.clone());
            state.pixel.engine.commit_patch(patch).map_err(|e| e.to_string())?;
            state.is_dirty = true;
            state.pixel.view.needs_full_redraw = true;
        }
        Ok(())
    }
}

pub struct DeletePathNodeCmd(pub usize);
impl Command for DeletePathNodeCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        let idx = self.0;
        let (store, _, _, id_gen) = state.pixel.engine.parts_mut();
        if idx < store.active_path.nodes.len() {
            let old_path = store.active_path.clone();
            store.active_path.nodes.remove(idx);
            let patch = crate::history::patch::ActionPatch::new_path_change(id_gen.generate(), old_path, store.active_path.clone());
            state.pixel.engine.commit_patch(patch).map_err(|e| e.to_string())?;
            state.is_dirty = true;
            state.pixel.view.needs_full_redraw = true;
        }
        Ok(())
    }
}

pub struct PenFillCmd;
impl Command for PenFillCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        if let Some(tool) = state.pixel.engine.tool_manager().tools.get(&ToolType::Pen) {
            if let Some(pen) = tool.as_any().downcast_ref::<crate::tools::pen::PenTool>() {
                if let Some(patch) = pen.fill(state.pixel.engine.store(), state.pixel.engine.id_gen()) {
                    state.pixel.engine.commit_patch(patch).map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }
}

pub struct PenStrokeCmd;
impl Command for PenStrokeCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        if let Some(tool) = state.pixel.engine.tool_manager().tools.get(&ToolType::Pen) {
            if let Some(pen) = tool.as_any().downcast_ref::<crate::tools::pen::PenTool>() {
                if let Some(patch) = pen.stroke(state.pixel.engine.store(), state.pixel.engine.id_gen()) {
                    state.pixel.engine.commit_patch(patch).map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }
}