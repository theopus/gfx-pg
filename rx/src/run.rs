use std::sync::Arc;
use std::time::{Duration, Instant};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopWindowTarget};

use crate::assets::{AssetsLoader, AssetsStorage};
use crate::events;
use crate::events::RxEvent;
#[cfg(feature = "hal")]
use crate::graphics::wrapper::ApiWrapper;
use crate::gui::ExampleRepaintSignal;
use crate::render_w::Renderer;
use crate::wgpu_graphics::{FrameState, State};
use crate::wgpu_graphics::pipeline::Pipeline;
use crate::window::WinitState;

pub struct Engine<T: 'static + Send + Clone> {
    winit_state: WinitState<T>,
    layers: Vec<Box<dyn Layer<T>>>,
    renderer: Renderer,
}

struct Test;

impl Pipeline for Test {
    fn process(&mut self, _frame: FrameState) {
        info!("test")
    }
}

impl<T: 'static + Send + Clone> Default for Engine<T> {
    fn default() -> Self {
        let winit_state: WinitState<T> = Default::default();
        let renderer = Renderer::new(winit_state.window.as_ref().unwrap()).unwrap();
        Self {
            winit_state,
            layers: Default::default(),
            renderer,
        }
    }
}

impl<T: Send + Clone> Engine<T> {
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
        let (events_loop, window): (winit::event_loop::EventLoop<RxEvent<T>>, winit::window::Window) = {
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
        let mut events: Vec<events::WinitEvent<T>> = Vec::new();
        let mut run_start = Instant::now();
        let mut last = Instant::now();

        let repaint_signal: Arc<ExampleRepaintSignal<T>> = std::sync::Arc::new(ExampleRepaintSignal(std::sync::Mutex::new(
            events_loop.create_proxy(),
        )));

        let mut egui_state = crate::gui::EguiState::new(&window, repaint_signal);


        // events.push(MyEvent::Resized(800, 600));
        use winit::dpi::PhysicalSize;
        let size = window.inner_size();
        events.push(Event::WindowEvent { window_id: window.id(), event: WindowEvent::Resized(size) });

        info!("Start!");
        let run_loop = move |o_event: Event<RxEvent<T>>,
                             _: &EventLoopWindowTarget<RxEvent<T>>,
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

                    let ctx = egui_state.frame(window.scale_factor(), &run_start);
                    Self::on_update(&mut layers, &mut events, elapsed, ctx.clone());
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
                }
                _ => {}
            }

            events::handle_event(&mut events, o_event);
        };
        events_loop.run(run_loop);
    }

    fn on_update(
        layers: &mut Vec<Box<dyn Layer<T>>>,
        events: &mut Vec<events::WinitEvent<T>>,
        elapsed: Duration,
        egui_ctx: egui::CtxRef,
    ) {
        for layer in layers.iter_mut() {
            let start = Instant::now();
            layer.on_update(FrameUpdate {
                events: &events,
                elapsed,
                egui_ctx: egui_ctx.clone(),
            });
            debug!("{:?} took {:?}", layer.name(), Instant::now() - start)
        }
        events.clear()
    }

    pub fn push_layer<L>(&mut self, layer: L)
        where
            L: Layer<T> + 'static,
    {
        self.layers.push(Box::new(layer));
    }
}


pub struct FrameUpdate<'a, T: 'static + Clone + Send> {
    pub events: &'a Vec<events::WinitEvent<T>>,
    pub elapsed: Duration,
    pub egui_ctx: egui::CtxRef,
}

pub trait Layer<T: Clone + Send> {
    fn on_update(&mut self, upd: FrameUpdate<T>);
    fn name(&self) -> &'static str;
}

impl<F, T: Clone + Send> Layer<T> for F
    where
        F: FnMut(FrameUpdate<T>),
{
    fn on_update(&mut self, upd: FrameUpdate<T>) {
        self(upd)
    }

    fn name(&self) -> &'static str {
        "stub"
    }
}
