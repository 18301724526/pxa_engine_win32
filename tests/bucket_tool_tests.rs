use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::core::color::Color;
use pxa_engine_win32::core::error::CoreError;
use pxa_engine_win32::core::symmetry::SymmetryMode;

/// 辅助函数：初始化一个指定大小的画布并返回 AppState
fn setup_bucket_test(width: u32, height: u32) -> AppState {
    let mut app = AppState::new();
    let (store, _, _) = app.engine.parts_mut();
    store.canvas_width = width;
    store.canvas_height = height;
    
    let layer_id = store.active_layer_id.clone().unwrap();
    if let Some(layer) = store.get_layer_mut(&layer_id) {
        layer.width = width;
        layer.height = height;
    }
    app
}

#[test]
fn test_bucket_fill_basic_boundary() {
    let mut app = setup_bucket_test(10, 10);
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();
    
    // 1. 绘制一个封闭的正方形边界 (2,2) 到 (5,5)
    app.engine.set_primary_color(Color::new(255, 255, 255, 255));
    app.set_tool(ToolType::Pencil);
    for i in 2..=5 {
        let _ = app.on_mouse_down(i, 2); let _ = app.on_mouse_up();
        let _ = app.on_mouse_down(i, 5); let _ = app.on_mouse_up();
        let _ = app.on_mouse_down(2, i); let _ = app.on_mouse_up();
        let _ = app.on_mouse_down(5, i); let _ = app.on_mouse_up();
    }

    // 2. 在内部 (3,3) 填充红色
    app.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.set_tool(ToolType::Bucket);
    let _ = app.on_mouse_down(3, 3);
    let _ = app.on_mouse_up();

    // 验证：内部填充，外部及边界不变
    let store = app.engine.store();
    assert_eq!(store.get_pixel(&layer_id, 3, 3).unwrap(), Color::new(255, 0, 0, 255));
    assert_eq!(store.get_pixel(&layer_id, 4, 4).unwrap(), Color::new(255, 0, 0, 255));
    assert_eq!(store.get_pixel(&layer_id, 2, 2).unwrap(), Color::new(255, 255, 255, 255));
    assert_eq!(store.get_pixel(&layer_id, 1, 1).unwrap().a, 0);
}

#[test]
fn test_bucket_fill_selection_restriction() {
    let mut app = setup_bucket_test(10, 10);
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();

    // 1. 创建选区
    app.set_tool(ToolType::RectSelect);
    let _ = app.on_mouse_down(0, 0);
    let _ = app.on_mouse_move(2, 2);
    let _ = app.on_mouse_up();

    // 2. 执行填充
    app.engine.set_primary_color(Color::new(0, 255, 0, 255));
    app.set_tool(ToolType::Bucket);
    let _ = app.on_mouse_down(1, 1);
    let _ = app.on_mouse_up();

    let store = app.engine.store();
    assert_eq!(store.get_pixel(&layer_id, 0, 0).unwrap().g, 255);
    assert_eq!(store.get_pixel(&layer_id, 3, 3).unwrap().a, 0, "选区外不应填充");
}

#[test]
fn test_bucket_fill_same_color_no_history() {
    let mut app = setup_bucket_test(10, 10);
    app.engine.set_primary_color(Color::transparent()); 
    
    // 记录执行前的历史栈深度
    let history_before = app.engine.history().undo_stack.len();
    
    app.set_tool(ToolType::Bucket);
    let _ = app.on_mouse_down(5, 5);
    let _ = app.on_mouse_up();

    // 验证：同色填充不产生历史记录
    let history_after = app.engine.history().undo_stack.len();
    assert_eq!(history_before, history_after);
}

#[test]
fn test_bucket_fill_locked_layer() {
    let mut app = setup_bucket_test(10, 10);
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();
    
    // 锁定图层
    if let Some(l) = app.engine.parts_mut().0.get_layer_mut(&layer_id) {
        l.locked = true;
    }

    app.set_tool(ToolType::Bucket);
    let result = app.on_mouse_down(5, 5);

    match result {
        Err(CoreError::LayerLocked) => {},
        _ => panic!("Expected LayerLocked error, got {:?}", result),
    }
}

#[test]
fn test_bucket_fill_diagonal_barrier() {
    let mut app = setup_bucket_test(10, 10);
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();

    // 1. 绘制一条从 (0,0) 到 (9,9) 的完整对角线，彻底分割画布
    app.engine.set_primary_color(Color::new(255, 255, 255, 255));
    app.set_tool(ToolType::Pencil);
    for i in 0..10 {
        let _ = app.on_mouse_down(i, i); 
        let _ = app.on_mouse_up();
    }

    // 2. 在左下角区域 (0,9) 填充红色
    app.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.set_tool(ToolType::Bucket);
    let _ = app.on_mouse_down(0, 9); 
    let _ = app.on_mouse_up();

    // 3. 验证
    let store = app.engine.store();
    assert_eq!(store.get_pixel(&layer_id, 0, 9).unwrap().r, 255, "填充点应为红色");
    assert_eq!(store.get_pixel(&layer_id, 9, 0).unwrap().a, 0, "贯穿对角线应阻断填充，另一侧应保持透明");
}

#[test]
fn test_bucket_fill_symmetry() {
    let mut app = setup_bucket_test(10, 10);
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();

    // 开启水平对称
    {
        let sym = app.engine.symmetry_mut();
        sym.mode = SymmetryMode::Horizontal;
        sym.axis_x = 5.0;
    }

    app.engine.set_primary_color(Color::new(255, 255, 0, 255));
    app.set_tool(ToolType::Bucket);
    
    let _ = app.on_mouse_down(2, 5);
    let _ = app.on_mouse_up();

    let store = app.engine.store();
    assert_eq!(store.get_pixel(&layer_id, 2, 5).unwrap().r, 255);
    assert_eq!(store.get_pixel(&layer_id, 8, 5).unwrap().r, 255, "对称点应同步填充");
}