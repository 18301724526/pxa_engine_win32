use pxa_engine_win32::app::state::{AppState, ToolType, AppMode};
use pxa_engine_win32::core::animation::bone::BoneData;
use pxa_engine_win32::core::animation::skeleton::Skeleton;
use pxa_engine_win32::app::commands::AppCommand;
use pxa_engine_win32::app::command_handler::CommandHandler;
use pxa_engine_win32::core::animation::timeline::{TimelineProperty, KeyframeValue, CurveType, Timeline};
use pxa_engine_win32::ui::timeline::TimelinePanel;
use egui::{Context, RawInput, Event, pos2, PointerButton};

fn simulate_create_bone(app: &mut AppState, start: (u32, u32), end: (u32, u32)) {
    app.set_tool(ToolType::CreateBone);
    app.on_mouse_down(start.0, start.1).unwrap();
    app.on_mouse_move(end.0, end.1).unwrap();
    app.on_mouse_up().unwrap();
}

#[test]
fn test_bone_chain_creation_flow() {
    let mut app = AppState::new();

    simulate_create_bone(&mut app, (10, 10), (50, 10));
    let root_id = app.ui.selected_bone_id.clone().expect("应选中新创建的根骨骼");

    simulate_create_bone(&mut app, (50, 10), (50, 50));
    let child_id = app.ui.selected_bone_id.clone().expect("应选中新创建的子骨骼");

    {
        let skeleton = &app.animation.project.skeleton;
        let child_bone = skeleton.bones.iter().find(|b| b.data.id == child_id).unwrap();
        assert_eq!(child_bone.data.parent_id.as_ref(), Some(&root_id));
    }

    simulate_create_bone(&mut app, (50, 50), (10, 50));
    let grandchild_id = app.ui.selected_bone_id.clone().unwrap();
    
    let skeleton = &app.animation.project.skeleton;
    let grandchild_bone = skeleton.bones.iter().find(|b| b.data.id == grandchild_id).unwrap();
    assert_eq!(grandchild_bone.data.parent_id.as_ref(), Some(&child_id));
}

#[test]
fn test_transform_math_accuracy() {
    let mut skel = Skeleton::new();

    let mut p_data = BoneData::new("P".into(), "Parent".into());
    p_data.local_transform.x = 100.0;
    p_data.local_transform.y = 100.0;
    p_data.local_transform.rotation = 90.0;
    skel.add_bone(p_data);

    let mut c_data = BoneData::new("C".into(), "Child".into());
    c_data.parent_id = Some("P".into());
    c_data.local_transform.x = 50.0;
    skel.add_bone(c_data);

    skel.update();

    let (wx, wy) = skel.get_bone_world_position("C").unwrap();
    assert!((wx - 100.0).abs() < 0.001);
    assert!((wy - 150.0).abs() < 0.001);
}

#[test]
fn test_mouse_translate_with_rotated_parent() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;

    let mut p_data = BoneData::new("P".into(), "P".into());
    p_data.local_transform.rotation = 90.0;
    app.animation.project.skeleton.add_bone(p_data);

    let mut c_data = BoneData::new("C".into(), "C".into());
    c_data.parent_id = Some("P".into());
    c_data.local_transform.x = 50.0;
    app.animation.project.skeleton.add_bone(c_data);
    app.animation.project.skeleton.update();

    app.ui.selected_bone_id = Some("C".into());
    app.set_tool(ToolType::BoneTranslate);

    app.on_mouse_down(0, 50).unwrap();
    app.on_mouse_move(0, 100).unwrap();
    app.on_mouse_up().unwrap();

    let skeleton = &app.animation.project.skeleton;
    let child = skeleton.bones.iter().find(|b| b.data.id == "C").unwrap();

    assert!((child.local_transform.x - 100.0).abs() < 0.1);
}

#[test]
fn test_branch_and_deselect() {
    let mut app = AppState::new();

    simulate_create_bone(&mut app, (100, 100), (150, 100));
    let id_a = app.ui.selected_bone_id.clone().unwrap();

    simulate_create_bone(&mut app, (150, 100), (150, 150));

    app.ui.selected_bone_id = Some(id_a.clone());

    simulate_create_bone(&mut app, (150, 100), (200, 100));
    let id_c = app.ui.selected_bone_id.clone().unwrap();

    app.ui.selected_bone_id = None;

    simulate_create_bone(&mut app, (300, 300), (350, 300));
    let id_d = app.ui.selected_bone_id.clone().unwrap();

    let skel = &app.animation.project.skeleton;
    assert_eq!(skel.bones.iter().find(|b| b.data.id == id_c).unwrap().data.parent_id.as_ref(), Some(&id_a));
    assert_eq!(skel.bones.iter().find(|b| b.data.id == id_d).unwrap().data.parent_id, None);
}

#[test]
fn test_bone_selection_logic() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;
    app.view.update_viewport(800.0, 600.0);

    let mut b1 = BoneData::new("B1".into(), "B1".into());
    b1.local_transform.x = 400.0;
    b1.local_transform.y = 300.0;
    b1.length = 10.0;
    app.animation.project.skeleton.add_bone(b1);
    app.animation.project.skeleton.update();

    app.on_mouse_down(400, 300).unwrap();
    assert_eq!(app.ui.selected_bone_id, Some("B1".into()));

    app.on_mouse_down(0, 0).unwrap();
    assert!(app.ui.selected_bone_id.is_none());
}

#[test]
fn test_create_and_select_animation() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;

    CommandHandler::execute(&mut app, AppCommand::CreateAnimation("Idle".into()));
    let idle_id = app.animation.project.active_animation_id.clone().unwrap();
    assert_eq!(app.animation.project.animations.get(&idle_id).unwrap().name, "Idle");

    CommandHandler::execute(&mut app, AppCommand::CreateAnimation("Run".into()));
    let run_id = app.animation.project.active_animation_id.clone().unwrap();
    assert_ne!(idle_id, run_id);
    assert_eq!(app.animation.project.animations.get(&run_id).unwrap().name, "Run");

    CommandHandler::execute(&mut app, AppCommand::SelectAnimation(idle_id.clone()));
    assert_eq!(app.animation.project.active_animation_id.unwrap(), idle_id);
    assert_eq!(app.animation.current_time, 0.0);
}

#[test]
fn test_keyframe_insertion_and_data_binding() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;

    // 先加骨骼，确保物理初始化逻辑生效
    app.animation.project.skeleton.add_bone(BoneData::new("BoneA".into(), "Arm".into()));
    CommandHandler::execute(&mut app, AppCommand::CreateAnimation("Attack".into()));
    
    let anim_id = app.animation.project.active_animation_id.clone().unwrap();
    {
        let anim = app.animation.project.animations.get_mut(&anim_id).unwrap();
        // 查找预初始化的轨道而不是 push
        let tl = anim.timelines.iter_mut().find(|t| t.target_id == "BoneA" && t.property == TimelineProperty::Rotation).unwrap();
        tl.add_keyframe(1.5, KeyframeValue::Rotate(45.0), CurveType::Linear);
    }

    let stored_anim = app.animation.project.animations.get(&anim_id).unwrap();
    // 一个骨骼物理初始化产生 3 条轨道（Translation, Rotation, Scale）
    assert_eq!(stored_anim.timelines.len(), 3);
    
    let rot_tl = stored_anim.timelines.iter().find(|t| t.property == TimelineProperty::Rotation).unwrap();
    assert_eq!(rot_tl.keyframes[0].time, 1.5);
    assert_eq!(rot_tl.keyframes[0].value, KeyframeValue::Rotate(45.0));
}

#[test]
fn test_multi_keyframe_drag_and_box_select() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;

    app.animation.project.skeleton.add_bone(BoneData::new("BoneA".into(), "Arm".into()));
    CommandHandler::execute(&mut app, AppCommand::CreateAnimation("Run".into()));
    
    let anim_id = app.animation.project.active_animation_id.clone().unwrap();
    {
        let anim = app.animation.project.animations.get_mut(&anim_id).unwrap();
        // 查找物理初始化的轨道
        let tl = anim.timelines.iter_mut().find(|t| t.target_id == "BoneA" && t.property == TimelineProperty::Rotation).unwrap();
        tl.add_keyframe(1.0, KeyframeValue::Rotate(10.0), CurveType::Linear);
        tl.add_keyframe(2.0, KeyframeValue::Rotate(20.0), CurveType::Linear);
    }

    app.ui.selected_keyframes = vec![
        ("BoneA".into(), Some(TimelineProperty::Rotation), 1.0),
        ("BoneA".into(), Some(TimelineProperty::Rotation), 2.0),
    ];

    CommandHandler::execute(&mut app, AppCommand::MoveSelectedKeyframes(0.5));

    let anim = app.animation.project.animations.get(&anim_id).unwrap();
    let rot_tl = anim.timelines.iter().find(|t| t.property == TimelineProperty::Rotation).unwrap();
    assert!((rot_tl.keyframes[0].time - 1.5).abs() < 0.001, "Frame 1 should be 1.5");
    assert!((rot_tl.keyframes[1].time - 2.5).abs() < 0.001, "Frame 2 should be 2.5");
}

#[test]
fn test_timeline_box_select_ui_logic() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;

    app.animation.project.skeleton.add_bone(BoneData::new("B1".into(), "B1".into()));
    CommandHandler::execute(&mut app, AppCommand::CreateAnimation("Anim1".into()));
    let anim_id = app.animation.project.active_animation_id.clone().unwrap();
    
    {
        let anim = app.animation.project.animations.get_mut(&anim_id).unwrap();
        let tl = anim.timelines.iter_mut().find(|t| t.target_id == "B1" && t.property == TimelineProperty::Rotation).unwrap();
        tl.add_keyframe(1.0, KeyframeValue::Rotate(90.0), CurveType::Linear);
    }

    let ctx = Context::default();

    let mut input1 = RawInput::default();
    input1.events.push(Event::PointerMoved(pos2(200.0, 105.0)));
    input1.events.push(Event::PointerButton { 
        pos: pos2(200.0, 105.0), button: PointerButton::Primary, pressed: true, modifiers: Default::default()
    });
    ctx.begin_frame(input1);
    egui::CentralPanel::default().show(&ctx, |ui| { TimelinePanel::show(ui, &mut app); });
    let _ = ctx.end_frame();

    let mut input2 = RawInput::default();
    input2.events.push(Event::PointerMoved(pos2(600.0, 200.0)));
    ctx.begin_frame(input2);
    egui::CentralPanel::default().show(&ctx, |ui| { TimelinePanel::show(ui, &mut app); });
    let _ = ctx.end_frame();

    let mut input3 = RawInput::default();
    input3.events.push(Event::PointerButton { 
        pos: pos2(600.0, 200.0), button: PointerButton::Primary, pressed: false, modifiers: Default::default()
    });
    ctx.begin_frame(input3);
    egui::CentralPanel::default().show(&ctx, |ui| { TimelinePanel::show(ui, &mut app); });
    let _ = ctx.end_frame();

    assert!(!app.ui.selected_keyframes.is_empty(), "UI 框选失败！没有任何关键帧被选中。");
}

#[test]
fn test_spine_cyclic_offset_logic() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;

    app.animation.project.skeleton.add_bone(BoneData::new("B1".into(), "B1".into()));
    app.animation.project.skeleton.add_bone(BoneData::new("B2".into(), "B2".into()));
    CommandHandler::execute(&mut app, AppCommand::CreateAnimation("Run".into()));
    
    let anim_id = app.animation.project.active_animation_id.clone().unwrap();
    {
        let anim = app.animation.project.animations.get_mut(&anim_id).unwrap();
        anim.duration = 2.0;

        let tl1 = anim.timelines.iter_mut().find(|t| t.target_id == "B1" && t.property == TimelineProperty::Rotation).unwrap();
        tl1.add_keyframe(1.8, KeyframeValue::Rotate(10.0), CurveType::Linear);

        let tl2 = anim.timelines.iter_mut().find(|t| t.target_id == "B2" && t.property == TimelineProperty::Rotation).unwrap();
        tl2.add_keyframe(1.8, KeyframeValue::Rotate(20.0), CurveType::Linear);
    }

    app.ui.selected_keyframes = vec![
        ("B1".into(), Some(TimelineProperty::Rotation), 1.8),
        ("B2".into(), Some(TimelineProperty::Rotation), 1.8),
    ];

    CommandHandler::execute(&mut app, AppCommand::ApplySpineOffset { mode: 1, fixed_frames: 15, step_frames: 1 });

    let anim = app.animation.project.animations.get(&anim_id).unwrap();

    assert_eq!(anim.duration, 2.0, "Spine Offset 绝不能改变动画总时长！");

    let b1_tl = anim.timelines.iter().find(|t| t.target_id == "B1" && t.property == TimelineProperty::Rotation).unwrap();
    // 补帧逻辑会向 0.0s 插入采样帧
    let b1_time = b1_tl.keyframes.iter().find(|k| (k.time - 0.3).abs() < 0.001).map(|k| k.time).expect("B1 关键帧应折返至 0.3s");
    assert!((b1_time - 0.3).abs() < 0.001);

    let b2_tl = anim.timelines.iter().find(|t| t.target_id == "B2" && t.property == TimelineProperty::Rotation).unwrap();
    let b2_time = b2_tl.keyframes.iter().find(|k| (k.time - 0.3333).abs() < 0.001).map(|k| k.time).expect("B2 关键帧应递增折返");
    assert!((b2_time - 0.3333).abs() < 0.001);
}

#[test]
fn test_animation_history_performance() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;

    for i in 0..100 {
        app.animation.project.skeleton.add_bone(BoneData::new(format!("Bone{}", i), format!("Bone{}", i)));
    }
    CommandHandler::execute(&mut app, AppCommand::CreateAnimation("HeavyAnim".into()));
    let anim_id = app.animation.project.active_animation_id.clone().unwrap();

    {
        let anim = app.animation.project.animations.get_mut(&anim_id).unwrap();
        for i in 0..100 {
            let bone_id = format!("Bone{}", i);
            let tl = anim.timelines.iter_mut().find(|t| t.target_id == bone_id && t.property == TimelineProperty::Rotation).unwrap();
            for f in 0..100 {
                tl.add_keyframe(f as f32 * 0.1, KeyframeValue::Rotate(f as f32), CurveType::Linear);
            }
        }
    }

    let start_commit = std::time::Instant::now();
    for _ in 0..100 {
        app.ui.selected_keyframes = vec![("Bone0".into(), Some(TimelineProperty::Rotation), 0.0)];
        CommandHandler::execute(&mut app, AppCommand::MoveSelectedKeyframes(0.1));
    }
    let elapsed_commit = start_commit.elapsed();

    let start_undo = std::time::Instant::now();
    for _ in 0..100 {
        CommandHandler::execute(&mut app, AppCommand::Undo);
    }
    let elapsed_undo = start_undo.elapsed();

    println!("100次 Commit 耗时: {:?}, 100次 Undo 耗时: {:?}", elapsed_commit, elapsed_undo);

    assert!(elapsed_commit.as_millis() < 50, "History commit 性能严重不达标!");
    assert!(elapsed_undo.as_millis() < 50, "History undo 性能严重不达标!");
}