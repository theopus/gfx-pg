use std::time::{Duration, Instant};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::event::{DeviceEvent, DeviceId, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::window::{Window, WindowId};

use crate::glm::e;
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
    pub fn run(mut self) {
        let WinitState {
            events_loop,
            window,
        } = self.winit_state;


        let mut layers = self.layers;
        let mut renderer = self.renderer;
        let mut events = Vec::with_capacity(300);
        let mut last = Instant::now();

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
                Event::MainEventsCleared => {
                    let current = Instant::now();
                    let elapsed = current - last;
                    Self::on_update(&mut layers, &mut events, elapsed);
                    window.request_redraw();
                    last = current
                }
                Event::RedrawRequested(_) => {
                    /*Render*/
                    renderer.render();
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(phys_size),
                    ..
                } => {
                    renderer._cam.update(&o_event);
                    renderer.reset_swapchain(phys_size);

                    let owned = Self::map_event(o_event);
                    if let Some(e) = owned {
                        Self::on_event(&mut events, e);
                    }
                }
                _ => {
                    let owned = Self::map_event(o_event);
                    if let Some(e) = owned {
                        Self::on_event(&mut events, e);
                    }
                },
            }
        };
        events_loop.run(run_loop);
    }


    fn on_update(layers: &mut Vec<Box<dyn Layer>>, events: &Vec<Event<()>>, elapsed: Duration) {
        for layer in layers.iter_mut() {
            layer.on_update(events, elapsed);
        }
    }

    pub fn push_layer<L>(&mut self, layer: L) where L: Layer + 'static {
        self.layers.push(Box::new(layer));
    }


    //TODO: find out adequate solution
    fn map_event<'a, 'b>(src: Event<'a, ()>) -> Option<Event<'b, ()>> {
        match src {
            Event::WindowEvent {
                window_id,
                event,
                ..
            } => {
                match event {
                    WindowEvent::Resized(_) => Engine::map_window_event(window_id, event),
                    WindowEvent::KeyboardInput { .. } => Engine::map_window_event(window_id, event),
                    _ => None
                }
            },
            Event::DeviceEvent {
                device_id,
                event
            } => {
                match event {
//                    DeviceEvent::MouseMotion { .. } => Engine::map_device_event(device_id, event),
                    _ => None
                }
            }
            _ => None
        }
    }


    fn map_window_event<'a, 'b>(window_id: WindowId, event: WindowEvent) -> Option<Event<'b, ()>> {
        Some(Event::WindowEvent {
            window_id,
            event: event.to_static().unwrap(),
        })
    }

    fn map_device_event<'a, 'b>(device_id: DeviceId, event: DeviceEvent) -> Option<Event<'b, ()>> {
        Some(Event::DeviceEvent {
            device_id,
            event: event.clone(),
        })
    }

    fn on_event<'a>(vec: &mut Vec<Event<'a, ()>>, event: Event<'a, ()>) {
        vec.push(event);
    }
}

pub trait Layer {
    fn on_update(&mut self, events: &Vec<Event<()>>, elapsed: Duration);
}

impl<F> Layer for F
where
    F: FnMut(&Vec<Event<()>>, Duration),
{
    fn on_update(&mut self, events: &Vec<Event<()>>, elapsed: Duration) {
        self(events, elapsed)
    }
}