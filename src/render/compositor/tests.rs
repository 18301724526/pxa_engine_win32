use super::*;
use crate::core::layer::Layer;
use crate::core::color::Color;
use crate::core::store::PixelStore;

fn full_view(s: &PixelStore) -> Viewport {
    Viewport { 
        screen_width: s.canvas_width, 
        screen_height: s.canvas_height, 
        zoom: 1.0, 
        pan_x: 0.0, 
        pan_y: 0.0 
    }
}

#[test]
fn test_compositor_flattening() {
    let mut s = PixelStore::new(1, 1);
    s.add_layer(Layer::new("L".into(), "L".into(), 1, 1));
    s.mut_set_pixel("L", 0, 0, Color::new(255, 0, 0, 255)).unwrap();
    let mut f = vec![0u8; 4];
    Compositor::render(&s, &mut f, full_view(&s));
    assert_eq!(f[0], 255);
}

#[test]
fn test_compositor_bg_fill() {
    let s = PixelStore::new(10, 10);
    let mut f = vec![0u8; 10 * 10 * 4];
    Compositor::render(&s, &mut f, full_view(&s));
    assert_eq!(f[0], 45); 
    let idx_8_0 = (8 * 4) as usize;
    assert_eq!(f[idx_8_0], 30);
}

#[test]
fn test_compositor_order() {
    let mut s = PixelStore::new(1, 1);
    let mut l0 = Layer::new("0".into(), "0".into(), 1, 1);
    l0.set_pixel(0, 0, Color::new(0, 255, 0, 255)).unwrap();
    s.add_layer(l0);
    let mut l1 = Layer::new("1".into(), "1".into(), 1, 1);
    l1.set_pixel(0, 0, Color::new(255, 0, 0, 255)).unwrap();
    s.add_layer(l1);
    let mut f = vec![0u8; 4];
    Compositor::render(&s, &mut f, full_view(&s));
    assert_eq!(f[0], 255); 
}

#[test]
fn test_compositor_empty() {
    let s = PixelStore::new(1, 1);
    let mut f = vec![0u8; 4];
    Compositor::render(&s, &mut f, full_view(&s));
    assert_eq!(f[0], 45);
}

#[test]
fn test_compositor_alpha_skip() {
    let mut s = PixelStore::new(1, 1);
    let mut l0 = Layer::new("0".into(), "0".into(), 1, 1);
    l0.set_pixel(0, 0, Color::new(0, 0, 255, 255)).unwrap();
    s.add_layer(l0);
    let mut l1 = Layer::new("1".into(), "1".into(), 1, 1);
    l1.set_pixel(0, 0, Color::new(255, 0, 0, 0)).unwrap(); 
    s.add_layer(l1);
    let mut f = vec![0u8; 4];
    Compositor::render(&s, &mut f, full_view(&s));
    assert_eq!(f[2], 255); 
}

#[test]
fn test_compositor_vis_skip() {
    let mut s = PixelStore::new(1, 1);
    let mut l = Layer::new("L".into(), "L".into(), 1, 1);
    l.set_pixel(0, 0, Color::new(255, 0, 0, 255)).unwrap();
    l.visible = false;
    s.add_layer(l);
    let mut f = vec![0u8; 4];
    Compositor::render(&s, &mut f, full_view(&s));
    assert_eq!(f[0], 45);
}

#[test]
fn test_compositor_stacking() {
    let mut s = PixelStore::new(1, 1);
    s.add_layer(Layer::new("L".into(), "L".into(), 1, 1));
    s.mut_set_pixel("L", 0, 0, Color::new(0, 0, 255, 255)).unwrap();
    let mut f = vec![0u8; 4];
    Compositor::render(&s, &mut f, full_view(&s));
    assert_eq!(f[2], 255);
}

#[test]
fn test_compositor_vis_filtering() {
    let mut s = PixelStore::new(1, 1);
    let mut l0 = Layer::new("0".into(), "0".into(), 1, 1);
    l0.set_pixel(0, 0, Color::new(0, 255, 0, 255)).unwrap();
    s.add_layer(l0);
    let mut l1 = Layer::new("1".into(), "1".into(), 1, 1);
    l1.set_pixel(0, 0, Color::new(255, 0, 0, 255)).unwrap();
    s.add_layer(l1);
    s.set_layer_visibility("1", false);
    let mut f = vec![0u8; 4];
    Compositor::render(&s, &mut f, full_view(&s));
    assert_eq!(f[1], 255); 
}