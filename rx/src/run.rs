use std::rc::Rc;
use std::sync::{
    Arc, Weak,
};
use std::time::{Duration, Instant};

use egui::FontDefinitions;
use egui_wgpu_backend::{epi, RenderPass};
use egui_wgpu_backend::epi::App;
use egui_winit_platform::PlatformDescriptor;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopWindowTarget};

use crate::assets::{AssetsLoader, AssetsStorage};
use crate::events::{map_event, MyEvent};
#[cfg(feature = "hal")]
use crate::graphics::wrapper::ApiWrapper;
use crate::gui::ExampleRepaintSignal;
use crate::render_w::Renderer;
use crate::wgpu_graphics::{FrameState, State};
use crate::wgpu_graphics::pipeline::Pipeline;
use crate::window::WinitState;

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

        let repaint_signal = std::sync::Arc::new(ExampleRepaintSignal(std::sync::Mutex::new(
            events_loop.create_proxy(),
        )));

        let mut egui_state = crate::gui::EguiState::new(&window, repaint_signal);


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
            egui_state.handle_event(&o_event);

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

                    let ctx = egui_state.frame(window.scale_factor());
                    let size = window.inner_size(); 
                    Self::on_update(&mut layers, &mut events, elapsed);
                    {
                        let start = Instant::now();
                        renderer.render(ctx, &mut egui_state);
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
