use pxa_engine_win32::app::state::{AppState, AppMode, ToolType};
use pxa_engine_win32::app::commands::AppCommand;
use pxa_engine_win32::app::command_handler::CommandHandler;
use pxa_engine_win32::core::animation::bone::BoneData;
use pxa_engine_win32::core::animation::timeline::TimelineProperty;

fn setup_anim_env() -> AppState {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;
    app.animation.project.skeleton.add_bone(BoneData::new("Bone1".into(), "Root".into()));
    app.animation.project.skeleton.update();
    CommandHandler::execute(&mut app, AppCommand::CreateAnimation("TestAnim".into()));
    app.ui.selected_bone_id = Some("Bone1".into());
    app
}

#[test]
fn test_bone_transform_undo() {
    let mut app = setup_anim_env();
    
    // 1. 模拟鼠标拖拽进行旋转 (BoneRotate)
    app.set_tool(ToolType::BoneRotate);
    // 修复：将起点设为 100，终点设为 50，以产生 -50 的位移，同时避免 u32 负号错误
    app.on_mouse_down(100, 100).unwrap();
    app.on_mouse_move(100, 50).unwrap();
    let angle_after_move = app.animation.project.skeleton.bones.iter().find(|b| b.data.id == "Bone1").unwrap().local_transform.rotation;
    assert_ne!(angle_after_move, 0.0);
    app.on_mouse_up().unwrap();

    // 2. 执行撤销
    CommandHandler::execute(&mut app, AppCommand::Undo);
    assert_eq!(app.animation.project.skeleton.bones.iter().find(|b| b.data.id == "Bone1").unwrap().local_transform.rotation, 0.0, "撤销后旋转角度应归零");

    // 3. 执行重做
    CommandHandler::execute(&mut app, AppCommand::Redo);
    assert_eq!(app.animation.project.skeleton.bones.iter().find(|b| b.data.id == "Bone1").unwrap().local_transform.rotation, angle_after_move, "重做后旋转角度应恢复");
}

#[test]
fn test_keyframe_crud_undo() {
    let mut app = setup_anim_env();
    
    // 1. 添加关键帧 (通过手动 K 帧指令)
    app.animation.current_time = 1.0;
    CommandHandler::execute(&mut app, AppCommand::InsertManualKeyframe("Bone1".into()));
    
    // 修复：使用 clone() 消除对 app 的借用占用
    let anim_id = app.animation.project.active_animation_id.clone().unwrap();
    {
        let anim = app.animation.project.animations.get(&anim_id).unwrap();
        let tl = anim.timelines.iter().find(|t| t.target_id == "Bone1" && t.property == TimelineProperty::Rotation).unwrap();
        assert!(tl.keyframes.iter().any(|k| k.time == 1.0));
    }

    // 2. 撤销添加
    CommandHandler::execute(&mut app, AppCommand::Undo);
    {
        let anim_after_undo = app.animation.project.animations.get(&anim_id).unwrap();
        let tl_after = anim_after_undo.timelines.iter().find(|t| t.target_id == "Bone1" && t.property == TimelineProperty::Rotation).unwrap();
        assert!(tl_after.keyframes.is_empty(), "撤销后关键帧列表应清空，但轨道应保留");
    }

    // 3. 移动关键帧并撤销
    app.animation.current_time = 0.5;
    // 修复：重新获取 anim_id 以确保借用检查通过
    CommandHandler::execute(&mut app, AppCommand::InsertManualKeyframe("Bone1".into()));
    app.ui.selected_keyframes = vec![("Bone1".into(), Some(TimelineProperty::Rotation), 0.5)];
    CommandHandler::execute(&mut app, AppCommand::MoveSelectedKeyframes(0.5)); // 移到 1.0
    
    assert_eq!(app.ui.selected_keyframes[0].2, 1.0);
    CommandHandler::execute(&mut app, AppCommand::Undo);
    let tl_final = app.animation.project.animations.get(&anim_id).unwrap().timelines.iter().find(|t| t.target_id == "Bone1" && t.property == TimelineProperty::Rotation).unwrap();
    assert!(tl_final.keyframes.iter().any(|k| k.time == 0.5), "移动撤销后位置应回退");
}

#[test]
fn test_composite_keyframe_move_undo() {
    let mut app = setup_anim_env();
    app.animation.project.skeleton.add_bone(BoneData::new("Bone2".into(), "Child".into()));
    
    // 在两个骨骼上分别创建关键帧
    app.animation.current_time = 0.0;
    CommandHandler::execute(&mut app, AppCommand::InsertManualKeyframe("Bone1".into()));
    CommandHandler::execute(&mut app, AppCommand::InsertManualKeyframe("Bone2".into()));
    
    // 框选两个关键帧并移动
    app.ui.selected_keyframes = vec![
        ("Bone1".into(), Some(TimelineProperty::Rotation), 0.0),
        ("Bone2".into(), Some(TimelineProperty::Rotation), 0.0),
    ];
    CommandHandler::execute(&mut app, AppCommand::MoveSelectedKeyframes(1.0));
    
    // 验证撤销是否作为整体（Composite Patch）执行
    CommandHandler::execute(&mut app, AppCommand::Undo);
    let anim_id = app.animation.project.active_animation_id.clone().unwrap();
    let anim = app.animation.project.animations.get(&anim_id).unwrap();
    assert!(anim.timelines.iter().filter(|tl| !tl.keyframes.is_empty() && (tl.target_id == "Bone1" || tl.target_id == "Bone2")).all(|tl| tl.keyframes[0].time == 0.0), "复合移动撤销必须同时作用于所有选中的 Timeline");
}