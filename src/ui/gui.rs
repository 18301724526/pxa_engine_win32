use crate::app::state::{AppState, AppMode, ToolType};
use crate::ui::cursor_overlay::CursorOverlay;
use egui::{FontData, FontDefinitions, FontFamily};
use crate::ui::title_bar::TitleBar;
use crate::app::commands::*;
use crate::ui::layer_panel::LayerPanel;
use crate::ui::timeline::TimelinePanel;
use crate::ui::toolbar_pixel::ToolbarPixel;
use crate::ui::toolbar_anim::ToolbarAnim;
use rust_i18n::t;
use crate::app::ui_context::UiContext;

pub struct Framework { pub gui: Gui }
pub struct Gui { 
    fonts_loaded: bool,
    pub ui_ctx: UiContext,
}

impl Gui {
    pub fn new() -> Self { Self { fonts_loaded: false, ui_ctx: UiContext::new() } }

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

        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Z)) { app.enqueue_command(Box::new(UndoCmd)); }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Y)) { app.enqueue_command(Box::new(RedoCmd)); }

        // 【核心修复 2】：工具快捷键必须在全局接受监听（由 shortcut_manager 判断当前模式下该干嘛）
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Text(text) = event {
                    if let Some(cmd) = app.shortcuts.handle_text_input(text, app.mode) {
                        app.enqueue_command(cmd);
                    }
                }
            }
        });

        if app.mode == AppMode::PixelEdit {
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::D)) { app.enqueue_command(Box::new(ClearSelectionCmd)); }     
        }

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(6.0, 6.0);
        style.visuals.widgets.active.bg_stroke.width = 2.0;
        ctx.set_style(style);
        CursorOverlay::draw(ctx, app);

        let frame = egui::Frame::none().fill(egui::Color32::from_rgb(25, 25, 25));
        egui::TopBottomPanel::top("top_bar").frame(frame).show(ctx, |ui| {
            TitleBar::show(ui, app, &mut self.ui_ctx);
        });

        if app.mode == AppMode::PixelEdit {
            egui::SidePanel::left("toolbar_pixel").resizable(false).default_width(115.0).show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);                    
                        ToolbarPixel::show(ui, app, &mut self.ui_ctx);
                        ui.add_space(10.0);
                        ui.label(format!("{}: {:.1}x", t!("toolbar.zoom"), app.pixel.view.zoom_level));
                        ui.add(egui::Slider::new(&mut app.pixel.view.zoom_level, 0.1..=10.0).step_by(0.1).show_value(false));
                    });
                });
            });
            egui::SidePanel::right("layer_panel").default_width(180.0).show(ctx, |ui| {
                LayerPanel::show(ui, app, &mut self.ui_ctx);
            });

        } else if app.mode == AppMode::Animation {
            egui::SidePanel::right("hierarchy_panel").default_width(220.0).show(ctx, |ui| {
                LayerPanel::show(ui, app, &mut self.ui_ctx);
            });
            egui::TopBottomPanel::bottom("timeline_panel")
                .resizable(false)
                .default_height(250.0)
                .show(ctx, |ui| {
                    TimelinePanel::show(ui, app, &mut self.ui_ctx);
                });

            egui::Window::new("Anim Tools")
                .title_bar(false).resizable(false).collapsible(false)
                .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(10.0, -10.0))
                .show(ctx, |ui| {
                    ToolbarAnim::show(ui, app, &mut self.ui_ctx);
                });

            egui::Window::new("Transform Panel")
                .title_bar(false).resizable(false).collapsible(false)
                .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -10.0))
                .show(ctx, |ui| {
                    crate::ui::bone_transform_panel::BoneTransformPanel::show(ui, app, &mut self.ui_ctx);
                });
        }

        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());
            
            let scale = ctx.pixels_per_point();
            let zoom = app.pixel.view.zoom_level as f32;
            let s_cx = app.pixel.view.width / 2.0;
            let s_cy = app.pixel.view.height / 2.0;
            let c_cx = app.pixel.engine.store().canvas_width as f32 / 2.0;
            let c_cy = app.pixel.engine.store().canvas_height as f32 / 2.0;
            let pan_x = app.pixel.view.pan_x;
            let pan_y = app.pixel.view.pan_y;

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
                if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                    let (cx, cy) = get_canvas_pos(pos);

                    if app.pixel.engine.tool_manager().active_type == ToolType::Pen {
                        let tool = app.pixel.engine.tool_manager().tools.get(&ToolType::Pen).unwrap();
                        let pen = tool.as_any().downcast_ref::<crate::tools::pen::PenTool>().unwrap();
                        if let (Some(idx), _) = pen.hit_test(&app.pixel.engine.store().active_path, cx as f32, cy as f32) {
                            self.ui_ctx.canvas_menu_pos = pos;
                            self.ui_ctx.selected_node_idx = Some(idx);
                            self.ui_ctx.show_canvas_menu = true;
                        }
                    } else if app.pixel.engine.store().selection.is_active {
                        self.ui_ctx.canvas_menu_pos = pos;
                        self.ui_ctx.show_canvas_menu = true;
                    }
                }
            }

            if self.ui_ctx.show_canvas_menu && app.mode == AppMode::PixelEdit {
                let area_response = egui::Area::new("canvas_context_menu")
                    .fixed_pos(self.ui_ctx.canvas_menu_pos)
                    .order(egui::Order::Foreground)
                    .constrain(true)
                    .show(ctx, |ui: &mut egui::Ui| {
                        egui::Frame::menu(ui.style()).show(ui, |ui| {
                            ui.set_max_width(200.0);
                            ui.set_min_width(120.0);
                            
                            if app.pixel.engine.tool_manager().active_type == ToolType::Pen {
                                if let Some(idx) = self.ui_ctx.selected_node_idx {
                                    if ui.button(t!("tool.convert_node")).clicked() {
                                        app.enqueue_command(Box::new(TogglePathNodeTypeCmd(idx)));
                                        self.ui_ctx.show_canvas_menu = false;
                                    }
                                    if ui.button(t!("tool.delete_node")).clicked() {
                                        app.enqueue_command(Box::new(DeletePathNodeCmd(idx)));
                                        self.ui_ctx.show_canvas_menu = false;
                                    }
                                }
                            } else {
                                if ui.button(t!("tool.deselect")).clicked() {
                                    app.enqueue_command(Box::new(ClearSelectionCmd));
                                    self.ui_ctx.show_canvas_menu = false;
                                }
                                if ui.button(t!("tool.invert_selection")).clicked() {
                                    app.enqueue_command(Box::new(InvertSelectionCmd));
                                    self.ui_ctx.show_canvas_menu = false;
                                }
                                ui.separator();
                                if ui.button(t!("tool.stroke_selection")).clicked() {
                                    app.enqueue_command(Box::new(StrokeSelectionCmd(1)));
                                    self.ui_ctx.show_canvas_menu = false;
                                }
                            }
                        });
                    });
                if ctx.input(|i| i.pointer.any_pressed()) {
                    let menu_rect = area_response.response.rect;
                    if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                        if !menu_rect.contains(pos) {
                            self.ui_ctx.show_canvas_menu = false;
                        }
                    }
                }
            }
            if response.hovered() {
                let scroll = ui.input(|i| i.scroll_delta.y);
                if scroll != 0.0 {
                    app.pixel.view.zoom_level = (app.pixel.view.zoom_level as f32 + scroll * 0.005).clamp(0.1, 10.0) as f64;
                    app.pixel.view.needs_full_redraw = true;
                }
            }
        });

        if self.ui_ctx.show_exit_modal {
            egui::Window::new(t!("dialog.unsaved_title"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(t!("dialog.unsaved_desc"));
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button(t!("dialog.save_exit")).clicked() {
                            app.enqueue_command(Box::new(SaveProjectCmd));
                            app.enqueue_command(Box::new(ConfirmExitCmd));
                        }
                        if ui.button(t!("dialog.exit_direct")).clicked() { app.enqueue_command(Box::new(ConfirmExitCmd)); }
                        if ui.button(t!("dialog.cancel")).clicked() { app.enqueue_command(Box::new(CancelExitCmd)); }
                    });
                });
        }

        if self.ui_ctx.show_resize_modal {
            egui::Window::new(t!("dialog.resize_title"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", t!("dialog.width")));
                        ui.add(egui::TextEdit::singleline(&mut self.ui_ctx.resize_new_width).desired_width(60.0));
                        ui.label("px");
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", t!("dialog.height")));
                        ui.add(egui::TextEdit::singleline(&mut self.ui_ctx.resize_new_height).desired_width(60.0));
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
                                    let is_selected = self.ui_ctx.resize_anchor == anchor;
                                    let text = if is_selected { "◉" } else { "○" };
                                    if ui.add_sized([30.0, 30.0], egui::Button::new(text)).clicked() {
                                        self.ui_ctx.resize_anchor = anchor;
                                    }
                                }
                            });
                        }
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button(t!("dialog.confirm")).clicked() {
                            let nw = self.ui_ctx.resize_new_width.parse::<u32>().unwrap_or(app.pixel.engine.store().canvas_width);
                            let nh = self.ui_ctx.resize_new_height.parse::<u32>().unwrap_or(app.pixel.engine.store().canvas_height);
                            app.enqueue_command(Box::new(ResizeCanvasCmd(nw, nh, self.ui_ctx.resize_anchor)));
                            self.ui_ctx.show_resize_modal = false;
                        }
                        if ui.button(t!("dialog.cancel")).clicked() { self.ui_ctx.show_resize_modal = false; }
                    });
                });
        }
        if let Some(err_msg) = self.ui_ctx.error_message.clone() {
            egui::Window::new(t!("dialog.prompt"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(&err_msg);
                    ui.add_space(10.0);
                    ui.vertical_centered(|ui| {
                        if ui.button(t!("dialog.confirm")).clicked() {
                            self.ui_ctx.error_message = None;
                        }
                    });
                });
        }
    }
}