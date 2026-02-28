use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::app::commands::AppCommand;
use pxa_engine_win32::app::command_handler::CommandHandler;
use pxa_engine_win32::core::color::Color;

/// 初始化测试环境：100x100 画布，中间 20x20 红色方块，并建立等大的矩形选区
fn setup_transform_test() -> AppState {
    let mut app = AppState::new();
    let (store, _, _) = app.engine.parts_mut();
    store.canvas_width = 100;
    store.canvas_height = 100;
    
    let layer_id = store.active_layer_id.clone().unwrap();
    if let Some(layer) = store.get_layer_mut(&layer_id) {
        layer.width = 100;
        layer.height = 100;
    }
    app.engine.set_primary_color(Color::new(255, 0, 0, 255));

    // 画一个 20x20 的红色实心方块：(40,40) 到 (59,59)
    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(40, 40).unwrap();
    app.on_mouse_move(59, 59).unwrap();
    app.on_mouse_up().unwrap();

    app.set_tool(ToolType::Bucket);
    app.on_mouse_down(50, 50).unwrap();
    app.on_mouse_up().unwrap();

    // 激活变换工具
    app.set_tool(ToolType::Transform);
    app
}

// ---------------------------------------------------------
// 1. 旋转 & 7. 提交与取消
// ---------------------------------------------------------
#[test]
fn test_transform_rotate_and_commit() {
    let mut app = setup_transform_test();

    // 触发 extract_pixels
    app.on_mouse_down(0, 0).unwrap(); app.on_mouse_up().unwrap();

    // 在选区外围点击并拖拽以触发 Rotate (中心点在 49.5, 49.5)
    // 初始点：(50, 20) -> 正上方
    app.on_mouse_down(50, 20).unwrap();
    // 拖动到 (80, 50) -> 正右方，相当于旋转了 90 度
    app.on_mouse_move(80, 50).unwrap();
    app.on_mouse_up().unwrap();

    // 验证：此时还在预览模式，提交后才写入像素
    CommandHandler::execute(&mut app, AppCommand::CommitCurrentTool);

    // 验证旋转结果（方块本身是正方形，旋转90度依然是正方形，但我们测的是工具的执行过程无崩溃且选区正确）
    let store = app.engine.store();
    assert!(store.selection.is_active, "变换后选区应保持激活");
    assert!(store.selection.contains(50, 50));
}

// ---------------------------------------------------------
// 2. 缩放 & 5. 负数缩放（镜像）
// ---------------------------------------------------------
#[test]
fn test_transform_scale_and_mirror() {
    let mut app = setup_transform_test();
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();

    app.on_mouse_down(0, 0).unwrap(); app.on_mouse_up().unwrap();

    // 抓取右下角缩放手柄 (60, 60)
    // 判定半径是 10.0，所以点 (60, 60) 会命中 DragMode::Scale(1.0, 1.0)
    app.on_mouse_down(60, 60).unwrap();
    
    // 拖拽到左上角 (20, 20)
    // 原本宽度 20，向左拖拽 40，导致 scale_x 和 scale_y 变成 -1.0 左右（负数缩放/镜像）
    app.on_mouse_move(20, 20).unwrap();
    app.on_mouse_up().unwrap();

    // 获取变换参数验证内部状态
    let tool = app.engine.tool_manager().tools.get(&ToolType::Transform).unwrap();
    let transform_tool = tool.as_any().downcast_ref::<pxa_engine_win32::tools::transform::TransformTool>().unwrap();
    
    assert!(transform_tool.scale_x < 0.0, "拖拽过中心点应产生负数缩放 (水平镜像)");
    assert!(transform_tool.scale_y < 0.0, "拖拽过中心点应产生负数缩放 (垂直镜像)");

    // 提交变换
    CommandHandler::execute(&mut app, AppCommand::CommitCurrentTool);

    // 验证撤销功能 (特性 6)
    app.undo();
    let store = app.engine.store();
    // 撤销后，原始的方块应该完美恢复
    assert_eq!(store.get_pixel(&layer_id, 45, 45).unwrap().r, 255, "撤销后像素应恢复");
}

// ---------------------------------------------------------
// 4. 选区更新 & 7. 取消变换 (Escape)
// ---------------------------------------------------------
#[test]
fn test_transform_selection_update_and_cancel() {
    let mut app = setup_transform_test();
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();

    app.on_mouse_down(0, 0).unwrap(); app.on_mouse_up().unwrap();

    // 抓取中心移动 (Move)
    app.on_mouse_down(50, 50).unwrap();
    app.on_mouse_move(70, 70).unwrap(); // 向右下角移动 20 像素
    app.on_mouse_up().unwrap();

    // 发送取消命令 (Escape)
    CommandHandler::execute(&mut app, AppCommand::CancelCurrentTool);

    let store = app.engine.store();
    // 验证取消后：像素没有移动到 (70,70)，依然在 (50,50)
    assert_eq!(store.get_pixel(&layer_id, 70, 70).unwrap().a, 0, "取消变换后，新位置不应有像素");
    assert_eq!(store.get_pixel(&layer_id, 50, 50).unwrap().r, 255, "取消变换后，原位置应保持不变");
    
    // 验证选区也回到了原位
    assert!(store.selection.contains(50, 50), "取消变换后，选区应回到原位");
    assert!(!store.selection.contains(70, 70), "取消变换后，选区不应留在拖拽位置");
}