use winit::dpi::PhysicalPosition;
use winit::event::{DeviceEvent, DeviceId, ElementState, Event, KeyboardInput, MouseButton, WindowEvent};

#[derive(Debug, Clone)]
pub enum RxEvent<T: 'static + Send + Clone> {
    TestEvent,
    ClientEvent(T),
}

pub type WinitEvent<T> = winit::event::Event<'static, RxEvent<T>>;

pub fn handle_event<T: Clone + Send>(buffer: &mut Vec<WinitEvent<T>>, event: winit::event::Event<RxEvent<T>>) {
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

fn wrap<T: Clone + Send>(buffer: &mut Vec<Event<RxEvent<T>>>, event: Event<RxEvent<T>>) {
    if let Some(e) = event.to_static().as_ref() {
        buffer.push(e.clone());
    }
}