use std::ops::Deref;

use hal::{device::Device, Backend};
use winit::window::Window;

use crate::graphics::memory::MemoryManager;
use crate::graphics::pipelines::PipelineV0;
use crate::graphics::state::HalStateV2;
use crate::graphics::swapchain::{CommonSwapchain, DeviceDrop};
use hal::window::Extent2D;

pub struct ApiWrapper<B: Backend> {
    pub(crate) hal_state: HalStateV2<B>,
    pub(crate) swapchain: CommonSwapchain<B>,
    pub(crate) pipeline: PipelineV0<B>,
    pub(crate) storage: MemoryManager<B>,
}

impl<B: Backend> Drop for ApiWrapper<B> {
    fn drop(&mut self) {
        let _ = self.hal_state.device_ref().wait_idle();
        unsafe {
            self.pipeline.manually_drop(&self.hal_state.device);
            self.storage.manually_drop(&self.hal_state.device);
            self.swapchain.manually_drop(&self.hal_state.device);
        }
    }
}

impl<B: Backend> ApiWrapper<B> {
    pub fn next_frame(
        &mut self,
    ) -> Result<
        (
            usize,
            &mut B::CommandBuffer,
            &B::Framebuffer,
            &B::RenderPass,
            &MemoryManager<B>,
            &PipelineV0<B>
        ),
        &str,
    > {
        let (o, r ,t ,y) = self.swapchain.next_frame(&self.hal_state.device)?;
        Ok((o, r ,t ,y, &self.storage, &self.pipeline))
    }
    pub fn present_buffer(&mut self, present: usize) -> Result<(), &str> {
        self.swapchain.present_buffer(present)
    }

    pub fn new(window: &Window, instance: B::Instance, surface: B::Surface) -> Result<Self, &str> {
        let (mut hal_state, mut queue_group) = HalStateV2::new(window, instance, surface)?;

        let storage = unsafe { MemoryManager::new(&hal_state) }?;
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
            storage,
        })
    }
}
