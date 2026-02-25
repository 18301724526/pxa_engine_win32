use std::ops::{Add, Sub, Mul};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self { Self { x, y } }
    
    pub fn distance(&self, other: Vec2) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self { Self::new(self.x + other.x, self.y + other.y) }
}
impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self { Self::new(self.x - other.x, self.y - other.y) }
}
impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self { Self::new(self.x * scalar, self.y * scalar) }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeType {
    Corner,
    Smooth,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BezierNode {
    pub anchor: Vec2,
    pub handle_in: Vec2,
    pub handle_out: Vec2,
    pub kind: NodeType,
}

impl BezierNode {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            anchor: Vec2::new(x, y),
            handle_in: Vec2::new(0.0, 0.0),
            handle_out: Vec2::new(0.0, 0.0),
            kind: NodeType::Corner,
        }
    }
    pub fn abs_in(&self) -> Vec2 { self.anchor + self.handle_in }
    pub fn abs_out(&self) -> Vec2 { self.anchor + self.handle_out }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BezierPath {
    pub nodes: Vec<BezierNode>,
    pub is_closed: bool,
}

impl BezierPath {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), is_closed: false }
    }

    pub fn add_node(&mut self, x: f32, y: f32) {
        self.nodes.push(BezierNode::new(x, y));
    }
    pub fn flatten(&self, tolerance: f32) -> Vec<Vec2> {
        let mut points = Vec::new();
        if self.nodes.is_empty() { return points; }

        points.push(self.nodes[0].anchor);

        let count = self.nodes.len();
        if count < 2 { return points; }
        let segments = if self.is_closed { count } else { count - 1 };

        for i in 0..segments {
            let n1 = &self.nodes[i];
            let n2 = &self.nodes[(i + 1) % count];
            let p0 = n1.anchor;
            let p1 = n1.abs_out();
            let p2 = n2.abs_in();
            let p3 = n2.anchor;

            if n1.handle_out == Vec2::new(0.0, 0.0) && n2.handle_in == Vec2::new(0.0, 0.0) {
                points.push(p3);
            } else {
                flatten_cubic_bezier(p0, p1, p2, p3, tolerance, &mut points);
            }
        }

        points
    }
}

fn flatten_cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, tolerance: f32, points: &mut Vec<Vec2>) {
    if is_flat(p0, p1, p2, p3, tolerance) {
        points.push(p3);
    } else {
        let (left, right) = split_cubic(p0, p1, p2, p3, 0.5);
        flatten_cubic_bezier(left.0, left.1, left.2, left.3, tolerance, points);
        flatten_cubic_bezier(right.0, right.1, right.2, right.3, tolerance, points);
    }
}

fn is_flat(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, tolerance: f32) -> bool {
    let ux = 3.0 * p1.x - 2.0 * p0.x - p3.x;
    let uy = 3.0 * p1.y - 2.0 * p0.y - p3.y;
    let vx = 3.0 * p2.x - 2.0 * p3.x - p0.x;
    let vy = 3.0 * p2.y - 2.0 * p3.y - p0.y;
    
    let ux = ux * ux;
    let uy = uy * uy;
    let vx = vx * vx;
    let vy = vy * vy;

    if ux < vx {
        ux + uy <= 16.0 * tolerance * tolerance
    } else {
        vx + vy <= 16.0 * tolerance * tolerance
    }
}

fn split_cubic(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> ((Vec2, Vec2, Vec2, Vec2), (Vec2, Vec2, Vec2, Vec2)) {
    let p01 = lerp(p0, p1, t);
    let p12 = lerp(p1, p2, t);
    let p23 = lerp(p2, p3, t);

    let p012 = lerp(p01, p12, t);
    let p123 = lerp(p12, p23, t);

    let mid = lerp(p012, p123, t);

    ((p0, p01, p012, mid), (mid, p123, p23, p3))
}

fn lerp(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a * (1.0 - t) + b * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_ops() {
        let v1 = Vec2::new(10.0, 20.0);
        let v2 = Vec2::new(5.0, 5.0);
        assert_eq!(v1 + v2, Vec2::new(15.0, 25.0));
        assert_eq!(v1 - v2, Vec2::new(5.0, 15.0));
        assert_eq!(v1 * 2.0, Vec2::new(20.0, 40.0));
    }

    #[test]
    fn test_flatten_straight_line() {
        let mut path = BezierPath::new();
        path.add_node(0.0, 0.0);
        path.add_node(100.0, 0.0);
        
        let points = path.flatten(1.0);
        assert_eq!(points.len(), 2);
        assert_eq!(points[0], Vec2::new(0.0, 0.0));
        assert_eq!(points[1], Vec2::new(100.0, 0.0));
    }

    #[test]
    fn test_flatten_curve() {
        let mut path = BezierPath::new();
        let mut n1 = BezierNode::new(0.0, 0.0);
        n1.handle_out = Vec2::new(50.0, 50.0);
        path.nodes.push(n1);

        let mut n2 = BezierNode::new(100.0, 0.0);
        n2.handle_in = Vec2::new(-50.0, 50.0);
        path.nodes.push(n2);

        let points = path.flatten(0.5);
        
        assert!(points.len() > 2);
        
        let mid_idx = points.len() / 2;
        let mid_pt = points[mid_idx];
        
        println!("Mid point: {:?}", mid_pt);
        assert!((mid_pt.x - 50.0).abs() < 5.0);
        assert!(mid_pt.y > 0.0);
    }
}