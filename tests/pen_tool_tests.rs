use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::app::commands::*;
use pxa_engine_win32::core::color::Color;
use pxa_engine_win32::core::path::NodeType;

fn exec(app: &mut AppState, cmd: Box<dyn pxa_engine_win32::app::command_handler::Command>) {
    app.enqueue_command(cmd);
    app.process_commands();
}

fn setup_pen_test() -> AppState {
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
    app
}

#[test]
fn test_pen_node_creation_and_history() {
    let mut app = setup_pen_test();
    app.set_tool(ToolType::Pen);

    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();
    assert_eq!(app.pixel.engine.store().active_path.nodes.len(), 1);

    app.on_mouse_down(50, 10).unwrap();
    app.on_mouse_move(50, 20).unwrap(); 
    app.on_mouse_up().unwrap();

    assert_eq!(app.pixel.engine.store().active_path.nodes[1].kind, NodeType::Smooth);

    app.undo();
    assert_eq!(app.pixel.engine.store().active_path.nodes.len(), 1);
    app.redo();
    assert_eq!(app.pixel.engine.store().active_path.nodes.len(), 2);
}

#[test]
fn test_pen_close_path_and_adjust_handles() {
    let mut app = setup_pen_test();
    app.set_tool(ToolType::Pen);

    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(50, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 40).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();

    let path = &app.pixel.engine.store().active_path;
    assert!(path.is_closed);

    app.on_mouse_down(50, 10).unwrap();
    app.on_mouse_move(60, 10).unwrap();
    app.on_mouse_up().unwrap();
    assert_eq!(app.pixel.engine.store().active_path.nodes[1].anchor.x, 60.0);
}

#[test]
fn test_pen_path_to_selection() {
    let mut app = setup_pen_test();
    app.set_tool(ToolType::Pen);

    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 30).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(10, 30).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();

    exec(&mut app, Box::new(CommitCurrentToolCmd));
    assert!(app.pixel.engine.store().selection.is_active);
}

#[test]
fn test_pen_fill_path() {
    let mut app = setup_pen_test();
    app.set_tool(ToolType::Pen);
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();

    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(50, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 40).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();

    exec(&mut app, Box::new(PenFillCmd));
    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 30, 20).unwrap().r, 255);
}

#[test]
fn test_pen_stroke_path() {
    let mut app = setup_pen_test();
    app.set_tool(ToolType::Pen);
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();

    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 10).unwrap(); app.on_mouse_up().unwrap();

    exec(&mut app, Box::new(PenStrokeCmd));
    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 20, 10).unwrap().r, 255);
}

#[test]
fn test_pen_node_type_switching_and_independence() {
    let mut app = setup_pen_test();
    app.set_tool(ToolType::Pen);

    app.on_mouse_down(50, 50).unwrap();
    app.on_mouse_move(50, 60).unwrap(); 
    app.on_mouse_up().unwrap();

    exec(&mut app, Box::new(TogglePathNodeTypeCmd(0)));
    assert_eq!(app.pixel.engine.store().active_path.nodes[0].kind, NodeType::Corner);

    app.on_mouse_down(50, 60).unwrap();
    app.on_mouse_move(70, 60).unwrap();
    app.on_mouse_up().unwrap();

    assert_eq!(app.pixel.engine.store().active_path.nodes[0].handle_out.x, 20.0);
    assert_eq!(app.pixel.engine.store().active_path.nodes[0].handle_in.x, 0.0);
}

#[test]
fn test_pen_interactive_node_deletion() {
    let mut app = setup_pen_test();
    app.set_tool(ToolType::Pen);

    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 30).unwrap(); app.on_mouse_up().unwrap();
    
    app.on_mouse_down(30, 30).unwrap();
    app.on_mouse_up().unwrap();

    assert_eq!(app.pixel.engine.store().active_path.nodes.len(), 1);
}

#[test]
fn test_pen_command_based_node_deletion() {
    let mut app = setup_pen_test();
    app.set_tool(ToolType::Pen);

    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(20, 20).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 30).unwrap(); app.on_mouse_up().unwrap();

    assert_eq!(app.pixel.engine.store().active_path.nodes.len(), 3);

    exec(&mut app, Box::new(DeletePathNodeCmd(1))); // 删除中间的节点

    assert_eq!(app.pixel.engine.store().active_path.nodes.len(), 2);
    assert_eq!(app.pixel.engine.store().active_path.nodes[0].anchor.x, 10.0);
    assert_eq!(app.pixel.engine.store().active_path.nodes[1].anchor.x, 30.0, "剩余节点应前移");

    app.undo();
    assert_eq!(app.pixel.engine.store().active_path.nodes.len(), 3, "撤销应恢复被删除的节点");
}

#[test]
fn test_pen_fill_stroke_undo_redo() {
    let mut app = setup_pen_test();
    app.set_tool(ToolType::Pen);
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();

    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 30).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(10, 30).unwrap(); app.on_mouse_up().unwrap();
    
    app.pixel.engine.set_primary_color(Color::new(0, 255, 0, 255));
    exec(&mut app, Box::new(PenStrokeCmd));
    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 20, 10).unwrap().g, 255);
    
    app.undo();
    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 20, 10).unwrap().a, 0, "钢笔描边应被正确撤销");
}