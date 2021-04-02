use hal::{Backend, device::Device};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::graphics::memory::MemoryManager;
use crate::graphics::pipelines::PipelineV0;
use crate::graphics::state::HalStateV2;
use crate::graphics::swapchain::{CommonSwapchain, DeviceDrop};
use crate::hal::adapter::Adapter;

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
            &PipelineV0<B>,
            &HalStateV2<B>
        ),
        &str,
    > {
        let (o, r, t, y) = self.swapchain.next_frame(&self.hal_state.device)?;
        Ok((o, r, t, y, &self.storage, &self.pipeline, &self.hal_state))
    }
    pub fn present_buffer(&mut self, present: usize) -> Result<(), &str> {
        self.swapchain.present_buffer(present)
    }
    pub fn reset_swapchain(&mut self, size: PhysicalSize<u32>) -> Result<(), &str> {
        self.swapchain.reset_inner(&mut self.hal_state, size)
    }

    pub fn new(window: &Window, instance: Option<B::Instance>, surface: B::Surface, adapters: Vec<Adapter<B>>) -> Result<Self, &'static str> {
        let (mut hal_state, queue_group) = HalStateV2::new(window, instance, surface, adapters)?;

        let swapchain = CommonSwapchain::new(&mut hal_state, queue_group)?;
        let storage = unsafe { MemoryManager::new(&hal_state, swapchain.img_count as u32) }?;

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
