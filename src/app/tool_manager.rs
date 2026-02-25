use std::collections::HashMap;
use crate::app::state::ToolType;
use crate::tools::tool_trait::Tool;
use crate::tools::pencil::PencilTool;
use crate::tools::bucket::BucketTool;
use crate::tools::eyedropper::EyedropperTool;
use crate::tools::rect_select::RectSelectTool;
use crate::tools::ellipse_select::EllipseSelectTool;
use crate::tools::move_tool::MoveTool;
use crate::tools::transform::TransformTool;
use crate::tools::pen::PenTool;
use crate::tools::create_bone::CreateBoneTool;
use crate::core::store::PixelStore;
use crate::history::patch::ActionPatch;
use crate::core::error::CoreError;
use crate::core::symmetry::SymmetryConfig;

pub struct ToolManager {
    pub active_type: ToolType,
    pub tools: HashMap<ToolType, Box<dyn Tool>>,
    pub is_drawing: bool,
}

impl ToolManager {
    pub fn new() -> Self {
        let mut tools: HashMap<ToolType, Box<dyn Tool>> = HashMap::new();
        tools.insert(ToolType::Pencil, Box::new(PencilTool::new(false)));
        tools.insert(ToolType::Eraser, Box::new(PencilTool::new(true)));
        tools.insert(ToolType::Bucket, Box::new(BucketTool::new()));
        tools.insert(ToolType::Eyedropper, Box::new(EyedropperTool::new()));
        tools.insert(ToolType::RectSelect, Box::new(RectSelectTool::new()));
        tools.insert(ToolType::EllipseSelect, Box::new(EllipseSelectTool::new()));
        tools.insert(ToolType::Move, Box::new(MoveTool::new()));
        tools.insert(ToolType::Transform, Box::new(TransformTool::new()));
        tools.insert(ToolType::Pen, Box::new(PenTool::new()));
        tools.insert(ToolType::CreateBone, Box::new(CreateBoneTool::new()));

        Self {
            active_type: ToolType::Pencil,
            tools,
            is_drawing: false,
        }
    }

    pub fn set_tool(&mut self, tool_type: ToolType) {
        self.active_type = tool_type;
    }

    pub fn handle_pointer_down(&mut self, x: u32, y: u32, store: &mut PixelStore, symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        self.is_drawing = true;
        if let Some(tool) = self.tools.get_mut(&self.active_type) {
            tool.on_pointer_down(x, y, store, symmetry)?;
        }
        Ok(())
    }

    pub fn handle_pointer_move(&mut self, x: u32, y: u32, store: &mut PixelStore, symmetry: &SymmetryConfig) -> Result<(), CoreError> {
        if !self.is_drawing { return Ok(()); }
        if let Some(tool) = self.tools.get_mut(&self.active_type) {
            tool.on_pointer_move(x, y, store, symmetry)?;
        }
        Ok(())
    }

    pub fn handle_pointer_up(&mut self, store: &mut PixelStore) -> Result<Option<ActionPatch>, CoreError> {
        if !self.is_drawing { return Ok(None); }
        self.is_drawing = false;
        if let Some(tool) = self.tools.get_mut(&self.active_type) {
            tool.on_pointer_up(store)
        } else {
            Ok(None)
        }
    }

    pub fn get_transform_params(&self) -> Option<(f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32)> {
        if self.active_type != ToolType::Transform { return None; }
        let tool = self.tools.get(&ToolType::Transform)?;
        tool.as_any().downcast_ref::<TransformTool>().and_then(|t| t.get_transform_params())
    }
}