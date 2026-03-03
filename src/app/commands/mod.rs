pub mod layer;
pub mod anim;
pub mod tool;
pub mod system;
pub mod palette;

// 重新导出所有子模块，保持外部 API 不变
pub use layer::*;
pub use anim::*;
pub use tool::*;
pub use system::*;
pub use palette::*;

// 共享的 UI 相关枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeAnchor {
    TopLeft,    TopCenter,    TopRight,
    MiddleLeft, Center,       MiddleRight,
    BottomLeft, BottomCenter, BottomRight,
}