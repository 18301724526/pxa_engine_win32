use pxa_engine_win32::app::state::{AppState, AppMode, ToolType};
use pxa_engine_win32::app::commands::AppCommand;
use pxa_engine_win32::app::command_handler::CommandHandler;
use pxa_engine_win32::core::animation::bone::BoneData;
use pxa_engine_win32::core::animation::timeline::{TimelineProperty, KeyframeValue, CurveType};

fn setup_test_context() -> AppState {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;
    app.animation.project.skeleton.add_bone(BoneData::new("Root".into(), "Root".into()));
    app.animation.project.skeleton.add_bone(BoneData::new("Child".into(), "Child".into()));
    app.animation.project.skeleton.update();
    CommandHandler::execute(&mut app, AppCommand::CreateAnimation("LoopTest".into()));
    app
}

#[test]
fn test_bone_transform_undo_consistency() {
    let mut app = setup_test_context();
    app.ui.selected_bone_id = Some("Root".into());
    app.set_tool(ToolType::BoneRotate);

    let initial_rot = app.animation.project.skeleton.bones[0].local_transform.rotation;
    app.on_mouse_down(100, 100).unwrap();
    app.on_mouse_move(100, 150).unwrap(); 
    app.on_mouse_up().unwrap();
    
    let mid_rot = app.animation.project.skeleton.bones[0].local_transform.rotation;
    assert_ne!(initial_rot, mid_rot);

    CommandHandler::execute(&mut app, AppCommand::Undo);
    assert_eq!(app.animation.project.skeleton.bones[0].local_transform.rotation, initial_rot);
}

#[test]
fn test_single_keyframe_move_undo() {
    let mut app = setup_test_context();
    let anim_id = app.animation.project.active_animation_id.clone().unwrap();

    app.animation.current_time = 0.5;
    CommandHandler::execute(&mut app, AppCommand::InsertManualKeyframe("Root".into()));

    // 单选：只选 Root 的 Rotation
    app.ui.selected_keyframes = vec![
        ("Root".into(), Some(TimelineProperty::Rotation), 0.5),
    ];
    CommandHandler::execute(&mut app, AppCommand::MoveSelectedKeyframes(0.5));

    let anim = app.animation.project.animations.get(&anim_id).unwrap();
    // 验证：只有 Rotation 移动了，Translation 还在原地
    let rot_tl = anim.timelines.iter().find(|t| t.target_id == "Root" && t.property == TimelineProperty::Rotation).unwrap();
    let pos_tl = anim.timelines.iter().find(|t| t.target_id == "Root" && t.property == TimelineProperty::Translation).unwrap();
    
    assert_eq!(rot_tl.keyframes[0].time, 1.0);
    assert_eq!(pos_tl.keyframes[0].time, 0.5);

    CommandHandler::execute(&mut app, AppCommand::Undo);
    let anim_restored = app.animation.project.animations.get(&anim_id).unwrap();
    assert_eq!(anim_restored.timelines[0].keyframes[0].time, 0.5);
}

#[test]
fn test_multi_keyframe_subset_move_undo() {
    let mut app = setup_test_context();
    let anim_id = app.animation.project.active_animation_id.clone().unwrap();

    app.animation.current_time = 0.5;
    CommandHandler::execute(&mut app, AppCommand::InsertManualKeyframe("Root".into()));
    CommandHandler::execute(&mut app, AppCommand::InsertManualKeyframe("Child".into()));

    // 子集多选：选两个骨骼的 Rotation，不选 Translation/Scale
    app.ui.selected_keyframes = vec![
        ("Root".into(), Some(TimelineProperty::Rotation), 0.5),
        ("Child".into(), Some(TimelineProperty::Rotation), 0.5),
    ];
    CommandHandler::execute(&mut app, AppCommand::MoveSelectedKeyframes(0.5));

    let anim = app.animation.project.animations.get(&anim_id).unwrap();
    for tl in &anim.timelines {
        match tl.property {
            TimelineProperty::Rotation => assert_eq!(tl.keyframes[0].time, 1.0, "Rotation 应该移动"),
            _ => assert_eq!(tl.keyframes[0].time, 0.5, "其他属性不应移动"),
        }
    }
}

#[test]
fn test_all_keyframes_move_undo() {
    let mut app = setup_test_context();
    let anim_id = app.animation.project.active_animation_id.clone().unwrap();

    app.animation.current_time = 0.5;
    CommandHandler::execute(&mut app, AppCommand::InsertManualKeyframe("Root".into()));

    // 全选：选中 Root 骨骼的所有属性
    app.ui.selected_keyframes = vec![
        ("Root".into(), Some(TimelineProperty::Rotation), 0.5),
        ("Root".into(), Some(TimelineProperty::Translation), 0.5),
        ("Root".into(), Some(TimelineProperty::Scale), 0.5),
    ];
    CommandHandler::execute(&mut app, AppCommand::MoveSelectedKeyframes(0.5));

    let anim = app.animation.project.animations.get(&anim_id).unwrap();
    assert!(anim.timelines.iter().filter(|t| !t.keyframes.is_empty()).all(|t| t.keyframes[0].time == 1.0), "全选时所有属性都应移动");

    CommandHandler::execute(&mut app, AppCommand::Undo);
    let anim_restored = app.animation.project.animations.get(&anim_id).unwrap();
    let affected_tls = anim_restored.timelines.iter().filter(|t| !t.keyframes.is_empty());
    assert!(affected_tls.count() > 0, "应该至少有被修改过的轨道存在");
    assert!(anim_restored.timelines.iter().filter(|t| !t.keyframes.is_empty()).all(|t| t.keyframes[0].time == 0.5));
}