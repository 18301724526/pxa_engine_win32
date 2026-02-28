use crate::core::store::PixelStore;
use crate::history::patch::ActionPatch;
use super::tool_trait::Tool;
use crate::core::symmetry::SymmetryConfig;
use crate::core::error::CoreError;
use crate::core::path::{BezierPath, Vec2, NodeType};
use std::any::Any;
use crate::core::id_gen;
use crate::core::selection::SelectionData;
use super::geometry::Geometry;

#[derive(Debug, Clone, Copy, PartialEq)]
enum PenMode {
    Idle,
    CreatingDrag,
    MovingAnchor(usize),
    AdjustingIn(usize),
    AdjustingOut(usize),
}

pub struct PenTool {
    snapshot: Option<BezierPath>,
    mode: PenMode,
    pub selected_node_idx: Option<usize>,
    pub hover_start_point: bool,
    pub hover_node_idx: Option<usize>,
    pub hover_handle: Option<(usize, bool)>,
    needs_redraw: bool,
}

impl PenTool {
    pub fn new() -> Self {
        Self {
            snapshot: None,
            mode: PenMode::Idle,
            selected_node_idx: None,
            hover_start_point: false,
            hover_node_idx: None,
            hover_handle: None,
            needs_redraw: false,
        }
    }

    pub fn hit_test(&self, path: &BezierPath, x: f32, y: f32) -> (Option<usize>, Option<(usize, bool)>) {
        let hit_radius = 6.0;
        let mouse = Vec2::new(x, y);

        if let Some(idx) = self.selected_node_idx {
            if let Some(node) = path.nodes.get(idx) {
                if node.handle_in != Vec2::new(0.0, 0.0) && node.abs_in().distance(mouse) < hit_radius {
                    return (None, Some((idx, true)));
                }
                if node.handle_out != Vec2::new(0.0, 0.0) && node.abs_out().distance(mouse) < hit_radius {
                    return (None, Some((idx, false)));
                }
            }
        }

        for (i, node) in path.nodes.iter().enumerate() {
            if node.anchor.distance(mouse) < hit_radius {
                return (Some(i), None);
            }
        }

        (None, None)
    }

    pub fn fill(&self, store: &PixelStore) -> Option<ActionPatch> {
        if store.active_path.nodes.len() < 3 { return None; }
        let layer_id = store.active_layer_id.as_ref()?;
        let layer = store.get_layer(layer_id)?;
        let points = store.active_path.flatten(0.5);
        let mut temp_sel = SelectionData::new(store.canvas_width, store.canvas_height);
        temp_sel.set_from_polygon(&points);
        let mut patch = ActionPatch::new_pixel_diff(id_gen::gen_id(), layer_id.clone());
        let color = store.primary_color;
        for y in 0..store.canvas_height {
            for x in 0..store.canvas_width {
                if temp_sel.contains(x, y) {
                    let lx = x as i32 - layer.offset_x;
                    let ly = y as i32 - layer.offset_y;
                    if lx >= 0 && ly >= 0 && lx < layer.width as i32 && ly < layer.height as i32 {
                        let old_color = layer.get_pixel(lx as u32, ly as u32).unwrap_or(crate::core::color::Color::transparent());
                        if old_color != color { patch.add_pixel_diff(lx as u32, ly as u32, old_color, color); }
                    }
                }
            }
        }
        if patch.is_empty() { None } else { Some(patch) }
    }

    pub fn stroke(&self, store: &PixelStore) -> Option<ActionPatch> {
        if store.active_path.nodes.is_empty() { return None; }
        let layer_id = store.active_layer_id.clone()?;
        let points = store.active_path.flatten(0.5);
        let mut patch = ActionPatch::new_pixel_diff(id_gen::gen_id(), layer_id.clone());
        let color = store.primary_color;
        let mut drawn_points = std::collections::HashSet::new();
        for i in 0..points.len() {
            if !store.active_path.is_closed && i == points.len() - 1 { break; }
            let p1 = points[i];
            let p2 = points[(i + 1) % points.len()];
            Geometry::bresenham_line(
                p1.x.round() as i32, p1.y.round() as i32,
                p2.x.round() as i32, p2.y.round() as i32,
                |x, y| {
                    if x >= 0 && y >= 0 && (x as u32) < store.canvas_width && (y as u32) < store.canvas_height {
                        if drawn_points.insert((x, y)) {
                            let old_color = store.get_pixel(&layer_id, x as u32, y as u32).unwrap_or(crate::core::color::Color::transparent());
                            patch.add_pixel_diff(x as u32, y as u32, old_color, color);
                        }
                    }
                }
            );
        }
        if patch.is_empty() { None } else { Some(patch) }
    }
}

impl Tool for PenTool {
    fn on_pointer_down(&mut self, x: u32, y: u32, store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        self.snapshot = Some(store.active_path.clone());

        let fx = x as f32;
        let fy = y as f32;

        let (hit_node, hit_handle) = self.hit_test(&store.active_path, fx, fy);

        if let Some((idx, is_in)) = hit_handle {
            self.mode = if is_in { PenMode::AdjustingIn(idx) } else { PenMode::AdjustingOut(idx) };
            return Ok(());
        }

        if let Some(idx) = hit_node {
            if idx == 0 && !store.active_path.is_closed && store.active_path.nodes.len() > 2 {
                store.active_path.is_closed = true;
                self.selected_node_idx = Some(0);
                self.mode = PenMode::Idle;
                return Ok(());
            }

            self.selected_node_idx = Some(idx);
            self.mode = PenMode::MovingAnchor(idx);
            return Ok(());
        }

        if store.active_path.is_closed {
            store.active_path.nodes.clear();
            store.active_path.is_closed = false;
        }

        store.active_path.add_node(fx, fy);
        let new_idx = store.active_path.nodes.len() - 1;
        self.selected_node_idx = Some(new_idx);
        self.mode = PenMode::CreatingDrag;

        Ok(())
    }

    fn on_pointer_move(&mut self, x: u32, y: u32, store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        let mouse_pos = Vec2::new(x as f32, y as f32);

        match self.mode {
            PenMode::Idle => {
                let (hit_node, hit_handle) = self.hit_test(&store.active_path, mouse_pos.x, mouse_pos.y);
                self.hover_node_idx = hit_node;
                self.hover_handle = hit_handle;
                self.hover_start_point = false;
                if !store.active_path.is_closed && store.active_path.nodes.len() > 2 {
                    if let Some(first) = store.active_path.nodes.first() {
                        if first.anchor.distance(mouse_pos) < 8.0 {
                            self.hover_start_point = true;
                        }
                    }
                }
                self.needs_redraw = true;
            },
            PenMode::CreatingDrag => {
                if let Some(idx) = self.selected_node_idx {
                    if let Some(node) = store.active_path.nodes.get_mut(idx) {
                        node.handle_out = mouse_pos - node.anchor;
                        node.handle_in = node.handle_out * -1.0;
                        node.kind = NodeType::Smooth;
                        self.needs_redraw = true;
                    }
                }
            },
            PenMode::MovingAnchor(idx) => {
                if let Some(node) = store.active_path.nodes.get_mut(idx) {
                    node.anchor = mouse_pos;
                    self.needs_redraw = true;
                }
            },
            PenMode::AdjustingOut(idx) => {
                if let Some(node) = store.active_path.nodes.get_mut(idx) {
                    node.handle_out = mouse_pos - node.anchor;
                    if node.kind == NodeType::Smooth {
                        node.handle_in = node.handle_out * -1.0;
                    }
                    self.needs_redraw = true;
                }
            },
            PenMode::AdjustingIn(idx) => {
                if let Some(node) = store.active_path.nodes.get_mut(idx) {
                    node.handle_in = mouse_pos - node.anchor;
                    if node.kind == NodeType::Smooth {
                        node.handle_out = node.handle_in * -1.0;
                    }
                    self.needs_redraw = true;
                }
            },
        }
        Ok(())
    }

    fn on_pointer_up(&mut self, store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError> {
        let mode_was = self.mode;
        self.mode = PenMode::Idle;
        
        if let Some(old_path) = self.snapshot.take() {
            let current_path = &store.active_path;
            if let PenMode::MovingAnchor(idx) = mode_was {
                if old_path == *current_path && idx < store.active_path.nodes.len() {
                    store.active_path.nodes.remove(idx);
                    self.selected_node_idx = None;
                    return Ok(Some(ActionPatch::new_path_change(id_gen::gen_id(), old_path, store.active_path.clone())));
                }
            }
            if old_path != *current_path {
                return Ok(Some(ActionPatch::new_path_change(id_gen::gen_id(), old_path, current_path.clone())));
            }
        }

        Ok(None)
    }

    fn take_dirty_rect(&mut self) -> Option<(u32, u32, u32, u32)> {
        if self.needs_redraw {
            self.needs_redraw = false;
            Some((0, 0, u32::MAX, u32::MAX))
        } else {
            None
        }
    }

    fn on_cancel(&mut self, store: &mut PixelStore) {
        store.active_path.nodes.clear();
        store.active_path.is_closed = false;

        
        self.mode = PenMode::Idle;
        self.selected_node_idx = None;
        self.hover_start_point = false;
    }

    fn on_commit(&mut self, store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError> {
        if store.active_path.nodes.len() < 3 { return Ok(None); }
        let points = store.active_path.flatten(0.5);
        let old_sel = store.selection.clone();
        let mut new_sel = old_sel.clone();
        new_sel.set_from_polygon(&points);

        store.active_path.nodes.clear();
        store.active_path.is_closed = false;
        self.selected_node_idx = None;

        Ok(Some(ActionPatch::new_selection_change(id_gen::gen_id(), old_sel, new_sel)))
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}