use std::time::{Duration, Instant, SystemTime};

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
        let mut winit_state: WinitState = Default::default();
        let renderer = Renderer::new(&mut winit_state).unwrap();
        Self {
            winit_state,
            layers: Default::default(),
            renderer,
        }
    }
}

struct InstantStopwatch {
    time: SystemTime
}


impl InstantStopwatch {
    pub fn new() -> Result<Self, &'static str> {
        Ok(Self { time: std::time::UNIX_EPOCH })
    }
}


impl Stopwatch for InstantStopwatch {
    fn start(&mut self) {
        self.time = SystemTime::now();
    }

    fn elapsed(&mut self) -> Duration {
        let current = SystemTime::now();

        let elapsed = current.duration_since(self.time).expect("");
        self.time = current;
        elapsed
    }
}


#[cfg(target_arch = "wasm32")]
pub mod websys_timer {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use crate::run::Stopwatch;

    pub struct WebSysStopwatch {
        time: std::time::SystemTime
    }

    impl WebSysStopwatch {
        pub fn new() -> Result<Self, &'static str> {
            Ok(Self { time: std::time::UNIX_EPOCH })
        }
    }

    impl Stopwatch for WebSysStopwatch {
        fn start(&mut self) {
            let window = web_sys::window().expect("");
            let performance = window.performance().expect("");
            self.time = to_system_time(performance.now())
        }

        fn elapsed(&mut self) -> Duration {
            let window = web_sys::window().expect("");
            let performance = window.performance().expect("");
            let current = to_system_time(performance.now());

            let elapsed = current.duration_since(self.time).expect("");
            self.time = current;
            elapsed
        }
    }

    fn to_system_time(amt: f64) -> SystemTime {
        let secs = (amt as u64) / 1_000;
        let nanos = ((amt as u32) % 1_000) * 1_000_000;
        UNIX_EPOCH + Duration::new(secs, nanos)
    }
}

pub trait Stopwatch {
    fn start(&mut self);
    fn elapsed(&mut self) -> Duration;
}

impl Engine {
    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }
    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }
    pub fn loader(&mut self) -> (&mut ApiWrapper<back::Backend>, &mut Option<AssetsLoader>, &mut AssetsStorage) {
        (&mut self.renderer.api, &mut self.renderer.loader, &mut self.renderer.storage)
    }

    pub fn run(self) -> Result<(), &'static str> {
        let (
            events_loop,
            window
        ) = {
            let WinitState {
                events_loop,
                window,
                ..
            } = self.winit_state;
            (events_loop, window.unwrap())
        };


        let mut layers = self.layers;
        let mut renderer = self.renderer;
        let mut events: Vec<MyEvent> = Vec::new();
         Self::on_event(&mut events, MyEvent::Resized(800,600));
        #[cfg(target_arch = "wasm32")]
            let mut timer = websys_timer::WebSysStopwatch::new()?;
        #[cfg(not(target_arch = "wasm32"))]
            let mut timer = InstantStopwatch::new()?;

        timer.start();


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
//                    {
//                        assert!(draw_req == 1, "Draw requests: {:?}", draw_req);
//                        draw_req -= 1;
//                    }
                    renderer.render();
                }
                Event::MainEventsCleared => {
//                    let current = Instant::now();
//                    let elapsed = current - last;
                    Self::on_update(&mut layers, &mut events, timer.elapsed());
                    window.request_redraw();
                    //[BUG#windows]: winit
//                    draw_req += 1;
//                    last = current
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(phys_size),
                    ..
                } => {
                    warn!("Resized {:?}", phys_size);

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