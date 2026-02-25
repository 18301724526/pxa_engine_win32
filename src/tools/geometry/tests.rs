use super::*;
#[test]
fn test_geom_line() {
    let mut p = Vec::new();
    Geometry::bresenham_line(0, 0, 2, 2, |x, y| p.push((x, y)));
    assert_eq!(p.len(), 3);
}
#[test]
fn test_geom_point() {
    let mut p = Vec::new();
    Geometry::bresenham_line(1, 1, 1, 1, |x, y| p.push((x, y)));
    assert_eq!(p.len(), 1);
}