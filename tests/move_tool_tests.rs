use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::core::color::Color;

#[test]
fn test_move_entire_layer() {
    let mut app = AppState::new();

    app.pixel.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.set_tool(ToolType::Pencil);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_up().unwrap();

    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();
    
    app.set_tool(ToolType::Move);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(20, 30).unwrap();
    app.on_mouse_up().unwrap();

    let layer = app.pixel.engine.store().get_layer(&layer_id).unwrap();
    assert_eq!(layer.offset_x, 10);
    assert_eq!(layer.offset_y, 20);

    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 20, 30).unwrap().r, 255);
    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 10, 10).unwrap_or(Color::transparent()).a, 0);
}

#[test]
fn test_move_selection_content() {
    let mut app = AppState::new();
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();

    app.pixel.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.set_tool(ToolType::Pencil);
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 30).unwrap(); app.on_mouse_up().unwrap();

    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(5, 5).unwrap();
    app.on_mouse_move(15, 15).unwrap();
    app.on_mouse_up().unwrap();

    app.set_tool(ToolType::Move);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(30, 10).unwrap();
    app.on_mouse_up().unwrap();

    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 10, 10).unwrap().a, 0);
    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 30, 10).unwrap().r, 255);
    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 30, 30).unwrap().r, 255);

    let sel = &app.pixel.engine.store().selection;
    assert!(!sel.contains(10, 10));
    assert!(sel.contains(30, 10));

    app.undo();
    app.process_commands();
    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 10, 10).unwrap().r, 255);
    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 30, 10).unwrap().a, 0);
    assert!(app.pixel.engine.store().selection.contains(10, 10));
}

#[test]
fn test_move_out_of_bounds() {
    let mut app = AppState::new();
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();

    app.pixel.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();

    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(0, 0).unwrap();
    app.on_mouse_move(20, 20).unwrap();
    app.on_mouse_up().unwrap();

    app.set_tool(ToolType::Move);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(150, 150).unwrap();
    app.on_mouse_up().unwrap();

    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 10, 10).unwrap().a, 0);
    assert!(app.pixel.engine.store().get_pixel(&layer_id, 150, 150).is_none());
}

#[test]
fn test_layer_offset_manual_set() {
    let mut app = AppState::new();
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();

    if let Some(layer) = app.pixel.engine.parts_mut().0.get_layer_mut(&layer_id) {
        layer.offset_x = 50;
        layer.offset_y = 50;
    }
    
    app.pixel.engine.set_primary_color(Color::new(0, 255, 0, 255));
    app.set_tool(ToolType::Pencil);
    app.on_mouse_down(60, 60).unwrap();
    app.on_mouse_up().unwrap();

    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 60, 60).unwrap().g, 255);
}