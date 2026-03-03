use crate::app::command_handler::{Command, AppEvent};
use crate::app::state::{AppState, AppMode};
use std::collections::VecDeque;

pub struct UndoCmd;
impl Command for UndoCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        if state.mode == AppMode::Animation {
            if state.anim.state.history.undo(&mut state.anim.state.project) {
                state.is_dirty = true; state.pixel.view.needs_full_redraw = true;
                crate::animation::controller::AnimationController::apply_current_pose(&mut state.anim.state);
            }
        } else {
            state.pixel.engine.undo().map_err(|e| e.to_string())?;
            state.is_dirty = true; state.pixel.view.needs_full_redraw = true;
        }
        Ok(())
    }
}

pub struct RedoCmd;
impl Command for RedoCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        if state.mode == AppMode::Animation {
            if state.anim.state.history.redo(&mut state.anim.state.project) {
                state.is_dirty = true; state.pixel.view.needs_full_redraw = true;
                crate::animation::controller::AnimationController::apply_current_pose(&mut state.anim.state);
            }
        } else {
            state.pixel.engine.redo().map_err(|e| e.to_string())?;
            state.is_dirty = true; state.pixel.view.needs_full_redraw = true;
        }
        Ok(())
    }
}

pub struct RequestExitCmd;
impl Command for RequestExitCmd {
    fn execute(&self, state: &mut AppState, events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        if state.is_dirty { events.push_back(AppEvent::ShowExitModal); }
        else { events.push_back(AppEvent::CloseWindow); }
        Ok(())
    }
}

pub struct ConfirmExitCmd;
impl Command for ConfirmExitCmd {
    fn execute(&self, _state: &mut AppState, events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        events.push_back(AppEvent::CloseWindow);
        Ok(())
    }
}

pub struct CancelExitCmd;
impl Command for CancelExitCmd {
    fn execute(&self, _state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        Ok(())
    }
}

pub struct SaveProjectCmd;
impl Command for SaveProjectCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.save_project_to_pxad();
        Ok(())
    }
}

pub struct LoadProjectCmd;
impl Command for LoadProjectCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.load_project_from_pxad();
        Ok(())
    }
}

pub struct SetLanguageCmd(pub String);
impl Command for SetLanguageCmd {
    fn execute(&self, _state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        rust_i18n::set_locale(&self.0);
        Ok(())
    }
}

pub struct WindowDragCmd;
impl Command for WindowDragCmd {
    fn execute(&self, _state: &mut AppState, events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        events.push_back(AppEvent::DragWindow);
        Ok(())
    }
}

pub struct WindowMinimizeCmd;
impl Command for WindowMinimizeCmd {
    fn execute(&self, _state: &mut AppState, events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        events.push_back(AppEvent::MinimizeWindow);
        Ok(())
    }
}

pub struct WindowMaximizeCmd;
impl Command for WindowMaximizeCmd {
    fn execute(&self, _state: &mut AppState, events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        events.push_back(AppEvent::MaximizeWindow);
        Ok(())
    }
}

pub struct ImportImageCmd;
impl Command for ImportImageCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.import_image();
        Ok(())
    }
}

pub struct ExportPngCmd;
impl Command for ExportPngCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.export_to_png();
        Ok(())
    }
}

pub struct ExportGifCmd;
impl Command for ExportGifCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.export_animation(true);
        Ok(())
    }
}

pub struct ExportSequenceCmd;
impl Command for ExportSequenceCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.export_animation(false);
        Ok(())
    }
}