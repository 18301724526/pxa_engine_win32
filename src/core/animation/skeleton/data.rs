use crate::core::animation::bone::BoneData;
use crate::core::animation::transform::Transform;

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