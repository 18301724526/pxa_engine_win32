use super::*;
use crate::core::layer::Layer;
use crate::core::color::Color;
use crate::core::store::PixelStore;
use crate::core::symmetry::SymmetryConfig;
use crate::render::compositor::Compositor;

#[test]
fn test_eyedropper_composite_picking() {
    let mut s = PixelStore::new(1, 1);
    let sym = SymmetryConfig::new(1, 1);
    
    let mut l1 = Layer::new("1".into(), "L1".into(), 1, 1);
    l1.set_pixel(0, 0, Color::new(0, 0, 255, 255)).unwrap();
    s.add_layer(l1);
    
    let mut l2 = Layer::new("2".into(), "L2".into(), 1, 1);
    l2.set_pixel(0, 0, Color::new(255, 0, 0, 255)).unwrap();
    s.add_layer(l2);
    Compositor::update_composite_cache(&mut s, None);
    let mut tool = EyedropperTool::new();
    
    let _ = tool.on_pointer_down(0, 0, &mut s, &sym);
    assert_eq!(s.primary_color.r, 255);

    s.get_layer_mut("2").unwrap().visible = false;
    Compositor::update_composite_cache(&mut s, None);
    let _ = tool.on_pointer_down(0, 0, &mut s, &sym);
    assert_eq!(s.primary_color.b, 255);
    
    s.get_layer_mut("2").unwrap().visible = true;
    s.get_layer_mut("2").unwrap().set_pixel(0, 0, Color::transparent()).unwrap();
    Compositor::update_composite_cache(&mut s, None);
    let _ = tool.on_pointer_down(0, 0, &mut s, &sym);
    assert_eq!(s.primary_color.b, 255);
}