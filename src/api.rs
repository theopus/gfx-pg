use std::{iter, ptr};
use std::hash::Hasher;
use std::io::Cursor;
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, size_of};
use std::ops::Deref;
use std::ptr::read;
use std::time::Instant;

use arrayvec::ArrayVec;
use hal::{adapter::{Adapter, Gpu, PhysicalDevice}, Backend, buffer::*, command, command::*, command::CommandBuffer, device::Device, format::{ChannelType, Swizzle}, image::{Extent, SubresourceRange, ViewKind}, IndexType, Instance, Limits, memory::*, MemoryTypeId, pass::Subpass, pool::CommandPool, pso::*, queue::*, queue::QueueType::Graphics, window::*, window::Surface};
use hal::pass::{SubpassDependency, SubpassRef};
use log::{debug, error, info, trace, warn};
use winit::window::Window;
use crate::utils::cast_slice;

pub struct DepthImage<B: Backend> {
    pub image: ManuallyDrop<B::Image>,
    pub requirements: Requirements,
    pub memory: ManuallyDrop<B::Memory>,
    pub image_view: ManuallyDrop<B::ImageView>,
}

pub struct LoadedImage<B: Backend> {
    pub image: ManuallyDrop<B::Image>,
    pub requirements: Requirements,
    pub memory: ManuallyDrop<B::Memory>,
    pub image_view: ManuallyDrop<B::ImageView>,
    pub sampler: ManuallyDrop<B::Sampler>,
}


pub struct BufferBundle<B: Backend> {
    pub buffer: ManuallyDrop<B::Buffer>,
    pub requirements: Requirements,
    pub memory: ManuallyDrop<B::Memory>,
    pub phantom: PhantomData<B::Device>,
}


pub struct HalState<B: Backend> {
    creation_instant: Instant,
    vertices: BufferBundle<B>,
    indexes: BufferBundle<B>,
    texture: LoadedImage<B>,
    descriptor_set: ManuallyDrop<B::DescriptorSet>,
    descriptor_pool: ManuallyDrop<B::DescriptorPool>,
    descriptor_set_layouts: Vec<B::DescriptorSetLayout>,
    pipeline_layout: ManuallyDrop<B::PipelineLayout>,
    graphics_pipeline: ManuallyDrop<B::GraphicsPipeline>,
    //
    current_frame: usize,
    swapchain_img_count: usize,
    swapchain_img_fences: Vec<B::Fence>,
    render_finished_semaphores: Vec<B::Semaphore>,
    image_available_semaphores: Vec<B::Semaphore>,
    command_buffers: Vec<B::CommandBuffer>,
    command_pool: ManuallyDrop<B::CommandPool>,
    framebuffers: Vec<B::Framebuffer>,
    image_views: Vec<(B::ImageView)>,
    depth_images: Vec<(DepthImage<B>)>,
    render_pass: ManuallyDrop<B::RenderPass>,
    render_area: Rect,
    queue_group: ManuallyDrop<QueueGroup<B>>,
    swapchain: ManuallyDrop<B::Swapchain>,
    device: ManuallyDrop<B::Device>,
    _adapter: hal::adapter::Adapter<B>,
    _surface: ManuallyDrop<B::Surface>,
    _instance: ManuallyDrop<B::Instance>,
}


impl<B: Backend> DepthImage<B> {
    pub fn new(adapter: &Adapter<B>, device: &B::Device, extent: Extent2D) -> Result<Self, &'static str> {
        unsafe {
            use hal::format::Format;
            use hal::format::Aspects;
            let mut the_image = device
                .create_image(
                    hal::image::Kind::D2(extent.width, extent.height, 1, 1),
                    1,
                    Format::D32Sfloat,
                    hal::image::Tiling::Optimal,
                    hal::image::Usage::DEPTH_STENCIL_ATTACHMENT,
                    hal::image::ViewCapabilities::empty(),
                )
                .map_err(|_| "Couldn't crate the image!")?;
            let requirements = device.get_image_requirements(&the_image);
            let memory_type_id = adapter
                .physical_device
                .memory_properties()
                .memory_types
                .iter()
                .enumerate()
                .find(|&(id, memory_type)| {
                    // BIG NOTE: THIS IS DEVICE LOCAL NOT CPU VISIBLE
                    requirements.type_mask & (1 << id) != 0
                        && memory_type.properties.contains(Properties::DEVICE_LOCAL)
                })
                .map(|(id, _)| MemoryTypeId(id))
                .ok_or("Couldn't find a memory type to support the image!")?;

            let memory = device
                .allocate_memory(memory_type_id, requirements.size)
                .map_err(|_| "Couldn't allocate image memory!")?;
            device
                .bind_image_memory(&memory, 0, &mut the_image)
                .map_err(|_| "Couldn't bind the image memory!")?;
            let image_view = device
                .create_image_view(
                    &the_image,
                    hal::image::ViewKind::D2,
                    Format::D32Sfloat,
                    hal::format::Swizzle::NO,
                    SubresourceRange {
                        aspects: Aspects::DEPTH,
                        levels: 0..1,
                        layers: 0..1,
                    },
                )
                .map_err(|_| "Couldn't create the image view!")?;
            Ok(Self {
                image: ManuallyDrop::new(the_image),
                requirements,
                memory: ManuallyDrop::new(memory),
                image_view: ManuallyDrop::new(image_view),
            })
        }
    }

    pub unsafe fn manually_drop(&self, device: &B::Device) {
        use core::ptr::read;
        device.destroy_image_view(ManuallyDrop::into_inner(read(&self.image_view)));
        device.destroy_image(ManuallyDrop::into_inner(read(&self.image)));
        device.free_memory(ManuallyDrop::into_inner(read(&self.memory)));
    }
}

impl<B: Backend> LoadedImage<B> {
    pub fn new(
        adapter: &Adapter<B>,
        device: &B::Device,
        command_pool: &mut <B as Backend>::CommandPool,
        command_queue: &mut <B as Backend>::CommandQueue,
        img: image::RgbaImage,
    ) -> Result<Self, &'static str> {
        use hal::image as i;
        unsafe {
            // 0. First we compute some memory related values.
            let pixel_size = size_of::<image::Rgba<u8>>();
            let row_size = pixel_size * (img.width() as usize);
            let limits: Limits = adapter.physical_device.limits();
            let row_alignment_mask = limits.optimal_buffer_copy_pitch_alignment as u32 - 1;
            let row_pitch = ((row_size as u32 + row_alignment_mask) & !row_alignment_mask) as usize;
            debug_assert!(row_pitch as usize >= row_size);

            // 1. make a staging buffer with enough memory for the image, and a
            //    transfer_src usage
            let required_bytes = row_pitch * img.height() as usize;
            let staging_bundle =
                BufferBundle::new(&adapter, device, required_bytes, Usage::TRANSFER_SRC)?;

            // 2. use mapping writer to put the image data into that buffer
            let mem_ref = staging_bundle.mem_ref();
            let mut writer = device
                .map_memory(mem_ref, 0..staging_bundle.requirements.size)
                .map_err(|_| "Couldn't acquire a mapping writer to the staging buffer!")?;

            for y in 0..img.height() as usize {
                let row = &(*img)[y * row_size..(y + 1) * row_size];
                ptr::copy_nonoverlapping(
                    row.as_ptr(),
                    writer.offset(y as isize * row_pitch as isize),
                    row_size,
                );
            }
            device.flush_mapped_memory_ranges(iter::once((mem_ref, 0..staging_bundle.requirements.size)))
                .map_err(|_| "Couldn't flush the memory range!")?;
            device.unmap_memory(mem_ref);

            // 3. Make an image with transfer_dst and SAMPLED usage
            let mut the_image = device
                .create_image(
                    hal::image::Kind::D2(img.width(), img.height(), 1, 1),
                    1,
                    hal::format::Format::Rgba8Srgb,
                    hal::image::Tiling::Optimal,
                    hal::image::Usage::TRANSFER_DST | hal::image::Usage::SAMPLED,
                    hal::image::ViewCapabilities::empty(),
                )
                .map_err(|_| "Couldn't create the image!")?;

            // 4. allocate memory for the image and bind it
            let requirements = device.get_image_requirements(&the_image);
            let memory_type_id = adapter
                .physical_device
                .memory_properties()
                .memory_types
                .iter()
                .enumerate()
                .find(|&(id, memory_type)| {
                    // BIG NOTE: THIS IS DEVICE LOCAL NOT CPU VISIBLE
                    requirements.type_mask & (1 << id) != 0
                        && memory_type.properties.contains(Properties::DEVICE_LOCAL)
                })
                .map(|(id, _)| MemoryTypeId(id))
                .ok_or("Couldn't find a memory type to support the image!")?;
            let memory = device
                .allocate_memory(memory_type_id, requirements.size)
                .map_err(|_| "Couldn't allocate image memory!")?;
            device
                .bind_image_memory(&memory, 0, &mut the_image)
                .map_err(|_| "Couldn't bind the image memory!")?;

            // 5. create image view and sampler
            let image_view = device
                .create_image_view(
                    &the_image,
                    hal::image::ViewKind::D2,
                    hal::format::Format::Rgba8Srgb,
                    hal::format::Swizzle::NO,
                    SubresourceRange {
                        aspects: hal::format::Aspects::COLOR,
                        levels: 0..1,
                        layers: 0..1,
                    },
                )
                .map_err(|_| "Couldn't create the image view!")?;
            let sampler = device
                .create_sampler(&hal::image::SamplerDesc::new(
                    hal::image::Filter::Nearest,
                    hal::image::WrapMode::Tile,
                ))
                .map_err(|_| "Couldn't create the sampler!")?;
            // 6. create a command buffer
            let mut cmd_buffer: B::CommandBuffer = command_pool.allocate_one(command::Level::Primary);
            cmd_buffer.begin_primary(command::CommandBufferFlags::ONE_TIME_SUBMIT);
            // 7. Use a pipeline barrier to transition the image from empty/undefined
            //    to TRANSFER_WRITE/TransferDstOptimal
            use hal::image::Layout;
            let image_barrier = hal::memory::Barrier::Image {
                states: (hal::image::Access::empty(), Layout::Undefined)
                    ..(
                    hal::image::Access::TRANSFER_WRITE,
                    Layout::TransferDstOptimal,
                ),
                target: &the_image,
                families: None,
                range: SubresourceRange {
                    aspects: hal::format::Aspects::COLOR,
                    levels: 0..1,
                    layers: 0..1,
                },
            };
            cmd_buffer.pipeline_barrier(
                PipelineStage::TOP_OF_PIPE..PipelineStage::TRANSFER,
                hal::memory::Dependencies::empty(),
                &[image_barrier],
            );
            // 8. perform copy from staging buffer to image
            cmd_buffer.copy_buffer_to_image(
                &staging_bundle.buffer,
                &the_image,
                Layout::TransferDstOptimal,
                &[hal::command::BufferImageCopy {
                    buffer_offset: 0,
                    buffer_width: (row_pitch / pixel_size) as u32,
                    buffer_height: img.height(),
                    image_layers: hal::image::SubresourceLayers {
                        aspects: hal::format::Aspects::COLOR,
                        level: 0,
                        layers: 0..1,
                    },
                    image_offset: hal::image::Offset { x: 0, y: 0, z: 0 },
                    image_extent: hal::image::Extent {
                        width: img.width(),
                        height: img.height(),
                        depth: 1,
                    },
                }],
            );
            // 9. use pipeline barrier to transition the image to SHADER_READ access/
            //    ShaderReadOnlyOptimal layout
            let image_barrier = hal::memory::Barrier::Image {
                states: (
                    hal::image::Access::TRANSFER_WRITE,
                    Layout::TransferDstOptimal,
                )
                    ..(
                    hal::image::Access::SHADER_READ,
                    Layout::ShaderReadOnlyOptimal,
                ),
                target: &the_image,
                families: None,
                range: SubresourceRange {
                    aspects: hal::format::Aspects::COLOR,
                    levels: 0..1,
                    layers: 0..1,
                },
            };
            cmd_buffer.pipeline_barrier(
                PipelineStage::TRANSFER..PipelineStage::FRAGMENT_SHADER,
                hal::memory::Dependencies::empty(),
                &[image_barrier],
            );
            // 10. Submit the cmd buffer to queue and wait for it
            cmd_buffer.finish();
            let upload_fence = device
                .create_fence(false)
                .map_err(|_| "Couldn't create an upload fence!")?;
            command_queue.submit_without_semaphores(Some(&cmd_buffer), Some(&upload_fence));
            device
                .wait_for_fence(&upload_fence, core::u64::MAX)
                .map_err(|_| "Couldn't wait for the fence!")?;
            device.destroy_fence(upload_fence);
            // 11. Destroy the staging bundle and one shot buffer now that we're done
            staging_bundle.manually_drop(device);
            command_pool.free(Some(cmd_buffer));
            Ok(Self {
                image: ManuallyDrop::new(the_image),
                requirements,
                memory: ManuallyDrop::new(memory),
                image_view: ManuallyDrop::new(image_view),
                sampler: ManuallyDrop::new(sampler),
            })
        }
    }

    pub unsafe fn manually_drop(&self, device: &B::Device) {
        use core::ptr::read;
        device.destroy_sampler(ManuallyDrop::into_inner(read(&self.sampler)));
        device.destroy_image_view(ManuallyDrop::into_inner(read(&self.image_view)));
        device.destroy_image(ManuallyDrop::into_inner(read(&self.image)));
        device.free_memory(ManuallyDrop::into_inner(read(&self.memory)));
    }
}

impl<B: Backend> BufferBundle<B> {
    pub fn mem_ref(&self) -> &B::Memory {
        &self.memory
    }

    pub fn new(adapter: &Adapter<B>, device: &B::Device, size: usize, usage: Usage) -> Result<Self, &'static str> {
        unsafe {
            let mut buffer = device
                .create_buffer(size as u64, usage)
                .map_err(|_| "Couldn't create a buffer!")?;
            let requirements = device.get_buffer_requirements(&buffer);
            let memory_type_id = adapter
                .physical_device
                .memory_properties()
                .memory_types
                .iter()
                .enumerate()
                .find(|&(id, memory_type)| {
                    requirements.type_mask & (1 << id) != 0 && memory_type.properties.contains(Properties::CPU_VISIBLE)
                })
                .map(|(id, _)| MemoryTypeId(id))
                .ok_or("Couldn't find a memory type to support the buffer!")?;
            let memory = device
                .allocate_memory(memory_type_id, requirements.size)
                .map_err(|_| "Couldn't allocate buffer memory!")?;
            device
                .bind_buffer_memory(&memory, 0, &mut buffer)
                .map_err(|_| "Couldn't bind the buffer memory!")?;
            Ok(Self {
                buffer: ManuallyDrop::new(buffer),
                requirements,
                memory: ManuallyDrop::new(memory),
                phantom: PhantomData,
            })
        }
    }

    pub unsafe fn manually_drop(&self, device: &B::Device) {
        use core::ptr::read;
        device.destroy_buffer(ManuallyDrop::into_inner(read(&self.buffer)));
        device.free_memory(ManuallyDrop::into_inner(read(&self.memory)));
    }
}

impl<B> Drop for HalState<B> where B: Backend {
    fn drop(&mut self) {
        let _ = self.device.wait_idle();
        unsafe {
            self.vertices.manually_drop(&self.device);
            self.indexes.manually_drop(&self.device);
            self.texture.manually_drop(&self.device);

            self.descriptor_pool.free_sets(Some(ManuallyDrop::into_inner(read(&mut self.descriptor_set))));
            self.device.destroy_descriptor_pool(ManuallyDrop::into_inner(read(&mut self.descriptor_pool)));

            for dsl in self.descriptor_set_layouts.drain(..) {
                self.device.destroy_descriptor_set_layout(dsl);
            }
            self.device.destroy_pipeline_layout(ManuallyDrop::into_inner(read(&mut self.pipeline_layout)));
            self.device.destroy_graphics_pipeline(ManuallyDrop::into_inner(read(&mut self.graphics_pipeline)));
            for fence in self.swapchain_img_fences.drain(..) {
                self.device.destroy_fence(fence)
            }
            for sp in self.render_finished_semaphores.drain(..) {
                self.device.destroy_semaphore(sp)
            }
            for sp in self.image_available_semaphores.drain(..) {
                self.device.destroy_semaphore(sp)
            }
            for fb in self.framebuffers.drain(..) {
                self.device.destroy_framebuffer(fb);
            }
            for iv in self.image_views.drain(..) {
                self.device.destroy_image_view(iv);
            }
            for di in self.depth_images.drain(..) {
                di.manually_drop(&self.device);
            }
            self
                .device
                .destroy_command_pool(ManuallyDrop::into_inner(read(&mut self.command_pool)));
            self
                .device
                .destroy_render_pass(ManuallyDrop::into_inner(read(&mut self.render_pass)));
            self
                .device
                .destroy_swapchain(ManuallyDrop::into_inner(read(&mut self.swapchain)));
            ManuallyDrop::drop(&mut self.queue_group);
            ManuallyDrop::drop(&mut self.device);

            let surface = ManuallyDrop::into_inner(ptr::read(&self._surface));
            self._instance.destroy_surface(surface);
            ManuallyDrop::drop(&mut self._instance);
        }
    }
}


pub const VERTEX_SOURCE: &'static str = include_str!("shaders/one.vert");

pub const FRAGMENT_SOURCE: &'static str = include_str!("shaders/one.frag");


impl<B: Backend> HalState<B> {
    pub fn new(window: &Window, instance: <B as Backend>::Instance, mut surface: B::Surface) -> Result<Self, &'static str> {
        let adapter = instance
            .enumerate_adapters()
            .into_iter()
            .find(|a| {
                a.queue_families
                    .iter()
                    .any(|qf| qf.queue_type().supports_graphics() && surface.supports_queue_family(qf))
            })
            .ok_or("Couldn't find a graphical Adapter!")?;
        info!("{:?}", adapter);
        //device stuff
        let (mut device, mut queue_group) = {
            let queue_family = adapter
                .queue_families
                .iter()
                .find(|qf| qf.queue_type().supports_graphics() && surface.supports_queue_family(qf))
                .ok_or("Couldn't find a QueueFamily with graphics!")?;

            let Gpu { device, mut queue_groups } = unsafe {
                adapter
                    .physical_device
                    .open(&[(&queue_family, &[1.0; 1])], hal::Features::empty())
                    .map_err(|_| "Couldn't open the PhysicalDevice!")?
            };

            let queue_group = queue_groups.pop().unwrap();
            let _ = if queue_group.queues.len() > 0 {
                Ok(())
            } else {
                Err("The QueueGroup did not have any CommandQueues available!")
            }?;
            (device, queue_group)
        };
        //swapchain stuff
        let (swapchain, extent, backbuffer, format, swapchain_img_count) = {
            let SurfaceCapabilities {
                image_count,
                current_extent,
                usage,
                present_modes,
                composite_alpha_modes,
                ..
            } = surface.capabilities(&adapter.physical_device);
            let formats = surface.supported_formats(&adapter.physical_device);
            info!("present modes: {:?}", present_modes);
            info!("formats {:?}", formats);

            let present_mode = {
                use hal::window::PresentMode;
                [PresentMode::MAILBOX, PresentMode::IMMEDIATE, PresentMode::FIFO, PresentMode::RELAXED]
                    .iter()
                    .cloned()
                    .find(|pm| present_modes.contains(*pm))
                    .ok_or("No PresentMode values specified!")?
            };

            info!("Selected present mode: {:?}", present_mode);
            let composite_alpha_mode = {
                use hal::window::CompositeAlphaMode;
                [CompositeAlphaMode::OPAQUE,
                    CompositeAlphaMode::INHERIT,
                    CompositeAlphaMode::PREMULTIPLIED,
                    CompositeAlphaMode::POSTMULTIPLIED]
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
            let extent = match current_extent {
                None => Extent2D {
                    width: 600,
                    height: 400,
                },
                Some(e) => e,
            };
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

            let swapchain_config = SwapchainConfig {
                present_mode,
                composite_alpha_mode,
                format,
                extent,
                image_count,
                image_layers,
                image_usage,
            };
            info!("extent {:?}", extent);
            info!("Swapchain config: {:?}", swapchain_config);
            let (swapchain, backbuffer) = unsafe {
                device
                    .create_swapchain(&mut surface, swapchain_config, None)
                    .map_err(|_| "Failed to create the swapchain!")?
            };
            (swapchain, extent, backbuffer, format, image_count as usize)
        };
        //synchronization objects
        let (image_available_semaphores, render_finished_semaphores, swapchain_img_fences) = {
            let mut image_available_semaphores: Vec<<B as Backend>::Semaphore> = vec![];
            let mut render_finished_semaphores: Vec<<B as Backend>::Semaphore> = vec![];
            let mut swapchain_img_fences: Vec<<B as Backend>::Fence> = vec![];
            for _ in 0..swapchain_img_count {
                swapchain_img_fences.push(device.create_fence(true).map_err(|_| "Could not create a fence!")?);
                image_available_semaphores.push(device.create_semaphore().map_err(|_| "Could not create a semaphore!")?);
                render_finished_semaphores.push(device.create_semaphore().map_err(|_| "Could not create a semaphore!")?);
            }
            (image_available_semaphores, render_finished_semaphores, swapchain_img_fences)
        };
        //render pass config
        let render_pass = {
            use hal::pass::{
                Attachment,
                AttachmentOps,
                AttachmentLoadOp,
                AttachmentStoreOp,
                SubpassDesc,
            };
            use hal::image::Layout;

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
            let in_dependency = SubpassDependency {
                passes: SubpassRef::External..SubpassRef::Pass(0),
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
                passes: SubpassRef::Pass(0)..SubpassRef::External,
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
            unsafe {
                device
                    .create_render_pass(
                        &[color_attachment, depth_attachment],
                        &[subpass],
                        &[in_dependency, out_dependency],
                    )
                    .map_err(|_| "Couldn't create a render pass!")?
            }
        };
        //image views

        let (image_views, depth_images, framebuffers) = {
            let image_views: Vec<<B as Backend>::ImageView> = {
                backbuffer.into_iter()
                    .map(|image| unsafe {
                        device
                            .create_image_view(
                                &image,
                                ViewKind::D2,
                                format,
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
                .map(|_| DepthImage::new(&adapter, &device, extent))
                .collect::<Result<Vec<_>, &str>>()?;

            let framebuffers: Vec<<B as Backend>::Framebuffer> = {
                image_views
                    .iter()
                    .zip(depth_images.iter())
                    .map(|(image_view, depth_image_view)| unsafe {
                        let attachments: ArrayVec<[_; 2]> = [image_view, &depth_image_view.image_view].into();
                        device
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
        use hal::pool::CommandPoolCreateFlags;
        let mut command_pool = unsafe {
            device
                .create_command_pool(
                    queue_group.family,
                    CommandPoolCreateFlags::RESET_INDIVIDUAL,
                )
                .map_err(|_| "Could not create the raw command pool!")?
        };
        let command_buffers: Vec<<B as Backend>::CommandBuffer> = {
            framebuffers
                .iter()
                .map(|_| unsafe { command_pool.allocate_one(hal::command::Level::Primary) })
                .collect()
        };
        let render_area: Rect = Rect {
            x: 0,
            y: 0,
            w: extent.width as i16,
            h: extent.height as i16,
        };
        let current_frame: usize = 0;
        info!("Extent: {:?}", extent);

        let (descriptor_set_layouts, desc_pool, desc_set, pipeline_layout, graphics_pipeline) = Self::create_pipeline(&mut device, extent, &render_pass)?;

        let (vertices, indexes) = unsafe {
            const F32_XY_RGB_UV_QUAD: usize = size_of::<f32>() * (2 + 3 + 2) * 4;
            let vertices = BufferBundle::new(&adapter, &device, F32_XY_RGB_UV_QUAD, Usage::VERTEX)?;

            const U16_QUAD_INDICES: usize = size_of::<u16>() * 2 * 3;
            let indexes = BufferBundle::new(&adapter, &device, U16_QUAD_INDICES, Usage::INDEX)?;


            unsafe {
                let mut data_target = device
                    .map_memory(indexes.mem_ref(), 0..indexes.requirements.size)
                    .map_err(|_| "Failed to acquire an index buffer mapping writer!")?;
                const INDEX_DATA: &[u16] = &[0, 1, 2, 2, 3, 0];
                ptr::copy(INDEX_DATA.as_ptr() as *const u8, data_target, INDEX_DATA.len() * size_of::<u16>());

                device.flush_mapped_memory_ranges(iter::once((indexes.mem_ref(), 0..indexes.requirements.size)))
                    .map_err(|_| "Couldn't flush the index buffer memory!")?;
                device.unmap_memory(indexes.mem_ref());
            }
            (vertices, indexes)
        };

        const CREATURE_BYTES: &[u8] = include_bytes!("data/image.png");

        let texture = LoadedImage::new(
            &adapter,
            &device,
            &mut command_pool,
            &mut queue_group.queues[0],
            image::load(Cursor::new(&CREATURE_BYTES[..]), image::PNG)
                .map_err(|_|"Binary corrupted!")?
                .to_rgba()
        )?;

        // 5. You write the descriptors into the descriptor set using
        //    write_descriptor_sets which you pass a set of DescriptorSetWrites
        //    which each write in one or more descriptors to the set
        unsafe {
            device.write_descriptor_sets(vec![
                hal::pso::DescriptorSetWrite {
                    set: &desc_set,
                    binding: 0,
                    array_offset: 0,
                    descriptors: Some(hal::pso::Descriptor::Image(
                        texture.image_view.deref(),
                        hal::image::Layout::ShaderReadOnlyOptimal,
                    )),
                },
                hal::pso::DescriptorSetWrite {
                    set: &desc_set,
                    binding: 1,
                    array_offset: 0,
                    descriptors: Some(hal::pso::Descriptor::Sampler(texture.sampler.deref())),
                },
            ]);
        }

        Ok(Self {
            creation_instant: Instant::now(),
            vertices,
            indexes,
            texture,
            descriptor_set: ManuallyDrop::new(desc_set),
            descriptor_pool: ManuallyDrop::new(desc_pool),
            descriptor_set_layouts,
            pipeline_layout: ManuallyDrop::new(pipeline_layout),
            graphics_pipeline: ManuallyDrop::new(graphics_pipeline),
            current_frame,
            swapchain_img_count,
            swapchain_img_fences,
            render_finished_semaphores,
            image_available_semaphores,
            command_buffers,
            command_pool: ManuallyDrop::new(command_pool),
            framebuffers,
            image_views,
            depth_images,
            render_area,
            render_pass: ManuallyDrop::new(render_pass),
            queue_group: ManuallyDrop::new(queue_group),
            swapchain: ManuallyDrop::new(swapchain),
            device: ManuallyDrop::new(device),
            _adapter: adapter,
            _surface: ManuallyDrop::new(surface),
            _instance: ManuallyDrop::new(instance),
        })
    }


    fn create_pipeline(
        device: &mut B::Device,
        extent: Extent2D,
        render_pass: &<B as Backend>::RenderPass,
    ) -> Result<
        (
            Vec<<B as Backend>::DescriptorSetLayout>,
            <B as Backend>::DescriptorPool,
            <B as Backend>::DescriptorSet,
            <B as Backend>::PipelineLayout,
            <B as Backend>::GraphicsPipeline,
        ),
        &'static str> {
        let mut compiler = shaderc::Compiler::new().ok_or("shaderc not found!")?;

        let vertex_compile_artifact = compiler
            .compile_into_spirv(
                VERTEX_SOURCE,
                shaderc::ShaderKind::Vertex,
                "vertex.vert",
                "main",
                None,
            )
            .map_err(|e| {
                error!("{}", e);
                "Couldn't compile vertex shader!"
            })?;
        let fragment_compile_artifact = compiler
            .compile_into_spirv(
                FRAGMENT_SOURCE,
                shaderc::ShaderKind::Fragment,
                "fragment.frag",
                "main",
                None,
            )
            .map_err(|e| {
                error!("{}", e);
                "Couldn't compile fragment shader!"
            })?;
        let vertex_shader_module = unsafe {
            device
                .create_shader_module(vertex_compile_artifact.as_binary())
                .map_err(|_| "Couldn't make the vertex module")?
        };
        let fragment_shader_module = unsafe {
            device
                .create_shader_module(fragment_compile_artifact.as_binary())
                .map_err(|_| "Couldn't make the fragment module")?
        };
        let (vs_entry, fs_entry): (EntryPoint<B>, EntryPoint<B>) = (
            EntryPoint {
                entry: "main",
                module: &vertex_shader_module,
                specialization: Specialization::default(),
            },
            EntryPoint {
                entry: "main",
                module: &fragment_shader_module,
                specialization: Specialization::default(),
            },
        );
        let shaders = GraphicsShaderSet {
            vertex: vs_entry,
            hull: None,
            domain: None,
            geometry: None,
            fragment: Some(fs_entry),
        };
        let vertex_buffers: Vec<VertexBufferDesc> = vec![VertexBufferDesc {
            binding: 0,
            stride: (size_of::<f32>() * (2 + 3 + 2)) as u32,
            rate: VertexInputRate::Vertex,
        }];
        let attributes: Vec<AttributeDesc> = vec![
            AttributeDesc {
                location: 0,
                binding: 0,
                element: Element {
                    format: hal::format::Format::Rg32Sfloat,
                    offset: 0,
                },
            },
            AttributeDesc {
                location: 1,
                binding: 0,
                element: Element {
                    format: hal::format::Format::Rgb32Sfloat,
                    offset: (size_of::<f32>() * 2) as u32,
                },
            },
            AttributeDesc {
                location: 2,
                binding: 0,
                element: Element {
                    format: hal::format::Format::Rg32Sfloat,
                    offset: (size_of::<f32>() * (2 + 3)) as u32,
                },
            }
        ];

        let input_assembler_desc = InputAssemblerDesc {
            primitive: Primitive::TriangleList,
            with_adjacency: false,
            restart_index: None,
        };

        let rasterizer = Rasterizer {
            polygon_mode: PolygonMode::Fill,
            cull_face: Face::NONE,
            front_face: FrontFace::Clockwise,
            depth_clamping: false,
            depth_bias: None,
            conservative: false,
        };

        let depth_stencil = DepthStencilDesc {
            depth: Some(DepthTest {
                fun: Comparison::LessEqual,
                write: true,
            }),
            depth_bounds: false,
            stencil: None,
        };

        let blender = {
            let blend_state = BlendState {
                color: BlendOp::Add {
                    src: Factor::One,
                    dst: Factor::Zero,
                },
                alpha: BlendOp::Add {
                    src: Factor::One,
                    dst: Factor::Zero,
                },
            };
            BlendDesc {
                logic_op: Some(LogicOp::Copy),
                targets: vec![ColorBlendDesc { mask: ColorMask::ALL, blend: Some(blend_state) }],
            }
        };

        let baked_states =
            BakedStates {
                viewport: Some(Viewport {
                    rect: extent.to_extent().rect(),
                    depth: (0.0..1.0),
                }),
                scissor: Some(extent.to_extent().rect()),
                blend_color: None,
                depth_bounds: None,
            };

        let bindings = Vec::<DescriptorSetLayoutBinding>::new();
        let immutable_samplers = Vec::<<B as Backend>::Sampler>::new();
        let descriptor_set_layouts: Vec<<B as Backend>::DescriptorSetLayout> = vec![unsafe {
            device
                .create_descriptor_set_layout(
                    &[
                        DescriptorSetLayoutBinding {
                            binding: 0,
                            ty: hal::pso::DescriptorType::SampledImage,
                            count: 1,
                            stage_flags: ShaderStageFlags::FRAGMENT,
                            immutable_samplers: false,
                        },
                        DescriptorSetLayoutBinding {
                            binding: 1,
                            ty: hal::pso::DescriptorType::Sampler,
                            count: 1,
                            stage_flags: ShaderStageFlags::FRAGMENT,
                            immutable_samplers: false,
                        },
                    ],
                    &[],
                )
                .map_err(|_| "Couldn't make a DescriptorSetLayout")?
        }];
        let mut descriptor_pool = unsafe {
            device
                .create_descriptor_pool(
                    1, // sets
                    &[
                        hal::pso::DescriptorRangeDesc {
                            ty: hal::pso::DescriptorType::SampledImage,
                            count: 1,
                        },
                        hal::pso::DescriptorRangeDesc {
                            ty: hal::pso::DescriptorType::Sampler,
                            count: 1,
                        },
                    ],
                    hal::pso::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET,
                )
                .map_err(|_| "Couldn't create a descriptor pool!")?
        };
        let descriptor_set = unsafe {
            descriptor_pool
                .allocate_set(&descriptor_set_layouts[0])
                .map_err(|_| "Couldn't make a Descriptor Set!")?
        };
//        let device: Device<B>;

        let push_constants = vec![
//            (ShaderStageFlags::FRAGMENT, 0..4),
            (ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT, 0..64),
        ];
        let layout = unsafe {
            device
                .create_pipeline_layout(&descriptor_set_layouts, push_constants)
                .map_err(|_| "Couldn't create a pipeline layout")?
        };

        let pipeline_desc = GraphicsPipelineDesc {
            shaders,
            rasterizer,
            vertex_buffers,
            attributes,
            input_assembler: input_assembler_desc,
            blender,
            depth_stencil,
            multisampling: None,
            baked_states,
            layout: &layout,
            subpass: Subpass {
                index: 0,
                main_pass: render_pass,
            },
            flags: PipelineCreationFlags::empty(),
            parent: BasePipeline::None,

        };
        let pipeline = unsafe {
            device
                .create_graphics_pipeline(&pipeline_desc, None)
                .map_err(|_| "Couldn't create a graphics pipeline!")?
        };
        unsafe { device.destroy_shader_module(vertex_shader_module) };
        unsafe { device.destroy_shader_module(fragment_shader_module) };
        Ok((descriptor_set_layouts, descriptor_pool, descriptor_set, layout, pipeline, ))
    }

    pub fn draw_clear_frame(&mut self, color: [f32; 4]) -> Result<(), &str> {
        // SETUP FOR THIS FRAME
        let image_available = &self.image_available_semaphores[self.current_frame];
        let render_finished = &self.render_finished_semaphores[self.current_frame];
        // Advance the frame _before_ we start using the `?` operator
        self.current_frame = (self.current_frame + 1) % self.swapchain_img_count;

        let (i_u32, i_usize) = unsafe {
            let image_index = self
                .swapchain
                .acquire_image(core::u64::MAX, Some(image_available), None)
                .map_err(|_| "Couldn't acquire an image from the swapchain!")?;
            let a = image_index.0.clone();
            (image_index, a as usize)
        };

        let flight_fence = &self.swapchain_img_fences[i_usize];
        unsafe {
            self.device
                .wait_for_fence(flight_fence, core::u64::MAX)
                .map_err(|_| "Failed to wait on the fence!")?;
            self.device
                .reset_fence(flight_fence)
                .map_err(|_| "Couldn't reset the fence!")?;
        }

        // RECORD COMMANDS
        unsafe {
            let buffer = &mut self.command_buffers[i_usize];
            let clear_values = [command::ClearValue {
                color: command::ClearColor {
                    float32: color,
                },
            }];
            buffer.begin_primary(command::CommandBufferFlags::empty());
            buffer.begin_render_pass(
                &self.render_pass,
                &self.framebuffers[i_usize],
                self.render_area,
                clear_values.iter(),
                command::SubpassContents::Inline,
            );
            buffer.end_render_pass();
            buffer.finish();
        }

        // SUBMISSION AND PRESENT
        let command_buffers = &self.command_buffers[i_usize..=i_usize];
        let wait_semaphores: ArrayVec<[_; 1]> =
            [(image_available, hal::pso::PipelineStage::COLOR_ATTACHMENT_OUTPUT)].into();
        let signal_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
        // yes, you have to write it twice like this. yes, it's silly.
        let present_wait_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
        let submission = Submission {
            command_buffers,
            wait_semaphores,
            signal_semaphores,
        };
        let the_command_queue = &mut self.queue_group.queues[0];
        unsafe {
            the_command_queue.submit(submission, Some(flight_fence));
            self.swapchain
                .present(the_command_queue, i_u32.0, present_wait_semaphores)
                .map_err(|_| "Failed to present into the swapchain!")
        };
        Ok(())
    }

    pub fn draw_quad_frame(&mut self, quad: crate::utils::Quad, cam: &crate::utils::Camera) -> Result<(), &'static str> {
        let duration = Instant::now().duration_since(self.creation_instant);
        let time_f32 = duration.as_secs() as f32 + duration.subsec_nanos() as f32 * 1e-9;

        // SETUP FOR THIS FRAME
        let image_available = &self.image_available_semaphores[self.current_frame];
        let render_finished = &self.render_finished_semaphores[self.current_frame];
        // Advance the frame _before_ we start using the `?` operator
        self.current_frame = (self.current_frame + 1) % self.swapchain_img_count;

        let (i_u32, i_usize) = unsafe {
            let image_index = self
                .swapchain
                .acquire_image(core::u64::MAX, Some(image_available), None)
                .map_err(|_| "Couldn't acquire an image from the swapchain!")?;
            let a = image_index.0.clone();
            (image_index, a as usize)
        };

        let flight_fence = &self.swapchain_img_fences[i_usize];
        unsafe {
            self.device
                .wait_for_fence(flight_fence, core::u64::MAX)
                .map_err(|_| "Failed to wait on the fence!")?;
            self.device
                .reset_fence(flight_fence)
                .map_err(|_| "Couldn't reset the fence!")?;
        }

        // WRITE THE TRIANGLE DATA
        unsafe {
            let mut data_target = self
                .device
                .map_memory(&self.vertices.memory, 0..self.vertices.requirements.size)
                .map_err(|_| "Failed to acquire a memory writer!")?;
            let points = quad.vertex_attributes();
            ptr::copy(points.as_ptr() as *const u8, data_target, points.len() * size_of::<f32>());
            // Here we must force the Deref impl of ManuallyDrop to play nice. / or call .deref() from impl Deref
            let memory_ref: &<B as Backend>::Memory = &self.vertices.memory;
            self.device
                .flush_mapped_memory_ranges(iter::once((*&memory_ref, 0..self.vertices.requirements.size)))
                .map_err(|_| "Failed to flush memory!")?;
            self.device
                .unmap_memory(&memory_ref);

//           MEM CHECK
//            let mut data_target = self
//                .device
//                .map_memory(&self.memory, 0..self.requirements.size)
//                .map_err(|_| "Failed to acquire a memory writer!")?;

//            let mut myslice: [f32; 6] = [0.0 as f32; 6];
//            ptr::copy(data_target, myslice.as_mut_ptr() as *mut u8, myslice.len() * size_of::<f32>());
//            info!("{:?}", myslice);

//            self.device
//                .flush_mapped_memory_ranges(iter::once((*&memory_ref, 0..self.requirements.size)))
//                .map_err(|_| "Failed to flush memory!")?;
//            self.device
//                .unmap_memory(&memory_ref);
        }

        // RECORD COMMANDS
        unsafe {
            let buffer = &mut self.command_buffers[i_usize];
            const TRIANGLE_CLEAR: [ClearValue; 2] = [
                command::ClearValue {
                    color: command::ClearColor {
                        float32: [0.1, 0.2, 0.3, 1.0],
                    }
                },
                command::ClearValue {
                    depth_stencil: command::ClearDepthStencil {
                        depth: 1.0,
                        stencil: 0,
                    }
                },
            ];
            buffer.begin_primary(command::CommandBufferFlags::empty());
            {
                let viewport = Viewport {
                    rect: self.render_area,
                    depth: (0.0..1.0),
                };
                buffer.set_viewports(0, &[viewport]);
                buffer.set_scissors(0, &[self.render_area]);
                // Here we must force the Deref impl of ManuallyDrop to play nice.
                let buffer_ref: &<B as Backend>::Buffer = &self.vertices.buffer;
                let buffers: ArrayVec<[_; 1]> = [(buffer_ref, 0)].into();

                buffer.bind_graphics_pipeline(&self.graphics_pipeline);
                buffer.bind_vertex_buffers(0, buffers);
                buffer.bind_index_buffer(IndexBufferView {
                    buffer: &self.indexes.buffer,
                    offset: 0,
                    index_type: IndexType::U16,
                });
                buffer.bind_graphics_descriptor_sets(
                    &self.pipeline_layout,
                    0,
                    Some(self.descriptor_set.deref()),
                    &[],
                );

                buffer.begin_render_pass(
                    &self.render_pass,
                    &self.framebuffers[i_usize],
                    self.render_area,
                    TRIANGLE_CLEAR.iter(),
                    command::SubpassContents::Inline,
                );
//                buffer.push_graphics_constants(
//                    &self.pipeline_layout,
//                    ShaderStageFlags::FRAGMENT,
//                    0,
//                    &[time_f32.to_bits()],
//                );
                buffer.push_graphics_constants(
                    &self.pipeline_layout,
                    ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT,
                    0,
                    cast_slice::<f32, u32>(&cam.view_projection().as_slice())
                            .expect("this cast never fails for same-aligned same-size data"),
                );
                buffer.draw_indexed(0..6, 0, 0..1);
                buffer.end_render_pass();
            }
            buffer.finish();
        }

        // SUBMISSION AND PRESENT
        let command_buffers = &self.command_buffers[i_usize..=i_usize];
        let wait_semaphores: ArrayVec<[_; 1]> =
            [(image_available, PipelineStage::COLOR_ATTACHMENT_OUTPUT)].into();
        let signal_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
        // yes, you have to write it twice like this. yes, it's silly.
        let present_wait_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
        let submission = Submission {
            command_buffers,
            wait_semaphores,
            signal_semaphores,
        };
        let the_command_queue = &mut self.queue_group.queues[0];
        unsafe {
            the_command_queue.submit(submission, Some(flight_fence));
            self.swapchain
                .present(the_command_queue, i_u32.0, present_wait_semaphores)
                .map_err(|_| "Failed to present into the swapchain!")?;
        };
        Ok(())
    }
}