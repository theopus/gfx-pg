#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::window::winit::event::{Event, WindowEvent};
use crate::window::winit::event_loop::{
    ControlFlow,
    EventLoop,
};
use crate::window::WinitState;

pub struct Engine {
    winit_state: WinitState,
    layers: Vec<Box<dyn Layer>>,
}


impl Default for Engine {
    fn default() -> Self {
        Self {
            winit_state: Default::default(),
            layers: Default::default(),
        }
    }
}

impl Engine {
    pub fn run(mut self) {
        let WinitState { events_loop, window } = self.winit_state;
        let mut layers = self.layers;

        events_loop.run(move |event, _, control_flow| {
            //Always poll
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    info!("The close button was pressed; stopping");
                    *control_flow = ControlFlow::Exit
                }
                Event::MainEventsCleared => {
                    Self::on_update(&mut layers);
                    // Queue a RedrawRequested event.
                    window.request_redraw();
                }
                Event::RedrawRequested(_) => { /*Render*/ }
                Event::LoopDestroyed => { /*Onclose*/ }
                _ => ()
            }
        });
    }

    pub fn push_layer<L>(&mut self, layer: L) where L: Layer + 'static {
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

impl<F> Layer for F where F: FnMut()  {
    fn on_update(&mut self) {
        self()
    }
}

