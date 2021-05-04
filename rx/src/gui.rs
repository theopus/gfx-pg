use std::rc::Rc;

use imgui::{FontSource, Ui};

use crate::wgpu_graphics::FrameState;
use crate::wgpu_graphics::pipeline::Pipeline;
use std::time;
use std::sync::Arc;

pub struct ImGuiRenderer {
    renderer: imgui_wgpu::Renderer,
}

impl ImGuiRenderer {
    fn process(&mut self, frame: FrameState) {
        let FrameState {
            frame,
            encoder,
            queue,
            imgui_ui,
            device,
            ..
        } = frame;

        if let Some(ui_rc) = imgui_ui {
            if let Ok(ui) = Arc::try_unwrap(ui_rc) {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });

                self.renderer
                    .render(ui.render(), &queue, &device, &mut rpass)
                    .expect("Rendering failed");
            }
        }
    }
}

pub struct ImGuiState {
    context: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    last_cursor: Option<imgui::MouseCursor>,
}

impl ImGuiState {
    pub fn new(window: &winit::window::Window) -> Self {
        let mut imgui = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );
        imgui.set_ini_filename(None);

        let hidpi_factor = window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);
        ImGuiState { context: imgui, platform: platform, last_cursor: None }
    }

    pub fn new_frame(&mut self, window: &winit::window::Window, dt: time::Duration) -> Arc<Ui> {
        self.context.io_mut().update_delta_time(dt);
        self.platform.prepare_frame(self.context.io_mut(), window);
        Arc::new(self.context.frame())
    }

    pub fn prepare_render(&mut self, ui: Rc<Ui>, window: &winit::window::Window) {
        if self.last_cursor != ui.mouse_cursor() {
            self.last_cursor = ui.mouse_cursor();
            self.platform.prepare_render(&ui, window);
        }
    }
}


