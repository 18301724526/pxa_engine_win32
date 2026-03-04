use rust_i18n::t;

#[derive(Debug, Clone)]
pub enum CoreError {
    LayerLocked,
    OutOfBounds { x: i32, y: i32 },
    LayerNotFound(String),
    BoneNotFound(String),
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::LayerLocked => write!(f, "{}", t!("error.layer_locked")),
            CoreError::OutOfBounds { x, y } => write!(f, "{}", t!("error.out_of_bounds", x = x, y = y)),
            CoreError::LayerNotFound(id) => write!(f, "{}", t!("error.layer_not_found", id = id)),
            CoreError::BoneNotFound(id) => write!(f, "Bone not found: {}", id),
        }
    }
}

impl std::error::Error for CoreError {}
pub type Result<T> = std::result::Result<T, CoreError>;