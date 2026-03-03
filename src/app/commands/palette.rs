use crate::app::command_handler::{Command, AppEvent};
use crate::app::state::AppState;
use crate::core::color::Color;
use std::collections::VecDeque;

pub struct SetPrimaryColorCmd(pub Color);
impl Command for SetPrimaryColorCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.pixel.engine.set_primary_color(self.0);
        Ok(())
    }
}

pub struct AddColorToPaletteCmd(pub Color);
impl Command for AddColorToPaletteCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.pixel.engine.add_color_to_palette(self.0);
        state.is_dirty = true;
        Ok(())
    }
}

pub struct RemovePaletteColorCmd(pub usize);
impl Command for RemovePaletteColorCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.pixel.engine.remove_palette_color(self.0);
        state.is_dirty = true;
        Ok(())
    }
}

pub struct SetPaletteCmd(pub crate::core::palette::Palette);
impl Command for SetPaletteCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.pixel.engine.set_palette(self.0.clone());
        state.is_dirty = true;
        Ok(())
    }
}

pub struct ExportPaletteCmd;
impl Command for ExportPaletteCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.export_palette();
        Ok(())
    }
}

pub struct ImportPaletteCmd;
impl Command for ImportPaletteCmd {
    fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
        state.import_palette();
        Ok(())
    }
}