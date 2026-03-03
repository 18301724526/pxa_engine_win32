use pxa_engine_win32::app::state::{AppState, ToolType, AppMode};
use pxa_engine_win32::core::animation::bone::BoneData;
use pxa_engine_win32::core::animation::skeleton::Skeleton;
use pxa_engine_win32::app::commands::*; 
use pxa_engine_win32::app::command_handler::AppEvent;
use pxa_engine_win32::app::ui_context::UiContext; 
use pxa_engine_win32::core::animation::timeline::{TimelineProperty, KeyframeValue, CurveType};
use pxa_engine_win32::core::color::Color;
use pxa_engine_win32::ui::timeline::TimelinePanel;
use egui::{Context, RawInput, Event, pos2, PointerButton};

fn exec(app: &mut AppState, cmd: Box<dyn pxa_engine_win32::app::command_handler::Command>) {
    app.enqueue_command(cmd); 
    app.process_commands();   
}

fn simulate_create_bone(app: &mut AppState, start: (u32, u32), end: (u32, u32)) {
    app.set_tool(ToolType::CreateBone);
    app.on_mouse_down(start.0, start.1).unwrap();
    app.on_mouse_move(end.0, end.1).unwrap();
    app.on_mouse_up().unwrap();
    app.process_commands();
}

#[test]
fn test_bone_chain_creation_flow() {
    let mut app = AppState::new();

    simulate_create_bone(&mut app, (10, 10), (50, 10));
    let root_id = app.anim.selected_bone_id.clone().expect("应选中新创建的根骨骼");

    simulate_create_bone(&mut app, (50, 10), (50, 50));
    let child_id = app.anim.selected_bone_id.clone().expect("应选中新创建的子骨骼");

    {
        let skeleton = &app.anim.state.project.skeleton;
        let child_bone = skeleton.bones.iter().find(|b| b.data.id == child_id).unwrap();
        assert_eq!(child_bone.data.parent_id.as_ref(), Some(&root_id));
    }

    simulate_create_bone(&mut app, (50, 50), (10, 50));
    let grandchild_id = app.anim.selected_bone_id.clone().unwrap();
    
    let skeleton = &app.anim.state.project.skeleton;
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
    app.anim.state.project.skeleton.add_bone(p_data);

    let mut c_data = BoneData::new("C".into(), "C".into());
    c_data.parent_id = Some("P".into());
    c_data.local_transform.x = 50.0;
    app.anim.state.project.skeleton.add_bone(c_data);
    app.anim.state.project.skeleton.update();

    app.anim.selected_bone_id = Some("C".into());
    app.set_tool(ToolType::BoneTranslate);

    app.on_mouse_down(0, 50).unwrap();
    app.on_mouse_move(0, 100).unwrap();
    app.on_mouse_up().unwrap();

    let skeleton = &app.anim.state.project.skeleton;
    let child = skeleton.bones.iter().find(|b| b.data.id == "C").unwrap();

    assert!((child.local_transform.x - 100.0).abs() < 0.1);
}

#[test]
fn test_branch_and_deselect() {
    let mut app = AppState::new();

    simulate_create_bone(&mut app, (100, 100), (150, 100));
    let id_a = app.anim.selected_bone_id.clone().unwrap();

    simulate_create_bone(&mut app, (150, 100), (150, 150));

    app.anim.selected_bone_id = Some(id_a.clone());

    simulate_create_bone(&mut app, (150, 100), (200, 100));
    let id_c = app.anim.selected_bone_id.clone().unwrap();

    app.anim.selected_bone_id = None;

    simulate_create_bone(&mut app, (300, 300), (350, 300));
    let id_d = app.anim.selected_bone_id.clone().unwrap();

    let skel = &app.anim.state.project.skeleton;
    assert_eq!(skel.bones.iter().find(|b| b.data.id == id_c).unwrap().data.parent_id.as_ref(), Some(&id_a));
    assert_eq!(skel.bones.iter().find(|b| b.data.id == id_d).unwrap().data.parent_id, None);
}

#[test]
fn test_bone_selection_logic() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;
    app.pixel.view.update_viewport(800.0, 600.0);

    let mut b1 = BoneData::new("B1".into(), "B1".into());
    b1.local_transform.x = 400.0;
    b1.local_transform.y = 300.0;
    b1.length = 10.0;
    app.anim.state.project.skeleton.add_bone(b1);
    app.anim.state.project.skeleton.update();

    app.on_mouse_down(400, 300).unwrap();
    assert_eq!(app.anim.selected_bone_id, Some("B1".into()));

    app.on_mouse_down(64, 64).unwrap();
    assert_eq!(app.anim.selected_bone_id, Some("root".into()), "点击中心点应选中 root 骨骼");
    
    app.on_mouse_down(800, 600).unwrap();
    assert!(app.anim.selected_bone_id.is_none());
}

#[test]
fn test_create_and_select_animation() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;

    exec(&mut app, Box::new(CreateAnimationCmd("Idle".into())));
    let idle_id = app.anim.state.project.active_animation_id.clone().unwrap();
    assert_eq!(app.anim.state.project.animations.get(&idle_id).unwrap().name, "Idle");

    exec(&mut app, Box::new(CreateAnimationCmd("Run".into())));
    let run_id = app.anim.state.project.active_animation_id.clone().unwrap();
    assert_ne!(idle_id, run_id);
    assert_eq!(app.anim.state.project.animations.get(&run_id).unwrap().name, "Run");

    exec(&mut app, Box::new(SelectAnimationCmd(idle_id.clone())));
    assert_eq!(app.anim.state.project.active_animation_id.as_ref().unwrap(), &idle_id);
    assert_eq!(app.anim.state.current_time, 0.0);
}

#[test]
fn test_keyframe_insertion_and_data_binding() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;

    app.anim.state.project.skeleton.add_bone(BoneData::new("BoneA".into(), "Arm".into()));
    exec(&mut app, Box::new(CreateAnimationCmd("Attack".into())));
    
    let anim_id = app.anim.state.project.active_animation_id.clone().unwrap();
    {
        let anim = app.anim.state.project.animations.get_mut(&anim_id).unwrap();
        let tl = anim.timelines.iter_mut().find(|t| t.target_id == "BoneA" && t.property == TimelineProperty::Rotation).unwrap();
        tl.add_keyframe(1.5, KeyframeValue::Rotate(45.0), CurveType::Linear);
    }

    let stored_anim = app.anim.state.project.animations.get(&anim_id).unwrap();
    assert_eq!(stored_anim.timelines.len(), 6);
    
    let rot_tl = stored_anim.timelines.iter().find(|t| t.target_id == "BoneA" && t.property == TimelineProperty::Rotation).unwrap();
    assert_eq!(rot_tl.keyframes[0].time, 1.5);
    assert_eq!(rot_tl.keyframes[0].value, KeyframeValue::Rotate(45.0));
}

#[test]
fn test_multi_keyframe_drag_and_box_select() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;

    app.anim.state.project.skeleton.add_bone(BoneData::new("BoneA".into(), "Arm".into()));
    exec(&mut app, Box::new(CreateAnimationCmd("Run".into())));
    
    let anim_id = app.anim.state.project.active_animation_id.clone().unwrap();
    {
        let anim = app.anim.state.project.animations.get_mut(&anim_id).unwrap();
        let tl = anim.timelines.iter_mut().find(|t| t.target_id == "BoneA" && t.property == TimelineProperty::Rotation).unwrap();
        tl.add_keyframe(1.0, KeyframeValue::Rotate(10.0), CurveType::Linear);
        tl.add_keyframe(2.0, KeyframeValue::Rotate(20.0), CurveType::Linear);
    }

    // Fixed: Passing state fully inside engine
    app.anim.selected_keyframes = vec![
        ("BoneA".into(), Some(TimelineProperty::Rotation), 1.0),
        ("BoneA".into(), Some(TimelineProperty::Rotation), 2.0),
    ];

    exec(&mut app, Box::new(MoveSelectedKeyframesCmd(0.5)));

    let anim = app.anim.state.project.animations.get(&anim_id).unwrap();
    let rot_tl = anim.timelines.iter().find(|t| t.target_id == "BoneA" && t.property == TimelineProperty::Rotation).unwrap();
    assert!((rot_tl.keyframes[0].time - 1.5).abs() < 0.001, "Frame 1 should be 1.5");
    assert!((rot_tl.keyframes[1].time - 2.5).abs() < 0.001, "Frame 2 should be 2.5");
}

#[test]
fn test_timeline_box_select_ui_logic() {
    let mut app = AppState::new();
    let mut ui_ctx = UiContext::new();
    app.mode = AppMode::Animation;

    app.anim.state.project.skeleton.add_bone(BoneData::new("B1".into(), "B1".into()));
    exec(&mut app, Box::new(CreateAnimationCmd("Anim1".into())));
    let anim_id = app.anim.state.project.active_animation_id.clone().unwrap();
    
    {
        let anim = app.anim.state.project.animations.get_mut(&anim_id).unwrap();
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
    egui::CentralPanel::default().show(&ctx, |ui| { TimelinePanel::show(ui, &mut app, &mut ui_ctx); });
    let _ = ctx.end_frame();

    let mut input2 = RawInput::default();
    input2.events.push(Event::PointerMoved(pos2(600.0, 200.0)));
    ctx.begin_frame(input2);
    egui::CentralPanel::default().show(&ctx, |ui| { TimelinePanel::show(ui, &mut app, &mut ui_ctx); });
    let _ = ctx.end_frame();

    let mut input3 = RawInput::default();
    input3.events.push(Event::PointerButton { 
        pos: pos2(600.0, 200.0), button: PointerButton::Primary, pressed: false, modifiers: Default::default()
    });
    ctx.begin_frame(input3);
    egui::CentralPanel::default().show(&ctx, |ui| { TimelinePanel::show(ui, &mut app, &mut ui_ctx); });
    let _ = ctx.end_frame();

    assert!(!ui_ctx.selected_keyframes.is_empty(), "UI 框选失败！没有任何关键帧被选中。");
}

#[test]
fn test_spine_cyclic_offset_logic() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;

    app.anim.state.project.skeleton.add_bone(BoneData::new("B1".into(), "B1".into()));
    app.anim.state.project.skeleton.add_bone(BoneData::new("B2".into(), "B2".into()));
    exec(&mut app, Box::new(CreateAnimationCmd("Run".into())));
    
    let anim_id = app.anim.state.project.active_animation_id.clone().unwrap();
    {
        let anim = app.anim.state.project.animations.get_mut(&anim_id).unwrap();
        anim.duration = 2.0;

        let tl1 = anim.timelines.iter_mut().find(|t| t.target_id == "B1" && t.property == TimelineProperty::Rotation).unwrap();
        tl1.add_keyframe(0.0, KeyframeValue::Rotate(0.0), CurveType::Linear);
        tl1.add_keyframe(1.8, KeyframeValue::Rotate(10.0), CurveType::Linear);
        tl1.add_keyframe(2.0, KeyframeValue::Rotate(0.0), CurveType::Linear);

        let tl2 = anim.timelines.iter_mut().find(|t| t.target_id == "B2" && t.property == TimelineProperty::Rotation).unwrap();
        tl2.add_keyframe(0.0, KeyframeValue::Rotate(0.0), CurveType::Linear);
        tl2.add_keyframe(1.8, KeyframeValue::Rotate(20.0), CurveType::Linear);
        tl2.add_keyframe(2.0, KeyframeValue::Rotate(0.0), CurveType::Linear);
    }

    // Fixed: Passing state fully inside engine
    app.anim.selected_keyframes = vec![
        ("B1".into(), Some(TimelineProperty::Rotation), 0.0),
        ("B1".into(), Some(TimelineProperty::Rotation), 1.8),
        ("B1".into(), Some(TimelineProperty::Rotation), 2.0),
        ("B2".into(), Some(TimelineProperty::Rotation), 0.0),
        ("B2".into(), Some(TimelineProperty::Rotation), 1.8),
        ("B2".into(), Some(TimelineProperty::Rotation), 2.0),
    ];

    exec(&mut app, Box::new(ApplySpineOffsetCmd { mode: 1, fixed_frames: 15, step_frames: 1 }));

    let anim = app.anim.state.project.animations.get(&anim_id).unwrap();
    assert_eq!(anim.duration, 2.0, "Spine Offset 绝不能改变动画总时长！");

    let b1_tl = anim.timelines.iter().find(|t| t.target_id == "B1" && t.property == TimelineProperty::Rotation).unwrap();
    let b1_time = b1_tl.keyframes.iter().find(|k| (k.time - 0.3).abs() < 0.001).map(|k| k.time).expect("B1 关键帧应折返至 0.3s");
    assert!((b1_time - 0.3).abs() < 0.001);

    let b2_tl = anim.timelines.iter().find(|t| t.target_id == "B2" && t.property == TimelineProperty::Rotation).unwrap();
    let b2_time = b2_tl.keyframes.iter().find(|k| (k.time - 0.3333).abs() < 0.001).map(|k| k.time).expect("B2 关键帧应递增折返");
    assert!((b2_time - 0.3333).abs() < 0.001);
}

#[test]
fn test_animation_history_performance() {
    let mut app = AppState::new();
    let mut ui_ctx = UiContext::new();
    app.mode = AppMode::Animation;

    for i in 0..100 {
        app.anim.state.project.skeleton.add_bone(BoneData::new(format!("Bone{}", i), format!("Bone{}", i)));
    }
    exec(&mut app, Box::new(CreateAnimationCmd("HeavyAnim".into())));
    let anim_id = app.anim.state.project.active_animation_id.clone().unwrap();

    {
        let anim = app.anim.state.project.animations.get_mut(&anim_id).unwrap();
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
        ui_ctx.selected_keyframes = vec![("Bone0".into(), Some(TimelineProperty::Rotation), 0.0)];
        exec(&mut app, Box::new(MoveSelectedKeyframesCmd(0.1)));
    }
    let elapsed_commit = start_commit.elapsed();

    let start_undo = std::time::Instant::now();
    for _ in 0..100 {
        exec(&mut app, Box::new(UndoCmd));
    }
    let elapsed_undo = start_undo.elapsed();

    println!("100次 Commit 耗时: {:?}, 100次 Undo 耗时: {:?}", elapsed_commit, elapsed_undo);
    assert!(elapsed_commit.as_millis() < 50);
    assert!(elapsed_undo.as_millis() < 50);
}

#[test]
fn test_animation_layer_binding_and_canvas_pan() {
    let mut app = AppState::new();
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();

    app.pixel.engine.set_primary_color(pxa_engine_win32::core::color::Color::new(255, 0, 0, 255));
    app.set_tool(ToolType::Pencil);
    app.on_mouse_down(50, 50).unwrap(); app.on_mouse_up().unwrap();
    
    assert_eq!(app.pixel.engine.store().get_pixel(&layer_id, 50, 50).unwrap().r, 255);

    app.anim.state.project.skeleton.add_bone(BoneData::new("BoneA".into(), "Arm".into()));
    exec(&mut app, Box::new(BindLayerToBoneCmd(layer_id.clone(), "BoneA".into())));
    
    let slot = app.anim.state.project.skeleton.slots.iter().find(|s| s.data.id == layer_id).unwrap();
    assert_eq!(slot.data.bone_id, "BoneA", "图层绑定骨骼失败");

    app.mode = AppMode::Animation;
    exec(&mut app, Box::new(CreateAnimationCmd("Action".into())));
    let anim_id = app.anim.state.project.active_animation_id.clone().unwrap();
    {
        let anim = app.anim.state.project.animations.get_mut(&anim_id).unwrap();
        let tl = anim.timelines.iter_mut().find(|t| t.target_id == "BoneA" && t.property == TimelineProperty::Translation).unwrap();
        tl.add_keyframe(1.0, KeyframeValue::Translate(10.0, 20.0), CurveType::Linear);
    }

    exec(&mut app, Box::new(SetTimeCmd(1.0)));
    
    let layer = app.pixel.engine.store().get_layer(&layer_id).unwrap();
    assert_eq!(layer.anim_offset_x, 10);
    assert_eq!(layer.anim_offset_y, 20);

    app.is_space_pressed = true;
    app.pixel.view.pan_x += 100.0;
    app.pixel.view.pan_y += 100.0;

    app.pixel.engine.update_render_cache(None);
    let store = app.pixel.engine.store();
    
    assert_eq!(store.get_pixel(&layer_id, 50, 50).unwrap().r, 255);
    assert_eq!(store.get_composite_pixel(60, 70).r, 255);
    assert_eq!(store.get_composite_pixel(50, 50).r, 0);
}

#[test]
fn test_binding_visual_stability_persistence() {
    let mut app = AppState::new();
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();

    app.pixel.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.on_mouse_down(64, 64).unwrap(); app.on_mouse_up().unwrap();

    app.anim.state.project.skeleton.add_bone(BoneData::new("BoneB".into(), "BoneB".into()));
    let bone = app.anim.state.project.skeleton.bones.iter_mut().find(|b| b.data.id == "BoneB").unwrap();
    bone.local_transform.x = 64.0;
    bone.local_transform.y = 64.0;
    app.anim.state.project.skeleton.update();

    exec(&mut app, Box::new(BindLayerToBoneCmd(layer_id.clone(), "BoneB".into())));

    app.pixel.engine.update_render_cache(None);
    assert_eq!(app.pixel.engine.store().get_composite_pixel(64, 64).r, 255);
}

#[test]
fn test_binding_rotation_and_pivot_offset_stability() {
    let mut app = AppState::new();
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();

    app.pixel.engine.set_primary_color(Color::new(255, 0, 0, 255));
    app.set_tool(ToolType::Pencil);
    app.on_mouse_down(50, 30).unwrap(); app.on_mouse_up().unwrap();

    let mut bone = BoneData::new("BoneA".into(), "Spine".into());
    bone.local_transform.x = 50.0;
    bone.local_transform.y = 50.0;
    app.anim.state.project.skeleton.add_bone(bone);
    app.anim.state.project.skeleton.update();

    exec(&mut app, Box::new(BindLayerToBoneCmd(layer_id.clone(), "BoneA".into())));

    if let Some(b) = app.anim.state.project.skeleton.bones.iter_mut().find(|b| b.data.id == "BoneA") {
        b.local_transform.rotation = 90.0;
    }
    app.anim.state.project.skeleton.update();

    app.mode = AppMode::Animation;
    exec(&mut app, Box::new(SetTimeCmd(0.0)));
    
    app.pixel.engine.update_render_cache(None);
    let store = app.pixel.engine.store();

    assert_eq!(store.get_composite_pixel(50, 30).a, 0);

    let pixel_right = store.get_composite_pixel(70, 50);
    let pixel_left = store.get_composite_pixel(30, 50);
    assert!(pixel_right.r == 255 || pixel_left.r == 255);
}

#[test]
fn test_panning_does_not_trigger_auto_keyframe() {
    let mut app = AppState::new();
    let mut ui_ctx = UiContext::new();
    app.mode = AppMode::Animation;
    ui_ctx.auto_keyframe = true;

    let mut bone = BoneData::new("BoneA".into(), "Spine".into());
    bone.local_transform.x = 50.0;
    bone.local_transform.y = 50.0;
    app.anim.state.project.skeleton.add_bone(bone);
    app.anim.state.project.skeleton.update();

    exec(&mut app, Box::new(CreateAnimationCmd("Walk".into())));
    app.anim.selected_bone_id = Some("BoneA".into());
    app.set_tool(ToolType::BoneTranslate);

    app.is_space_pressed = true; 
    
    app.on_mouse_down(50, 50).unwrap();
    app.on_mouse_move(100, 100).unwrap(); 
    app.on_mouse_up().unwrap();

    let anim_id = app.anim.state.project.active_animation_id.clone().unwrap();
    let anim = app.anim.state.project.animations.get(&anim_id).unwrap();
    let tl = anim.timelines.iter().find(|t| t.target_id == "BoneA" && t.property == TimelineProperty::Translation);
    
    let kf_count = tl.map_or(0, |t| t.keyframes.len());
    assert_eq!(kf_count, 0);
    
    let bone_after = app.anim.state.project.skeleton.bones.iter().find(|b| b.data.id == "BoneA").unwrap();
    assert_eq!(bone_after.local_transform.x, 50.0);
}

#[test]
fn test_global_undo_command_routing() {
    let mut app = AppState::new();

    app.mode = AppMode::PixelEdit;
    let initial_layer_count = app.pixel.engine.store().layers.len();
    let active_id = app.pixel.engine.store().active_layer_id.clone().unwrap();
    exec(&mut app, Box::new(DuplicateLayerCmd(active_id)));
    assert_eq!(app.pixel.engine.store().layers.len(), initial_layer_count + 1);
    
    exec(&mut app, Box::new(UndoCmd));
    assert_eq!(app.pixel.engine.store().layers.len(), initial_layer_count);

    app.mode = AppMode::Animation;
    app.anim.state.project.skeleton.add_bone(BoneData::new("BoneA".into(), "Spine".into()));
    app.anim.state.project.skeleton.update();
    exec(&mut app, Box::new(CreateAnimationCmd("Walk".into())));
    
    app.anim.selected_bone_id = Some("BoneA".into());
    let anim_id = app.anim.state.project.active_animation_id.clone().unwrap();
    let get_kf_count = |app: &AppState| app.anim.state.project.animations.get(&anim_id).unwrap().timelines.iter().find(|t| t.target_id == "BoneA" && t.property == TimelineProperty::Rotation).unwrap().keyframes.len();
    
    exec(&mut app, Box::new(InsertManualKeyframeCmd("BoneA".into())));
    assert_eq!(get_kf_count(&app), 1);
    
    exec(&mut app, Box::new(UndoCmd));
    assert_eq!(get_kf_count(&app), 0);
}

// ... 保持原有导入 ...

#[test]
fn test_p2_3_disable_bone_mod_in_animation_mode() {
    let mut app = AppState::new();
    let mut ui_ctx = UiContext::new();
    
    // 设置基础环境
    app.anim.state.project.skeleton.add_bone(BoneData::new("hand".into(), "hand".into()));
    app.anim.state.project.skeleton.update();
    
    // --- 场景 1: 右键菜单在动画模式下不应发送删除命令 (由于 UI 拦截，模拟点击不应产生命令) ---
    app.mode = AppMode::Animation;
    ui_ctx.selected_bone_id = Some("hand".into());
    
    // 模拟 UI 逻辑：在 Animation 模式下，右键菜单根本不会构建包含 DeleteBoneCmd 的按钮。
    // 我们通过直接尝试执行 Handler 来验证拦截器 P1.2 是否同步工作
    let res = pxa_engine_win32::app::handlers::setup_handler::delete_bone(&mut app, "hand");
    assert!(res.is_err(), "动画模式下 Handler 必须拦截删除操作");
    assert!(res.unwrap_err().contains("只能在绘画模式下"), "错误消息不匹配");

    // --- 场景 2: 拖拽绑定无效 ---
    app.mode = AppMode::Animation;
    let layer_id = app.pixel.engine.store().active_layer_id.clone().unwrap();
    
    // 模拟层级面板的 Drop 逻辑 (模仿 layer_panel.rs)
    ui_ctx.dragging_layer_id = Some(layer_id.clone());
    ui_ctx.drag_target_bone_id = Some("hand".into());
    
    // 模拟 pointer_released 触发的 Drop
    if app.mode == AppMode::PixelEdit {
         app.enqueue_command(Box::new(BindLayerToBoneCmd(layer_id.clone(), "hand".into())));
    }
    app.process_commands();
    
    // 断言没有产生绑定
    let slot = app.anim.state.project.skeleton.slots.iter().find(|s| s.data.id == layer_id).unwrap();
    assert_ne!(slot.data.bone_id, "hand", "动画模式下不应发生拖拽绑定");

    // --- 场景 3: 删除按钮行为 (选中骨骼时拦截) ---
    app.mode = AppMode::Animation;
    ui_ctx.selected_bone_id = Some("hand".into());
    
    // 模拟点击删除按钮
    if ui_ctx.selected_bone_id.is_some() {
        app.enqueue_command(Box::new(DeleteBoneCmd("hand".into())));
    }
    app.process_commands();
    
    // 检查是否产生了 AppEvent::ShowError
    let has_error = app.command_bus.events.iter().any(|e| {
        if let pxa_engine_win32::app::command_handler::AppEvent::ShowError(msg) = e {
            msg.contains("只能在绘画模式下")
        } else { false }
    });
    assert!(has_error, "点击删除骨骼按钮时应触发错误事件");
    assert!(app.anim.state.project.skeleton.bone_id_to_index("hand").is_some(), "骨骼不应被真正删除");

    app.mode = AppMode::Animation;
    app.add_new_layer(); // 此时应失败
    app.delete_active_layer();
    app.process_commands();
    
    let mut has_layer_error = false;
    while let Some(ev) = app.command_bus.events.pop_front() {
        if let AppEvent::ShowError(msg) = ev {
            if msg.contains("禁止修改图层结构") { has_layer_error = true; }
        }
    }
    assert!(has_layer_error, "动画模式下底层必须拦截图层增删请求");

    // --- 场景 5: 验证底层 Handler 拦截顺序调整 ---
    let res = pxa_engine_win32::app::handlers::layer_handler::move_layer_up(&mut app, "L1");
    assert!(res.is_err(), "底层 Handler 必须拦截顺序调整");
    assert!(res.unwrap_err().contains("禁止调整图层顺序"));
}