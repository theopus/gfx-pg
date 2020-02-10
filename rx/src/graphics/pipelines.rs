use std::mem::{ManuallyDrop, size_of};

use hal::{
    Backend,
    device::Device,
    pso::{
        AttributeDesc, BakedStates, BlendDesc, BlendOp, BlendState, ColorBlendDesc, ColorMask,
        Comparison, DepthStencilDesc, DepthTest, DescriptorPool, DescriptorSetLayoutBinding,
        Element, EntryPoint, Face, Factor, FrontFace, GraphicsPipelineDesc, GraphicsShaderSet,
        InputAssemblerDesc, LogicOp, PipelineCreationFlags, Primitive, Rasterizer,
        ShaderStageFlags, Specialization, VertexBufferDesc, Viewport,
    },
    window::Extent2D,
};
use hal::pass::Subpass;
use hal::pso::{BasePipeline, PolygonMode, VertexInputRate};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::graphics::api::{FRAGMENT_SOURCE, VERTEX_SOURCE};
use crate::graphics::swapchain::DeviceDrop;

pub struct PipelineV0<B: Backend> {
//    pub(crate)descriptor_set: ManuallyDrop<B::DescriptorSet>,
//    descriptor_pool: ManuallyDrop<B::DescriptorPool>,
//    descriptor_set_layouts: Vec<B::DescriptorSetLayout>,
    pub(crate)pipeline_layout: ManuallyDrop<B::PipelineLayout>,
    pub(crate)graphics_pipeline: ManuallyDrop<B::GraphicsPipeline>,
}

impl<B: Backend> DeviceDrop<B> for PipelineV0<B> {
    unsafe fn manually_drop(&mut self, device: &<B as Backend>::Device) {
        unsafe {
            use std::ptr::read;
//            self.descriptor_pool
//                .free_sets(Some(ManuallyDrop::into_inner(read(
//                    &mut self.descriptor_set,
//                ))));
//            device
//                .destroy_descriptor_pool(ManuallyDrop::into_inner(read(&mut self.descriptor_pool)));
//
//            for dsl in self.descriptor_set_layouts.drain(..) {
//                device.destroy_descriptor_set_layout(dsl);
//            }
            device
                .destroy_pipeline_layout(ManuallyDrop::into_inner(read(&mut self.pipeline_layout)));
            device.destroy_graphics_pipeline(ManuallyDrop::into_inner(read(
                &mut self.graphics_pipeline,
            )));
        }
    }
}

impl<B: Backend> PipelineV0<B> {
    pub fn new(
        device: &B::Device,
        extent: Extent2D,
        render_pass: &<B as Backend>::RenderPass,
    ) -> Result<Self, &'static str> {
        let vertex_compile_artifact = shader::compile(
            VERTEX_SOURCE,
            shaderc::ShaderKind::Vertex,
            "vertex.vert",
            "main",
        )?;
        let fragment_compile_artifact = shader::compile(
            FRAGMENT_SOURCE,
            shaderc::ShaderKind::Fragment,
            "fragment.frag",
            "main",
        )?;

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
        let mut vertex_buffers: Vec<VertexBufferDesc> = vec![VertexBufferDesc {
            binding: 0,
            stride: (size_of::<f32>() * (3 + 2 + 3)) as u32,
            rate: VertexInputRate::Vertex,
        }];

        //instanced
//        vertex_buffers.push(VertexBufferDesc {
//            binding: 1,
//            stride: (size_of::<f32>() * 16) as u32,
//            rate: VertexInputRate::Instance(1),
//        });
        let mut attributes: Vec<AttributeDesc> = vec![
            AttributeDesc {
                location: 0,
                binding: 0,
                element: Element {
                    format: hal::format::Format::Rgb32Sfloat,
                    offset: 0,
                },
            },
            AttributeDesc {
                location: 1,
                binding: 0,
                element: Element {
                    format: hal::format::Format::Rg32Sfloat,
                    offset: (size_of::<f32>() * 3) as u32,
                },
            },
            AttributeDesc {
                location: 2,
                binding: 0,
                element: Element {
                    format: hal::format::Format::Rgb32Sfloat,
                    offset: (size_of::<f32>() * (2 + 3)) as u32,
                },
            },
        ];

//        //instanced1
//        for i in 0..4 {
//            attributes.push(AttributeDesc {
//                location: 3 + i,
//                binding: 1,
//                element: Element {
//                    format: hal::format::Format::Rgba32Sfloat,
//                    offset: (size_of::<f32>() * 4) as u32 * i,
//                },
//            });
//        }

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
                targets: vec![ColorBlendDesc {
                    mask: ColorMask::ALL,
                    blend: Some(blend_state),
                }],
            }
        };

        let baked_states = BakedStates {
            viewport: Some(Viewport {
                rect: extent.to_extent().rect(),
                depth: (0.0..1.0),
            }),
            scissor: Some(extent.to_extent().rect()),
            blend_color: None,
            depth_bounds: None,
        };

//        let descriptor_set_layouts: Vec<<B as Backend>::DescriptorSetLayout> = vec![unsafe {
//            device
//                .create_descriptor_set_layout(
//                    &[
//                        DescriptorSetLayoutBinding {
//                            binding: 0,
//                            ty: hal::pso::DescriptorType::SampledImage,
//                            count: 1,
//                            stage_flags: ShaderStageFlags::FRAGMENT,
//                            immutable_samplers: false,
//                        },
//                        DescriptorSetLayoutBinding {
//                            binding: 1,
//                            ty: hal::pso::DescriptorType::Sampler,
//                            count: 1,
//                            stage_flags: ShaderStageFlags::FRAGMENT,
//                            immutable_samplers: false,
//                        },
//                    ],
//                    &[],
//                )
//                .map_err(|_| "Couldn't make a DescriptorSetLayout")?
//        }];
//        let mut descriptor_pool = unsafe {
//            device
//                .create_descriptor_pool(
//                    1, // sets
//                    &[
//                        hal::pso::DescriptorRangeDesc {
//                            ty: hal::pso::DescriptorType::SampledImage,
//                            count: 1,
//                        },
//                        hal::pso::DescriptorRangeDesc {
//                            ty: hal::pso::DescriptorType::Sampler,
//                            count: 1,
//                        },
//                    ],
//                    hal::pso::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET,
//                )
//                .map_err(|_| "Couldn't create a descriptor pool!")?
//        };
//        let descriptor_set = unsafe {
//            descriptor_pool
//                .allocate_set(&descriptor_set_layouts[0])
//                .map_err(|_| "Couldn't make a Descriptor Set!")?
//        };

        //            (ShaderStageFlags::FRAGMENT, 0..4),
        let mut descriptor_set_layouts: Vec<<B as Backend>::DescriptorSetLayout> = vec![];
        let push_constants = vec![(ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT, 0..64)];
        let layout = unsafe {
            device
                .create_pipeline_layout(
                    &descriptor_set_layouts,
                    push_constants)
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
        Ok(Self {
//            descriptor_set_layouts,
//            descriptor_pool: ManuallyDrop::new(descriptor_pool),
//            descriptor_set: ManuallyDrop::new(descriptor_set),
            pipeline_layout: ManuallyDrop::new(layout),
            graphics_pipeline: ManuallyDrop::new(pipeline),
        })
    }
}

mod shader {
    use log::error;
    use shaderc::CompilationArtifact;
    use shaderc::Compiler;
    use shaderc::ShaderKind;

    pub fn compile(
        source: &str,
        kind: shaderc::ShaderKind,
        name: &str,
        entry_point: &str,
    ) -> Result<CompilationArtifact, &'static str> {
        Compiler::new()
            .ok_or("shaderc not found!")?
            .compile_into_spirv(source, kind, name, entry_point, None)
            .map_err(|e| {
                error!("{}", e);
                "Couldn't compile vertex shader!"
            })
    }
}
