use pxa_engine_win32::app::state::{AppState, AppMode};
use pxa_engine_win32::app::commands::AppCommand;
use pxa_engine_win32::app::command_handler::CommandHandler;
use pxa_engine_win32::core::animation::bone::BoneData;
use pxa_engine_win32::core::animation::timeline::{TimelineProperty, KeyframeValue, CurveType};

#[test]
fn test_spine_offset_loop_maintenance() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;
    CommandHandler::execute(&mut app, AppCommand::CreateAnimation("Loop".into()));
    app.animation.project.skeleton.add_bone(BoneData::new("B1".into(), "B1".into()));
    
    let anim_id = app.animation.project.active_animation_id.clone().unwrap();
    {
        let anim = app.animation.project.animations.get_mut(&anim_id).unwrap();
        anim.duration = 2.0;
        let mut tl = pxa_engine_win32::core::animation::timeline::Timeline::new("B1".into(), TimelineProperty::Rotation);
        // 初始状态：0.0s = 0度, 1.0s = 90度, 2.0s = 0度
        tl.add_keyframe(0.0, KeyframeValue::Rotate(0.0), CurveType::Linear);
        tl.add_keyframe(1.0, KeyframeValue::Rotate(90.0), CurveType::Linear);
        tl.add_keyframe(2.0, KeyframeValue::Rotate(0.0), CurveType::Linear);
        anim.timelines.push(tl);
    }

    // 选中所有关键帧，偏移 30 帧 (1.0s)
    app.ui.selected_keyframes = vec![
        ("B1".into(), Some(TimelineProperty::Rotation), 0.0),
        ("B1".into(), Some(TimelineProperty::Rotation), 1.0),
        ("B1".into(), Some(TimelineProperty::Rotation), 2.0),
    ];
    
    CommandHandler::execute(&mut app, AppCommand::ApplySpineOffset { mode: 0, fixed_frames: 30, step_frames: 0 });

    let anim = app.animation.project.animations.get(&anim_id).unwrap();
    let tl = &anim.timelines[0];

    // 验证 1：原 1.0s 的帧（90度）现在应该移动到 2.0s 并 wrap 到 0.0s
    // 验证 2：补帧逻辑必须确保 0.0s 处有值。由于偏移了 1s，现在的 0.0s 应该是原 1.0s 的值（90度）
    let val_at_0 = tl.sample(0.0).unwrap();
    if let KeyframeValue::Rotate(deg) = val_at_0 {
        assert!((deg - 90.0).abs() < 0.001, "偏移后 0.0s 必须补齐正确的插值（预期 90.0, 实际 {}）", deg);
    }

    // 验证 3：撤销偏移
    CommandHandler::execute(&mut app, AppCommand::Undo);
    let tl_restored = &app.animation.project.animations.get(&anim_id).unwrap().timelines[0];
    assert!((tl_restored.sample(1.0).map(|v| match v { KeyframeValue::Rotate(d) => d, _ => 0.0 }).unwrap() - 90.0).abs() < 0.001);
}