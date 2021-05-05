use crate::wgpu_graphics::pipeline::Pipeline;
use crate::wgpu_graphics::FrameState;
use egui_wgpu_backend::epi;
use crate::egui::CtxRef;
use egui_wgpu_backend::epi::App;
use std::sync::{Arc, Mutex};

pub struct EguiState {
    platform: egui_winit_platform::Platform,
    scale_factor: f64,
    repaint_signal: Arc<ExampleRepaintSignal>

}

pub struct EguiPipeline {
    render_pass: egui_wgpu_backend::RenderPass,
    demo_app: egui_demo_lib::WrapApp,
    show_demo: bool
}

enum Event {
    RequestRedraw,
}

pub struct ExampleRepaintSignal(pub std::sync::Mutex<winit::event_loop::EventLoopProxy<()>>);

impl epi::RepaintSignal for ExampleRepaintSignal {
    fn request_repaint(&self) {
        self.0.lock().unwrap().send_event(()).ok();
    }
}

impl EguiPipeline {
    pub fn new(device: &wgpu::Device, show_demo: bool) -> Self {
        let mut egui_rpass = egui_wgpu_backend::RenderPass::new(device, wgpu::TextureFormat::Bgra8UnormSrgb);
        let mut demo_app = egui_demo_lib::WrapApp::default();
        EguiPipeline {render_pass: egui_rpass, demo_app, show_demo}
    }


    pub fn process(&mut self, state: FrameState, ctx: CtxRef, egui_state: &mut EguiState) {
        let FrameState {
            frame,
            encoder,
            queue,
            sc_desc,
            device,
            ..
        } = state;
        if self.show_demo {
            //egui_demo
            let mut app_output = epi::backend::AppOutput::default();
            let mut frame = epi::backend::FrameBuilder {
                info: epi::IntegrationInfo {
                    web_info: None,
                    cpu_usage: None,
                    seconds_since_midnight: None,
                    native_pixels_per_point: Some(egui_state.scale_factor as f32),
                },
                tex_allocator: &mut self.render_pass,
                output: &mut app_output,
                repaint_signal: egui_state.repaint_signal.clone(),
            }.build();
            self.demo_app.update(&ctx, &mut frame);
        }
        // let scale = window.scale_factor() as f32;
        let scale = egui_state.scale_factor;
        let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: sc_desc.width,
            physical_height: sc_desc.height,
            scale_factor: scale as f32,
        };


        let paint_commands= egui_state.end_frame();
        let paint_jobs = ctx.tessellate(paint_commands);


        self.render_pass.update_texture(&device, &queue, &ctx.texture());
        self.render_pass.update_user_textures(&device, &queue);
        self.render_pass.update_buffers(device, queue, &paint_jobs, &screen_descriptor);


        // Record all render passes.
        self.render_pass.execute(
            encoder,
            &frame.view,
            &paint_jobs,
            &screen_descriptor,
            None,
        );
    }
}

impl EguiState {


    pub fn frame(&mut self, scale_factor: f64) -> egui::CtxRef {
        self.scale_factor = scale_factor;
        self.platform.begin_frame();
        self.platform.context().clone()
    }

    pub fn handle_event(&mut self, e: &winit::event::Event<()>) {
        self.platform.handle_event(e);
    }

    pub fn end_frame(&mut self) -> Vec<egui::epaint::ClippedShape> {
        let (_output, paint_commands) = self.platform.end_frame();
        paint_commands
    }
    pub fn new(window: &winit::window::Window, loop_proxy: Arc<ExampleRepaintSignal>) -> Self {
        let size = window.inner_size();
        let mut platform = egui_winit_platform::Platform::new(egui_winit_platform::PlatformDescriptor {
            physical_width: size.width as u32,
            physical_height: size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: egui::FontDefinitions::default(),
            style: Default::default(),
        });
        EguiState { platform, scale_factor:  window.scale_factor(), repaint_signal: loop_proxy }
    }
}

