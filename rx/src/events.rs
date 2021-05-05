use winit::dpi::PhysicalPosition;
use winit::event::{DeviceEvent, DeviceId, ElementState, Event, KeyboardInput, MouseButton, WindowEvent};


#[derive(Debug, Clone)]
pub enum RxEvent<T: 'static + Send + Clone> {
    TestEvent,
    ClientEvent(T)
}

pub type WinitEvent<T> = winit::event::Event<'static, RxEvent<T>>;


pub fn handle_event<'a, T: Clone + Send>(buffer: &mut Vec<WinitEvent<T>>, event: winit::event::Event<'a, RxEvent<T>>) {
    match event {
        //ignore
        Event::NewEvents(_) => {}
        Event::UserEvent(_) => {}
        Event::Suspended => {}
        Event::Resumed => {}
        Event::MainEventsCleared => {}
        Event::RedrawRequested(_) => {}
        Event::RedrawEventsCleared => {}
        Event::LoopDestroyed => {}
        Event::DeviceEvent { .. } => {
            if let Some(e) = event.to_static().as_ref() {
                buffer.push(e.clone());
            }
        }
        //forward
        Event::WindowEvent { .. } => {
            if let Some(e) = event.to_static().as_ref() {
                buffer.push(e.clone());
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum MyEvent {
    Resized(u32, u32),
    KeyboardInput {
        device_id: DeviceId,
        input: KeyboardInput,
        is_synthetic: bool,
    },
    MouseMotion {
        delta: (f64, f64),
    },
    MouseInput {
        device_id: DeviceId,
        state: ElementState,
        button: MouseButton,
    },
    CursorMoved {
        device_id: DeviceId,
        position: PhysicalPosition<f64>,
    },
}

//TODO: find out adequate solution
pub fn map_event(src: Event<()>) -> Option<MyEvent> {
    match src {
        Event::WindowEvent {
            event,
            ..
        } => {
            match event {
                WindowEvent::Resized(sz) =>
                    Some(MyEvent::Resized(sz.width as u32, sz.height as u32)),
                WindowEvent::KeyboardInput {
                    device_id, input, is_synthetic
                } => Some(MyEvent::KeyboardInput {
                    device_id,
                    input,
                    is_synthetic,
                }),
                WindowEvent::MouseInput {
                    device_id,
                    state,
                    button,
                    ..
                } => Some(MyEvent::MouseInput {
                    device_id,
                    state,
                    button,
                }),
                WindowEvent::CursorMoved {
                    device_id, position, ..
                } => Some(MyEvent::CursorMoved {
                    device_id,
                    position,
                }),
                _ => None
            }
        }
        Event::DeviceEvent {
            event,
            ..
        } => {
            match event {
                DeviceEvent::MouseMotion {
                    delta
                } => Some(
                    MyEvent::MouseMotion {
                        delta
                    }
                ),
                _ => None
            }
        }
        _ => None
    }
}