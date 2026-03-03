use pxa_engine_win32::app::state::{AppState, AppMode, ToolType};
use pxa_engine_win32::app::commands::*;
use pxa_engine_win32::core::animation::bone::BoneData;
use pxa_engine_win32::core::animation::timeline::TimelineProperty;

// 辅助方法：模拟命令执行
fn exec(app: &mut AppState, cmd: Box<dyn pxa_engine_win32::app::command_handler::Command>) {
    app.enqueue_command(cmd);
    app.process_commands();
}

fn setup_anim_env() -> AppState {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;
    app.anim.state.project.skeleton.add_bone(BoneData::new("Bone1".into(), "Root".into()));
    app.anim.state.project.skeleton.update();
    exec(&mut app, Box::new(CreateAnimationCmd("TestAnim".into())));
    app.anim.selected_bone_id = Some("Bone1".into());
    app
}

#[test]
fn test_bone_transform_undo() {
    let mut app = setup_anim_env();
    
    app.set_tool(ToolType::BoneRotate);
    app.on_mouse_down(100, 100).unwrap();
    app.on_mouse_move(100, 50).unwrap();
    let angle_after_move = app.anim.state.project.skeleton.bones.iter().find(|b| b.data.id == "Bone1").unwrap().local_transform.rotation;
    assert_ne!(angle_after_move, 0.0);
    app.on_mouse_up().unwrap();

    exec(&mut app, Box::new(UndoCmd));
    assert_eq!(app.anim.state.project.skeleton.bones.iter().find(|b| b.data.id == "Bone1").unwrap().local_transform.rotation, 0.0);

    exec(&mut app, Box::new(RedoCmd));
    assert_eq!(app.anim.state.project.skeleton.bones.iter().find(|b| b.data.id == "Bone1").unwrap().local_transform.rotation, angle_after_move);
}

#[test]
fn test_keyframe_crud_undo() {
    let mut app = setup_anim_env();
    
    app.anim.state.current_time = 1.0;
    exec(&mut app, Box::new(InsertManualKeyframeCmd("Bone1".into())));
    
    let anim_id = app.anim.state.project.active_animation_id.clone().unwrap();
    {
        let anim = app.anim.state.project.animations.get(&anim_id).unwrap();
        let tl = anim.timelines.iter().find(|t| t.target_id == "Bone1" && t.property == TimelineProperty::Rotation).unwrap();
        assert!(tl.keyframes.iter().any(|k| k.time == 1.0));
    }

    exec(&mut app, Box::new(UndoCmd));
    {
        let anim_after_undo = app.anim.state.project.animations.get(&anim_id).unwrap();
        let tl_after = anim_after_undo.timelines.iter().find(|t| t.target_id == "Bone1" && t.property == TimelineProperty::Rotation).unwrap();
        assert!(tl_after.keyframes.is_empty());
    }

    app.anim.state.current_time = 0.5;
    exec(&mut app, Box::new(InsertManualKeyframeCmd("Bone1".into())));
    
    // 修复点：将选中状态真正注入到引擎的状态树中，而不是外部丢弃的变量
    app.anim.selected_keyframes = vec![("Bone1".into(), Some(TimelineProperty::Rotation), 0.5)];
    
    exec(&mut app, Box::new(MoveSelectedKeyframesCmd(0.5))); 
    
    let anim = app.anim.state.project.animations.get(&anim_id).unwrap();
    let tl = anim.timelines.iter().find(|t| t.target_id == "Bone1" && t.property == TimelineProperty::Rotation).unwrap();
    assert!(tl.keyframes.iter().any(|k| (k.time - 1.0).abs() < 0.001));

    exec(&mut app, Box::new(UndoCmd));
    let tl_final = app.anim.state.project.animations.get(&anim_id).unwrap().timelines.iter().find(|t| t.target_id == "Bone1" && t.property == TimelineProperty::Rotation).unwrap();
    assert!(tl_final.keyframes.iter().any(|k| (k.time - 0.5).abs() < 0.001));
}
#[test]
fn test_skeleton_structure_undo_redo() {
    let mut app = setup_anim_env();
    let initial_len = app.anim.state.project.skeleton.bones.len();

    assert!(app.anim.state.project.skeleton.bone_id_to_index("Bone1").is_some());

    app.mode = AppMode::PixelEdit;
    pxa_engine_win32::app::handlers::setup_handler::delete_bone(&mut app, "Bone1").unwrap();
    assert_eq!(app.anim.state.project.skeleton.bones.len(), initial_len - 1);
    assert!(app.anim.state.project.skeleton.bone_id_to_index("Bone1").is_none());

    exec(&mut app, Box::new(UndoCmd));
    assert_eq!(app.anim.state.project.skeleton.bones.len(), initial_len);
    assert!(app.anim.state.project.skeleton.bone_id_to_index("Bone1").is_some(), "撤销后内部状态必须被正确重建");

    exec(&mut app, Box::new(RedoCmd));
    assert_eq!(app.anim.state.project.skeleton.bones.len(), initial_len - 1);
    assert!(app.anim.state.project.skeleton.bone_id_to_index("Bone1").is_none());
}