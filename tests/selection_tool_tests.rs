use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::core::color::Color;
use pxa_engine_win32::app::commands::AppCommand;
use pxa_engine_win32::app::command_handler::CommandHandler;

/// 辅助：初始化测试环境
fn setup_selection_test() -> AppState {
    let mut app = AppState::new();
    {
        // 修正：通过 parts_mut() 获取可变 store
        let (store, _, _) = app.engine.parts_mut();
        store.canvas_width = 100;
        store.canvas_height = 100;
    }
    app
}

#[test]
fn test_rect_selection_creation_and_cancel() {
    let mut app = setup_selection_test();
    
    // 1. 拖拽创建矩形选区 (10,10) -> (20,20)
    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(20, 20).unwrap();
    app.on_mouse_up().unwrap();
    
    {
        let sel = &app.engine.store().selection;
        assert!(sel.is_active, "选区应处于激活状态");
        assert!(sel.contains(10, 10));
        assert!(sel.contains(20, 20));
        assert!(!sel.contains(9, 9));
    }

    // 2. 取消选区 (模拟 Ctrl+D)
    CommandHandler::execute(&mut app, AppCommand::ClearSelection);
    assert!(!app.engine.store().selection.is_active, "取消选择后选区应失效");
}

#[test]
fn test_ellipse_selection_and_invert() {
    let mut app = setup_selection_test();
    
    // 1. 创建椭圆选区
    app.set_tool(ToolType::EllipseSelect);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(30, 30).unwrap(); // 直径 20 的圆
    app.on_mouse_up().unwrap();
    
    let store = app.engine.store();
    assert!(store.selection.contains(20, 20), "圆心应在选区内");
    assert!(!store.selection.contains(10, 10), "矩形角点不应在圆内");

    // 2. 反向选择
    CommandHandler::execute(&mut app, AppCommand::InvertSelection);
    let store = app.engine.store();
    assert!(!store.selection.contains(20, 20), "反选后圆心应不在选区内");
    assert!(store.selection.contains(0, 0), "反选后外部区域应被选中");
}

#[test]
fn test_selection_stroke_accuracy() {
    let mut app = setup_selection_test();
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();
    
    // 1. 创建 3x3 的矩形选区 (10,10) -> (12,12)
    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(12, 12).unwrap();
    app.on_mouse_up().unwrap();

    // 2. 执行描边，宽度为 1
    app.engine.set_primary_color(Color::new(255, 0, 0, 255));
    CommandHandler::execute(&mut app, AppCommand::StrokeSelection(1));

    // 3. 验证像素：
    let store = app.engine.store();
    assert_eq!(store.get_pixel(&layer_id, 10, 10).unwrap().r, 255);
    assert_eq!(store.get_pixel(&layer_id, 11, 11).unwrap().a, 0, "描边不应填充内部像素");
}

#[test]
fn test_selection_interaction_with_layer_offset() {
    let mut app = setup_selection_test();
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();
    
    // 1. 设置图层偏移为 (10, 10)
    {
        let (store, _, _) = app.engine.parts_mut();
        if let Some(l) = store.get_layer_mut(&layer_id) {
            l.offset_x = 10;
            l.offset_y = 10;
        }
    }

    // 2. 创建一个在画布坐标 (15, 15) 的点状选区
    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(15, 15).unwrap();
    app.on_mouse_move(16, 15).unwrap();
    app.on_mouse_up().unwrap();

    // 3. 使用铅笔工具在 (15, 15) 绘画
    app.engine.set_primary_color(Color::new(0, 255, 0, 255));
    app.set_tool(ToolType::Pencil);
    app.on_mouse_down(15, 15).unwrap();
    app.on_mouse_up().unwrap();

    // 4. 验证：
    // 修正：直接从 store 取像素，不持有长期引用，解决借用冲突
    let pixel_on = app.engine.store().get_pixel(&layer_id, 15, 15).unwrap();
    assert_eq!(pixel_on.g, 255, "选区内绘画应成功");
    
    // 在选区外 (16, 16) 绘画应失败
    app.on_mouse_down(17, 17).unwrap();
    app.on_mouse_up().unwrap();
    let pixel_off = app.engine.store().get_pixel(&layer_id, 17, 17).unwrap();
    assert_eq!(pixel_off.a, 0, "选区外不应产生像素修改");
}

#[test]
fn test_selection_invalid_zero_size() {
    let mut app = setup_selection_test();
    
    // 尝试在同一点按下并抬起（长宽为0）
    app.set_tool(ToolType::RectSelect);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_up().unwrap();
    
    assert!(!app.engine.store().selection.is_active, "无效选区（长宽为0）不应激活");
}