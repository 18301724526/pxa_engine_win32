use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::app::command_handler::CommandHandler;
use pxa_engine_win32::core::color::Color;

fn process_app_logic(app: &mut AppState) {
    while let Some(cmd) = app.pop_command() {
        CommandHandler::execute(app, cmd);
    }
    if app.view.needs_full_redraw {
        app.engine.update_render_cache(None);
        app.view.needs_full_redraw = false;
    }
}

#[test]
fn test_pencil_tool_workflow() {
    let mut app = AppState::new();
    let layer_id = "L1";
    app.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.set_tool(ToolType::Pencil); 

    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(12, 10).unwrap();
    app.on_mouse_up().unwrap();

    assert_eq!(app.engine.store().get_pixel(layer_id, 10, 10).unwrap().r, 255);
    assert_eq!(app.engine.store().get_pixel(layer_id, 11, 10).unwrap().r, 255);
    assert_eq!(app.engine.store().get_pixel(layer_id, 12, 10).unwrap().r, 255);

    assert_eq!(app.engine.history().undo_stack.len(), 1);

    app.undo();
    assert_eq!(app.engine.store().get_pixel(layer_id, 11, 10).unwrap().a, 0);
}

#[test]
fn test_bucket_tool_workflow() {
    let mut app = AppState::new();
    let layer_id = "L1";

    app.set_tool(ToolType::Pencil);
    for x in 5..=7 {
        app.on_mouse_down(x, 5).unwrap(); app.on_mouse_up().unwrap();
        app.on_mouse_down(x, 7).unwrap(); app.on_mouse_up().unwrap();
    }
    for y in 5..=7 {
        app.on_mouse_down(5, y).unwrap(); app.on_mouse_up().unwrap();
        app.on_mouse_down(7, y).unwrap(); app.on_mouse_up().unwrap();
    }

    app.engine.set_primary_color(Color::new(0, 255, 0, 255));
    app.set_tool(ToolType::Bucket);
    app.on_mouse_down(6, 6).unwrap();
    app.on_mouse_up().unwrap();

    assert_eq!(app.engine.store().get_pixel(layer_id, 6, 6).unwrap().g, 255);
    assert_eq!(app.engine.store().get_pixel(layer_id, 4, 6).unwrap().a, 0);
}

#[test]
fn test_rect_and_ellipse_select_workflow() {
    let mut app = AppState::new();

    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(20, 20).unwrap();
    app.on_mouse_up().unwrap();
    
    assert!(app.engine.store().selection.is_active);
    assert!(app.engine.store().selection.contains(15, 15));
    assert!(!app.engine.store().selection.contains(25, 25));

    app.set_tool(ToolType::EllipseSelect);
    app.on_mouse_down(30, 30).unwrap();
    app.on_mouse_move(40, 40).unwrap();
    app.on_mouse_up().unwrap();

    assert!(app.engine.store().selection.contains(35, 35));
    assert!(!app.engine.store().selection.contains(30, 30));
}

#[test]
fn test_move_tool_workflow() {
    let mut app = AppState::new();
    let layer_id = "L1";

    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_up().unwrap();

    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(9, 9).unwrap();
    app.on_mouse_move(11, 11).unwrap();
    app.on_mouse_up().unwrap();

    app.set_tool(ToolType::Move);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(20, 20).unwrap();
    app.on_mouse_up().unwrap();

    assert_eq!(app.engine.store().get_pixel(layer_id, 10, 10).unwrap().a, 0);
    assert_eq!(app.engine.store().get_pixel(layer_id, 20, 20).unwrap().a, 255);
}

#[test]
fn test_transform_tool_workflow() {
    let mut app = AppState::new();
    let layer_id = "L1";

    app.engine.set_primary_color(Color::new(255, 255, 255, 255));
    {
        let (size, _, _) = app.engine.brush_settings_mut();
        *size = 8;
    }

    app.set_tool(ToolType::Pencil);
    app.on_mouse_down(40, 40).unwrap(); 
    app.on_mouse_up().unwrap();

    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(30, 30).unwrap(); 
    app.on_mouse_move(50, 50).unwrap();
    app.on_mouse_up().unwrap();

    app.set_tool(ToolType::Transform);

    app.on_mouse_down(40, 40).unwrap();
    app.on_mouse_move(64, 64).unwrap(); 
    app.on_mouse_up().unwrap();

    app.commit_current_tool(); 
    process_app_logic(&mut app);

    let store = app.engine.store();
    assert_eq!(store.get_pixel(layer_id, 64, 64).expect("Pixel read failed").a, 255, "目标中心应有像素");
    assert_eq!(store.get_pixel(layer_id, 40, 40).expect("Pixel read failed").a, 0, "原位置应为空");
}