use super::*;
use crate::core::color::Color;
use crate::core::layer::Layer;
use crate::core::store::PixelStore;
use crate::history::patch::ActionPatch;

#[test]
fn test_history_pixel_diff() {
    let mut s = PixelStore::new(10, 10);
    s.add_layer(Layer::new("L".into(), "L".into(), 10, 10));
    let mut h = HistoryManager::new(10);
    let mut p = ActionPatch::new_pixel_diff("p".into(), "L".into());
    p.add_pixel_diff(0, 0, Color::transparent(), Color::new(255, 0, 0, 255));
    let _ = h.commit(p, &mut s);
    assert_eq!(s.get_pixel("L", 0, 0).unwrap().r, 255);
    let _ = h.undo(&mut s);
    assert_eq!(s.get_pixel("L", 0, 0).unwrap().a, 0);
}

#[test]
fn test_history_max_steps() {
    let mut s = PixelStore::new(10, 10);
    s.add_layer(Layer::new("L".into(), "L".into(), 10, 10));
    let mut h = HistoryManager::new(1);
    let mut p1 = ActionPatch::new_pixel_diff("1".into(), "L".into());
    p1.add_pixel_diff(0, 0, Color::transparent(), Color::new(1, 1, 1, 255));
    let _ = h.commit(p1, &mut s);
    let mut p2 = ActionPatch::new_pixel_diff("2".into(), "L".into());
    p2.add_pixel_diff(0, 0, Color::transparent(), Color::new(1, 1, 1, 255));
    let _ = h.commit(p2, &mut s);
    assert_eq!(h.undo_stack.len(), 1);
}

#[test]
fn test_history_ignore_empty() {
    let mut s = PixelStore::new(10, 10);
    s.add_layer(Layer::new("L".into(), "L".into(), 10, 10));
    let mut h = HistoryManager::new(10);
    let _ = h.commit(ActionPatch::new_pixel_diff("e".into(), "L".into()), &mut s);
    assert_eq!(h.undo_stack.len(), 0);
}

#[test]
fn test_history_invalid_id_safety() {
    let mut s = PixelStore::new(10, 10);
    let mut h = HistoryManager::new(10);
    let mut p = ActionPatch::new_pixel_diff("p".into(), "X".into());
    p.add_pixel_diff(0, 0, Color::transparent(), Color::new(1, 1, 1, 255));
    assert!(h.commit(p, &mut s).is_err());
    assert_eq!(h.undo_stack.len(), 0);
}

#[test]
fn test_history_layer_add() {
    let mut s = PixelStore::new(10, 10);
    s.add_layer(Layer::new("BASE".into(), "BASE".into(), 10, 10)); 
    let mut h = HistoryManager::new(10);
    let l = Layer::new("L".into(), "L".into(), 10, 10);
    let _ = h.commit(ActionPatch::new_layer_add("p".into(), "L".into(), l, 1, None), &mut s);
    assert_eq!(s.layers.len(), 2);
    let _ = h.undo(&mut s);
    assert_eq!(s.layers.len(), 1);
}

#[test]
fn test_history_layer_remove() {
    let mut s = PixelStore::new(10, 10);
    s.add_layer(Layer::new("L1".into(), "B".into(), 10, 10));
    let l2 = Layer::new("L2".into(), "D".into(), 10, 10);
    s.add_layer(l2.clone());
    let active_id = s.active_layer_id.clone();
    let mut h = HistoryManager::new(10);
    let _ = h.commit(ActionPatch::new_layer_remove("p".into(), "L2".into(), l2, 1, active_id), &mut s);
    assert_eq!(s.layers.len(), 1);
    let _ = h.undo(&mut s);
    assert_eq!(s.layers.len(), 2);
}

#[test]
fn test_history_visibility() {
    let mut s = PixelStore::new(10, 10);
    s.add_layer(Layer::new("L".into(), "L".into(), 10, 10));
    let mut h = HistoryManager::new(10);
    let _ = h.commit(ActionPatch::new_layer_visibility("p".into(), "L".into(), false), &mut s);
    assert_eq!(s.get_layer("L").unwrap().visible, false);
    let _ = h.undo(&mut s);
    assert_eq!(s.get_layer("L").unwrap().visible, true);
}

#[test]
fn test_history_redo_clear() {
    let mut s = PixelStore::new(10, 10);
    s.add_layer(Layer::new("L".into(), "L".into(), 10, 10));
    let mut h = HistoryManager::new(10);
    let mut p1 = ActionPatch::new_pixel_diff("1".into(), "L".into());
    p1.add_pixel_diff(0, 0, Color::transparent(), Color::new(1, 1, 1, 255));
    let _ = h.commit(p1, &mut s);
    let _ = h.undo(&mut s);
    let mut p2 = ActionPatch::new_pixel_diff("2".into(), "L".into());
    p2.add_pixel_diff(0, 0, Color::transparent(), Color::new(2, 2, 2, 255));
    let _ = h.commit(p2, &mut s);
    assert_eq!(h.redo_stack.len(), 0);
}