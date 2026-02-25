use crate::core::color::Color;
use crate::core::blend_mode::BlendMode;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeAnchor {
    TopLeft,    TopCenter,    TopRight,
    MiddleLeft, Center,       MiddleRight,
    BottomLeft, BottomCenter, BottomRight,
}
#[derive(Debug, Clone, PartialEq)]
pub enum AppCommand {
    AddColorToPalette(Color),
    RemovePaletteColor(usize),
    SetPrimaryColor(Color),
    ToggleLayerLock(String),
    SetLayerOpacity(String, u8),
    MoveLayerUp(String),
    MoveLayerDown(String),
    RenameLayer(String, String),
    SetLayerBlendMode(String, BlendMode),
    ImportPalette,
    ExportPalette,
    SetPalette(crate::core::palette::Palette),
    ClearSelection,
    InvertSelection,
    StrokeSelection(u32),
    DuplicateLayer(String),
    MergeSelected(Vec<String>),
    MoveLayerToIndex(String, usize),

    WindowDrag,
    RequestExit,
    ConfirmExit,
    CancelExit,
    WindowClose,
    WindowMinimize,
    WindowMaximize,
    SaveProject,
    LoadProject,
    ImportImage,
    ExportPng,
    Undo,
    Redo,
    ResizeCanvas(u32, u32, ResizeAnchor),
    CommitCurrentTool,
    CancelCurrentTool,
    SetLanguage(String),
    PenFill,
    PenStroke,
}