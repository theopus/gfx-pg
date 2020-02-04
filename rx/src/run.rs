#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::render::Renderer;
use crate::window::WinitState;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

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

        info!("Start!");
        events_loop.run(move |event, _, control_flow| {
            //Always poll
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    info!("The close button was pressed; stopping");
                    *control_flow = ControlFlow::Exit
                }
                Event::MainEventsCleared => {
                    Self::on_update(&mut layers);
                    // Queue a RedrawRequested event.
                    window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    /*Render*/
                    renderer.render();
                }
                Event::LoopDestroyed => {
                    info!("On close handle")
                    /*On close*/
                }
                _ => (),
            }
        });
    }

    pub fn push_layer<L>(&mut self, layer: L)
    where
        L: Layer + 'static,
    {
        self.layers.push(Box::new(layer));
    }

    fn on_update(layers: &mut Vec<Box<dyn Layer>>) {
        for layer in layers.iter_mut() {
            layer.on_update();
        }
    }
}

pub trait Layer {
    fn on_update(&mut self);
}

impl<F> Layer for F
where
    F: FnMut(),
{
    fn on_update(&mut self) {
        self()
    }
}
