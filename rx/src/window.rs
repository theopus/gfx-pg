use winit::{
    dpi::LogicalSize,
    error::OsError,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::events::EngEvent;

#[derive(Debug)]
pub struct WinitState<T: 'static + Send + Clone> {
    pub events_loop: EventLoop<T>,
    pub window: Option<Window>,
    pub window_builder: Option<WindowBuilder>,
}

impl<T: Send + Clone> WinitState<T> {
    pub fn new<S: Into<String>>(title: S, size: LogicalSize<u32>) -> Result<Self, OsError> {
        let events_loop = EventLoop::with_user_event();

        let output = WindowBuilder::new()
            .with_decorations(true)
            .with_resizable(true)
            .with_transparent(false)
            .with_title(title)
            .with_inner_size(size);
        let option = Some(output.build(&events_loop).unwrap());
        Ok(Self {
            events_loop,
            window: option,
            window_builder: None,
        })
    }
}

pub const WINDOW_NAME: &str = "Sample window";

impl<T: Send + Clone> Default for WinitState<T> {
    fn default() -> Self {
        Self::new(
            WINDOW_NAME,
            LogicalSize {
                width: 800,
                height: 600,
            },
        )
            .expect("Could not create a window!")
    }
}

#[derive(Debug, Clone, Default)]
pub struct UserInput {
    pub end_requested: bool,
    pub new_frame_size: Option<(u32, u32)>,
    pub new_mouse_position: Option<(i32, i32)>,
}

impl UserInput {
    pub fn poll_events_loop(event: &Event<()>) -> Self {
        let mut output = UserInput::default();
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => output.end_requested = true,
            Event::WindowEvent {
                event: WindowEvent::Resized(logical),
                ..
            } => {
                output.new_frame_size = Some((logical.width, logical.height));
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                output.new_mouse_position = Some((position.x as i32, position.y as i32));
            }
            _ => (),
        };
        output
    }
}
