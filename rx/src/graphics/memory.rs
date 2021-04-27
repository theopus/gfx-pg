use core::iter;
use std::mem::ManuallyDrop;
use std::ops::{Deref, Range};

use hal::{
    adapter, adapter::PhysicalDevice, Backend, buffer, device::Device, memory, memory::Segment,
    MemoryTypeId,
};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::graphics::state::HalStateV2;
use crate::graphics::swapchain::DeviceDrop;

pub struct MemoryManager<B: Backend> {
    _memory_properties: adapter::MemoryProperties,
    pub(crate) mesh_bundle: BufBundle<B>,
    pub(crate) idx_bundle: BufBundle<B>,
    pub(crate) instanced_bundle: BufBundle<B>,
    instanced_mem: usize,
    instanced_par_count: usize,
}

impl<B: Backend> DeviceDrop<B> for MemoryManager<B> {
    unsafe fn manually_drop(&mut self, device: &<B as Backend>::Device) {
        self.mesh_bundle.manually_drop(device);
        self.idx_bundle.manually_drop(device);
        self.instanced_bundle.manually_drop(device);
    }
}

const MESH_MEMORY_SIZE: usize = 1_000_000;
const IDX_MEMORY_SIZE: usize = 1_000_000;
const INSTANCE_MEMORY_SIZE: usize = (64 + 4) * 40_000;

impl<B: Backend> MemoryManager<B> {
    pub unsafe fn new(state: &HalStateV2<B>, images_cnt: u32) -> Result<Self, &'static str> {
        let mem_props = state._adapter.physical_device.memory_properties();

        info!("Limits: {:?}", state._adapter.physical_device.limits());
        let mesh_storage = BufBundle::new(
            state.device_ref(),
            &mem_props,
            MESH_MEMORY_SIZE,
            buffer::Usage::VERTEX,
            memory::Properties::CPU_VISIBLE,
        )?;

        let idx_storage = BufBundle::new(
            state.device_ref(),
            &mem_props,
            IDX_MEMORY_SIZE,
            buffer::Usage::INDEX,
            memory::Properties::CPU_VISIBLE,
        )?;
        let insatnced_mem = INSTANCE_MEMORY_SIZE * images_cnt as usize;

        let insatnced_storage = BufBundle::new(
            state.device_ref(),
            &mem_props,
            insatnced_mem,
            buffer::Usage::VERTEX,
            memory::Properties::CPU_VISIBLE,
        )?;

        Ok(Self {
            _memory_properties: mem_props,
            mesh_bundle: mesh_storage,
            idx_bundle: idx_storage,
            instanced_bundle: insatnced_storage,
            instanced_mem: insatnced_mem,
            instanced_par_count: images_cnt as usize,
        })
    }

    pub fn instanced_offset(&self, index: usize) -> Range<usize> {
        let offset = (self.instanced_mem / self.instanced_par_count) * index;
        offset..offset + self.instanced_mem / self.instanced_par_count
    }
}

pub struct BufBundle<B: Backend> {
    pub(crate) buffer: ManuallyDrop<B::Buffer>,
    requirements: memory::Requirements,
    memory: ManuallyDrop<B::Memory>,
}

impl<B: Backend> DeviceDrop<B> for BufBundle<B> {
    unsafe fn manually_drop(&mut self, device: &<B as Backend>::Device) {
        use core::ptr::read;
        device.destroy_buffer(ManuallyDrop::into_inner(read(&self.buffer)));
        device.free_memory(ManuallyDrop::into_inner(read(&self.memory)));
    }
}

impl<B: Backend> BufBundle<B> {
    unsafe fn new(
        device: &B::Device,
        mem_props: &adapter::MemoryProperties,
        size: usize,
        usage: buffer::Usage,
        props: memory::Properties,
    ) -> Result<Self, &'static str> {
        let mut buffer = device
            .create_buffer(size as u64, usage)
            .map_err(|_| "Couldn't create a buffer!")?;
        let requirements = device.get_buffer_requirements(&buffer);
        let mem_id = get_mem_id::<B>(mem_props, requirements, props)?;
        let memory = device
            .allocate_memory(mem_id, requirements.size)
            .map_err(|_| "Couldn't allocate buffer memory!")?;
        device
            .bind_buffer_memory(&memory, 0, &mut buffer)
            .map_err(|_| "Couldn't bind the buffer memory!")?;
        info!(
            "Buffer bundle: {:?} bytes, usage: {:?}, props: {:?}",
            size, usage, props
        );
        Ok(Self {
            buffer: ManuallyDrop::new(buffer),
            requirements,
            memory: ManuallyDrop::new(memory),
        })
    }

    pub unsafe fn requirements(&self) -> &memory::Requirements {
        &self.requirements
    }

    pub unsafe fn map_mem_range(
        &self,
        device: &B::Device,
        range: Range<u64>,
    ) -> Result<*mut u8, &'static str> {
        device
            .map_memory(&self.memory, to_seg(&range))
            .map_err(|_| "Failed to acquire a memory writer!")
    }

    pub unsafe fn flush_mem_range(
        &self,
        device: &B::Device,
        _range: Range<u64>,
    ) -> Result<(), &'static str> {
        device
            .flush_mapped_memory_ranges(iter::once((self.memory.deref(), Segment {
                //flush seg doesn't matter?!
                offset: 0,
                size: Some(0),
            })))
            .map_err(|_| "Failed to flush memory!")
    }

    pub unsafe fn unmap(&self, device: &B::Device) -> Result<(), &'static str> {
        Ok(device.unmap_memory(&self.memory))
    }
}

fn get_mem_id<B>(
    memory_properties: &adapter::MemoryProperties,
    req: hal::memory::Requirements,
    props: hal::memory::Properties,
) -> Result<MemoryTypeId, &'static str>
    where
        B: Backend,
{
    Ok(memory_properties
        .memory_types
        .iter()
        .enumerate()
        .find(|&(id, memory_type)| {
            req.type_mask & (1 << id) as u64 != 0 && memory_type.properties.contains(props)
        })
        .map(|(id, _)| MemoryTypeId(id))
        .ok_or("Couldn't find a memory type to support the buffer!")?)
}


pub fn to_seg(range: &Range<u64>) -> Segment {
    Segment {
        offset: range.start,
        size: Some((range.end) + (range.start)),
    }
}