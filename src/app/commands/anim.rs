use crate::app::command_handler::{Command, AppEvent};
use crate::app::state::AppState;
use crate::app::handlers::anim_handler;
use crate::app::handlers::setup_handler;
use crate::core::animation::timeline::{TimelineProperty, CurveType};
use std::collections::VecDeque;

pub struct CreateAnimationCmd(pub String);
impl Command for CreateAnimationCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        anim_handler::create_animation(state, &self.0)
    }
}

pub struct SelectAnimationCmd(pub String);
impl Command for SelectAnimationCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        anim_handler::select_animation(state, &self.0)
    }
}

pub struct TogglePlaybackCmd;
impl Command for TogglePlaybackCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.anim.state.is_playing = !state.anim.state.is_playing;
        Ok(())
    }
}

pub struct SetTimeCmd(pub f32);
impl Command for SetTimeCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        anim_handler::set_time(state, self.0)
    }
}

pub struct StepFrameCmd(pub i32);
impl Command for StepFrameCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        anim_handler::step_frame(state, self.0)
    }
}

pub struct ToggleLoopCmd;
impl Command for ToggleLoopCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        anim_handler::toggle_loop(state)
    }
}

pub struct SetPlaybackSpeedCmd(pub f32);
impl Command for SetPlaybackSpeedCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.anim.state.playback_speed = self.0.max(0.1);
        Ok(())
    }
}

pub struct ToggleTimelineFilterCmd(pub TimelineProperty);
impl Command for ToggleTimelineFilterCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        anim_handler::toggle_timeline_filter(state, self.0.clone())
    }
}

pub struct InsertManualKeyframeCmd(pub String);
impl Command for InsertManualKeyframeCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        anim_handler::insert_manual_keyframe(state, &self.0)
    }
}

pub struct UpdateKeyframeCurveCmd(pub String, pub TimelineProperty, pub f32, pub CurveType);
impl Command for UpdateKeyframeCurveCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        anim_handler::update_keyframe_curve(state, &self.0, self.1.clone(), self.2, self.3)
    }
}

pub struct DeleteKeyframeCmd(pub String, pub Option<TimelineProperty>, pub f32);
impl Command for DeleteKeyframeCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        anim_handler::delete_keyframe(state, &self.0, self.1.clone(), self.2)
    }
}

pub struct MoveSelectedKeyframesCmd(pub f32);
impl Command for MoveSelectedKeyframesCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        anim_handler::move_selected_keyframes(state, self.0)
    }
}

pub struct BeginOffsetSnapshotCmd;
impl Command for BeginOffsetSnapshotCmd {
    fn execute(&self, _state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> { Ok(()) }
}

pub struct CommitOffsetSnapshotCmd;
impl Command for CommitOffsetSnapshotCmd {
    fn execute(&self, _state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> { Ok(()) }
}

pub struct OffsetSelectedKeyframesCmd(pub f32);
impl Command for OffsetSelectedKeyframesCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> { 
        anim_handler::offset_selected_keyframes(state, self.0) 
    }
}

pub struct ApplySpineOffsetCmd { pub mode: usize, pub fixed_frames: i32, pub step_frames: i32 }
impl Command for ApplySpineOffsetCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> { 
        anim_handler::apply_spine_offset(state, self.mode, self.fixed_frames, self.step_frames) 
    }
}

pub struct BindLayerToBoneCmd(pub String, pub String);
impl Command for BindLayerToBoneCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        setup_handler::bind_layer_to_bone(state, &self.0, &self.1)
    }
}

pub struct DeleteBoneCmd(pub String);
impl Command for DeleteBoneCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        setup_handler::delete_bone(state, &self.0)
    }
}

pub struct CreateBoneCmd {
    pub start: (f32, f32),
    pub end: (f32, f32),
    pub parent_id: Option<String>,
}

impl Command for CreateBoneCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        if let Some(new_id) = setup_handler::create_bone(state, self.start, self.end, self.parent_id.clone())? {
            state.anim.selected_bone_id = Some(new_id);
        }
        Ok(())
    }
}

pub struct ToggleTransformCoordinateSystemCmd;
impl Command for ToggleTransformCoordinateSystemCmd {
    fn execute(&self, _state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        // 该命令目前作为占位符，UI 层的坐标系切换逻辑目前主要由 UiContext.show_world_transform 控制
        Ok(())
    }
}