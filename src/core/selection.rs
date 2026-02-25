use crate::core::path::Vec2;

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionData {
    pub mask: Vec<bool>,
    pub width: u32,
    pub height: u32,
    pub is_active: bool,
}

impl SelectionData {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            mask: vec![false; (width * height) as usize],
            width,
            height,
            is_active: false,
        }
    }

    pub fn clear(&mut self) {
        self.mask.fill(false);
        self.is_active = false;
    }

    pub fn set_rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        self.clear();
        self.is_active = true;
        let end_x = (x + w).min(self.width);
        let end_y = (y + h).min(self.height);
        for cy in y..end_y {
            for cx in x..end_x {
                let idx = (cy * self.width + cx) as usize;
                self.mask[idx] = true;
            }
        }
    }

    pub fn invert(&mut self) {
        for val in self.mask.iter_mut() {
            *val = !*val;
        }
    }

    pub fn set_ellipse(&mut self, x: u32, y: u32, w: u32, h: u32) {
        self.clear();
        self.is_active = true;
        let end_x = (x + w).min(self.width);
        let end_y = (y + h).min(self.height);
        let cx = x as f32 + w as f32 / 2.0;
        let cy = y as f32 + h as f32 / 2.0;
        let rx = w as f32 / 2.0;
        let ry = h as f32 / 2.0;

        if rx <= 0.0 || ry <= 0.0 { return; }

        for py in y..end_y {
            for px in x..end_x {
                let dx = px as f32 + 0.5 - cx;
                let dy = py as f32 + 0.5 - cy;
                if (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry) <= 1.0 {
                    let idx = (py * self.width + px) as usize;
                    self.mask[idx] = true;
                }
            }
        }
    }

    pub fn set_from_polygon(&mut self, points: &[Vec2]) {
        self.clear();
        if points.len() < 3 { return; }
        
        self.is_active = true;

        let mut min_y = self.height as i32;
        let mut max_y = -1;

        for p in points {
            let y = p.y.round() as i32;
            if y < min_y { min_y = y; }
            if y > max_y { max_y = y; }
        }

        min_y = min_y.max(0);
        max_y = max_y.min(self.height as i32 - 1);

        for y in min_y..=max_y {
            let mut nodes = Vec::new();
            let count = points.len();

            for i in 0..count {
                let p1 = points[i];
                let p2 = points[(i + 1) % count];

                if (p1.y < y as f32 && p2.y >= y as f32) || (p2.y < y as f32 && p1.y >= y as f32) {

                    let x = p1.x + (y as f32 - p1.y) * (p2.x - p1.x) / (p2.y - p1.y);
                    nodes.push(x);
                }
            }

            nodes.sort_by(|a, b| a.partial_cmp(b).unwrap());

            for i in (0..nodes.len()).step_by(2) {
                if i + 1 >= nodes.len() { break; }
                
                let x_start = nodes[i].round() as i32;
                let x_end = nodes[i+1].round() as i32;

                let start = x_start.max(0).min(self.width as i32 - 1);
                let end = x_end.max(0).min(self.width as i32);

                for x in start..end {
                    let idx = (y as u32 * self.width + x as u32) as usize;
                    if idx < self.mask.len() {
                        self.mask[idx] = true;
                    }
                }
            }
        }
    }

    #[inline(always)]
    pub fn contains(&self, x: u32, y: u32) -> bool {
        if !self.is_active { return true; } 
        if x >= self.width || y >= self.height { return false; }
        self.mask[(y * self.width + x) as usize]
    }

    pub fn shift_and_resize(&mut self, dx: i32, dy: i32, new_width: u32, new_height: u32) {
        let mut new_mask = vec![false; (new_width * new_height) as usize];
        if self.is_active {
            for y in 0..self.height {
                for x in 0..self.width {
                    if self.mask[(y * self.width + x) as usize] {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < new_width as i32 && ny >= 0 && ny < new_height as i32 {
                            new_mask[(ny as u32 * new_width + nx as u32) as usize] = true;
                        }
                    }
                }
            }
        }
        self.mask = new_mask;
        self.width = new_width;
        self.height = new_height;
    }
}