use winit::{
    dpi::LogicalSize,
    error::OsError,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

#[derive(Debug)]
pub struct WinitState {
    pub events_loop: EventLoop<()>,
    pub window: Window,
}

impl WinitState {
    pub fn new<T: Into<String>>(title: T, size: LogicalSize<u32>) -> Result<Self, OsError> {
        let events_loop = EventLoop::new();

        let output = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(size)
            .build(&events_loop);
        output.map(|window| Self {
            events_loop,
            window,
        })
    }
}

pub const WINDOW_NAME: &str = "Sample window";

impl Default for WinitState {
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
                output.new_mouse_position = Some((position.x, position.y));
            }
            _ => (),
        };
        output
    }
}
