use pxa_engine_win32::app::state::{AppState, AppMode};
use pxa_engine_win32::app::ui_context::UiContext;
use pxa_engine_win32::core::animation::bone::BoneData;
use pxa_engine_win32::app::commands::DeleteBoneCmd;

#[test]
fn test_deletion_redirect_logic_bone_priority() {
    let mut app = AppState::new();
    let mut ui_ctx = UiContext::new();
    
    // 1. 准备环境：添加骨骼并模拟选中
    let bone_id = "target_bone".to_string();
    app.anim.state.project.skeleton.add_bone(BoneData::new(bone_id.clone(), "TestBone".into()));
    ui_ctx.selected_bone_id = Some(bone_id.clone());
    app.mode = AppMode::PixelEdit; // 在绘画模式下允许删除骨骼

    // 2. 模拟按钮点击逻辑 (对应 layer_panel.rs 逻辑)
    if let Some(id) = &ui_ctx.selected_bone_id {
        app.enqueue_command(Box::new(DeleteBoneCmd(id.clone())));
    } else {
        app.delete_active_layer();
    }
    
    app.process_commands();

    // 3. 验证骨骼是否被移除
    assert!(app.anim.state.project.skeleton.bone_id_to_index(&bone_id).is_none(), "选中骨骼时应优先删除骨骼");
}

#[test]
fn test_deletion_redirect_logic_fallback_to_layer() {
    let mut app = AppState::new();
    let mut ui_ctx = UiContext::new();
    
    // 1. 初始状态：无骨骼选中，有多个图层
    app.add_new_layer();
    let layer_count_before = app.pixel.engine.store().layers.len();
    ui_ctx.selected_bone_id = None;
    
    // 2. 模拟按钮点击逻辑
    if let Some(id) = &ui_ctx.selected_bone_id {
        app.enqueue_command(Box::new(DeleteBoneCmd(id.clone())));
    } else {
        app.delete_active_layer();
    }
    
    app.process_commands();

    // 3. 验证图层是否被减少
    assert_eq!(app.pixel.engine.store().layers.len(), layer_count_before - 1, "无骨骼选中时应删除图层");
}