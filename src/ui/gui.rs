use crate::app::state::{AppState, ToolType, AppMode};
use crate::ui::cursor_overlay::CursorOverlay;
use egui::{FontData, FontDefinitions, FontFamily};
use crate::ui::title_bar::TitleBar;
use crate::app::commands::AppCommand;
use crate::ui::layer_panel::LayerPanel;
use crate::ui::timeline_panel::TimelinePanel;
use crate::ui::toolbar_pixel::ToolbarPixel;
use crate::ui::toolbar_anim::ToolbarAnim;
use rust_i18n::t;

pub struct Framework { pub gui: Gui }
pub struct Gui { fonts_loaded: bool }

impl Gui {
    pub fn new() -> Self { Self { fonts_loaded: false } }

    fn setup_fonts(&mut self, ctx: &egui::Context) {
        if self.fonts_loaded { return; }
        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "icons".to_owned(),
            FontData::from_static(include_bytes!("../../assets/icons.ttf")),
        );
        fonts.font_data.insert(
            "my_font".to_owned(),
            FontData::from_static(include_bytes!("../../assets/my_font.ttf")),
        );

        if let Some(prop) = fonts.families.get_mut(&FontFamily::Proportional) {
            prop.insert(0, "icons".to_owned());
            prop.push("my_font".to_owned());
        }
        if let Some(mono) = fonts.families.get_mut(&FontFamily::Monospace) {
            mono.insert(0, "icons".to_owned());
            mono.push("my_font".to_owned());
        }

        ctx.set_fonts(fonts);
        self.fonts_loaded = true;
    }

    pub fn ui(&mut self, ctx: &egui::Context, app: &mut AppState) {
        self.setup_fonts(ctx);

        if app.mode == AppMode::PixelEdit {
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Z)) { app.undo(); }
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Y)) { app.redo(); }
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::D)) { app.enqueue_command(AppCommand::ClearSelection); }
            
            ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Text(text) = event {
                        if text == "[" { 
                            let (size, _, _) = app.engine.brush_settings_mut();
                            *size = size.saturating_sub(1).max(1); 
                        }
                        else if text == "]" { 
                            let (size, _, _) = app.engine.brush_settings_mut();
                            *size = (*size + 1).min(20); 
                        }
                        else if text == "p" { app.set_tool(ToolType::Pencil); }
                        else if text == "e" { app.set_tool(ToolType::Eraser); }
                        else if text == "b" { app.set_tool(ToolType::Bucket); }
                        else if text == "t" { app.set_tool(ToolType::Transform); }
                        else if text == "c" { app.set_tool(ToolType::Pen); }
                    }
                }
            });
        } else if app.mode == AppMode::Animation {
            ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Text(text) = event {
                        if text == "c" { app.set_tool(ToolType::BoneRotate); }
                        else if text == "v" { app.set_tool(ToolType::BoneTranslate); }
                    }
                }
            });
        }

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(6.0, 6.0);
        style.visuals.widgets.active.bg_stroke.width = 2.0;
        ctx.set_style(style);
        CursorOverlay::draw(ctx, app);

        let frame = egui::Frame::none().fill(egui::Color32::from_rgb(25, 25, 25));
        egui::TopBottomPanel::top("top_bar").frame(frame).show(ctx, |ui| {
            TitleBar::show(ui, app);
        });

        if app.mode == AppMode::PixelEdit {
            egui::SidePanel::left("toolbar_pixel").resizable(false).default_width(115.0).show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);                    
                        ToolbarPixel::show(ui, app);
                        ui.add_space(10.0);
                        ui.label(format!("{}: {:.1}x", t!("toolbar.zoom"), app.view.zoom_level));
                        ui.add(egui::Slider::new(&mut app.view.zoom_level, 0.1..=10.0).step_by(0.1).show_value(false));
                    });
                });
            });
            egui::SidePanel::right("layer_panel").default_width(180.0).show(ctx, |ui| {
                LayerPanel::show(ui, app);
            });

        } else if app.mode == AppMode::Animation {
            egui::SidePanel::right("hierarchy_panel").default_width(220.0).show(ctx, |ui| {
                LayerPanel::show(ui, app);
            });
            egui::TopBottomPanel::bottom("timeline_panel")
                .resizable(true)
                .default_height(200.0)
                .show(ctx, |ui| {
                    TimelinePanel::show(ui, app);
                });

            egui::Window::new("Anim Tools")
                .title_bar(false).resizable(false).collapsible(false)
                .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(10.0, -10.0))
                .show(ctx, |ui| {
                    ToolbarAnim::show(ui, app);
                });

            egui::Window::new("Transform Panel")
                .title_bar(false).resizable(false).collapsible(false)
                .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -10.0))
                .show(ctx, |ui| {
                    crate::ui::bone_transform_panel::BoneTransformPanel::show(ui, app);
                });
        }

        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());
            
            let scale = ctx.pixels_per_point();
            let zoom = app.view.zoom_level as f32;
            let s_cx = app.view.width / 2.0;
            let s_cy = app.view.height / 2.0;
            let c_cx = app.engine.store().canvas_width as f32 / 2.0;
            let c_cy = app.engine.store().canvas_height as f32 / 2.0;
            let pan_x = app.view.pan_x;
            let pan_y = app.view.pan_y;

            let get_canvas_pos = |pos: egui::Pos2| -> (u32, u32) {
                let phys_x = pos.x * scale;
                let phys_y = pos.y * scale;
                let cx = (phys_x - s_cx) / zoom + c_cx - pan_x;
                let cy = (phys_y - s_cy) / zoom + c_cy - pan_y;
                (cx.floor() as i32 as u32, cy.floor() as i32 as u32)
            };

            if response.drag_started() {
                if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                    let (cx, cy) = get_canvas_pos(pos);
                    let _ = app.on_mouse_down(cx, cy);
                }
            }

            if response.dragged() {
                if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                    let (cx, cy) = get_canvas_pos(pos);
                    let _ = app.on_mouse_move(cx, cy);
                }
            } else if response.hovered() {
                if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                    let (cx, cy) = get_canvas_pos(pos);
                    let _ = app.on_mouse_move(cx, cy);
                }
            }

            if response.drag_released() || (response.hovered() && ctx.input(|i| i.pointer.any_released() && !i.pointer.any_down())) {
                let _ = app.on_mouse_up();
            }

            if app.mode == AppMode::PixelEdit && response.secondary_clicked() {
                if app.engine.store().selection.is_active {
                    if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                        app.ui.canvas_menu_pos = pos;
                        app.ui.show_canvas_menu = true;
                    }
                }
            }

            if app.ui.show_canvas_menu && app.mode == AppMode::PixelEdit {
                let area_response = egui::Area::new("canvas_context_menu")
                    .fixed_pos(app.ui.canvas_menu_pos)
                    .order(egui::Order::Foreground)
                    .constrain(true)
                    .show(ctx, |ui: &mut egui::Ui| {
                        egui::Frame::menu(ui.style()).show(ui, |ui| {
                            ui.set_max_width(200.0);
                            ui.set_min_width(120.0);
                            
                            if ui.button(t!("tool.deselect")).clicked() {
                                app.enqueue_command(AppCommand::ClearSelection);
                                app.ui.show_canvas_menu = false;
                            }
                            if ui.button(t!("tool.invert_selection")).clicked() {
                                app.enqueue_command(AppCommand::InvertSelection);
                                app.ui.show_canvas_menu = false;
                            }
                            ui.separator();
                            if ui.button(t!("tool.stroke_selection")).clicked() {
                                app.enqueue_command(AppCommand::StrokeSelection(1));
                                app.ui.show_canvas_menu = false;
                            }
                        });
                    });
                if ctx.input(|i| i.pointer.any_pressed()) {
                    let menu_rect = area_response.response.rect;
                    if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                        if !menu_rect.contains(pos) {
                            app.ui.show_canvas_menu = false;
                        }
                    }
                }
            }
            let scroll = ui.input(|i| i.scroll_delta.y);
            if scroll != 0.0 {
                app.view.zoom_level = (app.view.zoom_level as f32 + scroll * 0.005).clamp(0.1, 10.0) as f64;
                app.view.needs_full_redraw = true;
            }
        });

        if app.ui.show_exit_modal {
            egui::Window::new(t!("dialog.unsaved_title"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(t!("dialog.unsaved_desc"));
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button(t!("dialog.save_exit")).clicked() {
                            app.enqueue_command(AppCommand::SaveProject);
                            app.enqueue_command(AppCommand::ConfirmExit);
                        }
                        if ui.button(t!("dialog.exit_direct")).clicked() { app.enqueue_command(AppCommand::ConfirmExit); }
                        if ui.button(t!("dialog.cancel")).clicked() { app.enqueue_command(AppCommand::CancelExit); }
                    });
                });
        }

        if app.ui.show_resize_modal {
            egui::Window::new(t!("dialog.resize_title"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", t!("dialog.width")));
                        ui.add(egui::TextEdit::singleline(&mut app.ui.resize_new_width).desired_width(60.0));
                        ui.label("px");
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", t!("dialog.height")));
                        ui.add(egui::TextEdit::singleline(&mut app.ui.resize_new_height).desired_width(60.0));
                        ui.label("px");
                    });

                    ui.add_space(10.0);
                    ui.label(format!("{}:", t!("dialog.anchor")));
                    
                    let anchors = [
                        [crate::app::commands::ResizeAnchor::TopLeft, crate::app::commands::ResizeAnchor::TopCenter, crate::app::commands::ResizeAnchor::TopRight],
                        [crate::app::commands::ResizeAnchor::MiddleLeft, crate::app::commands::ResizeAnchor::Center, crate::app::commands::ResizeAnchor::MiddleRight],
                        [crate::app::commands::ResizeAnchor::BottomLeft, crate::app::commands::ResizeAnchor::BottomCenter, crate::app::commands::ResizeAnchor::BottomRight],
                    ];

                    ui.vertical_centered(|ui| {
                        for row in anchors.iter() {
                            ui.horizontal(|ui| {
                                for &anchor in row.iter() {
                                    let is_selected = app.ui.resize_anchor == anchor;
                                    let text = if is_selected { "◉" } else { "○" };
                                    if ui.add_sized([30.0, 30.0], egui::Button::new(text)).clicked() {
                                        app.ui.resize_anchor = anchor;
                                    }
                                }
                            });
                        }
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button(t!("dialog.confirm")).clicked() {
                            let nw = app.ui.resize_new_width.parse::<u32>().unwrap_or(app.engine.store().canvas_width);
                            let nh = app.ui.resize_new_height.parse::<u32>().unwrap_or(app.engine.store().canvas_height);
                            app.enqueue_command(AppCommand::ResizeCanvas(nw, nh, app.ui.resize_anchor));
                            app.ui.show_resize_modal = false;
                        }
                        if ui.button(t!("dialog.cancel")).clicked() { app.ui.show_resize_modal = false; }
                    });
                });
        }
        if let Some(err_msg) = app.ui.error_message.clone() {
            egui::Window::new(t!("dialog.prompt"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(&err_msg);
                    ui.add_space(10.0);
                    ui.vertical_centered(|ui| {
                        if ui.button(t!("dialog.confirm")).clicked() {
                            app.ui.error_message = None;
                        }
                    });
                });
        }
    }
}