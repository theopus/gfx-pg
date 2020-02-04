use gfx_hal::Instance;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::graphics::api::HalState;
use crate::utils::Camera;
use back;

pub type DrawCmd = (u32, glm::Mat4);

pub struct Renderer {
    hal_state: HalState<back::Backend>,
    _cam: Camera,
}

impl Renderer {
    pub fn new(window: &winit::window::Window) -> Result<Self, &str> {
        Ok(Self {
            hal_state: HalState::typed(window)?,
            _cam: Default::default()
        })
    }

    pub fn render(&mut self) {
        do_the_render(&mut self.hal_state, &self._cam).unwrap()
    }
}

#[cfg(not(feature = "gl"))]
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

#[cfg(feature = "gl")]
impl HalState<back::Backend> {
    pub fn typed(window: &winit::window::Window) -> Result<Self, &str> {
        let builder = back::config_context(back::glutin::ContextBuilder::new(), ColorFormat::SELF, None)
            .with_vsync(true);
        builder.build_kek();
        let surface = back::Surface::from_context(context);
        info!("{:?}", instance);
        info!("{:?}", surface);
        HalState::new(window, instance, surface)
    }
}


fn do_the_render(hal_state: &mut HalState<back::Backend>, cam: &Camera) -> Result<(), &'static str> {
    let mtx = glm::translate(&glm::identity(), &glm::vec3(0.,0.,-30.));
    hal_state.draw_quad_frame(crate::utils::Quad {
        x: -0.5 as f32,
        y: -0.5 as f32,
        w: 1 as f32,
        h: 1 as f32,
    }, cam, &mtx)
}