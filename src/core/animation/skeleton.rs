use super::bone::BoneData;
use super::slot::RuntimeSlot;
use super::transform::Transform;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RuntimeBone {
    pub data: BoneData,
    pub local_transform: Transform,
    pub world_matrix: [f32; 6], 
    pub parent_index: Option<usize>,
}

impl RuntimeBone {
    pub fn new(data: BoneData) -> Self {
        let local_transform = data.local_transform;
        Self {
            data,
            local_transform,
            world_matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            parent_index: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Skeleton {
    pub bones: Vec<RuntimeBone>,
    pub slots: Vec<RuntimeSlot>,
    name_to_index: HashMap<String, usize>,
}

impl Skeleton {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_bone(&mut self, data: BoneData) {
        let mut bone = RuntimeBone::new(data);
        
        if let Some(parent_id) = &bone.data.parent_id {
            if let Some(&idx) = self.name_to_index.get(parent_id) {
                bone.parent_index = Some(idx);
            }
        }

        self.name_to_index.insert(bone.data.id.clone(), self.bones.len());
        self.bones.push(bone);
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
        let idx = self.name_to_index.get(id)?;
        let m = self.bones[*idx].world_matrix;
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

        let mut root_data = BoneData::new("root".into(), "Root".into());
        root_data.local_transform.x = 100.0;
        root_data.local_transform.y = 100.0;
        skel.add_bone(root_data);

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