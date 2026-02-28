use pxa_engine_win32::app::state::{AppState, ToolType, AppMode};
use pxa_engine_win32::core::color::Color;
use pxa_engine_win32::core::store::BrushShape;
use pxa_engine_win32::core::symmetry::SymmetryMode;
use pxa_engine_win32::app::command_handler::CommandHandler;

fn setup_app() -> AppState {
    let mut app = AppState::new();
    if app.engine.store().layers.is_empty() {
        app.add_new_layer();
    }
    app
}

// ---------------------------------------------------------
// 1. 橡皮工具独立测试 & 4. 撤销/重做后像素恢复
// ---------------------------------------------------------
#[test]
fn test_eraser_functionality_and_history() {
    let mut app = setup_app();
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();
    
    // 用铅笔画个红点
    app.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.set_tool(ToolType::Pencil);
    let _ = app.on_mouse_down(10, 10); let _ = app.on_mouse_up();
    
    // 切换橡皮擦擦除
    app.set_tool(ToolType::Eraser);
    let _ = app.on_mouse_down(10, 10);
    let _ = app.on_mouse_up();

    let store = app.engine.store();
    assert_eq!(store.get_pixel(&layer_id, 10, 10).unwrap().a, 0, "橡皮擦应使像素透明");

    // 撤销/重做恢复验证
    app.undo();
    assert_eq!(app.engine.store().get_pixel(&layer_id, 10, 10).unwrap().r, 255, "撤销后像素应恢复为红色");
    app.redo();
    assert_eq!(app.engine.store().get_pixel(&layer_id, 10, 10).unwrap().a, 0, "重做后像素应再次变透明");
}

// ---------------------------------------------------------
// 2. 抖动边界值（0 和 15）
// ---------------------------------------------------------
#[test]
fn test_jitter_boundary_values() {
    let mut app = setup_app();
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();
    app.engine.set_primary_color(Color::new(255, 255, 255, 255));
    app.set_tool(ToolType::Pencil);
    
    // Jitter = 0: 必须精确落在原点
    {
        let (_, _, jitter) = app.engine.brush_settings_mut();
        *jitter = 0;
    }
    let _ = app.on_mouse_down(30, 30); let _ = app.on_mouse_up();
    assert_eq!(app.engine.store().get_pixel(&layer_id, 30, 30).unwrap().a, 255, "Jitter 0 应精准涂色");
    assert_eq!(app.engine.store().get_pixel(&layer_id, 31, 31).unwrap().a, 0);

    // Jitter = 15: 应该有较大范围的偏移
    {
        let (_, _, jitter) = app.engine.brush_settings_mut();
        *jitter = 15;
    }
    // 多点几下增加随机样本
    for _ in 0..10 { let _ = app.on_mouse_down(50, 50); let _ = app.on_mouse_up(); }
    
    let store = app.engine.store();
    let mut has_remote_pixel = false;
    for x in 35..=65 {
        for y in 35..=65 {
            if (x != 50 || y != 50) && store.get_pixel(&layer_id, x, y).unwrap().a > 0 {
                has_remote_pixel = true; 
                break;
            }
        }
    }
    assert!(has_remote_pixel, "Jitter 15 应产生明显的随机偏移散布");
}

// ---------------------------------------------------------
// 3. 抖动与对称组合
// ---------------------------------------------------------
#[test]
fn test_jitter_symmetry_consistency() {
    let mut app = setup_app();
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();
    app.engine.set_primary_color(Color::new(255, 255, 255, 255));
    
    {
        let sym = app.engine.symmetry_mut();
        sym.mode = SymmetryMode::Horizontal;
        sym.axis_x = 64.0;
        let (_, _, jitter) = app.engine.brush_settings_mut();
        *jitter = 5;
    }
    app.set_tool(ToolType::Pencil);
    let _ = app.on_mouse_down(30, 30); let _ = app.on_mouse_up();

    // 验证对称位置是否有像素
    let store = app.engine.store();
    let mut found_symmetry = false;
    for x in 90..105 {
        for y in 25..35 { 
            if store.get_pixel(&layer_id, x, y).unwrap().a > 0 { found_symmetry = true; break; }
        }
    }
    assert!(found_symmetry, "对称模式下抖动点也应在对称轴另一侧同步产生");
}

// ---------------------------------------------------------
// 5. 画笔跨块绘制
// ---------------------------------------------------------
#[test]
fn test_brush_cross_chunk_boundary() {
    let mut app = setup_app();
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();
    
    // CHUNK_SIZE = 64. 绘制在 (63, 63)，size=4 应该跨越中心点周围的 4 个块
    {
        let (size, shape, _) = app.engine.brush_settings_mut();
        *size = 4;
        *shape = BrushShape::Square;
    }
    app.engine.set_primary_color(Color::new(255, 255, 255, 255));
    app.set_tool(ToolType::Pencil);
    let _ = app.on_mouse_down(63, 63);
    let _ = app.on_mouse_up();

    let layer = app.engine.store().get_layer(&layer_id).unwrap();
    // 验证 4 个 Chunk 是否都因涂色被动态创建了
    assert!(layer.chunks.contains_key(&(0, 0)), "Chunk (0,0) missing");
    assert!(layer.chunks.contains_key(&(1, 0)), "Chunk (1,0) missing");
    assert!(layer.chunks.contains_key(&(0, 1)), "Chunk (0,1) missing");
    assert!(layer.chunks.contains_key(&(1, 1)), "Chunk (1,1) missing");
}

// ---------------------------------------------------------
// 6. 极大笔刷在边缘绘制，验证防越界
// ---------------------------------------------------------
#[test]
fn test_huge_brush_edge_safety() {
    let mut app = setup_app();
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();
    
    {
        let (size, _, _) = app.engine.brush_settings_mut();
        *size = 20; // 极大笔刷
    }
    app.engine.set_primary_color(Color::new(255, 255, 255, 255));
    app.set_tool(ToolType::Pencil);
    
    // 在左上角边缘极致位置绘制，笔刷的一半在画布外
    let res = app.on_mouse_down(0, 0);
    assert!(res.is_ok(), "极大笔刷在边缘绘制不应触发越界报错");
    let _ = app.on_mouse_up();
    
    assert_eq!(app.engine.store().get_pixel(&layer_id, 0, 0).unwrap().a, 255);
}

// ---------------------------------------------------------
// 7. 快捷键 [ / ] 调整尺寸
// ---------------------------------------------------------
#[test]
fn test_brush_size_shortcut_simulation() {
    let mut app = setup_app();
    
    // 1. 设置初始尺寸为 10
    {
        let (size, _, _) = app.engine.brush_settings_mut();
        *size = 10;
    }

    // 2. 模拟按下 ']' (放大)
    let cmd_increase = app.shortcuts.handle_text_input("]", AppMode::PixelEdit).unwrap();
    CommandHandler::execute(&mut app, cmd_increase);
    assert_eq!(app.engine.store().brush_size, 11, "按下 ] 应使笔刷 +1");

    // 3. 模拟按下 '[' (缩小)
    let cmd_decrease = app.shortcuts.handle_text_input("[", AppMode::PixelEdit).unwrap();
    CommandHandler::execute(&mut app, cmd_decrease);
    assert_eq!(app.engine.store().brush_size, 10, "按下 [ 应使笔刷 -1");

    // 4. 边界下限测试：不断按 '[' 直到触底
    for _ in 0..20 {
        let cmd_dec = app.shortcuts.handle_text_input("[", AppMode::PixelEdit).unwrap();
        CommandHandler::execute(&mut app, cmd_dec);
    }
    assert_eq!(app.engine.store().brush_size, 1, "笔刷尺寸下限应限制为 1");

    // 5. 边界上限测试：不断按 ']' 直到触顶
    for _ in 0..30 {
        let cmd_inc = app.shortcuts.handle_text_input("]", AppMode::PixelEdit).unwrap();
        CommandHandler::execute(&mut app, cmd_inc);
    }
    assert_eq!(app.engine.store().brush_size, 20, "笔刷尺寸上限应限制为 20");
}