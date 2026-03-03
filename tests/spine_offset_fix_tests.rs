use pxa_engine_win32::app::state::{AppState, AppMode};
use pxa_engine_win32::app::commands::*;
use pxa_engine_win32::core::animation::bone::BoneData;
use pxa_engine_win32::core::animation::timeline::{TimelineProperty, KeyframeValue, CurveType};

fn exec(app: &mut AppState, cmd: Box<dyn pxa_engine_win32::app::command_handler::Command>) {
    app.enqueue_command(cmd);
    app.process_commands();
}

#[test]
fn test_spine_offset_loop_maintenance() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;
    exec(&mut app, Box::new(CreateAnimationCmd("Loop".into())));
    app.anim.state.project.skeleton.add_bone(BoneData::new("B1".into(), "B1".into()));
    
    let anim_id = app.anim.state.project.active_animation_id.clone().unwrap();
    {
        let anim = app.anim.state.project.animations.get_mut(&anim_id).unwrap();
        anim.duration = 2.0;
        let mut tl = pxa_engine_win32::core::animation::timeline::Timeline::new("B1".into(), TimelineProperty::Rotation);
        tl.add_keyframe(0.0, KeyframeValue::Rotate(0.0), CurveType::Linear);
        tl.add_keyframe(1.0, KeyframeValue::Rotate(90.0), CurveType::Linear);
        tl.add_keyframe(2.0, KeyframeValue::Rotate(0.0), CurveType::Linear);
        anim.timelines.push(tl);
    }

    app.anim.selected_keyframes = vec![
        ("B1".into(), Some(TimelineProperty::Rotation), 0.0),
        ("B1".into(), Some(TimelineProperty::Rotation), 1.0),
        ("B1".into(), Some(TimelineProperty::Rotation), 2.0),
    ];
    
    exec(&mut app, Box::new(ApplySpineOffsetCmd { mode: 0, fixed_frames: 30, step_frames: 0 }));

    let anim = app.anim.state.project.animations.get(&anim_id).unwrap();
    let tl = anim.timelines.iter().find(|t| t.target_id == "B1" && t.property == TimelineProperty::Rotation).unwrap();

    let val_at_0 = tl.sample(0.0).unwrap();
    if let KeyframeValue::Rotate(deg) = val_at_0 {
        assert!((deg - 90.0).abs() < 0.001);
    }

    exec(&mut app, Box::new(UndoCmd));
    let anim_restored = app.anim.state.project.animations.get(&anim_id).unwrap();
    let tl_restored = anim_restored.timelines.iter().find(|t| t.target_id == "B1" && t.property == TimelineProperty::Rotation).unwrap();
    assert!((tl_restored.sample(1.0).map(|v| match v { KeyframeValue::Rotate(d) => d, _ => 0.0 }).unwrap() - 90.0).abs() < 0.001);
}