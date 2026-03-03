use pxa_engine_win32::app::state::AppState;
use pxa_engine_win32::app::commands::*;
use pxa_engine_win32::app::command_handler::AppEvent;

// 辅助函数：简化命令执行
fn exec(app: &mut AppState, cmd: Box<dyn pxa_engine_win32::app::command_handler::Command>) {
    app.enqueue_command(cmd);
    app.process_commands();
}

#[test]
fn test_window_maximize_minimize_drag_events() {
    let mut app = AppState::new();
    
    // 1. 测试最大化指令
    exec(&mut app, Box::new(WindowMaximizeCmd));
    let max_event = app.command_bus.events.pop_front().expect("期望输出一个事件，但队列为空");
    assert!(
        matches!(max_event, AppEvent::MaximizeWindow), 
        "WindowMaximizeCmd 应该触发 MaximizeWindow 事件"
    );

    // 2. 测试最小化指令
    exec(&mut app, Box::new(WindowMinimizeCmd));
    let min_event = app.command_bus.events.pop_front().expect("期望输出一个事件，但队列为空");
    assert!(
        matches!(min_event, AppEvent::MinimizeWindow), 
        "WindowMinimizeCmd 应该触发 MinimizeWindow 事件"
    );

    // 3. 测试窗口拖拽指令
    exec(&mut app, Box::new(WindowDragCmd));
    let drag_event = app.command_bus.events.pop_front().expect("期望输出一个事件，但队列为空");
    assert!(
        matches!(drag_event, AppEvent::DragWindow), 
        "WindowDragCmd 应该触发 DragWindow 事件"
    );
}

#[test]
fn test_window_close_protection_logic() {
    let mut app = AppState::new();
    
    // --- 场景 A：文件未修改（干净状态），点击关闭 ---
    app.is_dirty = false;
    exec(&mut app, Box::new(RequestExitCmd));
    let clean_event = app.command_bus.events.pop_front().expect("期望输出一个事件，但队列为空");
    assert!(
        matches!(clean_event, AppEvent::CloseWindow), 
        "当 is_dirty 为 false 时，请求退出应该直接触发 CloseWindow"
    );

    // 清空可能残留的事件
    app.command_bus.events.clear();

    // --- 场景 B：文件已修改（脏状态），点击关闭 ---
    app.is_dirty = true;
    exec(&mut app, Box::new(RequestExitCmd));
    let dirty_event = app.command_bus.events.pop_front().expect("期望输出一个事件，但队列为空");
    assert!(
        matches!(dirty_event, AppEvent::ShowExitModal), 
        "当 is_dirty 为 true 时，请求退出必须触发 ShowExitModal 拦截"
    );

    // --- 场景 C：在拦截模态框中点击“确认直接退出” ---
    exec(&mut app, Box::new(ConfirmExitCmd));
    let confirm_event = app.command_bus.events.pop_front().expect("期望输出一个事件，但队列为空");
    assert!(
        matches!(confirm_event, AppEvent::CloseWindow), 
        "确认退出指令必须触发 CloseWindow"
    );
}