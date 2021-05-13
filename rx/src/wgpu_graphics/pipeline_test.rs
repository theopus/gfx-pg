use std::ops::Range;
use std::sync::mpsc;

use itertools::Itertools;

use crate::graphics_api::{DrawCmd, v0};
use crate::graphics_api::v0::VertexInstance;
use crate::utils::file_system;
use crate::wgpu_graphics::{FrameState, texture, State};
use crate::wgpu_graphics::memory::MemoryManager;
use crate::wgpu_graphics::pipeline::Pipeline;
use crate::shader::SpirvCompiler;

pub struct GridPipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
}

impl GridPipeline {
    fn pipeline(
        device: &mut wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        memory_manager: &mut MemoryManager,
    ) -> (wgpu::RenderPipeline, wgpu::BindGroup) {
        let vert_path = &["shaders", "grid.vert.spv"];
        let frag_path = &["shaders", "grid.frag.spv"];

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("grid uniform_bind_group_layout"),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: memory_manager.uniform_buffer.as_entire_binding(),
                }
            ],
            label: Some("grid: uniform_bind_group"),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("grid render_pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("grid render_pipeline_layout"),
                bind_group_layouts: &[
                    &uniform_bind_group_layout
                ],
                push_constant_ranges: &[],
            })),
            vertex: wgpu::VertexState {
                module: &device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: Some("grid.vert.spv"),
                    source: wgpu::util::make_spirv(file_system::read_file(vert_path).as_slice()),
                    flags: Default::default(),
                }),
                entry_point: "main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                polygon_mode: wgpu::PolygonMode::Fill,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                //culling
                cull_mode: wgpu::CullMode::None,
                topology: wgpu::PrimitiveTopology::TriangleList,
            },
            // depth_stencil: None,
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
                clamp_depth: false,
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: Some("grid.frag.spv"),
                    source: wgpu::util::make_spirv(file_system::read_file(frag_path).as_slice()),
                    flags: Default::default(),
                }),
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    color_blend: wgpu::BlendState {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendState {
                        src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
        });
        (pipeline, uniform_bind_group)
    }

    pub fn new(
        device: &mut wgpu::Device,
        memory_manager: &mut MemoryManager,
        sc_desc: &wgpu::SwapChainDescriptor
    ) -> Self {
        let (pipeline, uniform_bind_group) = Self::pipeline(device, sc_desc, memory_manager);
        GridPipeline {
            pipeline,
            uniform_bind_group,
        }
    }
}
impl Pipeline for GridPipeline {

    fn process(&mut self, state: FrameState) {
        let FrameState { target_texture: _, frame, encoder, depth_texture, mem, queue, .. } = state;
        encoder.push_debug_group("pipeline_grid");
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("PipelineGRID: renderpass"),
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    }
                ],
                // depth_stencil_attachment: None,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.push_debug_group("PipelineGrid: renderpass");
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

            render_pass.draw(0..6, 0..1);
            render_pass.pop_debug_group();
        }
        encoder.pop_debug_group();
    }

    fn reset(&mut self, compiler: &mut SpirvCompiler, state: &mut State) {
        compiler.compile_to_fs(file_system::path_from_root(&["shaders", "grid.vert"]));
        compiler.compile_to_fs(file_system::path_from_root(&["shaders", "grid.frag"]));
        let (a, b) = Self::pipeline(&mut state.device, &state.sc_desc, &mut state.memory_manager);
        self.pipeline = a;
        self.uniform_bind_group = b;
    }
}