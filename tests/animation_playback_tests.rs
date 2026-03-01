use pxa_engine_win32::app::state::{AppState, AppMode};
use pxa_engine_win32::app::commands::AppCommand;
use pxa_engine_win32::app::command_handler::CommandHandler;
use pxa_engine_win32::core::animation::timeline::{TimelineProperty, CurveType};
use pxa_engine_win32::core::animation::bone::BoneData;

fn setup_env() -> AppState {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;
    CommandHandler::execute(&mut app, AppCommand::CreateAnimation("Action".into()));
    app.animation.project.skeleton.add_bone(BoneData::new("Root".into(), "Root".into()));
    app
}

#[test]
fn test_playback_and_stepping_commands() {
    let mut app = setup_env();

    // 测试 1 & 2: 播放暂停和单帧步进
    assert!(!app.animation.is_playing);
    CommandHandler::execute(&mut app, AppCommand::TogglePlayback);
    assert!(app.animation.is_playing);

    app.animation.current_time = 0.0;
    CommandHandler::execute(&mut app, AppCommand::StepFrame(1));
    assert!((app.animation.current_time - (1.0 / 30.0)).abs() < 0.001, "下一帧应推进 1/30 秒");

    CommandHandler::execute(&mut app, AppCommand::StepFrame(-1));
    assert_eq!(app.animation.current_time, 0.0, "上一帧应回退，且不小于 0");

    // 测试 4: 跳转指定时间 (首末帧)
    CommandHandler::execute(&mut app, AppCommand::SetTime(2.0));
    assert_eq!(app.animation.current_time, 2.0);

    // 测试 3 & 5: 循环与速度
    CommandHandler::execute(&mut app, AppCommand::SetPlaybackSpeed(1.5));
    assert_eq!(app.animation.playback_speed, 1.5);
    
    let loop_state = app.animation.is_looping;
    CommandHandler::execute(&mut app, AppCommand::ToggleLoop);
    assert_eq!(app.animation.is_looping, !loop_state);
}

#[test]
fn test_timeline_filter_toggle() {
    let mut app = setup_env();
    
    // 默认开启 Rotation, Translation, Scale
    assert!(app.ui.timeline_filter.contains(&TimelineProperty::Rotation));
    
    // 发送指令关闭 Rotation 显示
    CommandHandler::execute(&mut app, AppCommand::ToggleTimelineFilter(TimelineProperty::Rotation));
    assert!(!app.ui.timeline_filter.contains(&TimelineProperty::Rotation), "筛选器应成功移除 Rotation");
    
    // 再次发送指令，应恢复
    CommandHandler::execute(&mut app, AppCommand::ToggleTimelineFilter(TimelineProperty::Rotation));
    assert!(app.ui.timeline_filter.contains(&TimelineProperty::Rotation));
}

#[test]
fn test_curve_editor_update_command() {
    let mut app = setup_env();
    
    // 强制加入一个关键帧
    CommandHandler::execute(&mut app, AppCommand::InsertManualKeyframe("Root".into()));
    
    // 验证曲线修改指令 (特性 2, 3)
    let new_curve = CurveType::Bezier(0.1, 0.2, 0.8, 0.9);
    CommandHandler::execute(&mut app, AppCommand::UpdateKeyframeCurve("Root".into(), TimelineProperty::Translation, 0.0, new_curve.clone()));
    
    let active_id = app.animation.project.active_animation_id.as_ref().unwrap();
    let anim = app.animation.project.animations.get(active_id).unwrap();
    let tl = anim.timelines.iter().find(|t| t.target_id == "Root" && t.property == TimelineProperty::Translation).unwrap();
    
    assert_eq!(tl.keyframes[0].curve, new_curve, "关键帧的曲线属性必须被正确更新并推入历史记录");
}