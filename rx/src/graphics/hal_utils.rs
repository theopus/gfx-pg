use std::mem::size_of;

use hal::format::Format::*;
use hal::pso::{
    AttributeDesc,
    Element,
};

use crate::hal::{Backend, MemoryTypeId};
use crate::hal::adapter::{Adapter, PhysicalDevice};

#[derive(Copy, Clone)]
#[repr(C)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl Vertex {
    pub fn attributes() -> Vec<AttributeDesc> {
        let pos = AttributeDesc {
            location: 0,
            binding: 0,
            element: Element {
                format: Rg32Sfloat,
                offset: 0,
            },
        };
        let tex = AttributeDesc {
            location: 1,
            binding: 0,
            element: Element {
                format: Rg32Sfloat,
                offset: (size_of::<[f32; 3]>()) as u32,
            },
        };
        vec![pos, tex]
    }

    pub fn flatten(&self) -> [f32; 4] {
        [self.position[0], self.position[1],
            self.tex_coords[0], self.position[1]]
    }
}


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