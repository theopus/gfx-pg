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
use crate::wgpu_graphics::pipeline::Pipeline;
use std::rc::{Rc};
use std::sync::{
    Arc, Weak
};
use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, epi};
use egui_winit_platform::PlatformDescriptor;
use egui_wgpu_backend::epi::App;

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

        // imgui.io_mut().update_delta_time();
        let mut layers = self.layers;
        let mut renderer = self.renderer;
        let mut events: Vec<MyEvent> = Vec::new();
        let mut last = Instant::now();

        let size = window.inner_size();

        // {
            const OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
            // let repaint_signal = std::sync::Arc::new(egui_winit_platform::ExampleRepaintSignal(std::sync::Mutex::new(
            //     event_loop.create_proxy(),
            // )));

            let mut platform = egui_winit_platform::Platform::new(PlatformDescriptor {
                physical_width: size.width as u32,
                physical_height: size.height as u32,
                scale_factor: window.scale_factor(),
                font_definitions: FontDefinitions::default(),
                style: Default::default(),
            });
            let mut egui_rpass = RenderPass::new(&renderer.wpgu_state.device, OUTPUT_FORMAT);
            let mut demo_app = egui_demo_lib::WrapApp::default();
        demo_app.update()
        egui::Window::new("wow").show()

        let mut frame = epi::backend::FrameBuilder {
            info: epi::IntegrationInfo {
                web_info: None,
                cpu_usage: previous_fra me_time,
                seconds_since_midnight: Some(seconds_since_midnight()),
                native_pixels_per_point: Some(window.scale_factor() as _),
            },
            tex_allocator: Some(&mut egui_rpass),
            output: &mut app_output,
            repaint_signal: repaint_signal.clone(),
        }
            .build();
        frame.se
        // }

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
            platform.handle_event(&o_event);

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

    fn on_update(
        layers: &mut Vec<Box<dyn Layer>>,
        events: &mut Vec<MyEvent>,
        elapsed: Duration
    ) {
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
