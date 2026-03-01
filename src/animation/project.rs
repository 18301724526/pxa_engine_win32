use crate::core::animation::skeleton::Skeleton;
use crate::core::animation::timeline::Animation;
use std::collections::HashMap;

#[derive(Debug)]
pub struct AnimProject {
    pub skeleton: Skeleton,
    pub animations: HashMap<String, Animation>,
    pub active_animation_id: Option<String>,
}

impl Default for AnimProject {
    fn default() -> Self {
        let mut skeleton = Skeleton::new();
        skeleton.add_bone(crate::core::animation::bone::BoneData::new("root".into(), "root".into()));
        Self {
            skeleton,
            animations: HashMap::new(),
            active_animation_id: None,
        }
    }
}

impl AnimProject {
    pub fn new() -> Self {
        Self::default()
    }
}