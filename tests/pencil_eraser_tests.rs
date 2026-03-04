use pxa_engine_win32::app::state::{AppState, ToolType, AppMode};
use pxa_engine_win32::core::color::Color;

fn setup_app() -> AppState {
    let mut app = AppState::new();
    if app.pixel.engine.store().layers.is_empty() {
        app.add_new_layer();
    }
    app
}

#[test]
fn test_eraser_functionality_and_history() {
    let mut app = setup_app();
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();
    
    app.pixel.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.set_tool(ToolType::Pencil);
    let _ = app.on_mouse_down(10, 10); let _ = app.on_mouse_up();
    
    app.set_tool(ToolType::Eraser);
    let _ = app.on_mouse_down(10, 10);
    let _ = app.on_mouse_up();

    let store = app.pixel.engine.store();
    assert_eq!(store.get_pixel(&layer_id, 10, 10).unwrap().a, 0);

    app.undo();
    app.process_commands();
    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 10, 10).unwrap().r, 255);
    app.redo();
    app.process_commands();
    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 10, 10).unwrap().a, 0);
}

#[test]
fn test_jitter_boundary_values() {
    let mut app = setup_app();
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();
    app.pixel.engine.set_primary_color(Color::new(255, 255, 255, 255));
    app.set_tool(ToolType::Pencil);
    
    {
        let (_, _, jitter) = app.pixel.engine.brush_settings_mut();
        *jitter = 0;
    }
    let _ = app.on_mouse_down(30, 30); let _ = app.on_mouse_up();
    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 30, 30).unwrap().a, 255);

    {
        let (_, _, jitter) = app.pixel.engine.brush_settings_mut();
        *jitter = 15;
    }
    for _ in 0..10 { let _ = app.on_mouse_down(50, 50); let _ = app.on_mouse_up(); }
    
    let store = app.pixel.engine.store();
    let mut has_remote_pixel = false;
    for x in 35..=65 {
        for y in 35..=65 {
            if (x != 50 || y != 50) && store.get_pixel(&layer_id, x, y).unwrap().a > 0 {
                has_remote_pixel = true; 
                break;
            }
        }
    }
    assert!(has_remote_pixel);
}

#[test]
fn test_brush_size_shortcut_simulation() {
    let mut app = setup_app();
    {
        let (size, _, _) = app.pixel.engine.brush_settings_mut();
        *size = 10;
    }

    // 修正：改用 Box<dyn Command> 的 enqueue 方式
    if let Some(cmd) = app.shortcuts.handle_text_input("]", AppMode::PixelEdit) {
        app.enqueue_command(cmd);
        app.process_commands();
    }
    assert_eq!(app.pixel.engine.store().brush_size, 11);

    if let Some(cmd) = app.shortcuts.handle_text_input("[", AppMode::PixelEdit) {
        app.enqueue_command(cmd);
        app.process_commands();
    }
    assert_eq!(app.pixel.engine.store().brush_size, 10);
}