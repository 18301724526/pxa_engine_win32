use crate::app::engine::PxaEngine;
use crate::app::view_state::ViewState;
use crate::app::state::ToolType;
use crate::animation::state::AnimationState;

pub struct PixelEditSession {
    pub engine: PxaEngine,
    pub view: ViewState,
    pub active_select_tool: ToolType,
}

impl PixelEditSession {
    pub fn new(engine: PxaEngine) -> Self {
        Self {
            engine,
            view: ViewState::new(),
            active_select_tool: ToolType::RectSelect,
        }
    }
}

pub struct AnimationSession {
    pub state: AnimationState,
    pub selected_bone_id: Option<String>,
    pub selected_keyframes: Vec<(String, Option<crate::core::animation::timeline::TimelineProperty>, f32)>,
    pub timeline_filter: Vec<crate::core::animation::timeline::TimelineProperty>,
    pub fps: f32,
}

impl AnimationSession {
    pub fn new() -> Self {
        Self {
            state: AnimationState::new(),
            selected_bone_id: None,
            selected_keyframes: Vec::new(),
            timeline_filter: vec![
                crate::core::animation::timeline::TimelineProperty::Rotation,
                crate::core::animation::timeline::TimelineProperty::Translation,
                crate::core::animation::timeline::TimelineProperty::Scale,
            ],
            fps: 30.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{AppState, AppMode};

    #[test]
    fn test_session_isolation() {
        let mut app = AppState::new();
        
        // 验证默认状态
        assert_eq!(app.mode, AppMode::PixelEdit);
        
        // 验证修改动画会话不影响像素会话
        app.anim.selected_bone_id = Some("head".to_string());
        assert!(app.anim.selected_bone_id.is_some());
        assert_eq!(app.pixel.active_select_tool, ToolType::RectSelect);
    }
}