use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::app::commands::*;
use pxa_engine_win32::core::color::Color;
use pxa_engine_win32::core::error::CoreError;

fn exec(app: &mut AppState, cmd: Box<dyn pxa_engine_win32::app::command_handler::Command>) {
    app.enqueue_command(cmd);
    app.process_commands();
}

fn setup_layer_test() -> AppState {
    rust_i18n::set_locale("zh-CN");
    AppState::new()
}

#[test]
fn test_layer_add_and_delete_fallback() {
    let mut app = setup_layer_test();
    app.add_new_layer();
    app.add_new_layer();
    assert_eq!(app.pixel.engine.store().layers.len(), 3);
    
    let id3 = app.pixel.engine.store().layers[2].id.clone();
    assert_eq!(app.pixel.engine.store().active_layer_id.as_ref(), Some(&id3));

    app.delete_active_layer();
    assert_eq!(app.pixel.engine.store().layers.len(), 2);
    
    app.delete_active_layer();
    app.delete_active_layer(); // 最后 1 个
    assert_eq!(app.pixel.engine.store().layers.len(), 1);
}

#[test]
fn test_layer_duplicate() {
    let mut app = setup_layer_test();
    let original_id = app.pixel.engine.store().active_layer_id.clone().unwrap();
    
    app.set_tool(ToolType::Pencil);
    app.pixel.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();

    exec(&mut app, Box::new(DuplicateLayerCmd(original_id.clone())));
    
    let store = app.pixel.engine.store();
    assert_eq!(store.layers.len(), 2);
    assert_eq!(store.layers[1].get_pixel(10, 10).unwrap().r, 255);
}

#[test]
fn test_layer_merge_selected() {
    let mut app = setup_layer_test();
    let id1 = app.pixel.engine.store().active_layer_id.clone().unwrap();
    app.set_tool(ToolType::Pencil);
    app.pixel.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();

    app.add_new_layer();
    let id2 = app.pixel.engine.store().active_layer_id.clone().unwrap();
    app.pixel.engine.set_primary_color(Color::new(0, 255, 0, 255));
    app.on_mouse_down(20, 20).unwrap(); app.on_mouse_up().unwrap();

    exec(&mut app, Box::new(MergeSelectedCmd(vec![id1, id2])));
    let store = app.pixel.engine.store();
    assert_eq!(store.layers.len(), 1);
}

#[test]
fn test_layer_reorder() {
    let mut app = setup_layer_test();
    let _id1 = app.pixel.engine.store().layers[0].id.clone();
    app.add_new_layer();
    let id2 = app.pixel.engine.store().layers[1].id.clone();
    
    // 修复点：被 AI 改成了 MoveLayerToIndexCmd
    exec(&mut app, Box::new(MoveLayerToIndexCmd(id2.clone(), 0)));
    assert_eq!(app.pixel.engine.store().layers[0].id, id2);
}

#[test]
fn test_layer_rename_conflict() {
    let mut app = setup_layer_test();
    let id1 = app.pixel.engine.store().layers[0].id.clone();
    
    exec(&mut app, Box::new(RenameLayerCmd(id1.clone(), "Body".into())));
    
    app.add_new_layer();
    let id2 = app.pixel.engine.store().layers[1].id.clone();
    exec(&mut app, Box::new(RenameLayerCmd(id2.clone(), "Body".into())));
    
    assert_eq!(app.pixel.engine.store().layers[1].name, "Body (2)");
}

#[test]
fn test_layer_lock_all_tools_rejection() {
    let mut app = setup_layer_test();
    let id1 = app.pixel.engine.store().active_layer_id.clone().unwrap();
    
    exec(&mut app, Box::new(ToggleLayerLockCmd(id1.clone())));
    
    app.set_tool(ToolType::Pencil);
    let res_pencil = app.on_mouse_down(10, 10);
    assert!(matches!(res_pencil, Err(CoreError::LayerLocked)));
}