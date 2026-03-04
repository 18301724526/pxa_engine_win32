#[derive(Debug, Clone)]
pub struct PixelizeConfig {
    pub target_w: u32,
    pub target_h: u32,
    pub contrast: f32,
    pub brightness: f32,
    pub color_count: usize,
    pub min_color_distance: u32,
    pub use_selout: bool, // 是否开启智能描边
}

impl Default for PixelizeConfig {
    fn default() -> Self {
        Self {
            target_w: 128,
            target_h: 128,
            contrast: 1.12,
            brightness: 0.0,
            color_count: 32,
            min_color_distance: 1500,
            use_selout: false,
        }
    }
}