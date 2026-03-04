use egui::{Slider, vec2, TextureOptions};

pub struct ImportModalView;

impl ImportModalView {
    pub fn show(ctx: &egui::Context, ui_ctx: &mut crate::app::ui_context::UiContext) -> bool {
        let state = &mut ui_ctx.import_modal;
        if !state.is_open { return false; }

        // 检查后台线程是否算完了
        if state.is_processing {
            let mut result_lock = state.pending_result.lock().unwrap();
            if let Some((color_image, raw_data)) = result_lock.take() {
                state.preview_texture = Some(ctx.load_texture("preview_tex", color_image, TextureOptions::NEAREST));
                state.cached_pixel_data = Some(raw_data);
                state.is_processing = false;
            }
        }

        let mut needs_recompute = false;
        let mut is_confirmed = false;

        egui::Window::new("🖼 图片像素化预览")
            .collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // 左侧：滑块面板
                    ui.vertical(|ui| {
                        ui.set_width(220.0);
                        ui.label(egui::RichText::new("⚙ 参数调节").strong());
                        ui.add_space(10.0);
                        
                        let cfg = &mut state.config;
                        ui.label("输出宽度:");
                        if ui.add(Slider::new(&mut cfg.target_w, 32..=512)).drag_released() { needs_recompute = true; }
                        ui.label("输出高度:");
                        if ui.add(Slider::new(&mut cfg.target_h, 32..=512)).drag_released() { needs_recompute = true; }
                        
                        ui.separator();
                        
                        ui.label("对比度 (Contrast):");
                        if ui.add(Slider::new(&mut cfg.contrast, 0.8..=1.5)).drag_released() { needs_recompute = true; }
                        ui.label("亮度 (Brightness):");
                        if ui.add(Slider::new(&mut cfg.brightness, -50.0..=50.0)).drag_released() { needs_recompute = true; }
                        
                        ui.separator();
                        
                        ui.label("色数预算 (Color Count):");
                        if ui.add(Slider::new(&mut cfg.color_count, 8..=64)).drag_released() { needs_recompute = true; }
                        ui.label("色阶硬度 (Pruning):");
                        if ui.add(Slider::new(&mut cfg.min_color_distance, 500..=5000)).drag_released() { needs_recompute = true; }
                        
                        ui.separator();
                        
                        if ui.checkbox(&mut cfg.use_selout, "✒ 开启智能描边").clicked() { needs_recompute = true; }

                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            let btn_text = if state.is_processing { "⌛ 处理中..." } else { "✅ 确认导入" };
                            if ui.add_sized([100.0, 30.0], egui::Button::new(btn_text)).clicked() && !state.is_processing {
                                is_confirmed = true;
                            }
                            if ui.add_sized([100.0, 30.0], egui::Button::new("❌ 取消")).clicked() {
                                state.close();
                            }
                        });
                    });

                    ui.separator();

                    // 右侧：实时预览图
                    ui.vertical_centered(|ui| {
                        ui.set_width(400.0);
                        ui.set_height(400.0);
                        if state.is_processing {
                            ui.centered_and_justified(|ui| { ui.spinner(); });
                        } else if let Some(tex) = &state.preview_texture {
                            // 保持比例缩放显示，并使用 NEAREST 保证像素清晰
                            let size = tex.size_vec2();
                            let aspect = size.x / size.y;
                            let draw_size = if aspect > 1.0 { vec2(380.0, 380.0 / aspect) } else { vec2(380.0 * aspect, 380.0) };
                            ui.image(tex, draw_size);
                        }
                    });
                });
            });

        if needs_recompute {
            state.trigger_preview_update();
        }
        is_confirmed
    }
}