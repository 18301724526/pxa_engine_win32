use pxa_engine_win32::app::state::{AppState, ToolType, AppMode};
use pxa_engine_win32::app::commands::*;
use pxa_engine_win32::app::ui_context::UiContext;
use pxa_engine_win32::core::animation::timeline::TimelineProperty;
use pxa_engine_win32::tools::create_bone::CreateBoneTool;
use pxa_engine_win32::app::handlers::setup_handler;

fn exec(app: &mut AppState, cmd: Box<dyn pxa_engine_win32::app::command_handler::Command>) {
    app.enqueue_command(cmd);
    app.process_commands();
}

fn setup_anim_test() -> AppState {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;
    exec(&mut app, Box::new(CreateAnimationCmd("Walk".into())));
    app
}

#[test]
fn test_bone_creation_math_and_preview() {
    let mut app = AppState::new();
    app.set_tool(ToolType::CreateBone);
    
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(40, 50).unwrap();

    {
        let tool = app.pixel.engine.tool_manager().tools.get(&ToolType::CreateBone).unwrap();
        let bone_tool = tool.as_any().downcast_ref::<CreateBoneTool>().unwrap();
        assert_eq!(bone_tool.start_pos, Some((10.0, 10.0)));
        assert_eq!(bone_tool.preview_end, Some((40.0, 50.0)), "拖拽中必须保持 preview_end 用于渲染虚线");
    }

    app.on_mouse_up().unwrap();
    app.process_commands();
    let skel = &app.anim.state.project.skeleton;

    assert_eq!(skel.bones.len(), 2, "Bone should be added to the skeleton (root + new)");
    let bone = &skel.bones[1];

    assert!((bone.data.length - 50.0).abs() < 0.001, "骨骼长度计算必须精确符合拖拽距离");
    let expected_angle = (40.0f32).atan2(30.0).to_degrees();
    assert!((bone.data.local_transform.rotation - expected_angle).abs() < 0.001, "旋转角度应与拖拽向量完全一致");
    assert_eq!(app.anim.selected_bone_id, Some(bone.data.id.clone()), "Newly created bone should be selected");
}

#[test]
fn test_bone_tool_availability_in_modes() {
    let mut app = AppState::new();
    app.mode = AppMode::Animation;
    app.set_tool(ToolType::CreateBone);
    app.on_mouse_down(10, 10).unwrap();
    app.on_mouse_move(40, 50).unwrap();
    app.on_mouse_up().unwrap();
    app.process_commands();

    // 由于拦截器生效，骨骼不应被添加，依然只含有默认的 1 个 root
    assert_eq!(app.anim.state.project.skeleton.bones.len(), 1, "动画模式下不应创建骨骼");

    let event = app.command_bus.events.pop_front().expect("拦截器应该产生错误提示事件");
    assert!(matches!(event, pxa_engine_win32::app::command_handler::AppEvent::ShowError(_)));
}

#[test]
fn test_setup_guard_in_animation_mode() {
    let mut app = setup_anim_test();
    app.anim.state.project.skeleton.add_bone(pxa_engine_win32::core::animation::bone::BoneData::new("TestGuard".into(), "Root".into()));
    app.anim.state.project.skeleton.update();
    let initial_count = app.anim.state.project.skeleton.bones.len();

    let res = setup_handler::delete_bone(&mut app, "TestGuard");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), "只能在绘画模式下修改骨骼结构");
    assert_eq!(app.anim.state.project.skeleton.bones.len(), initial_count, "拦截后骨骼不应被删除");
}

#[test]
fn test_animation_keyframe_insertion_modes() {
    let mut app = setup_anim_test();
    let mut ui_ctx = UiContext::new(); // 引入独立的 UI Context
    let bone_data = pxa_engine_win32::core::animation::bone::BoneData::new("Bone1".into(), "Arm".into());
    app.anim.state.project.skeleton.add_bone(bone_data);
    app.anim.state.project.skeleton.update();
    
    app.anim.selected_bone_id = Some("Bone1".into()); // 修复：移至 app.anim
    app.anim.state.current_time = 1.0;

    ui_ctx.auto_keyframe = true; // 修复：移至 ui_ctx
    app.anim.state.auto_key_bone("Bone1", TimelineProperty::Rotation);
    
    {
        let active_id = app.anim.state.project.active_animation_id.as_ref().unwrap();
        let anim = app.anim.state.project.animations.get(active_id).unwrap();
        let rot_tl = anim.timelines.iter().find(|t| t.target_id == "Bone1" && t.property == TimelineProperty::Rotation).unwrap();
        assert_eq!(rot_tl.keyframes.len(), 1, "启用自动 K 帧时，属性改变应立即插入关键帧");
    }

    app.anim.state.current_time = 2.0;
    exec(&mut app, Box::new(InsertManualKeyframeCmd("Bone1".into())));

    {
        let active_id = app.anim.state.project.active_animation_id.as_ref().unwrap();
        let anim = app.anim.state.project.animations.get(active_id).unwrap();
        let pos_tl = anim.timelines.iter().find(|t| t.target_id == "Bone1" && t.property == TimelineProperty::Translation).unwrap();
        assert_eq!(pos_tl.keyframes.len(), 1, "手动 K 帧指令应强制为未改变的属性创建关键帧");
    }
}

#[test]
fn test_parent_child_world_transform_update() {
    let mut p_data = pxa_engine_win32::core::animation::bone::BoneData::new("P".into(), "Parent".into());
    p_data.local_transform.x = 100.0;
    p_data.local_transform.y = 100.0;
    p_data.length = 50.0;
    let mut skel = pxa_engine_win32::core::animation::skeleton::Skeleton::new();
    skel.add_bone(p_data);

    let mut c_data = pxa_engine_win32::core::animation::bone::BoneData::new("C".into(), "Child".into());
    c_data.parent_id = Some("P".into());
    c_data.local_transform.x = 50.0;
    c_data.local_transform.y = 0.0;
    skel.add_bone(c_data);
    
    skel.update();
    
    let (cx, cy) = skel.get_bone_world_position("C").unwrap();
    assert!((cx - 150.0).abs() < 0.001);
    assert!((cy - 100.0).abs() < 0.001);

    if let Some(p) = skel.bones.iter_mut().find(|b| b.data.id == "P") {
        p.local_transform.rotation = 90.0;
    }
    skel.update();

    let (new_cx, new_cy) = skel.get_bone_world_position("C").unwrap();
    assert!((new_cx - 100.0).abs() < 0.001);
    assert!((new_cy - 150.0).abs() < 0.001);
}
#[test]
fn test_delete_bone_cleans_timelines() {
    let mut app = setup_anim_test();
    let bone_id = "BoneToClean".to_string();
    app.anim.state.project.skeleton.add_bone(pxa_engine_win32::core::animation::bone::BoneData::new(bone_id.clone(), bone_id.clone()));
    
    // 1. 为骨骼添加关键帧，产生 Timeline
    app.anim.state.current_time = 0.5;
    exec(&mut app, Box::new(InsertManualKeyframeCmd(bone_id.clone())));
    
    let anim_id = app.anim.state.project.active_animation_id.clone().unwrap();
    assert!(app.anim.state.project.animations.get(&anim_id).unwrap().timelines.iter().any(|t| t.target_id == bone_id));

    // 2. 切换模式并删除骨骼
    app.mode = AppMode::PixelEdit;
    setup_handler::delete_bone(&mut app, &bone_id).unwrap();

    // 3. 验证骨骼已删，且所有 Timeline 已被清理
    assert!(app.anim.state.project.skeleton.bone_id_to_index(&bone_id).is_none());
    let anim = app.anim.state.project.animations.get(&anim_id).unwrap();
    let timeline_exists = anim.timelines.iter().any(|t| t.target_id == bone_id);
    assert!(!timeline_exists, "删除骨骼后对应的 Timeline 必须被清理");
}