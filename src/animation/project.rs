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
        Self {
            skeleton: Skeleton::new(),
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