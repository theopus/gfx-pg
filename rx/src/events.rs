use winit::event::{DeviceEvent, DeviceId, ElementState, Event, KeyboardInput, MouseButton, WindowEvent};

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