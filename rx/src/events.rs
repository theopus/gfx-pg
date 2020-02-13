use winit::event::{DeviceEvent, DeviceId, Event, KeyboardInput, WindowEvent};
use winit::window::WindowId;

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
}

//TODO: find out adequate solution
pub fn map_event(src: Event<()>) -> Option<MyEvent> {
    match src {
        Event::WindowEvent {
            window_id,
            event,
            ..
        } => {
            match event {
                WindowEvent::Resized(sz) =>
                    Some(MyEvent::Resized(sz.width, sz.height)),
                WindowEvent::KeyboardInput {
                    device_id, input, is_synthetic
                } => Some(MyEvent::KeyboardInput {
                    device_id,
                    input,
                    is_synthetic,
                }),
                _ => None
            }
        }
        Event::DeviceEvent {
            device_id,
            event
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