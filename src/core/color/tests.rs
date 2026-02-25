use super::*;
#[test]
fn test_color_new() {
    let c = Color::new(1, 2, 3, 4);
    assert_eq!(c.r, 1);
}
#[test]
fn test_color_trans() {
    let c = Color::transparent();
    assert_eq!(c.a, 0);
}