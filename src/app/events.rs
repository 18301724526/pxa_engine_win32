use crate::core::error::CoreError;
#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    PointerDown { x: i32, y: i32 },
    PointerMove { x: i32, y: i32 },
    PointerUp,
    CancelTool,
    CommitTool,
}
#[derive(Debug)]
pub enum EngineEffect {
    None,
    RedrawCanvas,
    RedrawRect(i32, i32, u32, u32),
    ToolCommitted,
    Error(CoreError),
}

impl EngineEffect {
    pub fn merge(self, other: EngineEffect) -> EngineEffect {
        match (self, other) {
            (EngineEffect::Error(e), _) => EngineEffect::Error(e),
            (_, EngineEffect::Error(e)) => EngineEffect::Error(e),
            (EngineEffect::RedrawCanvas, _) | (_, EngineEffect::RedrawCanvas) => EngineEffect::RedrawCanvas,
            (EngineEffect::RedrawRect(x1, y1, w1, h1), EngineEffect::RedrawRect(x2, y2, w2, h2)) => {
                let min_x = x1.min(x2);
                let min_y = y1.min(y2);
                let max_x = (x1 + w1 as i32).max(x2 + w2 as i32);
                let max_y = (y1 + h1 as i32).max(y2 + h2 as i32);
                EngineEffect::RedrawRect(min_x, min_y, (max_x - min_x) as u32, (max_y - min_y) as u32)
            }
            (e, EngineEffect::None) => e,
            (EngineEffect::None, e) => e,
            _ => EngineEffect::RedrawCanvas,
        }
    }
}