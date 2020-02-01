use std::mem::size_of;

use hal::format::Format::*;
use hal::pso::{
    AttributeDesc,
    Element,
};

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