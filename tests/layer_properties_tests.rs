use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::app::commands::AppCommand;
use pxa_engine_win32::app::command_handler::CommandHandler;
use pxa_engine_win32::core::color::Color;
use pxa_engine_win32::core::blend_mode::BlendMode;

fn setup_properties_test() -> AppState {
    AppState::new()
}

// ---------------------------------------------------------
// 1. 透明度滑块 (合成像素验证) & 3. 历史记录 (撤销/重做)
// ---------------------------------------------------------
#[test]
fn test_layer_opacity_and_history() {
    let mut app = setup_properties_test();
    
    // 1. 底层 L1：画纯红色 (255, 0, 0)
    let _id1 = app.engine.store().active_layer_id.clone().unwrap();
    app.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.set_tool(ToolType::Pencil);
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();

    // 2. 顶层 L2：画纯绿色 (0, 255, 0)
    app.add_new_layer();
    let id2 = app.engine.store().active_layer_id.clone().unwrap();
    app.engine.set_primary_color(Color::new(0, 255, 0, 255));
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();

    // 刷新渲染缓存，验证目前 L2 (绿色) 应该完全遮盖 L1 (红色)
    app.engine.update_render_cache(None);
    let px_before = app.engine.store().get_composite_pixel(10, 10);
    assert_eq!(px_before.g, 255, "不透明时，顶层应完全显示");
    assert_eq!(px_before.r, 0, "不透明时，底层应完全被遮挡");

    // 3. 修改顶层 L2 透明度为 128 (约 50%)
    CommandHandler::execute(&mut app, AppCommand::SetLayerOpacity(id2.clone(), 128));
    app.engine.update_render_cache(None);

    // 验证透明度合成：红色和绿色应该混合
    let px_mixed = app.engine.store().get_composite_pixel(10, 10);
    assert!(px_mixed.r > 0 && px_mixed.r < 255, "透明度降低后，底层红色应该透出来");
    assert!(px_mixed.g > 0 && px_mixed.g < 255, "透明度降低后，顶层绿色应该变淡");

    // 4. 验证撤销 (Undo)
    app.undo();
    app.engine.update_render_cache(None);
    let px_undo = app.engine.store().get_composite_pixel(10, 10);
    assert_eq!(px_undo.r, 0, "撤销后，透明度恢复 255，底层红光应该再次消失");
    assert_eq!(px_undo.g, 255, "撤销后，顶层绿色恢复 100% 显示");

    // 5. 验证重做 (Redo)
    app.redo();
    app.engine.update_render_cache(None);
    let px_redo = app.engine.store().get_composite_pixel(10, 10);
    assert_eq!(px_redo.g, px_mixed.g, "重做后，透明度应再次变为 50% 的混合效果");
}

// ---------------------------------------------------------
// 2. 批量修改：选中多个图层同时调整属性
// ---------------------------------------------------------
#[test]
fn test_batch_layer_properties_modification() {
    let mut app = setup_properties_test();
    
    // 创建 3 个图层: L1 (默认), L2, L3
    app.add_new_layer();
    app.add_new_layer();
    
    let store = app.engine.store();
    assert_eq!(store.layers.len(), 3);
    let id1 = store.layers[0].id.clone();
    let id2 = store.layers[1].id.clone();
    let id3 = store.layers[2].id.clone();

    // UI 模拟：用户在面板上同时选中了 L2 和 L3
    app.ui.selected_layer_ids = vec![id2.clone(), id3.clone()];

    // UI 模拟：用户拖动透明度滑块到 50，并修改混合模式为 Multiply
    // (UI 层会遍历选中图层并下发对应的指令)
    let selected_ids = app.ui.selected_layer_ids.clone();
    for target_id in selected_ids {
        CommandHandler::execute(&mut app, AppCommand::SetLayerOpacity(target_id.clone(), 50));
        CommandHandler::execute(&mut app, AppCommand::SetLayerBlendMode(target_id.clone(), BlendMode::Multiply));
    }

    // 验证结果：
    let store_after = app.engine.store();
    
    // L1 (未选中) 属性必须保持原样
    assert_eq!(store_after.get_layer(&id1).unwrap().opacity, 255, "未选中的图层透明度不应被修改");
    assert_eq!(store_after.get_layer(&id1).unwrap().blend_mode, BlendMode::Normal, "未选中的图层混合模式不应被修改");

    // L2, L3 (已选中) 属性必须全部更新
    assert_eq!(store_after.get_layer(&id2).unwrap().opacity, 50, "选中的图层 L2 透明度应被批量修改");
    assert_eq!(store_after.get_layer(&id2).unwrap().blend_mode, BlendMode::Multiply, "选中的图层 L2 混合模式应被修改");
    
    assert_eq!(store_after.get_layer(&id3).unwrap().opacity, 50, "选中的图层 L3 透明度应被批量修改");
    assert_eq!(store_after.get_layer(&id3).unwrap().blend_mode, BlendMode::Multiply, "选中的图层 L3 混合模式应被修改");
}