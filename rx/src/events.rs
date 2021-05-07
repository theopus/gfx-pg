
use winit::event::{DeviceEvent, Event};

#[derive(Debug, Clone)]
pub enum RxEvent<T: 'static + Send + Clone> {
    None,
    ClientEvent(T),
    WinitEvent(winit::event::Event<'static, ()>)
}

pub type WinitEvent<T> = winit::event::Event<'static, RxEvent<T>>;

pub fn handle_event<T: Clone + Send>(buffer: &mut Vec<RxEvent<T>>, event: winit::event::Event<()>) {
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

fn wrap<T: Clone + Send>(buffer: &mut Vec<RxEvent<T>>, event: winit::event::Event<()>) {
    if let Some(e) = event.to_static().as_ref() {
        buffer.push(RxEvent::WinitEvent(e.clone()));
    }
}