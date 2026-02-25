use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub shear_x: f32,
    pub shear_y: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            shear_x: 0.0,
            shear_y: 0.0,
        }
    }
}

impl Transform {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn to_matrix(&self) -> [f32; 6] {
        let r = self.rotation * PI / 180.0;
        let sx = self.shear_x * PI / 180.0;
        let sy = self.shear_y * PI / 180.0;

        let cos = r.cos();
        let sin = r.sin();

        let la = cos * self.scale_x;
        let lb = sin * self.scale_x;
        let lc = -sin * self.scale_y;
        let ld = cos * self.scale_y;

        let tan_sx = sx.tan();
        let tan_sy = sy.tan();

        let a = la + tan_sy * lc;
        let b = lb + tan_sy * ld;
        let c = lc + tan_sx * la;
        let d = ld + tan_sx * lb;

        [a, b, c, d, self.x, self.y]
    }

    pub fn apply_parent(&self, parent: &Transform) -> Transform {
        Transform {
            x: parent.x + self.x,
            y: parent.y + self.y,
            rotation: parent.rotation + self.rotation,
            scale_x: parent.scale_x * self.scale_x,
            scale_y: parent.scale_y * self.scale_y,
            shear_x: parent.shear_x + self.shear_x,
            shear_y: parent.shear_y + self.shear_y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_matrix_identity() {
        let t = Transform::default();
        let m = t.to_matrix();
        assert!((m[0] - 1.0).abs() < 1e-5);
        assert!((m[1] - 0.0).abs() < 1e-5);
        assert!((m[2] - 0.0).abs() < 1e-5);
        assert!((m[3] - 1.0).abs() < 1e-5);
        assert!((m[4] - 0.0).abs() < 1e-5);
        assert!((m[5] - 0.0).abs() < 1e-5);
    }
}