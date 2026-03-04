use pxa_engine_win32::app::state::AppState;
use pxa_engine_win32::app::events::InputEvent;
use pxa_engine_win32::core::store::PixelStore;

#[test]
fn test_coordinate_mapping_accuracy() {
    let mut app = AppState::new();

    // 假设画布尺寸是默认的 128x128
    let canvas_w = app.pixel.engine.store().canvas_width as f32; // 128.0
    let canvas_h = app.pixel.engine.store().canvas_height as f32; // 128.0

    // ==========================================
    // 场景 1: 模拟没有 DPI 缩放的标准界面
    // ==========================================
    
    // 假设 UI 右侧有一个 800x600 的工作区 (CentralPanel)
    let panel_width = 800.0;
    let panel_height = 600.0;

    // 模拟 gui.rs 里的 app.pixel.view.update_viewport
    app.pixel.view.update_viewport(panel_width, panel_height);

    // 【断点 1：视口中心点】
    let screen_cx = app.pixel.view.width / 2.0; // 预期 400.0
    let screen_cy = app.pixel.view.height / 2.0; // 预期 300.0
    assert_eq!(screen_cx, 400.0, "视口宽度更新失败");

    // 动作 A：用户刚好点击了工作区的正中心 (400, 300)
    let (cx, cy) = app.pixel.view.screen_to_canvas_raw(app.pixel.engine.store(), screen_cx, screen_cy);
    
    // 断言 A：屏幕中心必须严丝合缝地对应画布的中心 (64, 64)
    assert_eq!(cx, canvas_w / 2.0, "❌ 致命错误：X轴没有对齐画布中心点！当前计算结果: {}", cx);
    assert_eq!(cy, canvas_h / 2.0, "❌ 致命错误：Y轴没有对齐画布中心点！当前计算结果: {}", cy);

    // 动作 B：用户点击了画布的左上角 (预期在屏幕上的位置是 中心点 - 画布的一半)
    // 屏幕上的画布左上角坐标：X = 400 - 64 = 336, Y = 300 - 64 = 236
    let (cx_topleft, cy_topleft) = app.pixel.view.screen_to_canvas_raw(app.pixel.engine.store(), 336.0, 236.0);
    assert_eq!(cx_topleft, 0.0, "❌ 致命错误：无法准确定位画布左上角(0, 0)！计算出的是 ({}, {})", cx_topleft, cy_topleft);

    // ==========================================
    // 场景 2: 模拟用户放大了 2.0 倍 (Zoom)
    // ==========================================
    app.pixel.view.zoom_level = 2.0;

    // 动作 C：由于放大了 2 倍，屏幕上的 10 像素偏移，在画布上应该只移动 5 像素
    // 点击屏幕中心偏右 10 像素 (410, 300)
    let (cx_zoom, cy_zoom) = app.pixel.view.screen_to_canvas_raw(app.pixel.engine.store(), 410.0, 300.0);
    assert_eq!(cx_zoom, 64.0 + 5.0, "❌ 致命错误：缩放倍率计算错误！");

    // ==========================================
    // 场景 3: 模拟用户向左平移画布 (Pan)
    // ==========================================
    app.pixel.view.pan_x = 20.0; // 画布往左平移了 20 像素
    let (cx_pan, cy_pan) = app.pixel.view.screen_to_canvas_raw(app.pixel.engine.store(), screen_cx, screen_cy);
    
    // 断言 C：点屏幕中心，因为画布往左跑了，所以点到的应该是画布偏右侧 (64 - 20 = 44)
    assert_eq!(cx_pan, 64.0 - 20.0, "❌ 致命错误：平移坐标计算失效！");

    println!("✅ 恭喜！引擎底层所有坐标系数学推导 100% 完美无瑕！");
}