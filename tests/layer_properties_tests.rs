use pxa_engine_win32::app::state::AppState;
use pxa_engine_win32::app::commands::*;
use pxa_engine_win32::app::ui_context::UiContext;

fn exec(app: &mut AppState, cmd: Box<dyn pxa_engine_win32::app::command_handler::Command>) {
    app.enqueue_command(cmd);
    app.process_commands();
}

#[test]
fn test_bulk_opacity_change() {
    let mut app = AppState::new();
    let mut ui_ctx = UiContext::new(); 
    
    app.add_new_layer();
    app.add_new_layer();
    
    let id2 = app.pixel.engine.store().layers[1].id.clone();
    let id3 = app.pixel.engine.store().layers[2].id.clone();
    
    // 操作本地 ui_ctx
    ui_ctx.selected_layer_ids = vec![id2.clone(), id3.clone()];
    let selected_ids = ui_ctx.selected_layer_ids.clone();

    for target_id in selected_ids {
        // 修正：具体结构体实例化，解决 E0282
        exec(&mut app, Box::new(SetLayerOpacityCmd(target_id, 128)));
    }

    assert_eq!(app.pixel.engine.store().get_layer(&id2).unwrap().opacity, 128);
    assert_eq!(app.pixel.engine.store().get_layer(&id3).unwrap().opacity, 128);
}