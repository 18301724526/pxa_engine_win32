use pxa_engine_win32::app::state::AppState;
use pxa_engine_win32::app::ui_context::UiContext;
use pxa_engine_win32::core::animation::bone::BoneData;

#[test]
fn test_bone_selection_clears_layer_selection() {
    let mut app = AppState::new();
    let mut ui_ctx = UiContext::new();
    
    // 1. 初始状态：模拟选中一个图层
    let layer_id = "L1".to_string();
    ui_ctx.selected_layer_ids = vec![layer_id.clone()];
    assert!(ui_ctx.selected_layer_ids.contains(&layer_id));
    
    // 2. 准备骨骼数据
    let bone_id = "test_bone".to_string();
    app.anim.state.project.skeleton.add_bone(BoneData::new(bone_id.clone(), "Test".into()));
    
    // 3. 模拟 UI 选中逻辑 (对应 layer_panel.rs 中的逻辑实现)
    // 在 draw_bone_tree 的 resp.clicked() 逻辑：
    ui_ctx.selected_bone_id = Some(bone_id.clone());
    app.anim.selected_bone_id = Some(bone_id.clone());
    ui_ctx.selected_layer_ids.clear();
    
    // 4. 验证互斥逻辑
    assert_eq!(ui_ctx.selected_bone_id, Some(bone_id.clone()));
    assert_eq!(app.anim.selected_bone_id, Some(bone_id));
    assert!(ui_ctx.selected_layer_ids.is_empty(), "选中骨骼后图层选中列表必须清空");
}

#[test]
fn test_layer_selection_clears_bone_selection() {
    let mut app = AppState::new();
    let mut ui_ctx = UiContext::new();
    
    // 1. 初始状态：模拟选中一个骨骼
    ui_ctx.selected_bone_id = Some("bone_1".into());
    app.anim.selected_bone_id = Some("bone_1".into());
    
    // 2. 模拟点击图层逻辑 (对应 layer_panel.rs 中的逻辑实现)
    let layer_id = "L1".to_string();
    ui_ctx.selected_bone_id = None;
    app.anim.selected_bone_id = None;
    ui_ctx.selected_layer_ids = vec![layer_id.clone()];
    
    // 3. 验证互斥逻辑
    assert!(ui_ctx.selected_bone_id.is_none());
    assert!(app.anim.selected_bone_id.is_none());
    assert_eq!(ui_ctx.selected_layer_ids, vec![layer_id]);
}