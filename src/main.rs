extern crate gfx_hal as hal;
extern crate nalgebra_glm as glm;

use std::cell::RefCell;

use env_logger;
//#[cfg(feature = "dx12")]
//use gfx_backend_dx12 as back;
//#[cfg(feature = "gl")] //INFO: gl requires specific initialisation.
//use gfx_backend_gl as back;
//#[cfg(feature = "metal")]
//use gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as back;
use gfx_hal::Instance;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::api::HalState;
use crate::local::LocalState;
use crate::utils::{Camera, Quad, Triangle};
use crate::window::WinitState;

mod window;
mod api;
mod local;
mod utils;

//fn do_the_render<'a>(hal_state: &'a mut HalState, local_state: &'a LocalState) -> Result<(), &'a str> {
//    let r = (local_state.mouse_x as f32 / local_state.frame_width as f32) as f32;
//    let g = (local_state.mouse_y as f32 / local_state.frame_height as f32) as f32;
//    let b = (r + g) * 0.3;
//    let a = 1.0;
//    let color = [r, g, b, a];
//    hal_state.draw_clear_frame(color)
//}

impl HalState<back::Backend> {
    pub fn typed(window: &winit::window::Window) -> Result<Self, &str> {
        let instance = back::Instance::create("gfx-rs quad", 1)
            .expect("Failed to create an instance!");
        info!("{:?}", instance);
        let mut surface = unsafe {
            instance.create_surface(window).expect("Failed to create a surface!")
        };
        info!("{:?}", surface);
        HalState::new(window, instance, surface)
    }
}

fn do_the_render(hal_state: &mut HalState<back::Backend>, local_state: &LocalState, cam: &Camera) -> Result<(), &'static str> {
    let r = (local_state.mouse_x as f32 / local_state.frame_width as f32) as f32;
    let g = (local_state.mouse_y as f32 / local_state.frame_height as f32) as f32;
    let b = (r + g) * 0.3;

    let x = ((local_state.mouse_x as f64 / local_state.frame_width as f64) * 2.0) - 1.0;
    let y = ((local_state.mouse_y as f64 / local_state.frame_height as f64) * 2.0) - 1.0;
    let triangle = Triangle {
        points: [[-0.5, 0.5], [-0.5, -0.5], [x as f32, y as f32]],
        colors: [
            [r / local_state.mouse_x as f32 - 0.1, g, b],
            [r, g/ local_state.mouse_x as f32, b],
            [r, g, b/ local_state.mouse_x as f32],
        ],
    };
    hal_state.draw_quad_frame(crate::utils::Quad {
        x: -0.5 as f32,
        y: -0.5 as f32,
        w: 1 as f32,
        h: 1 as f32,
    }, cam)
}

fn main() {
    env_logger::from_env(
        env_logger::Env::default()
            .default_filter_or("info,winit::platform_impl::platform::event_loop::runner=error,gfx_backend_vulkan=info"))
        .init();
    let WinitState { events_loop, window } = WinitState::default();

    let mut local_state = local::LocalState {
        frame_width: window.inner_size().width,
        frame_height: window.inner_size().height,
        mouse_x: 0,
        mouse_y: 0,
    };


    let mut hal_state: RefCell<Option<HalState<back::Backend>>> = RefCell::new(None);

    let mut camera: RefCell<Option<Camera>> = RefCell::new(None);

    events_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;


        let input = window::UserInput::poll_events_loop(&event);
        if input.new_frame_size.is_some() {
            info!("Resize occurred");
            drop(hal_state.replace(None).unwrap());
            drop(camera.replace(None).unwrap());
        }

        if hal_state.borrow().is_none() {
            info!("Creating state.");
            hal_state.replace(Some(HalState::typed(&window).expect("")));
            let size = window.inner_size();
            camera.replace(Some(Camera::default_with_aspect(size.width as f32 / size.height as f32)));
        }
        local_state.update_from_input(input);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            }
            Event::MainEventsCleared => {
                // Application update code.

                // Queue a RedrawRequested event.
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                do_the_render(hal_state.borrow_mut().as_mut().unwrap(), &local_state, camera.borrow().as_ref().unwrap());
            }
            _ => ()
        }
    });
}
