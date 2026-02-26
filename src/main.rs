#![windows_subsystem = "windows"]
use pxa_engine_win32::render::compositor::{Compositor, Viewport};
use pxa_engine_win32::app::state::AppState;
use pxa_engine_win32::app::commands::AppCommand;
use pxa_engine_win32::app::command_handler::CommandHandler;
use pxa_engine_win32::ui::gui::Gui;
use pxa_engine_win32::ui::framework::GuiFramework;
use pxa_engine_win32::render::anim_compositor::AnimCompositor; 
use pxa_engine_win32::animation::controller::AnimationController; 
use pxa_engine_win32::app::state::AppMode; 


use winit::{
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::LogicalSize,
};
use pixels::wgpu;
use pixels::{Pixels, SurfaceTexture};
use winit::window::{ResizeDirection, CursorIcon};
use rust_i18n::t;

fn get_resize_direction(pos: (f64, f64), size: winit::dpi::PhysicalSize<u32>, edge: f64) -> Option<ResizeDirection> {
    let (x, y) = pos;
    let (w, h) = (size.width as f64, size.height as f64);
    let left = x < edge;
    let right = x > w - edge;
    let top = y < edge;
    let bottom = y > h - edge;

    if left && top { Some(ResizeDirection::NorthWest) }
    else if right && top { Some(ResizeDirection::NorthEast) }
    else if left && bottom { Some(ResizeDirection::SouthWest) }
    else if right && bottom { Some(ResizeDirection::SouthEast) }
    else if left { Some(ResizeDirection::West) }
    else if right { Some(ResizeDirection::East) }
    else if top { Some(ResizeDirection::North) }
    else if bottom { Some(ResizeDirection::South) }
    else { None }
}

rust_i18n::i18n!("locales");
fn main() -> Result<(), Box<dyn std::error::Error>> {
    rust_i18n::set_locale("zh-CN");
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(t!("app.title").to_string())
        .with_inner_size(LogicalSize::new(1024.0, 768.0))
        .with_decorations(false)
        .with_resizable(true)
        .build(&event_loop)?;

    let window_size = window.inner_size();
    let mut app_state = AppState::new();
    app_state.view.update_viewport(window_size.width as f32, window_size.height as f32);
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    let mut pixels = match Pixels::new(window_size.width, window_size.height, surface_texture) {
        Ok(p) => p,
        Err(e) => {
            rfd::MessageDialog::new()
                .set_title(&t!("error.hardware_init_title").to_string())
                .set_description(&t!("error.hardware_init_desc", error = e.to_string()).to_string())
                .set_level(rfd::MessageLevel::Error)
                .show();
            return Err(e.into());
        }
    };

    let mut framework = GuiFramework::new(
        &window, 
        window_size.width, 
        window_size.height, 
        window.scale_factor() as f32, 
        &pixels, 
        Gui::new(),
        &event_loop 
    );

    let mut cursor_pos = (0.0, 0.0);
    let mut last_cursor_pos = (0.0, 0.0);
    let mut is_mouse_down = false;
    let mut current_resize_dir: Option<ResizeDirection> = None;
    let mut active_resize_dir: Option<ResizeDirection> = None;
    let mut resize_start_window_pos = winit::dpi::PhysicalPosition::new(0, 0);
    let mut resize_start_window_size = winit::dpi::PhysicalSize::new(0, 0);
    let mut resize_start_cursor_global = (0.0, 0.0);
    let mut anim_renderer = AnimCompositor::new(pixels.device(), pixels.render_texture_format());
    let mut last_frame_inst = std::time::Instant::now();
    app_state.engine.update_render_cache(None);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => {
                let window_pos = window.outer_position().unwrap_or(winit::dpi::PhysicalPosition::new(0, 0));

                if let WindowEvent::CursorMoved { position, .. } = &event {
                    cursor_pos = (position.x, position.y);
                    
                    if let Some(dir) = active_resize_dir {
                        let global_x = window_pos.x as f64 + position.x;
                        let global_y = window_pos.y as f64 + position.y;
                        let dx = global_x - resize_start_cursor_global.0;
                        let dy = global_y - resize_start_cursor_global.1;

                        let mut new_w = resize_start_window_size.width as f64;
                        let mut new_h = resize_start_window_size.height as f64;
                        let mut new_x = resize_start_window_pos.x as f64;
                        let mut new_y = resize_start_window_pos.y as f64;

                        match dir {
                            ResizeDirection::East => new_w += dx,
                            ResizeDirection::West => { new_x += dx; new_w -= dx; },
                            ResizeDirection::South => new_h += dy,
                            ResizeDirection::North => { new_y += dy; new_h -= dy; },
                            ResizeDirection::SouthEast => { new_w += dx; new_h += dy; },
                            ResizeDirection::SouthWest => { new_x += dx; new_w -= dx; new_h += dy; },
                            ResizeDirection::NorthEast => { new_y += dy; new_w += dx; new_h -= dy; },
                            ResizeDirection::NorthWest => { new_x += dx; new_y += dy; new_w -= dx; new_h -= dy; },
                        }

                        let (min_w, min_h) = (400.0, 300.0);
                        if new_w < min_w {
                            if dir == ResizeDirection::West || dir == ResizeDirection::NorthWest || dir == ResizeDirection::SouthWest { new_x -= min_w - new_w; }
                            new_w = min_w;
                        }
                        if new_h < min_h {
                            if dir == ResizeDirection::North || dir == ResizeDirection::NorthWest || dir == ResizeDirection::NorthEast { new_y -= min_h - new_h; }
                            new_h = min_h;
                        }

                        window.set_inner_size(winit::dpi::PhysicalSize::new(new_w as u32, new_h as u32));
                        window.set_outer_position(winit::dpi::PhysicalPosition::new(new_x as i32, new_y as i32));
                        return;
                    }
                }

                if let WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } = &event {
                    if active_resize_dir.is_some() {
                        active_resize_dir = None;
                        return;
                    }
                }

                if active_resize_dir.is_none() {
                    let edge = 8.0 * window.scale_factor();
                    current_resize_dir = if !window.is_maximized() { get_resize_direction(cursor_pos, window.inner_size(), edge) } else { None };

                    let icon = if let Some(dir) = current_resize_dir {
                        match dir {
                            ResizeDirection::East | ResizeDirection::West => CursorIcon::EwResize,
                            ResizeDirection::North | ResizeDirection::South => CursorIcon::NsResize,
                            ResizeDirection::NorthWest | ResizeDirection::SouthEast => CursorIcon::NwseResize,
                            ResizeDirection::NorthEast | ResizeDirection::SouthWest => CursorIcon::NeswResize,
                        }
                    } else if app_state.is_space_pressed { CursorIcon::Hand } else { CursorIcon::Default };
                    window.set_cursor_icon(icon);
                }

                if let WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } = &event {
                    if let Some(dir) = current_resize_dir {
                        active_resize_dir = Some(dir);
                        resize_start_window_pos = window_pos;
                        resize_start_window_size = window.inner_size();
                        resize_start_cursor_global = (
                            window_pos.x as f64 + cursor_pos.0,
                            window_pos.y as f64 + cursor_pos.1,
                        );
                        return;
                    }
                }
                let _ = framework.handle_event(&event);

                match event {
                    WindowEvent::Resized(size) => {
                        let _ = pixels.resize_surface(size.width, size.height);
                        framework.resize(size.width, size.height);
                        app_state.view.update_viewport(size.width as f32, size.height as f32);
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(winit::event::VirtualKeyCode::Space) = input.virtual_keycode {
                            app_state.is_space_pressed = input.state == ElementState::Pressed;
                            if current_resize_dir.is_none() {
                                if app_state.is_space_pressed {
                                    window.set_cursor_icon(winit::window::CursorIcon::Hand);
                                } else {
                                    window.set_cursor_icon(winit::window::CursorIcon::Default);
                                }
                            }
                            } else if let Some(winit::event::VirtualKeyCode::Return) = input.virtual_keycode {
                                if input.state == ElementState::Pressed {
                                    app_state.enqueue_command(AppCommand::CommitCurrentTool);
                                }
                            } else if let Some(winit::event::VirtualKeyCode::Escape) = input.virtual_keycode {
                                if input.state == ElementState::Pressed {
                                    app_state.enqueue_command(AppCommand::CancelCurrentTool);
                                }
                            }
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        framework.scale_factor(scale_factor as f32);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let dx = (cursor_pos.0 - last_cursor_pos.0) as f32;
                        let dy = (cursor_pos.1 - last_cursor_pos.1) as f32;
                        last_cursor_pos = (position.x, position.y);
                        if app_state.is_space_pressed && is_mouse_down {
                            let zoom = app_state.view.zoom_level as f32;
                            app_state.view.pan_x += dx / zoom;
                            app_state.view.pan_y += dy / zoom;                        
                        }
                    }
                    WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                        is_mouse_down = state == ElementState::Pressed;
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                let dt = now.duration_since(last_frame_inst);
                last_frame_inst = now;

                while let Some(cmd) = app_state.pop_command() {
                    match cmd {
                        AppCommand::WindowClose => *control_flow = ControlFlow::Exit,
                        AppCommand::WindowDrag => { let _ = window.drag_window(); },
                        AppCommand::WindowMinimize => window.set_minimized(true),
                        AppCommand::WindowMaximize => {
                            window.set_maximized(!window.is_maximized());
                        },
                        _ => CommandHandler::execute(&mut app_state, cmd),
                    }
                }
                framework.prepare(&window);
                let ctx = framework.egui_ctx.clone();
                framework.gui.ui(&ctx, &mut app_state);
                let width = app_state.view.width as u32;
                let height = app_state.view.height as u32;

                if let Err(e) = pixels.resize_buffer(width, height) {
                    eprintln!("Resize buffer failed: {}", e);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                let viewport = Viewport { 
                    screen_width: width, 
                    screen_height: height,
                    zoom: app_state.view.zoom_level as f32,
                    pan_x: app_state.view.pan_x,
                    pan_y: app_state.view.pan_y,
                };

                if app_state.view.needs_full_redraw {
                    app_state.engine.update_render_cache(None);
                    app_state.view.needs_full_redraw = false;
                } else if let Some(rect) = app_state.view.dirty_rect.take() {
                    app_state.engine.update_render_cache(Some(rect));
                }

                match app_state.mode {
                    AppMode::PixelEdit => {
                        Compositor::render(app_state.engine.store(), pixels.frame_mut(), viewport);
                        if app_state.engine.tool_manager().active_type == pxa_engine_win32::app::state::ToolType::CreateBone {
                             AnimCompositor::render_cpu(app_state.engine.store(), &app_state.animation.project.skeleton, pixels.frame_mut(), viewport, app_state.ui.selected_bone_id.as_ref());
                             
                             if let Some(tool) = app_state.engine.tool_manager().tools.get(&pxa_engine_win32::app::state::ToolType::CreateBone) {
                                 if let Some(bone_tool) = tool.as_any().downcast_ref::<pxa_engine_win32::tools::create_bone::CreateBoneTool>() {
                                     if let (Some(start), Some(end)) = (bone_tool.start_pos, bone_tool.preview_end) {
                                         let mut temp_skel = pxa_engine_win32::core::animation::skeleton::Skeleton::new();
                                         let mut temp_bone = pxa_engine_win32::core::animation::bone::BoneData::new("preview".into(), "preview".into());
                                         temp_bone.local_transform.x = start.0;
                                         temp_bone.local_transform.y = start.1;
                                         temp_bone.length = ((end.0 - start.0).powi(2) + (end.1 - start.1).powi(2)).sqrt();
                                         temp_bone.local_transform.rotation = (end.1 - start.1).atan2(end.0 - start.0).to_degrees();
                                         temp_skel.add_bone(temp_bone);
                                         temp_skel.update();
                                         AnimCompositor::render_cpu(app_state.engine.store(), &temp_skel, pixels.frame_mut(), viewport, None);
                                     }
                                 }
                             }
                        }
                    }
                    AppMode::Animation => {
                        AnimationController::update(&mut app_state.animation, dt);
                        Compositor::render(app_state.engine.store(), pixels.frame_mut(), viewport);
                        AnimCompositor::render_cpu(app_state.engine.store(), &app_state.animation.project.skeleton, pixels.frame_mut(), viewport, app_state.ui.selected_bone_id.as_ref());
                        anim_renderer.prepare_textures(pixels.device(), pixels.queue(), app_state.engine.store(), &app_state.animation.project.skeleton);
                    }
                }

                let render_result = pixels.render_with(|encoder, render_target, context| {
                    context.scaling_renderer.render(encoder, render_target);
                    if app_state.mode == AppMode::Animation {
                        use pixels::wgpu::util::DeviceExt;
                        let skeleton = &app_state.animation.project.skeleton;
                        let store = app_state.engine.store();

                        let mut instances = Vec::new();
                        for slot in &skeleton.slots {
                            if let (Some(layer_id), Some(bone_idx)) = (&slot.current_attachment, skeleton.bones.iter().position(|b| b.data.id == slot.data.bone_id)) {
                                if let Some(layer) = store.get_layer(layer_id) {
                                    let m = skeleton.bones[bone_idx].world_matrix;
                                    let final_matrix = [
                                        [m[0] * layer.width as f32, m[1] * layer.width as f32, 0.0, 0.0],
                                        [m[2] * layer.height as f32, m[3] * layer.height as f32, 0.0, 0.0],
                                        [0.0, 0.0, 1.0, 0.0],
                                        [m[4], m[5], 0.0, 1.0],
                                    ];
                                    let c = slot.current_color;
                                    let color_f32 = [c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0, c.a as f32 / 255.0];
                                    
                                    let mut instance_data = Vec::with_capacity(80);
                                    instance_data.extend_from_slice(bytemuck::cast_slice(&final_matrix));
                                    instance_data.extend_from_slice(bytemuck::cast_slice(&color_f32));

                                    instances.push(context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                        label: None,
                                        contents: &instance_data,
                                        usage: wgpu::BufferUsages::VERTEX,
                                    }));
                                }
                            }
                        }

                        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("anim_gpu_pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: render_target,
                                resolve_target: None,
                                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: true },
                            })],
                            depth_stencil_attachment: None,
                        });
                        
                        anim_renderer.render_gpu(&context.queue, &mut rpass, store, skeleton, viewport, app_state.ui.selected_bone_id.as_ref(), &instances);
                    }
                    framework.render(encoder, render_target, context);
                    Ok(())
                });

                if render_result.is_err() { *control_flow = ControlFlow::Exit; }
            }
            _ => (),
        }
    });
}