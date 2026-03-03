use std::any::Any;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymmetryMode {
    None,
    Horizontal,
    Vertical,
    Quad,
    Translational,
}
pub trait SymmetryProvider: Send + Sync {
    fn apply_symmetry(&self, x: i32, y: i32, callback: &mut dyn FnMut(i32, i32));
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Debug, Clone, Copy)]
pub struct SymmetryConfig {
    pub mode: SymmetryMode,
    pub axis_x: f32,
    pub axis_y: f32,
    pub translation_dx: i32,
    pub translation_dy: i32,
    pub visible_guides: bool,
}

impl SymmetryConfig {
    pub fn new(canvas_width: u32, canvas_height: u32) -> Self {
        Self {
            mode: SymmetryMode::None,
            axis_x: canvas_width as f32 / 2.0,
            axis_y: canvas_height as f32 / 2.0,
            translation_dx: 50,
            translation_dy: 0,
            visible_guides: true,
        }
    }
}

impl SymmetryProvider for SymmetryConfig {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }

    fn apply_symmetry(&self, x: i32, y: i32, callback: &mut dyn FnMut(i32, i32)) {
        callback(x, y);

        match self.mode {
            SymmetryMode::None => {}
            SymmetryMode::Horizontal => {
                let sym_x = (2.0 * self.axis_x - x as f32).round() as i32;
                if sym_x != x {
                    callback(sym_x, y);
                }
            }
            SymmetryMode::Vertical => {
                let sym_y = (2.0 * self.axis_y - y as f32).round() as i32;
                if sym_y != y {
                    callback(x, sym_y);
                }
            }
            SymmetryMode::Quad => {
                let sym_x = (2.0 * self.axis_x - x as f32).round() as i32;
                let sym_y = (2.0 * self.axis_y - y as f32).round() as i32;
                
                if sym_x != x { callback(sym_x, y); }
                if sym_y != y { callback(x, sym_y); }
                if sym_x != x && sym_y != y { callback(sym_x, sym_y); }
            }
            SymmetryMode::Translational => {
                callback(x + self.translation_dx, y + self.translation_dy);
            }
        }
    }
}
impl SymmetryConfig {
    #[inline]
    pub fn apply_symmetry_inline<F>(&self, x: i32, y: i32, mut callback: F)
    where
        F: FnMut(i32, i32),
    {
        self.apply_symmetry(x, y, &mut callback);
    }
}