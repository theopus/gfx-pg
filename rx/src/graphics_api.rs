use crate::assets::MeshPtr;
use crate::graphics_api::v0::VertexInstance;

pub type DrawCmd = (MeshPtr, glm::Mat4, glm::Mat4);

pub enum RenderCommand {
    PushView(glm::Mat4),
    PushLight(glm::Vec3),
    PushState,
    Draw,
}

impl Into<v0::VertexInstance> for DrawCmd {
    fn into(self) -> VertexInstance {
        v0::VertexInstance {
            mvp: self.1.into(),
            model: self.2.into(),
        }
    }
}

pub mod v0 {
    use std::mem;

    #[repr(C)]
    #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
    pub struct Vertex {
        pub position: [f32; 3],
        pub uv: [f32; 2],
        pub normal: [f32; 3],
    }

    impl Vertex {
        pub fn wgpu_attr<'a>() -> wgpu::VertexBufferLayout<'a> {
            wgpu::VertexBufferLayout {
                array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float3,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float,
                        offset: (mem::size_of::<[f32; 3]>()) as u64,
                        shader_location: 1,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float3,
                        offset: (
                            mem::size_of::<[f32; 3]>() + mem::size_of::<[f32; 2]>()
                        ) as u64,
                        shader_location: 2,
                    },
                ],
            }
        }
        pub fn offsets(location_offset: u32) -> u32 {
            location_offset + 2
        }
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    pub struct VertexInstance {
        pub mvp: [[f32; 4]; 4],
        pub model: [[f32; 4]; 4],
    }

    impl VertexInstance {
        pub fn wgpu_attr<'a>() -> wgpu::VertexBufferLayout<'a> {
            wgpu::VertexBufferLayout {
                array_stride: (mem::size_of::<[f32; 4]>() * 8) as u64,
                step_mode: wgpu::InputStepMode::Instance,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float4,
                        offset: (mem::size_of::<[f32; 4]>() * 0) as u64,
                        shader_location: 3,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float4,
                        offset: (mem::size_of::<[f32; 4]>() * 1) as u64,
                        shader_location: 4,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float4,
                        offset: (mem::size_of::<[f32; 4]>() * 2) as u64,
                        shader_location: 5,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float4,
                        offset: (mem::size_of::<[f32; 4]>() * 3) as u64,
                        shader_location: 6,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float4,
                        offset: (mem::size_of::<[f32; 4]>() * 4) as u64,
                        shader_location: 7,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float4,
                        offset: (mem::size_of::<[f32; 4]>() * 5) as u64,
                        shader_location: 8,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float4,
                        offset: (mem::size_of::<[f32; 4]>() * 6) as u64,
                        shader_location: 9,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float4,
                        offset: (mem::size_of::<[f32; 4]>() * 7) as u64,
                        shader_location: 10,
                    },
                ],
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    pub struct Uniforms {
        pub view: [[f32; 4]; 4],
        pub light_pos: [f32; 4],
        pub light_intensity: [f32; 4],
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    pub struct ViewMtx {
        pub view: [[f32; 4]; 4]
    }

    impl Uniforms {
        pub const VIEW_OFFSET: u64 = 0;
        pub const LIGHT_POS_OFFSET: u64 = mem::size_of::<[[f32; 4]; 4]>() as u64;
        pub const LIGHT_INTENSITY_OFFSET: u64 = (mem::size_of::<[[f32; 4]; 4]>() + mem::size_of::<[f32; 4]>()) as u64;
    }
}


pub struct EngRenderer {}