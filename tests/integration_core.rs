use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::app::commands::{AppCommand, ResizeAnchor};
use pxa_engine_win32::app::command_handler::CommandHandler;
use pxa_engine_win32::core::color::Color;
use pxa_engine_win32::core::blend_mode::BlendMode;
use pxa_engine_win32::core::symmetry::SymmetryMode;

#[test]
fn test_blend_mode_stacking() {
    let mut app = AppState::new();
    
    app.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.set_tool(ToolType::Pencil);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_up().unwrap();

    app.add_new_layer();
    let l2_id = app.engine.store().layers[1].id.clone();
    app.engine.set_active_layer(l2_id.clone());
    app.engine.set_primary_color(Color::new(0, 0, 255, 255));
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_up().unwrap();

    CommandHandler::execute(&mut app, AppCommand::SetLayerBlendMode(l2_id, BlendMode::Add));
    app.engine.update_render_cache(None);

    let composite_pixel = app.engine.store().get_composite_pixel(10, 10);
    assert_eq!(composite_pixel.r, 255);
    assert_eq!(composite_pixel.b, 255);
    assert_eq!(composite_pixel.g, 0);
}

#[test]
fn test_selection_invert_and_clear() {
    let mut app = AppState::new();
    
    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(0, 0).unwrap();
    app.on_mouse_move(9, 9).unwrap();
    app.on_mouse_up().unwrap();
    
    assert!(app.engine.store().selection.is_active);
    assert!(app.engine.store().selection.contains(5, 5));
    assert!(!app.engine.store().selection.contains(15, 15));

    CommandHandler::execute(&mut app, AppCommand::InvertSelection);
    assert!(!app.engine.store().selection.contains(5, 5));
    assert!(app.engine.store().selection.contains(15, 15));

    CommandHandler::execute(&mut app, AppCommand::ClearSelection);
    assert!(!app.engine.store().selection.is_active);
}

#[test]
fn test_symmetry_pixel_distribution() {
    let mut app = AppState::new();
    let layer_id = "L1";
    
    {
        let sym = app.engine.symmetry_mut();
        sym.mode = SymmetryMode::Quad;
        sym.axis_x = 64.0;
        sym.axis_y = 64.0;
    }

    app.engine.set_primary_color(Color::new(255, 255, 255, 255));
    app.set_tool(ToolType::Pencil);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_up().unwrap();

    let store = app.engine.store();
    assert_eq!(store.get_pixel(layer_id, 10, 10).unwrap().a, 255);
    assert_eq!(store.get_pixel(layer_id, 118, 10).unwrap().a, 255);
    assert_eq!(store.get_pixel(layer_id, 10, 118).unwrap().a, 255);
    assert_eq!(store.get_pixel(layer_id, 118, 118).unwrap().a, 255);
}

#[test]
fn test_layer_naming_conflict() {
    rust_i18n::set_locale("zh-CN"); 

    let mut app = AppState::new();
    
    app.add_new_layer();
    app.add_new_layer();
    app.add_new_layer();
    
    let layers = &app.engine.store().layers;
    assert_eq!(layers.len(), 4);
    
    let names: Vec<String> = layers.iter().map(|l| l.name.clone()).collect();
    
    assert!(names.contains(&"图层 1".to_string()));
    assert!(names.contains(&"图层 2".to_string()));
    assert!(names.contains(&"图层 3".to_string()));
    assert!(names.contains(&"图层 4".to_string()));
}

#[test]
fn test_move_across_chunk_boundary() {
    let mut app = AppState::new();
    let layer_id = "L1";
    
    app.engine.set_primary_color(Color::new(255, 255, 255, 255));
    app.set_tool(ToolType::Pencil);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_up().unwrap();

    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(5, 5).unwrap();
    app.on_mouse_move(15, 15).unwrap();
    app.on_mouse_up().unwrap();

    app.set_tool(ToolType::Move);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(80, 80).unwrap();
    app.on_mouse_up().unwrap();

    let store = app.engine.store();
    assert_eq!(store.get_pixel(layer_id, 10, 10).unwrap().a, 0);
    assert_eq!(store.get_pixel(layer_id, 80, 80).unwrap().a, 255);
}

#[test]
fn test_canvas_resize() {
    let mut app = AppState::new();
    let layer_id = "L1";
    
    app.engine.set_primary_color(Color::new(255, 255, 255, 255));
    app.set_tool(ToolType::Pencil);
    app.on_mouse_down(0, 0).unwrap();
    app.on_mouse_up().unwrap();

    CommandHandler::execute(&mut app, AppCommand::ResizeCanvas(256, 256, ResizeAnchor::Center));
    
    let store = app.engine.store();
    assert_eq!(store.canvas_width, 256);
    assert_eq!(store.canvas_height, 256);
    
    assert_eq!(store.get_pixel(layer_id, 64, 64).unwrap().a, 255);
    assert_eq!(store.get_pixel(layer_id, 0, 0).unwrap().a, 0);
}