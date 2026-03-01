use crate::animation::project::AnimProject;
use crate::animation::history::AnimHistory;

pub struct AnimationState {
    pub project: AnimProject,
    pub history: AnimHistory,
    pub drag_start_skeleton: Option<crate::core::animation::skeleton::Skeleton>,
    pub drag_start_animation: Option<crate::core::animation::timeline::Animation>,
    pub current_time: f32,
    pub is_playing: bool,
    pub playback_speed: f32,
    pub create_bone_tool: crate::tools::create_bone::CreateBoneTool,
    pub is_looping: bool,
}

impl AnimationState {
    pub fn new() -> Self {
        Self {
            project: AnimProject::new(),
            history: AnimHistory::new(),
            drag_start_skeleton: None,
            drag_start_animation: None,
            current_time: 0.0,
            is_playing: false,
            playback_speed: 1.0,
            create_bone_tool: crate::tools::create_bone::CreateBoneTool::new(),
            is_looping: true,
        }
    }
    
    pub fn auto_key_bone(&mut self, bone_id: &str, property: crate::core::animation::timeline::TimelineProperty) {
        let time = self.current_time;
        let active_id = match &self.project.active_animation_id {
            Some(id) => id.clone(),
            None => return,
        };
        
        let transform = match self.project.skeleton.bones.iter().find(|b| b.data.id == bone_id) {
            Some(b) => b.local_transform,
            None => return,
        };
        
        if let Some(anim) = self.project.animations.get_mut(&active_id) {
            let idx = anim.timelines.iter().position(|t| t.target_id == bone_id && t.property == property)
                .unwrap_or_else(|| {
                    anim.timelines.push(crate::core::animation::timeline::Timeline::new(bone_id.to_string(), property.clone()));
                    anim.timelines.len() - 1
                });

            let timeline = &mut anim.timelines[idx];
            use crate::core::animation::timeline::{KeyframeValue, CurveType};
            match property {
                crate::core::animation::timeline::TimelineProperty::Rotation => 
                    timeline.add_keyframe(time, KeyframeValue::Rotate(transform.rotation), CurveType::Linear),
                crate::core::animation::timeline::TimelineProperty::Translation => 
                    timeline.add_keyframe(time, KeyframeValue::Translate(transform.x, transform.y), CurveType::Linear),
                crate::core::animation::timeline::TimelineProperty::Scale => 
                    timeline.add_keyframe(time, KeyframeValue::Scale(transform.scale_x, transform.scale_y), CurveType::Linear),
                _ => {}
            }
            anim.recalculate_duration();
        }
    }
}