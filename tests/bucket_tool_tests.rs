use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::core::color::Color;

#[test]
fn test_bucket_basic_fill() {
    let mut app = AppState::new();

    app.set_tool(ToolType::Pencil);
    for x in 10..=20 {
        app.on_mouse_down(x, 10).unwrap(); app.on_mouse_up().unwrap();
        app.on_mouse_down(x, 20).unwrap(); app.on_mouse_up().unwrap();
    }
    for y in 10..=20 {
        app.on_mouse_down(10, y).unwrap(); app.on_mouse_up().unwrap();
        app.on_mouse_down(20, y).unwrap(); app.on_mouse_up().unwrap();
    }

    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();
    
    app.pixel.engine.set_primary_color(Color::new(255, 255, 255, 255));
    app.set_tool(ToolType::Bucket);
    app.on_mouse_down(15, 15).unwrap();
    app.on_mouse_up().unwrap();

    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 15, 15).unwrap().r, 255);
    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 9, 15).unwrap().a, 0);
}

#[test]
fn test_bucket_tolerance_fill() {
    let mut app = AppState::new();
    app.pixel.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();

    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();

    app.pixel.engine.set_primary_color(Color::new(0, 255, 0, 255));
    app.set_tool(ToolType::Bucket);
    
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_up().unwrap();

    let store = app.pixel.engine.store();
    assert_eq!(store.get_pixel(&layer_id, 10, 10).unwrap().g, 255);
}

#[test]
fn test_bucket_fill_transparent_logic() {
    let mut app = AppState::new();

    app.pixel.engine.set_primary_color(Color::transparent());
    app.set_tool(ToolType::Bucket);
    
    let history_before = app.pixel.engine.history().undo_stack.len();
    
    app.on_mouse_down(50, 50).unwrap();
    app.on_mouse_up().unwrap();

    let history_after = app.pixel.engine.history().undo_stack.len();
    assert_eq!(history_before, history_after, "透明区域填充透明色不应产生历史记录");
}

#[test]
fn test_bucket_fill_with_symmetry() {
    let mut app = AppState::new();
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();

    {
        let sym = app.pixel.engine.symmetry_mut();
        sym.mode = pxa_engine_win32::core::symmetry::SymmetryMode::Horizontal;
        sym.axis_x = 64.0;
    }

    app.pixel.engine.set_primary_color(Color::new(255, 255, 0, 255));
    app.set_tool(ToolType::Bucket);
    app.on_mouse_down(30, 30).unwrap();
    app.on_mouse_up().unwrap();

    let store = app.pixel.engine.store();
    assert_eq!(store.get_pixel(&layer_id, 30, 30).unwrap().r, 255);
    assert_eq!(store.get_pixel(&layer_id, 98, 30).unwrap().r, 255, "填充工具应支持对称");
}