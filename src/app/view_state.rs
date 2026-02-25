use crate::core::store::PixelStore;

pub struct ViewState {
    pub width: f32,
    pub height: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub zoom_level: f64,
    pub dirty_rect: Option<(u32, u32, u32, u32)>,
    pub needs_full_redraw: bool,
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            zoom_level: 1.0,
            dirty_rect: None,
            needs_full_redraw: true,
        }
    }

    pub fn update_viewport(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn screen_to_canvas(&self, store: &PixelStore, screen_x: f32, screen_y: f32) -> Option<(u32, u32)> {
        let zoom = self.zoom_level as f32;
        let screen_cx = self.width / 2.0;
        let screen_cy = self.height / 2.0;
        let canvas_cx = store.canvas_width as f32 / 2.0;
        let canvas_cy = store.canvas_height as f32 / 2.0;
        let canvas_x = (screen_x - screen_cx) / zoom + canvas_cx - self.pan_x;
        let canvas_y = (screen_y - screen_cy) / zoom + canvas_cy - self.pan_y;
        let tx = canvas_x.floor() as i32;
        let ty = canvas_y.floor() as i32;
        if tx >= 0 && tx < store.canvas_width as i32 && ty >= 0 && ty < store.canvas_height as i32 {
            Some((tx as u32, ty as u32))
        } else {
            None
        }
    }

    pub fn mark_dirty_path(&mut self, store: &PixelStore, x1: u32, y1: u32, x2: u32, y2: u32) {
        let zoom = self.zoom_level as f32;
        let brush_size = store.brush_size as f32;
        
        let screen_cx = self.width / 2.0;
        let screen_cy = self.height / 2.0;
        let canvas_cx = store.canvas_width as f32 / 2.0;
        let canvas_cy = store.canvas_height as f32 / 2.0;

        let to_screen = |cx: u32, cy: u32| -> (f32, f32) {
             (
                (cx as f32 - canvas_cx + self.pan_x) * zoom + screen_cx,
                (cy as f32 - canvas_cy + self.pan_y) * zoom + screen_cy
             )
        };

        let (sx1, sy1) = to_screen(x1, y1);
        let (sx2, sy2) = to_screen(x2, y2);
        let radius = (brush_size * zoom / 2.0).ceil() as u32 + 4;
        
        let min_x = sx1.min(sx2) as u32;
        let min_y = sy1.min(sy2) as u32;
        let max_x = sx1.max(sx2) as u32;
        let max_y = sy1.max(sy2) as u32;
        
        let x = min_x.saturating_sub(radius);
        let y = min_y.saturating_sub(radius);
        let w = (max_x - min_x) + radius * 2;
        let h = (max_y - min_y) + radius * 2;

        self.union_dirty_rect(x, y, w, h);
    }

    pub fn mark_dirty_canvas_rect(&mut self, store: &PixelStore, x: u32, y: u32, w: u32, h: u32) {
        let zoom = self.zoom_level as f32;
        let s_cx = self.width / 2.0;
        let s_cy = self.height / 2.0;
        let c_cx = store.canvas_width as f32 / 2.0;
        let c_cy = store.canvas_height as f32 / 2.0;

        let to_screen_x = |cx: f32| (cx - c_cx + self.pan_x) * zoom + s_cx;
        let to_screen_y = |cy: f32| (cy - c_cy + self.pan_y) * zoom + s_cy;

        let sx1 = to_screen_x(x as f32);
        let sy1 = to_screen_y(y as f32);
        let sx2 = to_screen_x((x + w) as f32);
        let sy2 = to_screen_y((y + h) as f32);

        let rx = sx1.min(sx2).floor() as u32;
        let ry = sy1.min(sy2).floor() as u32;
        let rw = (sx1.max(sx2) - sx1.min(sx2)).ceil() as u32;
        let rh = (sy1.max(sy2) - sy1.min(sy2)).ceil() as u32;

        self.union_dirty_rect(rx, ry, rw, rh);
    }

    fn union_dirty_rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        match self.dirty_rect {
            None => self.dirty_rect = Some((x, y, w, h)),
            Some((ox, oy, ow, oh)) => {
                let min_x = ox.min(x);
                let min_y = oy.min(y);
                let max_x = (ox + ow).max(x + w);
                let max_y = (oy + oh).max(y + h);
                self.dirty_rect = Some((min_x, min_y, max_x - min_x, max_y - min_y));
            }
        }
    }
}