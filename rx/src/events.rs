
use winit::event::{DeviceEvent, Event};
use crossbeam_channel::Sender;
use crate::crossbeam_channel::TrySendError;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};


#[derive(Debug, Clone)]
pub enum EngEvent<T: 'static + Send + Clone> {
    TestEvent,
    ClientEvent(T),
}

#[derive(Debug, Clone)]
pub enum RxEvent<T: 'static + Send + Clone> {
    TestEvent,
    ClientEvent(T),
    WinitEvent(WinitEvent)
}

pub type WinitEvent = winit::event::Event<'static, ()>;

pub fn handle_event<T: 'static + Send + Clone>(sender: &Sender<RxEvent<T>>, buffer: &mut Vec<winit::event::Event<()>>, event: winit::event::Event<()>) {
    let e = match event {
        //forward
        Event::WindowEvent { .. } => { wrap(event) }
        //for raw mouse move
        Event::DeviceEvent { event: DeviceEvent::MouseMotion { .. }, .. } => { wrap(event) }
        Event::UserEvent(_) => { wrap(event)}
        //skip
        Event::NewEvents(_) => None,
        Event::Suspended => None,
        Event::Resumed => None,
        Event::MainEventsCleared => None,
        Event::RedrawRequested(_) => None,
        Event::RedrawEventsCleared => None,
        Event::LoopDestroyed => None,
        _ => None
    };
    if let Some(w_e) = e {
        match sender.try_send(RxEvent::WinitEvent(w_e)) {
            Err(_) => warn!("Throttled event."),
            _ => {}
        }
    }
}

fn wrap(event: winit::event::Event<()>) -> Option<Event<'static, ()>> {
    event.to_static().clone()
}