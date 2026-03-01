use crate::app::commands::ResizeAnchor;
use crate::app::state::ToolType;

pub struct UiState {
    pub renaming_layer_id: Option<String>,
    pub renaming_buffer: String,
    pub show_exit_modal: bool,
    pub show_resize_modal: bool,
    pub resize_new_width: String,
    pub resize_new_height: String,
    pub resize_anchor: ResizeAnchor,
    pub selected_layer_ids: Vec<String>,
    pub last_clicked_layer_id: Option<String>,
    pub dragging_layer_id: Option<String>,
    pub drop_target_idx: Option<usize>,
    pub error_message: Option<String>,
    pub language: String,
    pub selected_bone_id: Option<String>,
    pub active_select_tool: ToolType,
    pub show_canvas_menu: bool,
    pub canvas_menu_pos: egui::Pos2,
    pub show_new_anim_modal: bool,
    pub new_anim_name: String,
    pub selected_keyframes: Vec<(String, Option<crate::core::animation::timeline::TimelineProperty>, f32)>,
    pub box_select_start: Option<egui::Pos2>,
    pub show_curve_editor: bool,
    pub graph_pan: egui::Vec2,
    pub graph_zoom: egui::Vec2,
    pub timeline_zoom: f32,
    pub show_offset_modal: bool,
    pub offset_fixed_frames: i32,
    pub offset_step_frames: i32,
    pub offset_mode: usize,
    pub selected_node_idx: Option<usize>,
    pub show_world_transform: bool,
    pub auto_keyframe: bool,
    pub timeline_filter: Vec<crate::core::animation::timeline::TimelineProperty>,
    pub offset_drag_start_x: Option<f32>,
    pub offset_snapshot_anim: Option<crate::core::animation::timeline::Animation>,
    pub offset_snapshot_selection: Vec<(String, Option<crate::core::animation::timeline::TimelineProperty>, f32)>,
    pub is_offset_mode_active: bool,
}

impl UiState {
    pub fn new() -> Self {
        let filter = vec![
            crate::core::animation::timeline::TimelineProperty::Rotation,
            crate::core::animation::timeline::TimelineProperty::Translation,
            crate::core::animation::timeline::TimelineProperty::Scale,
        ];
        Self {
            renaming_layer_id: None,
            renaming_buffer: String::new(),
            show_exit_modal: false,
            show_resize_modal: false,
            resize_new_width: String::new(),
            resize_new_height: String::new(),
            resize_anchor: ResizeAnchor::Center,
            selected_layer_ids: Vec::new(),
            last_clicked_layer_id: None,
            dragging_layer_id: None,
            drop_target_idx: None,
            error_message: None,
            language: "zh-CN".to_string(),
            selected_bone_id: None,
            active_select_tool: ToolType::RectSelect,
            show_canvas_menu: false,
            canvas_menu_pos: egui::Pos2::ZERO,
            show_new_anim_modal: false,
            new_anim_name: String::new(),
            selected_keyframes: Vec::new(),
            box_select_start: None,
            show_curve_editor: false,
            graph_pan: egui::vec2(20.0, 0.0),
            graph_zoom: egui::vec2(100.0, 1.0),
            timeline_zoom: 1.0,
            show_offset_modal: false,
            offset_fixed_frames: 5,
            offset_step_frames: 1,
            offset_mode: 0,
            selected_node_idx: None,
            show_world_transform: false,
            auto_keyframe: true,
            timeline_filter: filter,
            offset_drag_start_x: None,
            offset_snapshot_anim: None,
            offset_snapshot_selection: Vec::new(),
            is_offset_mode_active: false,
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}