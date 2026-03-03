use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::app::commands::*;
use pxa_engine_win32::core::color::Color;

fn exec(app: &mut AppState, cmd: Box<dyn pxa_engine_win32::app::command_handler::Command>) {
    app.enqueue_command(cmd);
    app.process_commands();
}

fn setup_transform_test() -> AppState {
    let mut app = AppState::new();
    let (store, _, _, _) = app.pixel.engine.parts_mut();
    store.canvas_width = 100;
    store.canvas_height = 100;
    
    let layer_id = store.active_layer_id.clone().unwrap();
    if let Some(layer) = store.get_layer_mut(&layer_id) {
        layer.width = 100;
        layer.height = 100;
    }
    app.pixel.engine.set_primary_color(Color::new(255, 0, 0, 255));

    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(40, 40).unwrap();
    app.on_mouse_move(59, 59).unwrap();
    app.on_mouse_up().unwrap();

    app.set_tool(ToolType::Bucket);
    app.on_mouse_down(50, 50).unwrap();
    app.on_mouse_up().unwrap();

    app.set_tool(ToolType::Transform);
    app
}

#[test]
fn test_transform_rotate_and_commit() {
    let mut app = setup_transform_test();

    app.on_mouse_down(0, 0).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(50, 20).unwrap();
    app.on_mouse_move(80, 50).unwrap();
    app.on_mouse_up().unwrap();

    exec(&mut app, Box::new(CommitCurrentToolCmd));

    let store = app.pixel.engine.store();
    assert!(store.selection.is_active);
    assert!(store.selection.contains(50, 50));
}

#[test]
fn test_transform_scale_and_mirror() {
    let mut app = setup_transform_test();
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();

    app.on_mouse_down(0, 0).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(60, 60).unwrap();
    app.on_mouse_move(20, 20).unwrap();
    app.on_mouse_up().unwrap();

    let tool = app.pixel.engine.tool_manager().tools.get(&ToolType::Transform).unwrap();
    let transform_tool = tool.as_any().downcast_ref::<pxa_engine_win32::tools::transform::TransformTool>().unwrap();
    
    assert!(transform_tool.scale_x < 0.0);
    assert!(transform_tool.scale_y < 0.0);

    exec(&mut app, Box::new(CommitCurrentToolCmd));
    app.undo();
    
    let store = app.pixel.engine.store();
    assert_eq!(store.get_pixel(&layer_id, 45, 45).unwrap().r, 255);
}

#[test]
fn test_transform_selection_update_and_cancel() {
    let mut app = setup_transform_test();
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();

    app.on_mouse_down(0, 0).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(50, 50).unwrap();
    app.on_mouse_move(70, 70).unwrap(); 
    app.on_mouse_up().unwrap();

    exec(&mut app, Box::new(CancelCurrentToolCmd));

    let store = app.pixel.engine.store();
    assert_eq!(store.get_pixel(&layer_id, 70, 70).unwrap().a, 0);
    assert_eq!(store.get_pixel(&layer_id, 50, 50).unwrap().r, 255);
    assert!(store.selection.contains(50, 50));
}