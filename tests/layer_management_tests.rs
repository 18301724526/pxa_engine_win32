use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::app::commands::AppCommand;
use pxa_engine_win32::app::command_handler::CommandHandler;
use pxa_engine_win32::core::color::Color;
use pxa_engine_win32::core::error::CoreError;

fn setup_layer_test() -> AppState {
    rust_i18n::set_locale("zh-CN"); // 确保名称默认逻辑符合预期
    AppState::new()
}

// ---------------------------------------------------------
// 1. 新建图层 (默认名称递增) & 2. 删除活动层 (Fallback & 唯一性保护)
// ---------------------------------------------------------
#[test]
fn test_layer_add_and_delete_fallback() {
    let mut app = setup_layer_test();
    assert_eq!(app.engine.store().layers.len(), 1);
    
    // 1. 新增图层并验证名称递增
    app.add_new_layer();
    app.add_new_layer();
    assert_eq!(app.engine.store().layers.len(), 3);
    assert_eq!(app.engine.store().layers[1].name, "图层 2");
    assert_eq!(app.engine.store().layers[2].name, "图层 3");
    
    let id3 = app.engine.store().layers[2].id.clone();
    assert_eq!(app.engine.store().active_layer_id.as_ref(), Some(&id3), "新建后应自动成为活动图层");

    // 2. 删除活动层 (删除 Top)
    app.delete_active_layer();
    assert_eq!(app.engine.store().layers.len(), 2, "应成功删除一个图层");
    
    let id2 = app.engine.store().layers[1].id.clone();
    assert_eq!(app.engine.store().active_layer_id.as_ref(), Some(&id2), "删除后，下一个可用图层应成为活动层");

    // 2. 唯一性保护：删除直到剩下最后 1 个
    app.delete_active_layer(); // 删掉 Layer 2，剩 Layer 1
    assert_eq!(app.engine.store().layers.len(), 1);
    
    app.delete_active_layer(); // 尝试删除最后一个
    assert_eq!(app.engine.store().layers.len(), 1, "当只剩一个图层时，无法执行删除");
}

// ---------------------------------------------------------
// 3. 复制图层 (内容相同，带副本标识)
// ---------------------------------------------------------
#[test]
fn test_layer_duplicate() {
    let mut app = setup_layer_test();
    let original_id = app.engine.store().active_layer_id.clone().unwrap();
    
    // 在原图层画一个红色像素
    app.set_tool(ToolType::Pencil);
    app.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();

    // 执行复制
    CommandHandler::execute(&mut app, AppCommand::DuplicateLayer(original_id.clone()));
    
    let store = app.engine.store();
    assert_eq!(store.layers.len(), 2, "应生成新图层");
    
    let dup_layer = &store.layers[1];
    assert_eq!(dup_layer.get_pixel(10, 10).unwrap().r, 255, "副本图层内容必须与原图层完全一致");
    assert!(dup_layer.name.contains("图层 1") || dup_layer.name.contains("副本") || dup_layer.name.contains("copy"), "副本名称应包含原名称及衍生标识");
}

// ---------------------------------------------------------
// 4. 合并多个图层 (合成像素正确)
// ---------------------------------------------------------
#[test]
fn test_layer_merge_selected() {
    let mut app = setup_layer_test();
    
    // 底层 L1：画红色 (10, 10)
    let id1 = app.engine.store().active_layer_id.clone().unwrap();
    app.set_tool(ToolType::Pencil);
    app.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();

    // 顶层 L2：画绿色 (20, 20)
    app.add_new_layer();
    let id2 = app.engine.store().active_layer_id.clone().unwrap();
    app.engine.set_primary_color(Color::new(0, 255, 0, 255));
    app.on_mouse_down(20, 20).unwrap(); app.on_mouse_up().unwrap();

    // 执行合并
    CommandHandler::execute(&mut app, AppCommand::MergeSelected(vec![id1, id2]));
    
    let store = app.engine.store();
    assert_eq!(store.layers.len(), 1, "合并后原图层应被销毁，只保留一个合成层");
    
    let merged_id = store.layers[0].id.clone();
    assert_eq!(store.get_pixel(&merged_id, 10, 10).unwrap().r, 255, "底层像素应被保留在合并层中");
    assert_eq!(store.get_pixel(&merged_id, 20, 20).unwrap().g, 255, "顶层像素应被保留在合并层中");
}

// ---------------------------------------------------------
// 5. 调整顺序 (拖拽视觉顺序)
// ---------------------------------------------------------
#[test]
fn test_layer_reorder() {
    let mut app = setup_layer_test();
    let id1 = app.engine.store().layers[0].id.clone();
    
    app.add_new_layer();
    let id2 = app.engine.store().layers[1].id.clone();
    
    // 当前顺序：[id1, id2]。将 id2 下移
    CommandHandler::execute(&mut app, AppCommand::MoveLayerDown(id2.clone()));
    
    let store = app.engine.store();
    assert_eq!(store.layers[0].id, id2, "图层 2 应该移动到了底部 (索引0)");
    assert_eq!(store.layers[1].id, id1, "图层 1 应该被顶到了上方 (索引1)");
}

// ---------------------------------------------------------
// 6. 重命名图层 (冲突时自动添加序号)
// ---------------------------------------------------------
#[test]
fn test_layer_rename_conflict() {
    let mut app = setup_layer_test();
    let id1 = app.engine.store().layers[0].id.clone();
    
    // 重命名第一层为 "Body"
    CommandHandler::execute(&mut app, AppCommand::RenameLayer(id1.clone(), "Body".into()));
    
    // 新建第二层，并强行再次重命名为 "Body"
    app.add_new_layer();
    let id2 = app.engine.store().layers[1].id.clone();
    CommandHandler::execute(&mut app, AppCommand::RenameLayer(id2.clone(), "Body".into()));
    
    let store = app.engine.store();
    assert_eq!(store.layers[0].name, "Body", "原图层名称保持不变");
    assert_eq!(store.layers[1].name, "Body (2)", "冲突名称应自动添加后缀序号防重");
}

// ---------------------------------------------------------
// 7. 锁定/解锁 (各种工具的彻底阻断)
// ---------------------------------------------------------
#[test]
fn test_layer_lock_all_tools_rejection() {
    let mut app = setup_layer_test();
    let id1 = app.engine.store().active_layer_id.clone().unwrap();
    
    // 锁定图层
    CommandHandler::execute(&mut app, AppCommand::ToggleLayerLock(id1.clone()));
    
    // 测试 1：铅笔工具应当拒绝对锁定层的修改
    app.set_tool(ToolType::Pencil);
    let res_pencil = app.on_mouse_down(10, 10);
    assert!(matches!(res_pencil, Err(CoreError::LayerLocked)), "铅笔工具必须拦截锁定层");
    let _ = app.on_mouse_up();

    // 测试 2：变换工具应当拒绝提取像素
    app.set_tool(ToolType::Transform);
    let res_transform = app.on_mouse_down(10, 10);
    assert!(matches!(res_transform, Err(CoreError::LayerLocked)), "变换工具必须拦截锁定层");
    let _ = app.on_mouse_up();
}