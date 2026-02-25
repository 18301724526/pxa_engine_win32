use rust_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Add,
}

impl Default for BlendMode {
    fn default() -> Self { Self::Normal }
}

impl BlendMode {
    pub fn name(&self) -> String {
        match self {
            Self::Normal => t!("blend_mode.normal").to_string(),
            Self::Multiply => t!("blend_mode.multiply").to_string(),
            Self::Screen => t!("blend_mode.screen").to_string(),
            Self::Add => t!("blend_mode.add").to_string(),
        }
    }

    pub fn to_u8(self) -> u8 {
        match self { Self::Normal => 0, Self::Multiply => 1, Self::Screen => 2, Self::Add => 3 }
    }

    pub fn from_u8(v: u8) -> Self {
        match v { 1 => Self::Multiply, 2 => Self::Screen, 3 => Self::Add, _ => Self::Normal }
    }
}