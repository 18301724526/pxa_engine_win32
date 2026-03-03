pub mod data;
pub mod ops;

pub use data::RuntimeBone;

use crate::core::animation::bone::BoneData;
use crate::core::animation::slot::RuntimeSlot;
use crate::core::animation::storage::SkeletonStorage;
use std::collections::HashMap;
use std::any::Any;

#[derive(Debug, Clone, Default)]
pub struct Skeleton {
    pub bones: Vec<RuntimeBone>,
    pub slots: Vec<RuntimeSlot>,
    // 使用 pub(super) 让 ops.rs 可以访问它，但对整个引擎的其他部分保持私有
    pub(super) name_to_index: HashMap<String, usize>, 
}

impl SkeletonStorage for Skeleton {
    fn get_bone_world_position(&self, id: &str) -> Option<(f32, f32)> { self.get_bone_world_position(id) }
    fn get_parent_world_matrix(&self, bone_idx: usize) -> [f32; 6] { self.get_parent_world_matrix(bone_idx) }
    fn update(&mut self) { self.update() }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Skeleton {
    pub fn new() -> Self {
        let mut skel = Self::default();
        skel.add_bone(BoneData::new("root".into(), "root".into()));
        skel
    }

    /// 根据骨骼 ID 获取内部索引
    pub fn bone_id_to_index(&self, id: &str) -> Option<usize> {
        self.name_to_index.get(id).copied()
    }

    pub fn add_bone(&mut self, data: BoneData) {
        let bone = RuntimeBone::new(data);
        self.bones.push(bone);
        
        // 委托给统一的重建逻辑，确保状态一致
        self.rebuild_internal_state();
    }

    pub fn update(&mut self) {
        for i in 0..self.bones.len() {
            let (parent_matrix, local_matrix) = {
                let bone = &self.bones[i];
                let pm = if let Some(p_idx) = bone.parent_index {
                    Some(self.bones[p_idx].world_matrix)
                } else {
                    None
                };
                (pm, bone.local_transform.to_matrix())
            };

            let final_matrix = match parent_matrix {
                None => local_matrix,
                Some(pm) => {
                    let pa = pm[0]; let pb = pm[1];
                    let pc = pm[2]; let pd = pm[3];
                    let px = pm[4]; let py = pm[5];

                    let la = local_matrix[0]; let lb = local_matrix[1];
                    let lc = local_matrix[2]; let ld = local_matrix[3];
                    let lx = local_matrix[4]; let ly = local_matrix[5];
                    let wa = pa * la + pc * lb;
                    let wb = pb * la + pd * lb;
                    let wc = pa * lc + pc * ld;
                    let wd = pb * lc + pd * ld;
                    let wx = pa * lx + pc * ly + px;
                    let wy = pb * lx + pd * ly + py;

                    [wa, wb, wc, wd, wx, wy]
                }
            };

            self.bones[i].world_matrix = final_matrix;
        }
    }
    
    pub fn get_bone_world_position(&self, id: &str) -> Option<(f32, f32)> {
        let idx = self.bone_id_to_index(id)?;
        let m = self.bones[idx].world_matrix;
        Some((m[4], m[5])) 
    }

    pub fn get_parent_world_matrix(&self, bone_idx: usize) -> [f32; 6] {
        match self.bones[bone_idx].parent_index {
            Some(p_idx) => self.bones[p_idx].world_matrix,
            None => [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_hierarchy_math() {
        let mut skel = Skeleton::new();

        if let Some(idx) = skel.bone_id_to_index("root") {
            skel.bones[idx].local_transform.x = 100.0;
            skel.bones[idx].local_transform.y = 100.0;
        }

        let mut child_data = BoneData::new("child".into(), "Child".into());
        child_data.parent_id = Some("root".into());
        child_data.local_transform.x = 50.0;
        skel.add_bone(child_data);

        skel.update();
        
        let (cx, cy) = skel.get_bone_world_position("child").unwrap();
        assert!((cx - 150.0).abs() < 0.001, "Child X 应该是 150, 实际: {}", cx);
        assert!((cy - 100.0).abs() < 0.001, "Child Y 应该是 100, 实际: {}", cy);
    }
}