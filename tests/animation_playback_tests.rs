use pxa_engine_win32::app::state::{AppState, AppMode};
use pxa_engine_win32::app::commands::*; // 修正 1: 引入结构体命令
use pxa_engine_win32::core::animation::timeline::TimelineProperty;

// 辅助函数：模拟旧的执行逻辑
fn exec(app: &mut AppState, cmd: Box<dyn pxa_engine_win32::app::command_handler::Command>) {
    app.enqueue_command(cmd);
    app.process_commands();
}

#[test]
fn test_playback_controls() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;

    assert!(!app.anim.state.is_playing);
    
    exec(&mut app, Box::new(TogglePlaybackCmd));
    assert!(app.anim.state.is_playing);

    exec(&mut app, Box::new(TogglePlaybackCmd));
    assert!(!app.anim.state.is_playing);
}

#[test]
fn test_looping_toggle() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;

    // 1. 顺应引擎的新设定：默认状态下循环播放是开启的
    assert!(app.anim.state.is_looping, "默认状态应该为开启循环");
    
    // 2. 触发一次切换指令
    exec(&mut app, Box::new(ToggleLoopCmd));
    // 此时应该变成关闭状态
    assert!(!app.anim.state.is_looping, "切换后应该关闭循环");

    // 3. 再触发一次切换指令
    exec(&mut app, Box::new(ToggleLoopCmd));
    // 此时应该恢复为开启状态
    assert!(app.anim.state.is_looping, "再次切换应该恢复循环");
}

#[test]
fn test_playback_speed_limits() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;

    exec(&mut app, Box::new(SetPlaybackSpeedCmd(2.0)));
    assert_eq!(app.anim.state.playback_speed, 2.0);

    exec(&mut app, Box::new(SetPlaybackSpeedCmd(-1.0)));
    assert!(app.anim.state.playback_speed >= 0.1, "播放速度下限保护失效");
}

#[test]
fn test_timeline_filter_logic() {
    let mut app = AppState::new();
    // 修正 2：根据 Session 拆分，filter 现在存储在 app.anim
    assert!(app.anim.timeline_filter.contains(&TimelineProperty::Rotation));

    exec(&mut app, Box::new(ToggleTimelineFilterCmd(TimelineProperty::Rotation)));
    assert!(!app.anim.timeline_filter.contains(&TimelineProperty::Rotation), "筛选器应成功移除 Rotation");

    exec(&mut app, Box::new(ToggleTimelineFilterCmd(TimelineProperty::Rotation)));
    assert!(app.anim.timeline_filter.contains(&TimelineProperty::Rotation));
}