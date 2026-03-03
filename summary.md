# 项目摘要

## 项目结构 (拆分层级: 2)

## 完整项目结构

```text
Cargo.toml
README.md
locales/
    en.yml
    zh-CN.yml
src/
    animation/
        controller.rs
        history.rs
        mod.rs
        project.rs
        state.rs
    app/
        command_handler.rs
        commands/
            anim.rs
            layer.rs
            mod.rs
            palette.rs
            system.rs
            tool.rs
        context.rs
        engine.rs
        error.rs
        events.rs
        handlers/
            anim_handler.rs
            layer_handler.rs
            mod.rs
            setup_handler.rs
        input_handler.rs
        io_service.rs
        layer_service.rs
        mod.rs
        session.rs
        shortcut_manager.rs
        state.rs
        tool_manager.rs
        ui_context.rs
        ui_state.rs
        view_state.rs
    core/
        animation/
            bone.rs
            mod.rs
            skeleton/
                data.rs
                mod.rs
                ops.rs
            slot.rs
            storage.rs
            tests.rs
            tests_z_order.rs
            timeline.rs
            transform.rs
        blend_mode.rs
        color/
            tests.rs
        color.rs
        error.rs
        id.rs
        id_gen.rs
        layer/
            tests.rs
        layer.rs
        mod.rs
        palette.rs
        path.rs
        selection.rs
        storage.rs
        store/
            tests.rs
        store.rs
        symmetry.rs
    format/
        block.rs
        error.rs
        header.rs
        hex_palette.rs
        mod.rs
        payload.rs
        stream.rs
    history/
        manager/
            tests.rs
        manager.rs
        mod.rs
        patch/
            tests.rs
        patch.rs
    lib.rs
    main.rs
    render/
        anim_compositor.rs
        blend.rs
        compositor/
            tests.rs
        compositor.rs
        mod.rs
        texture_manager.rs
    tools/
        bucket.rs
        create_bone/
            tests.rs
        create_bone.rs
        ellipse_select.rs
        eyedropper/
            tests.rs
        eyedropper.rs
        geometry/
            tests.rs
        geometry.rs
        mod.rs
        move_tool.rs
        pen.rs
        pencil.rs
        rect_select.rs
        tool_trait.rs
        transform.rs
    ui/
        bone_transform_panel.rs
        cursor_overlay.rs
        framework.rs
        gui.rs
        layer_panel.rs
        menu_file.rs
        menu_image.rs
        mod.rs
        palette_panel.rs
        symmetry_panel.rs
        timeline/
            curve_editor.rs
            dopesheet.rs
            mod.rs
            offset_modal.rs
            toolbar.rs
        title_bar.rs
        toolbar_anim.rs
        toolbar_pixel.rs
        window_controls.rs
tests/
    animation_integrity_tests.rs
    animation_playback_tests.rs
    animation_undo_redo_tests.rs
    bone_and_animation_tests.rs
    bucket_tool_tests.rs
    eyedropper_tests.rs
    integration_animation.rs
    integration_core.rs
    integration_tools.rs
    layer_management_tests.rs
    layer_properties_tests.rs
    move_tool_tests.rs
    palette_tests.rs
    pen_tool_tests.rs
    pencil_eraser_tests.rs
    selection_tool_tests.rs
    spine_offset_fix_tests.rs
    stress_and_precision.rs
    transform_tool_tests.rs
    ui_deletion_redirect_tests.rs
    ui_selection_sync_tests.rs
    window_controls_tests.rs
```

## 模块列表

- **本地化** (`locales`)： 本地化文件
- **根目录文件** (`root`)： (暂无描述)
- **源代码** (`src`)： 核心源代码
- **动画模块** (`src/animation`)： 源代码子模块 `src/animation`
- **应用模块** (`src/app`)： 源代码子模块 `src/app`
- **核心模块** (`src/core`)： 源代码子模块 `src/core`
- **格式模块** (`src/format`)： 源代码子模块 `src/format`
- **历史记录** (`src/history`)： 源代码子模块 `src/history`
- **渲染模块** (`src/render`)： 源代码子模块 `src/render`
- **工具模块** (`src/tools`)： 源代码子模块 `src/tools`
- **界面模块** (`src/ui`)： 源代码子模块 `src/ui`
- **测试** (`tests`)： 集成测试

## 如何获取详细代码

如果你需要查看某个模块的详细代码，请告诉我模块键名（如 `src/core`、`src/app`），我会生成对应的快照文件。
