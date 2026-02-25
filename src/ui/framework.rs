use egui::{Context, Visuals};
use egui_wgpu::{renderer::ScreenDescriptor, Renderer};
use egui_winit::State;
use pixels::{wgpu, PixelsContext};
use winit::window::Window;
use winit::event::WindowEvent;
use winit::event_loop::EventLoopWindowTarget;

use crate::ui::gui::Gui;

pub struct GuiFramework {
    pub egui_ctx: Context,
    pub egui_state: State,
    pub screen_descriptor: ScreenDescriptor,
    pub renderer: Renderer,
    pub gui: Gui,
}

impl GuiFramework {
    pub fn new<T>(
        _window: &Window, 
        width: u32,
        height: u32,
        scale_factor: f32,
        pixels: &pixels::Pixels,
        gui: Gui,
        event_loop: &EventLoopWindowTarget<T>,
    ) -> Self {
        let egui_ctx = Context::default();
        let mut egui_state = State::new(event_loop);
        egui_state.set_pixels_per_point(scale_factor);
        
        let device = pixels.device();
        let render_format = pixels.render_texture_format();
        let renderer = Renderer::new(device, render_format, None, 1);
        
        egui_ctx.set_visuals(Visuals::dark());

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: scale_factor,
        };

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            renderer,
            gui,
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        self.egui_state.on_event(&self.egui_ctx, event).consumed
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.screen_descriptor.size_in_pixels = [width, height];
    }

    pub fn scale_factor(&mut self, scale_factor: f32) {
        self.screen_descriptor.pixels_per_point = scale_factor;
    }

    pub fn prepare(&mut self, window: &Window) {
        let raw_input = self.egui_state.take_egui_input(window);
        self.egui_ctx.begin_frame(raw_input);
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) {
        let full_output = self.egui_ctx.end_frame();
        let paint_jobs = self.egui_ctx.tessellate(full_output.shapes);

        for (id, image_delta) in full_output.textures_delta.set {
            self.renderer.update_texture(&context.device, &context.queue, id, &image_delta);
        }

        self.renderer.update_buffers(
            &context.device,
            &context.queue,
            encoder,
            &paint_jobs,
            &self.screen_descriptor,
        );

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: render_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true, 
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.renderer.render(&mut rpass, &paint_jobs, &self.screen_descriptor);
        }

        for id in full_output.textures_delta.free {
            self.renderer.free_texture(&id);
        }
    }
}