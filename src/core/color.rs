#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn transparent() -> Self {
        Self { r: 0, g: 0, b: 0, a: 0 }
    }

    pub fn distance_squared(&self, other: &Self) -> u32 {
        let dr = self.r as i32 - other.r as i32;
        let dg = self.g as i32 - other.g as i32;
        let db = self.b as i32 - other.b as i32;
        (2 * dr * dr + 4 * dg * dg + 1 * db * db) as u32
    }

    /// 按比例加深颜色，用于 Selout 描边（factor: 0.0 ~ 1.0）
    pub fn darken(&self, factor: f32) -> Self {
        let clamp = |v: f32| v.max(0.0).min(255.0) as u8;
        Self::new(
            clamp(self.r as f32 * (1.0 - factor)),
            clamp(self.g as f32 * (1.0 - factor)),
            clamp(self.b as f32 * (1.0 - factor)),
            self.a,
        )
    }
}

#[cfg(test)]
mod tests;