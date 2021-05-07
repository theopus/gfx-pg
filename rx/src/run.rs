use std::sync::Arc;
use std::time::{Duration, Instant};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopWindowTarget};

use crate::assets::{AssetsLoader, AssetsStorage};
use crate::{events, WinitEvent, EventSender, EventReceiver};
use crate::events::{EngEvent, RxEvent};
#[cfg(feature = "hal")]
use crate::graphics::wrapper::ApiWrapper;
use crate::gui::ExampleRepaintSignal;
use crate::render_w::Renderer;
use crate::wgpu_graphics::{FrameState, State};
use crate::wgpu_graphics::pipeline::Pipeline;
use crate::window::WinitState;

pub struct Engine<T: 'static + Send + Clone> {
    winit_state: WinitState<()>,
    layers: Vec<Box<dyn Layer>>,
    renderer: Renderer,
    channel: (crossbeam_channel::Sender<RxEvent<T>>, crossbeam_channel::Receiver<RxEvent<T>>)
}

struct Test;

impl Pipeline for Test {
    fn process(&mut self, _frame: FrameState) {
        info!("test")
    }
}

impl<T: 'static + Send + Clone> Default for Engine<T> {
    fn default() -> Self {
        let winit_state: WinitState<()> = Default::default();
        let renderer = Renderer::new(winit_state.window.as_ref().unwrap()).unwrap();
        let channel = crossbeam_channel::unbounded();
        Self {
            winit_state,
            layers: Default::default(),
            renderer,
            channel
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
        let (events_loop, window): (winit::event_loop::EventLoop<()>, winit::window::Window) = {
            let WinitState {
                events_loop,
                window,
                ..
            } = self.winit_state;
            (events_loop, window.unwrap())
        };

        let (mut sender, mut receiver) = self.channel;
        let mut layers = self.layers;
        for l in &layers {
            l.setup(sender, receiver);
        }
        let mut renderer = self.renderer;
        let mut events: Vec<events::WinitEvent> = Vec::new();
        let run_start = Instant::now();
        let mut last = Instant::now();

        let repaint_signal: Arc<ExampleRepaintSignal<()>> = std::sync::Arc::new(ExampleRepaintSignal(std::sync::Mutex::new(
            events_loop.create_proxy(),
        )));

        let mut egui_state = crate::gui::EguiState::new(&window, repaint_signal);

        let size = window.inner_size();
        events.push(Event::WindowEvent { window_id: window.id(), event: WindowEvent::Resized(size) });

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

            events::handle_event(&mut sender,&mut events, o_event);
        };
        events_loop.run(run_loop);
    }

    fn on_update(
        layers: &mut Vec<Box<dyn Layer>>,
        events: &mut Vec<events::WinitEvent>,
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
            L: Layer + 'static,
    {
        self.layers.push(Box::new(layer));
    }
}


pub struct FrameUpdate<'a> {
    pub events: &'a Vec<events::WinitEvent>,
    pub elapsed: Duration,
    pub egui_ctx: egui::CtxRef,
}

pub trait Layer<T> {
    fn on_update(&mut self, upd: FrameUpdate);
    fn setup(&mut self, sender: EventSender<T>, receiver: EventReceiver<T>){}
    fn name(&self) -> &'static str  {
        "stub"
    }
}

impl<F> Layer for F
    where F: FnMut(FrameUpdate)
{
    fn on_update(&mut self, upd: FrameUpdate) {
        self(upd)
    }

    fn name(&self) -> &'static str {
        "stub"
    }
}
