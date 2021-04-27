use std::mem::ManuallyDrop;

use arrayvec::ArrayVec;
use hal::{
    Backend,
    device::Device,
    format::{ChannelType, Swizzle},
    image::{Extent, SubresourceRange, ViewKind},
    pool::CommandPool,
    pso::*, queue::*, window::*, window::Surface,
};
use hal::pass::{SubpassDependency};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::dpi::PhysicalSize;

use crate::graphics::hal_utils::DepthImage;
use crate::graphics::state::HalStateV2;

pub trait DeviceDrop<B: Backend> {
    unsafe fn manually_drop(&mut self, device: &B::Device);
}

pub struct CommonSwapchain<B: Backend> {
    current_frame: usize,
    pub(crate) img_count: usize,
    img_fences: Vec<B::Fence>,
    render_finished_semaphores: Vec<B::Semaphore>,
    image_available_semaphores: Vec<B::Semaphore>,
    command_buffers: Vec<B::CommandBuffer>,
    command_pool: ManuallyDrop<B::CommandPool>,
    queue_group: ManuallyDrop<QueueGroup<B>>,
    render_pass: ManuallyDrop<B::RenderPass>,

    swapchain_config: SwapchainConfig,
    base: BaseSwapchain<B>,
}

impl<B: Backend> DeviceDrop<B> for CommonSwapchain<B> {
    unsafe fn manually_drop(&mut self, device: &<B as Backend>::Device) {
        for fence in self.img_fences.drain(..) {
            device.destroy_fence(fence)
        }
        for sp in self.render_finished_semaphores.drain(..) {
            device.destroy_semaphore(sp)
        }
        for sp in self.image_available_semaphores.drain(..) {
            device.destroy_semaphore(sp)
        }
        for buff in self.command_buffers.drain(..) {
            self.command_pool.free(vec![buff])
        }
        self.base.manually_drop(device);
        use std::ptr::read;
        device.destroy_command_pool(ManuallyDrop::into_inner(read(&mut self.command_pool)));
        ManuallyDrop::drop(&mut self.queue_group);
        device.destroy_render_pass(ManuallyDrop::into_inner(read(&mut self.render_pass)));
    }
}

pub struct BaseSwapchain<B: Backend> {
    img_count: usize,
    framebuffers: Vec<B::Framebuffer>,
    image_views: Vec<B::ImageView>,
    depth_images: Vec<DepthImage<B>>,
    swapchain: ManuallyDrop<B::Swapchain>,
    extent: Extent2D,
}

impl<B: Backend> DeviceDrop<B> for BaseSwapchain<B> {
    unsafe fn manually_drop(&mut self, device: &<B as Backend>::Device) {
        for fb in self.framebuffers.drain(..) {
            device.destroy_framebuffer(fb);
        }
        for iv in self.image_views.drain(..) {
            device.destroy_image_view(iv);
        }
        for di in self.depth_images.drain(..) {
            di.manually_drop(device);
        }
        use std::ptr::read;
        device.destroy_swapchain(ManuallyDrop::into_inner(read(&mut self.swapchain)));
    }
}

impl<B: Backend> BaseSwapchain<B> {
    fn pop_old_swapchain(&mut self, device: &<B as Backend>::Device) -> B::Swapchain {
        unsafe {
            for fb in self.framebuffers.drain(..) {
                device.destroy_framebuffer(fb);
            }
            for iv in self.image_views.drain(..) {
                device.destroy_image_view(iv);
            }
            for di in self.depth_images.drain(..) {
                di.manually_drop(device);
            }
        }
        use std::ptr::read;
        unsafe { ManuallyDrop::into_inner(read(&mut self.swapchain)) }
    }

    fn new(
        state: &mut HalStateV2<B>,
        render_pass: &B::RenderPass,
        config: SwapchainConfig,
        old_chain: Option<B::Swapchain>,
    ) -> Result<Self, &'static str> {
        let (swapchain, extent, backbuffer, config) = {
            let SurfaceCapabilities { current_extent: _, .. } =
                state._surface.capabilities(&state._adapter.physical_device);
            let extent = config.extent;
            let swapchain_config = SwapchainConfig { extent, ..config };
            info!("Swapchain config: {:?}", swapchain_config);
            let (swapchain, backbuffer) = if let Some(old) = old_chain {
                unsafe {
                    info!("Recreating swapchain");
                    state
                        .device
                        .create_swapchain(&mut state._surface, swapchain_config, Some(old))
                        .map_err(|_| "Failed to create the swapchain!")?
                }
            } else {
                unsafe {
                    state
                        .device
                        .create_swapchain(&mut state._surface, swapchain_config, None)
                        .map_err(|_| "Failed to create the swapchain!")?
                }
            };
            (swapchain, extent, backbuffer, config)
        };
        dbg!();
        let (image_views, depth_images, framebuffers) = {
            let image_views: Vec<<B as Backend>::ImageView> = {
                backbuffer
                    .into_iter()
                    .map(|image| unsafe {
                        state
                            .device
                            .create_image_view(
                                &image,
                                ViewKind::D2,
                                config.format,
                                Swizzle::NO,
                                SubresourceRange {
                                    aspects: hal::format::Aspects::COLOR,
                                    levels: 0..1,
                                    layers: 0..1,
                                },
                            )
                            .map_err(|_| "Couldn't create the image_view for the image!")
                    })
                    .collect::<Result<Vec<_>, &str>>()?
            };
            let depth_images = image_views
                .iter()
                .map(|_| DepthImage::new(&state._adapter, &state.device, extent))
                .collect::<Result<Vec<_>, &str>>()?;

            let framebuffers: Vec<<B as Backend>::Framebuffer> = {
                image_views
                    .iter()
                    .zip(depth_images.iter())
                    .map(|(image_view, depth_image_view)| unsafe {
                        let attachments: ArrayVec<[_; 2]> =
                            [image_view, &depth_image_view.image_view].into();
                        state
                            .device
                            .create_framebuffer(
                                &render_pass,
                                attachments,
                                Extent {
                                    width: extent.width as u32,
                                    height: extent.height as u32,
                                    depth: 1,
                                },
                            )
                            .map_err(|_| "Failed to create a framebuffer!")
                    })
                    .collect::<Result<Vec<_>, &str>>()?
            };
            (image_views, depth_images, framebuffers)
        };

        Ok(Self {
            swapchain: ManuallyDrop::new(swapchain),
            image_views,
            depth_images,
            framebuffers,
            img_count: config.image_count as usize,
            extent,
        })
    }
}

impl<B: Backend> CommonSwapchain<B> {
    pub fn reset_inner(&mut self, state: &mut HalStateV2<B>, size: PhysicalSize<u32>) -> Result<(), &'static str> {
        for fence in self.img_fences.iter() {
            unsafe {
                state.device
                    .wait_for_fence(fence, core::u64::MAX)
                    .map_err(|_| "Failed to wait on the fence!")?;
            };
        }

        let swapchain = &mut self.base;
        self.swapchain_config.extent = Extent2D {
            width: size.width as u32,
            height: size.height as u32,
        };
        let old = swapchain.pop_old_swapchain(&state.device);
        self.base = BaseSwapchain::new(state, &self.render_pass, self.swapchain_config.clone(), Some(old))?;
        info!("New extent: {:?}", self.base.extent);
        Ok(())
    }

    fn create_render_pass<'a>(
        device: &'a <B as Backend>::Device,
        format: hal::format::Format,
    ) -> Result<B::RenderPass, &'static str> {
        //todo move desc's upper
        use hal::image::Layout;
        use hal::pass::{
            Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, SubpassDesc,
        };
        let color_attachment = Attachment {
            format: Some(format),
            samples: 1,
            ops: AttachmentOps {
                load: AttachmentLoadOp::Clear,
                store: AttachmentStoreOp::Store,
            },
            stencil_ops: AttachmentOps::DONT_CARE,
            layouts: Layout::Undefined..Layout::Present,
        };
        let depth_attachment = Attachment {
            format: Some(hal::format::Format::D32Sfloat),
            samples: 1,
            ops: AttachmentOps {
                load: AttachmentLoadOp::Clear,
                store: AttachmentStoreOp::DontCare,
            },
            stencil_ops: AttachmentOps::DONT_CARE,
            layouts: Layout::Undefined..Layout::DepthStencilAttachmentOptimal,
        };
        //pre frag stage check
        use hal::image::Access;
        use hal::memory::Dependencies;
        let in_dependency = SubpassDependency {
            passes: None..Some(0),
            stages: PipelineStage::COLOR_ATTACHMENT_OUTPUT
                ..PipelineStage::COLOR_ATTACHMENT_OUTPUT | PipelineStage::EARLY_FRAGMENT_TESTS,
            accesses: Access::empty()
                ..(Access::COLOR_ATTACHMENT_READ
                | Access::COLOR_ATTACHMENT_WRITE
                | Access::DEPTH_STENCIL_ATTACHMENT_READ
                | Access::DEPTH_STENCIL_ATTACHMENT_WRITE),
            flags: Dependencies::empty(),
        };
        let out_dependency = SubpassDependency {
            passes: Some(0)..None,
            stages: PipelineStage::COLOR_ATTACHMENT_OUTPUT | PipelineStage::EARLY_FRAGMENT_TESTS
                ..PipelineStage::COLOR_ATTACHMENT_OUTPUT,
            accesses: (Access::COLOR_ATTACHMENT_READ
                | Access::COLOR_ATTACHMENT_WRITE
                | Access::DEPTH_STENCIL_ATTACHMENT_READ
                | Access::DEPTH_STENCIL_ATTACHMENT_WRITE)..Access::empty(),
            flags: Dependencies::empty(),
        };

        let subpass = SubpassDesc {
            colors: &[(0, Layout::ColorAttachmentOptimal)],
            depth_stencil: Some(&(1, Layout::DepthStencilAttachmentOptimal)),
            inputs: &[],
            resolves: &[],
            preserves: &[],
        };
        Ok(unsafe {
            device
                .create_render_pass(
                    &[color_attachment, depth_attachment],
                    &[subpass],
                    &[in_dependency, out_dependency],
                )
                .map_err(|_| "Couldn't create a render pass!")?
        })
    }

    pub fn current_extent(&self) -> Extent2D {
        self.base.extent
    }

    pub fn render_pass(&self) -> &B::RenderPass {
        &self.render_pass
    }

    pub fn new<'a>(
        state: &'a mut HalStateV2<B>,
        queue_group: QueueGroup<B>,
    ) -> Result<Self, &'static str> {
        let swapchain_config = {
            let SurfaceCapabilities {
                image_count,
                usage,
                present_modes,
                composite_alpha_modes,
                current_extent,
                ..
            } = state._surface.capabilities(&state._adapter.physical_device);
            let formats = state
                ._surface
                .supported_formats(&state._adapter.physical_device);
            info!("present modes: {:?}", present_modes);
            info!("formats {:?}", formats);

            let present_mode = {
                [
                    PresentMode::MAILBOX,
                    PresentMode::FIFO,
                    PresentMode::IMMEDIATE,
                    PresentMode::RELAXED,
                ]
                    .iter()
                    .cloned()
                    .find(|pm| present_modes.contains(*pm))
                    .ok_or("No PresentMode valuesmut specified!")?
            };

            info!("Selected present mode: {:?}", present_mode);
            let composite_alpha_mode = {
                [
                    CompositeAlphaMode::OPAQUE,
                    CompositeAlphaMode::INHERIT,
                    CompositeAlphaMode::PREMULTIPLIED,
                    CompositeAlphaMode::POSTMULTIPLIED,
                ]
                    .iter()
                    .cloned()
                    .find(|ca| composite_alpha_modes.contains(*ca))
                    .ok_or("No CompositeAlpha values specified!")?
            };

            info!("Selected composite alpha mode: {:?}", composite_alpha_mode);
            let format = formats.map_or(hal::format::Format::Rgba8Srgb, |formats| {
                formats
                    .iter()
                    .find(|format| format.base_format().1 == ChannelType::Srgb)
                    .map(|format| *format)
                    .unwrap_or(formats[0])
            });
            info!("Selected format mode: {:?}", format);
            let image_count = if present_mode == hal::window::PresentMode::MAILBOX {
                (image_count.end() - 1).min(*image_count.start().max(&3))
            } else {
                (image_count.end() - 1).min(*image_count.start().max(&2))
            };
            info!("Image count: {:?}", image_count);
            let image_layers = 1;
            info!("Image layers: {:?}", image_layers);
            let image_usage = if usage.contains(hal::image::Usage::COLOR_ATTACHMENT) {
                hal::image::Usage::COLOR_ATTACHMENT
            } else {
                Err("The Surface isn't capable of supporting color!")?
            };
            info!("Image usage: {:?}", image_usage);
            let extent = match current_extent {
                None => Extent2D {
                    width: 600,
                    height: 400,
                },
                Some(e) => e,
            };
            SwapchainConfig {
                present_mode,
                composite_alpha_mode,
                format,
                extent,
                image_count,
                image_layers,
                image_usage: usage,
            }
        };

        let render_pass = Self::create_render_pass(&state.device, swapchain_config.format)?;

        let base = BaseSwapchain::new(state, &render_pass, swapchain_config.clone(), None)?;
        let (image_available_semaphores, render_finished_semaphores, swapchain_img_fences) = {
            let mut image_available_semaphores: Vec<<B as Backend>::Semaphore> = vec![];
            let mut render_finished_semaphores: Vec<<B as Backend>::Semaphore> = vec![];
            let mut swapchain_img_fences: Vec<<B as Backend>::Fence> = vec![];
            for _ in 0..base.img_count {
                swapchain_img_fences.push(
                    state
                        .device
                        .create_fence(true)
                        .map_err(|_| "Could not create a fence!")?,
                );
                image_available_semaphores.push(
                    state
                        .device
                        .create_semaphore()
                        .map_err(|_| "Could not create a semaphore!")?,
                );
                render_finished_semaphores.push(
                    state
                        .device
                        .create_semaphore()
                        .map_err(|_| "Could not create a semaphore!")?,
                );
            }
            (
                image_available_semaphores,
                render_finished_semaphores,
                swapchain_img_fences,
            )
        };

        let (command_pool, command_buffers) = {
            let mut command_pool = unsafe {
                state
                    .device
                    .create_command_pool(
                        queue_group.family,
                        hal::pool::CommandPoolCreateFlags::RESET_INDIVIDUAL,
                    )
                    .map_err(|_| "Could not create the raw command pool!")?
            };
            let command_buffers: Vec<<B as Backend>::CommandBuffer> = {
                (0..base.img_count)
                    .map(|_| unsafe { command_pool.allocate_one(hal::command::Level::Primary) })
                    .collect()
            };
            (command_pool, command_buffers)
        };
        Ok(Self {
            current_frame: 0,
            img_count: base.img_count,
            img_fences: swapchain_img_fences,
            render_finished_semaphores,
            image_available_semaphores,
            command_buffers,
            queue_group: ManuallyDrop::new(queue_group),
            command_pool: ManuallyDrop::new(command_pool),
            base,
            render_pass: ManuallyDrop::new(render_pass),
            swapchain_config,
        })
    }

    pub fn next_frame(
        &mut self,
        device: &B::Device,
    ) -> Result<
        (
            usize,
            &mut B::CommandBuffer,
            &B::Framebuffer,
            &B::RenderPass,
        ),
        &str,
    > {
        let image_available = &self.image_available_semaphores[self.current_frame];

        let (_i_u32, i_usize) = unsafe {
            let image_index = self
                .base
                .swapchain
                .acquire_image(core::u64::MAX, Some(image_available), None)
                .map_err(|e| {
                    error!("{:?}", e);
                    "Couldn't acquire an image from the swapchain!"
                })?;
            let a = image_index.0.clone();
            (image_index, a as usize)
        };

        let flight_fence = &self.img_fences[i_usize];
        unsafe {
            device
                .wait_for_fence(flight_fence, core::u64::MAX)
                .map_err(|_| "Failed to wait on the fence!")?;
            device
                .reset_fence(flight_fence)
                .map_err(|_| "Couldn't reset the fence!")?;
        };
        Ok((
            i_usize,
            &mut self.command_buffers[i_usize],
            &self.base.framebuffers[i_usize],
            &self.render_pass,
        ))
    }

    pub fn present_buffer(&mut self, frame: usize) -> Result<(), &str> {
        let image_available = &self.image_available_semaphores[self.current_frame];
        let render_finished = &self.render_finished_semaphores[self.current_frame];
        self.current_frame = (self.current_frame + 1) % self.img_count;
        let flight_fence = &self.img_fences[frame];

        let command_buffers = &self.command_buffers[frame..=frame];
        let wait_semaphores: ArrayVec<[_; 1]> = [(
            image_available,
            hal::pso::PipelineStage::COLOR_ATTACHMENT_OUTPUT,
        )]
            .into();
        let signal_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
        let present_wait_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
        let submission = Submission {
            command_buffers,
            wait_semaphores,
            signal_semaphores,
        };
        let the_command_queue = &mut self.queue_group.queues[0];
        unsafe {
            the_command_queue.submit(submission, Some(flight_fence));
            self.base
                .swapchain
                .present(the_command_queue, frame as u32, present_wait_semaphores)
                .map_err(|_| "Failed to present into the swapchain!")
        }?;
        Ok(())
    }
}
