use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::core::color::Color;
use pxa_engine_win32::app::commands::*; // 更新引入

fn exec(app: &mut AppState, cmd: Box<dyn pxa_engine_win32::app::command_handler::Command>) {
    app.enqueue_command(cmd);
    app.process_commands();
}

fn setup_selection_test() -> AppState {
    let mut app = AppState::new();
    {
        let (store, _, _, _) = app.pixel.engine.parts_mut();
        store.canvas_width = 100;
        store.canvas_height = 100;
    }
    app
}

#[test]
fn test_rect_selection_creation_and_cancel() {
    let mut app = setup_selection_test();
    
    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(20, 20).unwrap();
    app.on_mouse_up().unwrap();
    
    {
        let sel = &app.pixel.engine.store().selection;
        assert!(sel.is_active);
        assert!(sel.contains(10, 10));
    }

    exec(&mut app, Box::new(ClearSelectionCmd));
    assert!(!app.pixel.engine.store().selection.is_active);
}

#[test]
fn test_ellipse_selection_and_invert() {
    let mut app = setup_selection_test();
    
    app.set_tool(ToolType::EllipseSelect);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(30, 30).unwrap(); 
    app.on_mouse_up().unwrap();
    
    assert!(app.pixel.engine.store().selection.contains(20, 20));

    exec(&mut app, Box::new(InvertSelectionCmd));
    assert!(!app.pixel.engine.store().selection.contains(20, 20));
    assert!(app.pixel.engine.store().selection.contains(0, 0));
}

#[test]
fn test_selection_stroke_accuracy() {
    let mut app = setup_selection_test();
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();
    
    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(12, 12).unwrap();
    app.on_mouse_up().unwrap();

    app.pixel.engine.set_primary_color(Color::new(255, 0, 0, 255));
    exec(&mut app, Box::new(StrokeSelectionCmd(1)));

    let store = app.pixel.engine.store();
    assert_eq!(store.get_pixel(&layer_id, 10, 10).unwrap().r, 255);
    assert_eq!(store.get_pixel(&layer_id, 11, 11).unwrap().a, 0);
}

#[test]
fn test_selection_interaction_with_layer_offset() {
    let mut app = setup_selection_test();
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();
    
    {
        let (store, _, _, _) = app.pixel.engine.parts_mut();
        if let Some(l) = store.get_layer_mut(&layer_id) {
            l.offset_x = 10;
            l.offset_y = 10;
        }
    }

    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(15, 15).unwrap();
    app.on_mouse_move(16, 15).unwrap();
    app.on_mouse_up().unwrap();

    app.pixel.engine.set_primary_color(Color::new(0, 255, 0, 255));
    app.set_tool(ToolType::Pencil);
    app.on_mouse_down(15, 15).unwrap(); app.on_mouse_up().unwrap();

    let pixel_on = app.pixel.engine.store().get_pixel(&layer_id, 15, 15).unwrap();
    assert_eq!(pixel_on.g, 255);
    
    app.on_mouse_down(17, 17).unwrap(); app.on_mouse_up().unwrap();
    let pixel_off = app.pixel.engine.store().get_pixel(&layer_id, 17, 17).unwrap();
    assert_eq!(pixel_off.a, 0);
}

#[test]
fn test_selection_invalid_zero_size() {
    let mut app = setup_selection_test();
    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_up().unwrap();
    assert!(!app.pixel.engine.store().selection.is_active);
}

#[test]
fn test_selection_stroke_thickness_and_undo() {
    let mut app = setup_selection_test();
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();
    
    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(20, 20).unwrap();
    app.on_mouse_up().unwrap();

    app.pixel.engine.set_primary_color(Color::new(0, 255, 0, 255));
    exec(&mut app, Box::new(StrokeSelectionCmd(2))); // 测试线宽为 2

    let store = app.pixel.engine.store();
    assert_eq!(store.get_pixel(&layer_id, 10, 10).unwrap().g, 255, "外边缘应该被染色");
    assert_eq!(store.get_pixel(&layer_id, 11, 11).unwrap().g, 255, "线宽为2时，内边缘也应该被染色");
    assert_eq!(store.get_pixel(&layer_id, 15, 15).unwrap().a, 0, "内部中心必须是透明的");

    app.undo();
    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 10, 10).unwrap().a, 0, "撤销后应恢复透明");
}