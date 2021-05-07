
use winit::event::{DeviceEvent, Event};

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

pub fn handle_event(buffer: &mut Vec<winit::event::Event<()>>, event: winit::event::Event<()>) {
    match event {
        //forward
        Event::WindowEvent { .. } => { wrap(buffer, event) }
        //for raw mouse move
        Event::DeviceEvent { event: DeviceEvent::MouseMotion { .. }, .. } => { wrap(buffer, event) }
        Event::UserEvent(_) => { wrap(buffer, event) }
        //skip
        Event::NewEvents(_) => {}
        Event::Suspended => {}
        Event::Resumed => {}
        Event::MainEventsCleared => {}
        Event::RedrawRequested(_) => {}
        Event::RedrawEventsCleared => {}
        Event::LoopDestroyed => {}
        _ => {}
    }
}

fn wrap(buffer: &mut Vec<winit::event::Event<()>>, event: winit::event::Event<()>) {
    if let Some(e) = event.to_static().as_ref() {
        buffer.push(e.clone());
    }
}