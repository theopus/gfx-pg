use std::hash::Hasher;

use back;
use gfx_hal::Instance;
use hal::{
    command,
    command::ClearValue,
    command::CommandBuffer,
    pso::{Rect, Viewport},
};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::graphics::api::HalState;
use crate::graphics::state::HalStateV2;
use crate::graphics::wrapper::ApiWrapper;
use crate::utils::Camera;

pub type DrawCmd = (u32, glm::Mat4);

pub struct Renderer {
    api: ApiWrapper<back::Backend>,
    _cam: Camera,
}

impl Renderer {
    pub fn new(window: &winit::window::Window) -> Result<Self, &str> {
        Ok(Self {
            api: ApiWrapper::typed(window)?,
            _cam: Default::default(),
        })
    }

    pub fn render(&mut self) {
        let ex = self.api.swapchain.current_extent();
        let frame = {
            let (frame, buffer, fb, render_pass) = self.api.next_frame().expect("");
            //lmao dude move this outta my eyes
            unsafe {
                const TRIANGLE_CLEAR: [ClearValue; 2] = [
                    command::ClearValue {
                        color: command::ClearColor {
                            float32: [0.1, 0.2, 0.3, 1.0],
                        },
                    },
                    command::ClearValue {
                        depth_stencil: command::ClearDepthStencil {
                            depth: 1.0,
                            stencil: 0,
                        },
                    },
                ];

                let render_area = Rect {
                    x: 0,
                    y: 0,
                    w: ex.width as i16,
                    h: ex.height as i16,
                };
                let viewport = Viewport {
                    rect: render_area,
                    depth: (0.0..1.0),
                };

                buffer.begin_primary(command::CommandBufferFlags::empty());
                buffer.set_viewports(0, &[viewport]);
                buffer.set_scissors(0, &[render_area]);
                buffer.begin_render_pass(
                    &render_pass,
                    &fb,
                    render_area,
                    TRIANGLE_CLEAR.iter(),
                    command::SubpassContents::Inline,
                );
                buffer.end_render_pass();
                buffer.finish();
            }

            frame
        };

        self.api.present_buffer(frame).expect("");
    }
}

#[cfg(not(feature = "gl"))]
impl ApiWrapper<back::Backend> {
    pub fn typed(window: &winit::window::Window) -> Result<Self, &str> {
        let instance =
            back::Instance::create("gfx-rs quad", 1).expect("Failed to create an instance!");
        info!("{:?}", instance);
        let mut surface = unsafe {
            instance
                .create_surface(window)
                .expect("Failed to create a surface!")
        };
        info!("{:?}", surface);
        ApiWrapper::new(window, instance, surface)
    }
}

#[cfg(feature = "gl")]
impl ApiWrapper<back::Backend> {
    pub fn typed(window: &winit::window::Window) -> Result<Self, &str> {
        let builder =
            back::config_context(back::glutin::ContextBuilder::new(), ColorFormat::SELF, None)
                .with_vsync(true);
        builder.build_kek();
        let surface = back::Surface::from_context(context);
        info!("{:?}", instance);
        info!("{:?}", surface);
        ApiWrapper::new(window, instance, surface)
    }
}

fn do_the_render(
    hal_state: &mut HalState<back::Backend>,
    cam: &Camera,
) -> Result<(), &'static str> {
    let mtx = glm::translate(&glm::identity(), &glm::vec3(0., 0., -30.));
    hal_state.draw_quad_frame(
        crate::utils::Quad {
            x: -0.5 as f32,
            y: -0.5 as f32,
            w: 1 as f32,
            h: 1 as f32,
        },
        cam,
        &mtx,
    )
}
