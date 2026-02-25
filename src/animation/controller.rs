use crate::app::state::AnimationState;
use std::time::Duration;

pub struct AnimationController;

impl AnimationController {
    pub fn update(state: &mut AnimationState, delta: Duration) {
        if !state.is_playing {
            Self::apply_current_pose(state);
            return;
        }

        let dt = delta.as_secs_f32() * state.playback_speed;
        state.current_time += dt;

        Self::apply_current_pose(state);
    }

    pub fn apply_current_pose(state: &mut AnimationState) {
        let project = &mut state.project;
        let active_anim_id = match &project.active_animation_id {
            Some(id) => id,
            None => return,
        };

        if let Some(anim) = project.animations.get(active_anim_id) {
            anim.apply(&mut project.skeleton, state.current_time);
            project.skeleton.update();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::animation::bone::BoneData;
    use crate::core::animation::timeline::{Animation, Timeline, TimelineProperty, KeyframeValue, CurveType};

    #[test]
    fn test_controller_playback_flow() {
        let mut state = AnimationState::new();
        
        let root = BoneData::new("root".into(), "Root".into()); 
        state.project.skeleton.add_bone(root);

        let mut anim = Animation::new("test".into(), 1.0);
        let mut tl = Timeline::new("root".into(), TimelineProperty::Rotation);
        tl.add_keyframe(0.0, KeyframeValue::Rotate(0.0), CurveType::Linear);
        tl.add_keyframe(1.0, KeyframeValue::Rotate(100.0), CurveType::Linear);
        anim.timelines.push(tl);
        
        state.project.animations.insert("anim1".into(), anim);
        state.project.active_animation_id = Some("anim1".into());

        state.is_playing = true;
        AnimationController::update(&mut state, Duration::from_millis(500));
        assert!((state.current_time - 0.5).abs() < 0.001);
        
        let rot = state.project.skeleton.bones[0].local_transform.rotation;
        assert!((rot - 50.0).abs() < 0.001);
    }
}