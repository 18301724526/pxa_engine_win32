use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::core::color::Color;
use pxa_engine_win32::core::error::CoreError;

/// 辅助：初始化移动工具测试环境
fn setup_move_test() -> AppState {
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
    app
}

// ---------------------------------------------------------
// 1. 无选区时移动整个图层，偏移量正确
// ---------------------------------------------------------
#[test]
fn test_move_entire_layer_no_selection() {
    let mut app = setup_move_test();
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();
    
    // 在 (10, 10) 绘制一个红点
    app.set_tool(ToolType::Pencil);
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();
    
    // 切换到移动工具，在无选区的情况下拖拽 (50, 50) -> (60, 70)
    // 预期：整个图层产生 dx=10, dy=20 的偏移
    app.set_tool(ToolType::Move);
    app.on_mouse_down(50, 50).unwrap();
    app.on_mouse_move(60, 70).unwrap(); 
    app.on_mouse_up().unwrap();
    
    let layer = app.engine.store().get_layer(&layer_id).unwrap();
    assert_eq!(layer.offset_x, 10, "无选区移动应正确修改图层 offset_x");
    assert_eq!(layer.offset_y, 20, "无选区移动应正确修改图层 offset_y");
    
    // 验证实际的像素读取（画布坐标系下，该点应移动到了 20, 30）
    assert_eq!(app.engine.store().get_pixel(&layer_id, 20, 30).unwrap().r, 255);
    assert_eq!(app.engine.store().get_pixel(&layer_id, 10, 10).unwrap_or(Color::transparent()).a, 0, "原画布坐标系位置应为空");
}

// ---------------------------------------------------------
// 2. 有选区时移动选区内容 & 5. 撤销/重做恢复
// ---------------------------------------------------------
#[test]
fn test_move_selection_and_history() {
    let mut app = setup_move_test();
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();
    
    // 绘制两个点：(10, 10) 和 (30, 30)
    app.set_tool(ToolType::Pencil);
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 30).unwrap(); app.on_mouse_up().unwrap();
    
    // 框选 (5, 5) 到 (15, 15)，只包含 (10, 10) 这个点
    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(5, 5).unwrap();
    app.on_mouse_move(15, 15).unwrap();
    app.on_mouse_up().unwrap();
    
    // 使用移动工具将选区向右移动 20 像素 (10, 10) -> (30, 10)
    app.set_tool(ToolType::Move);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(30, 10).unwrap();
    app.on_mouse_up().unwrap();
    
    // 验证 2：原位置清空，新位置绘制，非选区像素不受影响
    assert_eq!(app.engine.store().get_pixel(&layer_id, 10, 10).unwrap().a, 0, "选区内原位置应被清空");
    assert_eq!(app.engine.store().get_pixel(&layer_id, 30, 10).unwrap().r, 255, "像素应移动到新位置");
    assert_eq!(app.engine.store().get_pixel(&layer_id, 30, 30).unwrap().r, 255, "选区外的像素不应发生移动");
    
    // 验证 2：选区遮罩必须跟随移动
    let sel = &app.engine.store().selection;
    assert!(sel.contains(30, 10), "选区遮罩必须跟随内容移动");
    assert!(!sel.contains(10, 10), "选区不应留在原处");
    
    // 验证 5：撤销历史记录
    app.undo();
    assert_eq!(app.engine.store().get_pixel(&layer_id, 10, 10).unwrap().r, 255, "撤销后像素应回到原位");
    assert_eq!(app.engine.store().get_pixel(&layer_id, 30, 10).unwrap().a, 0, "撤销后新位置应被清空");
    assert!(app.engine.store().selection.contains(10, 10), "撤销后选区遮罩也必须回到原位");
}

// ---------------------------------------------------------
// 3. 移动时选区边界裁剪（部分移出画布）
// ---------------------------------------------------------
#[test]
fn test_move_out_of_bounds_clipping() {
    let mut app = setup_move_test();
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();
    
    // 在右下角边缘 (90, 90) 绘制
    app.set_tool(ToolType::Pencil);
    app.on_mouse_down(90, 90).unwrap(); app.on_mouse_up().unwrap();
    
    // 框选 (80, 80) 到 (95, 95)
    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(80, 80).unwrap();
    app.on_mouse_move(95, 95).unwrap();
    app.on_mouse_up().unwrap();
    
    // 往右下角拖拽 30 像素，这会导致选区内容移到 (120, 120)，超出画布 100x100 的范围
    app.set_tool(ToolType::Move);
    app.on_mouse_down(90, 90).unwrap();
    let res = app.on_mouse_move(120, 120);
    assert!(res.is_ok(), "移出画布边界不应导致程序崩溃 (Panic)");
    app.on_mouse_up().unwrap();
    
    // 验证原位置已被清空
    assert_eq!(app.engine.store().get_pixel(&layer_id, 90, 90).unwrap().a, 0, "即便移出画布，原画布位置也应被清空");
}

// ---------------------------------------------------------
// 4. 图层锁定状态下移动应报错
// ---------------------------------------------------------
#[test]
fn test_move_locked_layer_error() {
    let mut app = setup_move_test();
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();
    
    // 锁定当前图层
    if let Some(layer) = app.engine.parts_mut().0.get_layer_mut(&layer_id) {
        layer.locked = true;
    }
    
    // 尝试在无选区下移动图层
    app.set_tool(ToolType::Move);
    let result = app.on_mouse_down(10, 10);
    
    assert!(matches!(result, Err(CoreError::LayerLocked)), "在锁定图层上使用移动工具必须返回 LayerLocked 错误");
}