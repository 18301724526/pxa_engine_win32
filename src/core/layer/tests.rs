use super::*;
use crate::core::color::Color;
#[test]
fn test_layer_init() {
    let l = Layer::new("I".into(), "N".into(), 2, 2);
    assert_eq!(l.chunks_count(), 0);
}
#[test]
fn test_layer_pixel() {
    let mut l = Layer::new("I".into(), "N".into(), 2, 2);
    l.set_pixel(0, 0, Color::new(1, 1, 1, 255)).unwrap();
    assert_eq!(l.get_pixel(0, 0).unwrap().r, 1);
}
#[test]
fn test_layer_oob() {
    let mut l = Layer::new("I".into(), "N".into(), 2, 2);
    assert!(l.set_pixel(2, 2, Color::transparent()).is_err());
}
#[test]
fn test_layer_lock() {
    let mut l = Layer::new("I".into(), "N".into(), 2, 2);
    l.locked = true;
    assert!(l.set_pixel(0, 0, Color::transparent()).is_err());
}

#[test]
fn test_layer_shift_and_resize() {
    let mut l = Layer::new("1".into(), "1".into(), 10, 10);
    l.set_pixel(5, 5, Color::new(255, 0, 0, 255)).unwrap();
    
    l.shift_and_resize(10, 10, 20, 20);
    assert_eq!(l.width, 20);
    assert_eq!(l.get_pixel(15, 15).unwrap().r, 255);
    assert_eq!(l.get_pixel(5, 5).unwrap().a, 0);
    
    l.shift_and_resize(-20, -20, 5, 5);
    assert_eq!(l.chunks_count(), 0);
}