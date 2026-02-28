use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::core::color::Color;
use pxa_engine_win32::core::layer::Layer;

const CANVAS_BG_COLOR: Color = Color { r: 35, g: 35, b: 35, a: 255 };
/// 辅助：创建一个带两个预设图层的 App
fn setup_eyedropper_env() -> AppState {
    let mut app = AppState::new();
    let (store, _, _) = app.engine.parts_mut();
    
    // 清理默认图层，手动构建测试环境
    store.layers.clear();
    
    // 底层：纯红色 (255, 0, 0, 255)
    let mut bottom = Layer::new("bottom".into(), "Bottom".into(), 128, 128);
    bottom.set_pixel(10, 10, Color::new(255, 0, 0, 255)).unwrap();
    store.add_layer(bottom);
    
    // 顶层：半透明蓝色 (0, 0, 255, 128)
    let mut top = Layer::new("top".into(), "Top".into(), 128, 128);
    top.set_pixel(10, 10, Color::new(0, 0, 255, 128)).unwrap();
    store.add_layer(top);
    
    // 强制更新复合缓存，取色器依赖此缓存
    app.engine.update_render_cache(None);
    app
}

#[test]
fn test_eyedropper_basic_and_transparent_picking() {
    let mut app = setup_eyedropper_env();
    app.set_tool(ToolType::Eyedropper);

    // 1. 从有颜色的区域取色 (10, 10)
    app.on_mouse_down(10, 10).unwrap();
    let picked = app.engine.store().primary_color;
    
    // 验证：颜色不应该是纯红或纯蓝，而是混合后的结果
    // 混合计算大致：Red*(1-0.5) + Blue*0.5 (假设 Normal 混合)
    assert!(picked.r < 255 && picked.b > 0, "取色应为混合后的颜色");

    // 2. 从完全透明区域取色 (50, 50)
    app.on_mouse_down(50, 50).unwrap();
    let transparent_picked = app.engine.store().primary_color;
    assert_eq!(transparent_picked, CANVAS_BG_COLOR, "透明区域取色应拿到画布底色");
}

#[test]
fn test_eyedropper_drag_realtime_update() {
    let mut app = setup_eyedropper_env();
    app.set_tool(ToolType::Eyedropper);

    // 1. 在透明处按下
    app.on_mouse_down(50, 50).unwrap();
    assert_eq!(app.engine.store().primary_color, CANVAS_BG_COLOR);

    // 2. 拖拽到有色区域 (10, 10)，主色应实时更新
    app.on_mouse_move(10, 10).unwrap();
    let dragging_color = app.engine.store().primary_color;
    assert!(dragging_color.a > 0, "拖拽过程中颜色应随光标更新");

    // 3. 释放鼠标，主色定格
    app.on_mouse_up().unwrap();
    assert_eq!(app.engine.store().primary_color, dragging_color);
}

#[test]
fn test_eyedropper_layer_visibility_impact() {
    let mut app = setup_eyedropper_env();
    app.set_tool(ToolType::Eyedropper);

    // 隐藏顶层蓝色图层
    app.engine.toggle_layer_visibility("top").unwrap();
    app.engine.update_render_cache(None); // 模拟隐藏后的重新渲染

    // 此时取色应只拿到背景的红色
    app.on_mouse_down(10, 10).unwrap();
    let picked = app.engine.store().primary_color;
    assert_eq!(picked, Color::new(255, 0, 0, 255), "隐藏图层后取色不应包含该图层颜色");
}

#[test]
fn test_eyedropper_tool_switching() {
    let mut app = AppState::new();
    
    // 验证工具切换逻辑
    app.set_tool(ToolType::Eyedropper);
    assert_eq!(app.engine.tool_manager().active_type, ToolType::Eyedropper);
    
    // 验证切换回铅笔
    app.set_tool(ToolType::Pencil);
    assert_eq!(app.engine.tool_manager().active_type, ToolType::Pencil);
}