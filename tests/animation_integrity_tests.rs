use pxa_engine_win32::app::state::{AppState, AppMode, ToolType};
use pxa_engine_win32::app::commands::*;
use pxa_engine_win32::app::ui_context::UiContext; // 引入独立 UI 上下文
use pxa_engine_win32::core::animation::bone::BoneData;
use pxa_engine_win32::core::animation::timeline::TimelineProperty;

fn exec(app: &mut AppState, cmd: Box<dyn pxa_engine_win32::app::command_handler::Command>) {
    app.enqueue_command(cmd);
    app.process_commands();
}

#[test]
fn test_skeleton_bone_hierarchy_integrity() {
    let mut app = AppState::new();
    
    // 1. 选中 root
    app.anim.selected_bone_id = Some("root".into());

    // 2. 模拟使用工具生成新骨骼
    app.set_tool(ToolType::CreateBone);
    app.on_mouse_down(0, 0).unwrap();
    app.on_mouse_move(10, 10).unwrap();
    app.on_mouse_up().unwrap();
    app.process_commands();

    // 3. 验证层级完整性
    let skel = &app.anim.state.project.skeleton;
    
    // --- 调试信息：如果数量不符，打印出到底有哪些骨骼 ---
    if skel.bones.len() != 2 {
        println!("\n=== 调试：发现 {} 个骨骼 ===", skel.bones.len());
        for (i, bone) in skel.bones.iter().enumerate() {
            println!("索引 {}: ID = '{}', 名称 = '{}'", i, bone.data.id, bone.data.name);
        }
        println!("==========================\n");
    }
    
    assert_eq!(skel.bones.len(), 2, "骨骼总数必须为 2（root + 新骨骼）");
    assert_eq!(skel.bones[1].data.parent_id, Some("root".into()));
}

#[test]
fn test_keyframe_deletion_integrity() {
    let mut app = AppState::new();
    let mut ui_ctx = UiContext::new();
    app.mode = AppMode::Animation;

    app.anim.state.project.skeleton.add_bone(BoneData::new("Bone1".into(), "Bone1".into()));
    exec(&mut app, Box::new(CreateAnimationCmd("Anim".into())));
    
    exec(&mut app, Box::new(InsertManualKeyframeCmd("Bone1".into())));
    
    // 模拟 UI 选中关键帧操作
    ui_ctx.selected_keyframes = vec![
        ("Bone1".into(), Some(TimelineProperty::Rotation), 0.0)
    ];

    exec(&mut app, Box::new(DeleteKeyframeCmd("Bone1".into(), Some(TimelineProperty::Rotation), 0.0)));

    let anim_id = app.anim.state.project.active_animation_id.as_ref().unwrap();
    let anim = app.anim.state.project.animations.get(anim_id).unwrap();
    let tl = anim.timelines.iter().find(|t| t.target_id == "Bone1" && t.property == TimelineProperty::Rotation).unwrap();
    
    assert!(tl.keyframes.is_empty(), "关键帧删除失败");
}