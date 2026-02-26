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
    pub selected_keyframe: Option<(String, f32)>,
}

impl UiState {
    pub fn new() -> Self {
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
            selected_keyframe: None,
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}