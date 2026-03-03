use crate::app::command_handler::{Command, AppEvent};
use crate::app::state::AppState;
use crate::app::handlers::layer_handler;
use crate::core::blend_mode::BlendMode;
use std::collections::VecDeque;

pub struct ToggleLayerLockCmd(pub String);
impl Command for ToggleLayerLockCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        layer_handler::toggle_layer_lock(state, &self.0)
    }
}

pub struct SetLayerOpacityCmd(pub String, pub u8);
impl Command for SetLayerOpacityCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        layer_handler::set_layer_opacity(state, &self.0, self.1)
    }
}

pub struct SetLayerBlendModeCmd(pub String, pub BlendMode);
impl Command for SetLayerBlendModeCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        layer_handler::set_layer_blend_mode(state, &self.0, self.1)
    }
}

pub struct RenameLayerCmd(pub String, pub String);
impl Command for RenameLayerCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        layer_handler::rename_layer(state, &self.0, &self.1)
    }
}

pub struct DuplicateLayerCmd(pub String);
impl Command for DuplicateLayerCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        layer_handler::duplicate_layer(state, &self.0)
    }
}

pub struct MergeSelectedCmd(pub Vec<String>);
impl Command for MergeSelectedCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        layer_handler::merge_selected(state, self.0.clone())
    }
}

pub struct MoveLayerToIndexCmd(pub String, pub usize);
impl Command for MoveLayerToIndexCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        layer_handler::move_layer_to_index(state, &self.0, self.1)
    }
}