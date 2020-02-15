use std::mem::ManuallyDrop;
use std::ptr::read;

use hal::{
    adapter::{Adapter, PhysicalDevice},
    Backend,
    device::Device,
    image::SubresourceRange,
    memory::Properties,
    memory::Requirements,
    MemoryTypeId,
    window::Extent2D,
};

pub fn get_mem_id<B>(
    adapter: &Adapter<B>,
    req: hal::memory::Requirements,
    props: hal::memory::Properties,
) -> Result<MemoryTypeId, &'static str>
    where
        B: Backend {
    Ok(adapter
        .physical_device
        .memory_properties()
        .memory_types
        .iter()
        .enumerate()
        .find(|&(id, memory_type)| {
            req.type_mask & (1 << id) as u64 != 0 && memory_type.properties.contains(props)
        })
        .map(|(id, _)| MemoryTypeId(id))
        .ok_or("Couldn't find a memory type to support the buffer!")?)
}


pub struct DepthImage<B: Backend> {
    pub image: ManuallyDrop<B::Image>,
    pub requirements: Requirements,
    pub memory: ManuallyDrop<B::Memory>,
    pub image_view: ManuallyDrop<B::ImageView>,
}


impl<B: Backend> DepthImage<B> {
    pub fn new(
        adapter: &Adapter<B>,
        device: &B::Device,
        extent: Extent2D,
    ) -> Result<Self, &'static str> {
        unsafe {
            use hal::format::Aspects;
            use hal::format::Format;
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
            let memory_type_id = get_mem_id(adapter, requirements, Properties::DEVICE_LOCAL)?;

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
        device.destroy_image_view(ManuallyDrop::into_inner(read(&self.image_view)));
        device.destroy_image(ManuallyDrop::into_inner(read(&self.image)));
        device.free_memory(ManuallyDrop::into_inner(read(&self.memory)));
    }
}

//pub struct LoadedImage<B: Backend> {
//    pub image: ManuallyDrop<B::Image>,
//    pub requirements: Requirements,
//    pub memory: ManuallyDrop<B::Memory>,
//    pub image_view: ManuallyDrop<B::ImageView>,
//    pub sampler: ManuallyDrop<B::Sampler>,
//}

//impl<B: Backend> LoadedImage<B> {
//    pub fn new(
//        adapter: &Adapter<B>,
//        device: &B::Device,
//        command_pool: &mut <B as Backend>::CommandPool,
//        command_queue: &mut <B as Backend>::CommandQueue,
//        img: image::RgbaImage,
//    ) -> Result<Self, &'static str> {
//        unsafe {
//            // 0. First we compute some memory related values.
//            let pixel_size = size_of::<image::Rgba<u8>>();
//            let row_size = pixel_size * (img.width() as usize);
//            let limits: Limits = adapter.physical_device.limits();
//            let row_alignment_mask = limits.optimal_buffer_copy_pitch_alignment as u32 - 1;
//            let row_pitch = ((row_size as u32 + row_alignment_mask) & !row_alignment_mask) as usize;
//            debug_assert!(row_pitch as usize >= row_size);
//
//            // 1. make a staging buffer with enough memory for the image, and a
//            //    transfer_src usage
//            let required_bytes = row_pitch * img.height() as usize;
//            let staging_bundle =
//                BufferBundle::new(&adapter, device, required_bytes, Usage::TRANSFER_SRC)?;
//
//            // 2. use mapping writer to put the image data into that buffer
//            let mem_ref = staging_bundle.mem_ref();
//            let writer = device
//                .map_memory(mem_ref, 0..staging_bundle.requirements.size)
//                .map_err(|_| "Couldn't acquire a mapping writer to the staging buffer!")?;
//
//            for y in 0..img.height() as usize {
//                let row = &(*img)[y * row_size..(y + 1) * row_size];
//                ptr::copy_nonoverlapping(
//                    row.as_ptr(),
//                    writer.offset(y as isize * row_pitch as isize),
//                    row_size,
//                );
//            }
//            device
//                .flush_mapped_memory_ranges(iter::once((
//                    mem_ref,
//                    0..staging_bundle.requirements.size,
//                )))
//                .map_err(|_| "Couldn't flush the memory range!")?;
//            device.unmap_memory(mem_ref);
//
//            // 3. Make an image with transfer_dst and SAMPLED usage
//            let mut the_image = device
//                .create_image(
//                    hal::image::Kind::D2(img.width(), img.height(), 1, 1),
//                    1,
//                    hal::format::Format::Rgba8Srgb,
//                    hal::image::Tiling::Optimal,
//                    hal::image::Usage::TRANSFER_DST | hal::image::Usage::SAMPLED,
//                    hal::image::ViewCapabilities::empty(),
//                )
//                .map_err(|_| "Couldn't create the image!")?;
//
//            // 4. allocate memory for the image and bind it
//            let requirements = device.get_image_requirements(&the_image);
//            let memory_type_id = adapter
//                .physical_device
//                .memory_properties()
//                .memory_types
//                .iter()
//                .enumerate()
//                .find(|&(id, memory_type)| {
//                    // BIG NOTE: THIS IS DEVICE LOCAL NOT CPU VISIBLE
//                    requirements.type_mask & (1 << id) != 0
//                        && memory_type.properties.contains(Properties::DEVICE_LOCAL)
//                })
//                .map(|(id, _)| MemoryTypeId(id))
//                .ok_or("Couldn't find a memory type to support the image!")?;
//            let memory = device
//                .allocate_memory(memory_type_id, requirements.size)
//                .map_err(|_| "Couldn't allocate image memory!")?;
//            device
//                .bind_image_memory(&memory, 0, &mut the_image)
//                .map_err(|_| "Couldn't bind the image memory!")?;
//
//            // 5. create image view and sampler
//            let image_view = device
//                .create_image_view(
//                    &the_image,
//                    hal::image::ViewKind::D2,
//                    hal::format::Format::Rgba8Srgb,
//                    hal::format::Swizzle::NO,
//                    SubresourceRange {
//                        aspects: hal::format::Aspects::COLOR,
//                        levels: 0..1,
//                        layers: 0..1,
//                    },
//                )
//                .map_err(|_| "Couldn't create the image view!")?;
//            let sampler = device
//                .create_sampler(&hal::image::SamplerDesc::new(
//                    hal::image::Filter::Nearest,
//                    hal::image::WrapMode::Tile,
//                ))
//                .map_err(|_| "Couldn't create the sampler!")?;
//            // 6. create a command buffer
//            let mut cmd_buffer: B::CommandBuffer =
//                command_pool.allocate_one(command::Level::Primary);
//            cmd_buffer.begin_primary(command::CommandBufferFlags::ONE_TIME_SUBMIT);
//            // 7. Use a pipeline barrier to transition the image from empty/undefined
//            //    to TRANSFER_WRITE/TransferDstOptimal
//            use hal::image::Layout;
//            let image_barrier = hal::memory::Barrier::Image {
//                states: (hal::image::Access::empty(), Layout::Undefined)
//                    ..(
//                    hal::image::Access::TRANSFER_WRITE,
//                    Layout::TransferDstOptimal,
//                ),
//                target: &the_image,
//                families: None,
//                range: SubresourceRange {
//                    aspects: hal::format::Aspects::COLOR,
//                    levels: 0..1,
//                    layers: 0..1,
//                },
//            };
//            cmd_buffer.pipeline_barrier(
//                PipelineStage::TOP_OF_PIPE..PipelineStage::TRANSFER,
//                hal::memory::Dependencies::empty(),
//                &[image_barrier],
//            );
//            // 8. perform copy from staging buffer to image
//            cmd_buffer.copy_buffer_to_image(
//                &staging_bundle.buffer,
//                &the_image,
//                Layout::TransferDstOptimal,
//                &[hal::command::BufferImageCopy {
//                    buffer_offset: 0,
//                    buffer_width: (row_pitch / pixel_size) as u32,
//                    buffer_height: img.height(),
//                    image_layers: hal::image::SubresourceLayers {
//                        aspects: hal::format::Aspects::COLOR,
//                        level: 0,
//                        layers: 0..1,
//                    },
//                    image_offset: hal::image::Offset { x: 0, y: 0, z: 0 },
//                    image_extent: hal::image::Extent {
//                        width: img.width(),
//                        height: img.height(),
//                        depth: 1,
//                    },
//                }],
//            );
//            // 9. use pipeline barrier to transition the image to SHADER_READ access/
//            //    ShaderReadOnlyOptimal layout
//            let image_barrier = hal::memory::Barrier::Image {
//                states: (
//                    hal::image::Access::TRANSFER_WRITE,
//                    Layout::TransferDstOptimal,
//                )
//                    ..(
//                    hal::image::Access::SHADER_READ,
//                    Layout::ShaderReadOnlyOptimal,
//                ),
//                target: &the_image,
//                families: None,
//                range: SubresourceRange {
//                    aspects: hal::format::Aspects::COLOR,
//                    levels: 0..1,
//                    layers: 0..1,
//                },
//            };
//            cmd_buffer.pipeline_barrier(
//                PipelineStage::TRANSFER..PipelineStage::FRAGMENT_SHADER,
//                hal::memory::Dependencies::empty(),
//                &[image_barrier],
//            );
//            // 10. Submit the cmd buffer to queue and wait for it
//            cmd_buffer.finish();
//            let upload_fence = device
//                .create_fence(false)
//                .map_err(|_| "Couldn't create an upload fence!")?;
//            command_queue.submit_without_semaphores(Some(&cmd_buffer), Some(&upload_fence));
//            device
//                .wait_for_fence(&upload_fence, core::u64::MAX)
//                .map_err(|_| "Couldn't wait for the fence!")?;
//            device.destroy_fence(upload_fence);
//            // 11. Destroy the staging bundle and one shot buffer now that we're done
//            staging_bundle.manually_drop(device);
//            command_pool.free(Some(cmd_buffer));
//            Ok(Self {
//                image: ManuallyDrop::new(the_image),
//                requirements,
//                memory: ManuallyDrop::new(memory),
//                image_view: ManuallyDrop::new(image_view),
//                sampler: ManuallyDrop::new(sampler),
//            })
//        }
//    }
//
//    pub unsafe fn manually_drop(&self, device: &B::Device) {
//        device.destroy_sampler(ManuallyDrop::into_inner(read(&self.sampler)));
//        device.destroy_image_view(ManuallyDrop::into_inner(read(&self.image_view)));
//        device.destroy_image(ManuallyDrop::into_inner(read(&self.image)));
//        device.free_memory(ManuallyDrop::into_inner(read(&self.memory)));
//    }
//}