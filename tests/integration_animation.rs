use pxa_engine_win32::app::state::{AppState, ToolType, AppMode};
use pxa_engine_win32::core::animation::bone::BoneData;
use pxa_engine_win32::core::animation::skeleton::Skeleton;

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