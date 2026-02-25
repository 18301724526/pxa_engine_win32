use super::*;
use crate::core::layer::Layer;

#[test]
fn test_store_defaults() {
    let s = PixelStore::new(10, 10);
    assert_eq!(s.brush_size, 1);
}

#[test]
fn test_store_add_layer() {
    let mut s = PixelStore::new(10, 10);
    s.add_layer(Layer::new("L".into(), "L".into(), 10, 10));
    assert_eq!(s.layers.len(), 1);
}

#[test]
fn test_store_active_id_logic() {
    let mut s = PixelStore::new(10, 10);
    s.add_layer(Layer::new("1".into(), "1".into(), 10, 10));
    s.add_layer(Layer::new("2".into(), "2".into(), 10, 10));
    assert_eq!(s.active_layer_id, Some("1".into()));
}

#[test]
fn test_store_pixel_routing() {
    let mut s = PixelStore::new(10, 10);
    s.add_layer(Layer::new("L".into(), "L".into(), 10, 10));
    s.mut_set_pixel("L", 0, 0, Color::new(1, 1, 1, 255)).unwrap();
    assert_eq!(s.get_pixel("L", 0, 0).unwrap().r, 1);
}

#[test]
fn test_store_invalid_id() {
    let mut s = PixelStore::new(10, 10);
    assert!(s.mut_set_pixel("X", 0, 0, Color::transparent()).is_err());
}

#[test]
fn test_store_layer_ordering() {
    let mut s = PixelStore::new(10, 10);
    s.add_layer(Layer::new("1".into(), "1".into(), 10, 10));
    s.add_layer_at(Layer::new("0".into(), "0".into(), 10, 10), 0);
    assert_eq!(s.layers[0].id, "0");
}

#[test]
fn test_store_remove_active_switch() {
    let mut s = PixelStore::new(10, 10);
    s.add_layer(Layer::new("1".into(), "1".into(), 10, 10));
    s.add_layer(Layer::new("2".into(), "2".into(), 10, 10));
    s.active_layer_id = Some("2".into());
    s.remove_layer_by_id("2");
    assert_eq!(s.active_layer_id, Some("1".into()));
}

#[test]
fn test_store_visibility() {
    let mut s = PixelStore::new(10, 10);
    s.add_layer(Layer::new("L".into(), "L".into(), 10, 10));
    s.set_layer_visibility("L", false);
    assert_eq!(s.get_layer("L").unwrap().visible, false);
}

#[test]
fn test_store_bounds_check() {
    let mut s = PixelStore::new(10, 10);
    s.add_layer(Layer::new("L".into(), "L".into(), 10, 10));
    assert!(s.get_pixel("L", 10, 10).is_none());
}