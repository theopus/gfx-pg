use crate::wgpu_graphics::pipeline::Pipeline;
use crate::wgpu_graphics::FrameState;
use imgui::FontSource;

pub struct ImGuiRenderer {
    renderer: imgui_wgpu::Renderer
}

impl ImGuiRenderer {
    fn process(&mut self, f: imgui::Ui) {
        // f.render();
        // self.renderer
        //     .render()
    }
}


pub struct ImGuiState {
    context: imgui::Context,
    platform: imgui_winit_support::WinitPlatform
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
        ImGuiState { context: imgui, platform: platform }
    }
}


