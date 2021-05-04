use std::time::{Duration, Instant};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopWindowTarget};
use crate::assets::{AssetsLoader, AssetsStorage};
use crate::events::{map_event, MyEvent};
#[cfg(feature = "hal")]
use crate::graphics::wrapper::ApiWrapper;
use crate::render_w::Renderer;
use crate::window::WinitState;
use crate::wgpu_graphics::{State, FrameState};
use imgui::FontSource;
use crate::wgpu_graphics::pipeline::Pipeline;

pub struct Engine {
    winit_state: WinitState,
    layers: Vec<Box<dyn Layer>>,
    renderer: Renderer,
}

struct Test;
impl Pipeline for Test {
    fn process(&mut self, frame: FrameState) {
        info!("test")
    }
}

impl Default for Engine {
    fn default() -> Self {
        let mut winit_state: WinitState = Default::default();
        let renderer = Renderer::new(&mut winit_state).unwrap();
        Self {
            winit_state,
            layers: Default::default(),
            renderer,
        }
    }
}

impl Engine {
    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }
    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }
    pub fn loader(
        &mut self,
    ) -> (
        &mut State,
        &mut AssetsLoader,
        &mut AssetsStorage,
    ) {
        (
            &mut self.renderer.wpgu_state,
            &mut self.renderer.loader,
            &mut self.renderer.storage,
        )
    }

    pub fn run(self) {
        let (events_loop, window) = {
            let WinitState {
                events_loop,
                window,
                ..
            } = self.winit_state;
            (events_loop, window.unwrap())
        };
        // {
        //     let mut imgui = imgui::Context::create();
        //     let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        //     platform.attach_window(
        //         imgui.io_mut(),
        //         window,
        //         imgui_winit_support::HiDpiMode::Default,
        //     );
        //     imgui.set_ini_filename(None);
        //
        //     let hidpi_factor = display.window.scale_factor();
        //     let font_size = (13.0 * hidpi_factor) as f32;
        //     imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        //     imgui.fonts().add_font(&[FontSource::DefaultFontData {
        //         config: Some(imgui::FontConfig {
        //             oversample_h: 1,
        //             pixel_snap_h: true,
        //             size_pixels: font_size,
        //             ..Default::default()
        //         }),
        //     }]);
        //     let renderer_config = imgui_wgpu::RendererConfig {
        //         texture_format: display.sc_desc.format,
        //         ..Default::default()
        //     };
        //     let renderer = imgui_wgpu::Renderer::new(&mut imgui, &display.device, &display.queue, renderer_config);
        //     // imgui.io_mut().update_delta_time();
        //
        //
        //     platform.prepare_frame(imgui.io_mut(), window);
        //     platform.prepare_render()
        //     let ui = &imgui.frame();
        //
        // }

        let mut layers = self.layers;
        let mut renderer = self.renderer;
        let mut events: Vec<MyEvent> = Vec::new();
        let mut last = Instant::now();

        events.push(MyEvent::Resized(800, 600));
        use winit::dpi::PhysicalSize;
        renderer.reset_swapchain(PhysicalSize {
            width: 800,
            height: 600,
        });

        info!("Start!");
        let run_loop = move |o_event: Event<()>,
                             _: &EventLoopWindowTarget<()>,
                             control_flow: &mut ControlFlow| {
            //Always poll
            *control_flow = ControlFlow::Poll;

            match o_event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    info!("The close button was pressed; stopping");
                    *control_flow = ControlFlow::Exit
                }
                Event::LoopDestroyed => {
                    info!("On close handle")
                    /*On close*/
                }
                Event::RedrawRequested(_w) => {
                    /*Render*/
                }
                Event::MainEventsCleared => {
                    let current = Instant::now();
                    let elapsed = current - last;
                    Self::on_update(&mut layers, &mut events, elapsed);

                    {
                        let start =  Instant::now();
                        renderer.render();
                        debug!("render took {:?}", Instant::now() - start);
                    }

                    last = current;
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(phys_size),
                    ..
                } => {
                    info!("{:?}", phys_size);

                    renderer.reset_swapchain(phys_size);

                    let owned = map_event(o_event);
                    if let Some(e) = owned {
                        Self::on_event(&mut events, e);
                    }
                }
                _ => {
                    let owned = map_event(o_event);
                    if let Some(e) = owned {
                        Self::on_event(&mut events, e);
                    }
                }
            }
        };
        events_loop.run(run_loop);
    }

    fn on_update(layers: &mut Vec<Box<dyn Layer>>, events: &mut Vec<MyEvent>, elapsed: Duration) {
        for layer in layers.iter_mut() {
            let start =  Instant::now();
            layer.on_update(events, elapsed);
            debug!("{:?} took {:?}", layer.name(), Instant::now() - start)
        }
        events.clear()
    }

    pub fn push_layer<L>(&mut self, layer: L)
        where
            L: Layer + 'static,
    {
        self.layers.push(Box::new(layer));
    }

    fn on_event(vec: &mut Vec<MyEvent>, event: MyEvent) {
        vec.push(event);
    }
}

pub trait Layer {
    fn on_update(&mut self, events: &Vec<MyEvent>, elapsed: Duration);
    fn name(&self) -> &'static str;
}

impl<F> Layer for F
    where
        F: FnMut(&Vec<MyEvent>, Duration),
{
    fn on_update(&mut self, events: &Vec<MyEvent>, elapsed: Duration) {
        self(events, elapsed)
    }

    fn name(&self) -> &'static str {
        "stub"
    }
}
