use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::app::commands::AppCommand;
use pxa_engine_win32::app::command_handler::CommandHandler;
use pxa_engine_win32::core::color::Color;
use pxa_engine_win32::core::path::NodeType;

fn setup_pen_test() -> AppState {
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
// 1. 节点创建 (点击角点, 拖拽平滑点) & 7. 撤销重做
// ---------------------------------------------------------
#[test]
fn test_pen_node_creation_and_history() {
    let mut app = setup_pen_test();
    app.set_tool(ToolType::Pen);

    // 点击创建角点
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_up().unwrap();
    
    let path = &app.engine.store().active_path;
    assert_eq!(path.nodes.len(), 1);
    assert_eq!(path.nodes[0].kind, NodeType::Corner, "单次点击应创建角点");

    // 拖拽创建平滑点
    app.on_mouse_down(50, 10).unwrap();
    app.on_mouse_move(50, 20).unwrap(); // 向下拖拽 10 像素
    app.on_mouse_up().unwrap();

    let path = &app.engine.store().active_path;
    assert_eq!(path.nodes.len(), 2);
    assert_eq!(path.nodes[1].kind, NodeType::Smooth, "拖拽应创建平滑点");
    assert_eq!(path.nodes[1].handle_out.y, 10.0, "手柄输出应记录拖拽偏移量");
    assert_eq!(path.nodes[1].handle_in.y, -10.0, "手柄输入应与输出对称");

    // 验证撤销逻辑 (撤销第二个节点)
    app.undo();
    assert_eq!(app.engine.store().active_path.nodes.len(), 1, "撤销后应只剩 1 个节点");

    app.redo();
    assert_eq!(app.engine.store().active_path.nodes.len(), 2, "重做后应恢复 2 个节点");
}

// ---------------------------------------------------------
// 2. 手柄调整 & 3. 闭合路径
// ---------------------------------------------------------
#[test]
fn test_pen_close_path_and_adjust_handles() {
    let mut app = setup_pen_test();
    app.set_tool(ToolType::Pen);

    // 绘制三个点形成一个未闭合的三角形
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(50, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 40).unwrap(); app.on_mouse_up().unwrap();

    assert!(!app.engine.store().active_path.is_closed);

    // 点击起始点 (10, 10) 闭合路径
    app.on_mouse_down(10, 10).unwrap(); 
    app.on_mouse_up().unwrap();

    let path = &app.engine.store().active_path;
    assert!(path.is_closed, "点击起始点应闭合路径");
    assert_eq!(path.nodes.len(), 3, "闭合操作不应增加额外节点");

    // 手柄调整：拖拽 (50, 10) 的节点来调整其位置
    // 注意：hit_test 判定半径为 6.0，所以点 50, 10 能命中节点 1
    app.on_mouse_down(50, 10).unwrap();
    app.on_mouse_move(60, 10).unwrap();
    app.on_mouse_up().unwrap();

    let path = &app.engine.store().active_path;
    assert_eq!(path.nodes[1].anchor.x, 60.0, "锚点位置应随拖拽更新");
}

// ---------------------------------------------------------
// 4. 路径转选区
// ---------------------------------------------------------
#[test]
fn test_pen_path_to_selection() {
    let mut app = setup_pen_test();
    app.set_tool(ToolType::Pen);

    // 画一个 20x20 的正方形路径: (10,10) -> (30,10) -> (30,30) -> (10,30)
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 30).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(10, 30).unwrap(); app.on_mouse_up().unwrap();
    // 闭合
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();

    // 触发 "Sel" 命令
    CommandHandler::execute(&mut app, AppCommand::CommitCurrentTool);

    let store = app.engine.store();
    assert!(store.selection.is_active, "转换后选区应激活");
    assert!(store.selection.contains(20, 20), "路径内部应在选区内");
    assert!(!store.selection.contains(5, 5), "路径外部不应在选区内");
    assert!(store.active_path.nodes.is_empty(), "转换为选区后，工作路径应被清空");
}

// ---------------------------------------------------------
// 5. 填充路径
// ---------------------------------------------------------
#[test]
fn test_pen_fill_path() {
    let mut app = setup_pen_test();
    app.set_tool(ToolType::Pen);
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();

    // 画一个三角形: (10,10) -> (50,10) -> (30,40)
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(50, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 40).unwrap(); app.on_mouse_up().unwrap();
    // 闭合
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();

    // 触发 "Fill" 命令
    CommandHandler::execute(&mut app, AppCommand::PenFill);

    let store = app.engine.store();
    // 验证内部填充 (30, 20 肯定在三角形内)
    assert_eq!(store.get_pixel(&layer_id, 30, 20).unwrap().r, 255, "路径内部应被填充前景色");
    // 验证外部未填充 (5, 5 在外部)
    assert_eq!(store.get_pixel(&layer_id, 5, 5).unwrap().a, 0, "路径外部不应被填充");
}

// ---------------------------------------------------------
// 6. 描边路径
// ---------------------------------------------------------
#[test]
fn test_pen_stroke_path() {
    let mut app = setup_pen_test();
    app.set_tool(ToolType::Pen);
    let layer_id = app.engine.store().active_layer_id.clone().unwrap();

    // 画一条直线路径 (10, 10) 到 (30, 10)
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 10).unwrap(); app.on_mouse_up().unwrap();

    // 触发 "Strk" 命令
    CommandHandler::execute(&mut app, AppCommand::PenStroke);

    let store = app.engine.store();
    // 线上应该有像素
    assert_eq!(store.get_pixel(&layer_id, 20, 10).unwrap().r, 255, "路径线段上应被描边");
    // 偏离线的点不应有像素
    assert_eq!(store.get_pixel(&layer_id, 20, 11).unwrap().a, 0, "偏离路径的地方不应有颜色");
}

// ---------------------------------------------------------
// 8. 节点类型切换 (对标 PS 转换工具) & 手柄独立性
// ---------------------------------------------------------
#[test]
fn test_pen_node_type_switching_and_independence() {
    let mut app = setup_pen_test();
    app.set_tool(ToolType::Pen);

    // 创建一个平滑点 (Node 0)
    app.on_mouse_down(50, 50).unwrap();
    app.on_mouse_move(50, 60).unwrap(); // 产生镜像手柄
    app.on_mouse_up().unwrap();

    // 验证初始为平滑点且联动
    {
        let node = &app.engine.store().active_path.nodes[0];
        assert_eq!(node.kind, NodeType::Smooth);
        assert_eq!(node.handle_in.y, -10.0);
    }

    // 切换为角点
    CommandHandler::execute(&mut app, AppCommand::TogglePathNodeType(0));
    assert_eq!(app.engine.store().active_path.nodes[0].kind, NodeType::Corner);

    // 在角点模式下，调整 handle_out，handle_in 应该保持不动 (打破联动)
    app.on_mouse_down(50, 60).unwrap();
    app.on_mouse_move(70, 60).unwrap();
    app.on_mouse_up().unwrap();

    {
        let node = &app.engine.store().active_path.nodes[0];
        assert_eq!(node.handle_out.x, 20.0);
        assert_eq!(node.handle_in.x, 0.0, "角点手柄应相互独立，不联动");
    }

    // 撤销切换：回到平滑点并恢复联动
    app.undo();
    app.undo();
    assert_eq!(app.engine.store().active_path.nodes[0].kind, NodeType::Smooth);
}

// ---------------------------------------------------------
// 9. 交互式节点删除 (对标 PS 钢笔点击删点逻辑)
// ---------------------------------------------------------
#[test]
fn test_pen_interactive_node_deletion() {
    let mut app = setup_pen_test();
    app.set_tool(ToolType::Pen);

    // 创建两个点
    app.on_mouse_down(10, 10).unwrap(); app.on_mouse_up().unwrap();
    app.on_mouse_down(30, 30).unwrap(); app.on_mouse_up().unwrap();
    assert_eq!(app.engine.store().active_path.nodes.len(), 2);

    // 模拟点击第二个点 (30, 30)，触发删除
    app.on_mouse_down(30, 30).unwrap();
    app.on_mouse_up().unwrap();

    assert_eq!(app.engine.store().active_path.nodes.len(), 1, "点击现有节点应自动删除");
    
    // 验证撤销删除
    app.undo();
    assert_eq!(app.engine.store().active_path.nodes.len(), 2, "撤销后节点应恢复");
}