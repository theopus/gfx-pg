use std::ops::Deref;

use hal::Backend;
use winit::window::Window;

use crate::graphics::pipelines::PipelineV0;
use crate::graphics::state::HalStateV2;
use crate::graphics::swapchain::{CommonSwapchain, DeviceDrop};

pub struct ApiWrapper<B: Backend> {
    hal_state: HalStateV2<B>,
    swapchain: CommonSwapchain<B>,
    pipeline: PipelineV0<B>,
}

impl<B: Backend> Drop for ApiWrapper<B> {
    fn drop(&mut self) {
        unsafe {
            self.pipeline.drop(&self.hal_state.device);
            self.swapchain.drop(&self.hal_state.device);
        }
    }
}

impl<B: Backend> ApiWrapper<B> {
    fn new(window: &Window, instance: B::Instance, surface: B::Surface) -> Result<Self, &str> {
        let (mut hal_state, mut queue_group) = HalStateV2::new(window, instance, surface)?;

        let mut swapchain = CommonSwapchain::new(&mut hal_state, queue_group)?;
        let pipeline = PipelineV0::new(
            hal_state.device_ref(),
            swapchain.current_extent(),
            swapchain.render_pass(),
        )?;
        Ok(Self {
            hal_state,
            swapchain,
            pipeline,
        })
    }
}
