use std::time::{Duration, Instant};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopWindowTarget};

use crate::assets::{AssetsLoader, AssetsStorage};
use crate::events::{map_event, MyEvent};
use crate::graphics::wrapper::ApiWrapper;
use crate::render::Renderer;
use crate::window::WinitState;

pub struct Engine {
    winit_state: WinitState,
    layers: Vec<Box<dyn Layer>>,
    renderer: Renderer,
}

impl Default for Engine {
    fn default() -> Self {
        let winit_state: WinitState = Default::default();
        let window = &winit_state.window;
        let renderer = Renderer::new(window).unwrap();
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
    pub fn loader(&mut self) -> (&mut ApiWrapper<back::Backend>, &mut AssetsLoader, &mut AssetsStorage) {
        (&mut self.renderer.api, &mut self.renderer.loader, &mut self.renderer.storage)
    }

    pub fn run(self) {
        let WinitState {
            events_loop,
            window,
        } = self.winit_state;


        let mut layers = self.layers;
        let mut renderer = self.renderer;
        let mut events: Vec<MyEvent> = Vec::new();
        let mut last = Instant::now();


        //[BUG#windows]: winit
        let mut draw_req = 0;

        info!("Start!");
        let run_loop = move |o_event: Event<()>, _: &EventLoopWindowTarget<()>, control_flow: &mut ControlFlow| {
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
                Event::RedrawRequested(_) => {
                    /*Render*/
                    //[BUG#windows]: winit
                    {
                        assert!(draw_req == 1, "Draw requests: {:?}", draw_req);
                        draw_req -= 1;
                    }
                    renderer.render();
                }
                Event::MainEventsCleared => {
                    let current = Instant::now();
                    let elapsed = current - last;
                    Self::on_update(&mut layers, &mut events, elapsed);
                    window.request_redraw();
                    //[BUG#windows]: winit
                    draw_req += 1;
                    last = current
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(phys_size),
                    ..
                } => {
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
            layer.on_update(events, elapsed);
        }
        events.clear()
    }

    pub fn push_layer<L>(&mut self, layer: L) where L: Layer + 'static {
        self.layers.push(Box::new(layer));
    }

    fn on_event(vec: &mut Vec<MyEvent>, event: MyEvent) {
        vec.push(event);
    }
}

pub trait Layer {
    fn on_update(&mut self, events: &Vec<MyEvent>, elapsed: Duration);
}

impl<F> Layer for F
    where
        F: FnMut(&Vec<MyEvent>, Duration),
{
    fn on_update(&mut self, events: &Vec<MyEvent>, elapsed: Duration) {
        self(events, elapsed)
    }
}