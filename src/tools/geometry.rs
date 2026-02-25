pub struct Geometry;

impl Geometry {
    pub fn bresenham_line<F>(mut x1: i32, mut y1: i32, x2: i32, y2: i32, mut callback: F)
    where
        F: FnMut(i32, i32),
    {
        let dx = (x2 - x1).abs();
        let dy = -(y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            callback(x1, y1);
            if x1 == x2 && y1 == y2 { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; x1 += sx; }
            if e2 <= dx { err += dx; y1 += sy; }
        }
    }
}

#[cfg(test)]
mod tests;