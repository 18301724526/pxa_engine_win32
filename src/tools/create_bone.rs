use crate::core::store::PixelStore;
use crate::history::patch::ActionPatch;
use crate::tools::tool_trait::Tool;
use crate::core::symmetry::SymmetryConfig;
use crate::core::error::CoreError;
use crate::core::animation::bone::BoneData;
use crate::core::animation::skeleton::Skeleton;
use crate::core::id_gen;
use std::any::Any;

pub struct CreateBoneTool {
    pub start_pos: Option<(f32, f32)>,
    pub preview_end: Option<(f32, f32)>,
    pub parent_bone_id: Option<String>,
}

impl CreateBoneTool {
    pub fn new() -> Self {
        Self {
            start_pos: None,
            preview_end: None,
            parent_bone_id: None,
        }
    }

    pub fn commit_to_skeleton(&self, skeleton: &mut Skeleton) -> Option<String> {
        if let (Some(start), Some(end)) = (self.start_pos, self.preview_end) {
            let world_dx = end.0 - start.0;
            let world_dy = end.1 - start.1;
            let length = (world_dx * world_dx + world_dy * world_dy).sqrt();
            if length < 1.0 { return None; }

            let id = format!("bone_{}", id_gen::gen_id());
            let mut bone_data = BoneData::new(id.clone(), id.clone());
            
            bone_data.parent_id = self.parent_bone_id.clone();
            bone_data.length = length;
            if let Some(parent_id) = &self.parent_bone_id {
                if let Some(parent_idx) = skeleton.bones.iter().position(|b| b.data.id == *parent_id) {
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
            return Some(id);
        }
        None
    }
}

impl Tool for CreateBoneTool {
    fn on_pointer_down(&mut self, x: u32, y: u32, _store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        self.start_pos = Some((x as f32, y as f32));
        self.preview_end = Some((x as f32, y as f32));
        Ok(())
    }

    fn on_pointer_move(&mut self, x: u32, y: u32, _store: &mut PixelStore, _symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        if self.start_pos.is_some() {
            self.preview_end = Some((x as f32, y as f32));
        }
        Ok(())
    }

    fn on_pointer_up(&mut self, _store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError> {
        self.start_pos = None;
        self.preview_end = None;
        Ok(None)
    }

    fn take_dirty_rect(&mut self) -> Option<(u32, u32, u32, u32)> {
        Some((0, 0, u32::MAX, u32::MAX))
    }

    fn on_cancel(&mut self, _store: &mut PixelStore) {
        self.start_pos = None;
        self.preview_end = None;
    }

    fn on_commit(&mut self, _store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError> {
        self.on_cancel(_store);
        Ok(None)
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[cfg(test)]
mod tests;