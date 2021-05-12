use std::ops::Range;
use std::sync::mpsc;

use itertools::Itertools;

use crate::graphics_api::{DrawCmd, v0};
use crate::graphics_api::v0::VertexInstance;
use crate::utils::file_system;
use crate::wgpu_graphics::{FrameState, texture};
use crate::wgpu_graphics::memory::MemoryManager;

pub trait Pipeline {
    fn process(&mut self, frame: FrameState);
}

pub struct PipelineV0 {
    pipeline: wgpu::RenderPipeline,
    receiver: mpsc::Receiver<DrawCmd>,
    uniform_bind_group: wgpu::BindGroup,
}

impl PipelineV0 {
    fn pipeline(
        device: &mut wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        memory_manager: &mut MemoryManager,
    ) -> (wgpu::RenderPipeline, wgpu::BindGroup) {
        let vert_path = &["shaders", "one.vert.spv"];
        let frag_path = &["shaders", "one.frag.spv"];

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
            label: Some("uniform_bind_group_layout"),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: memory_manager.uniform_buffer.as_entire_binding(),
                }
            ],
            label: Some("uniform_bind_group"),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("v0 render_pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("v0 render_pipeline_layout"),
                bind_group_layouts: &[
                    &uniform_bind_group_layout
                ],
                push_constant_ranges: &[],
            })),
            vertex: wgpu::VertexState {
                module: &device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: Some("one.vert.spv"),
                    source: wgpu::util::make_spirv(file_system::read_file(vert_path).as_slice()),
                    flags: Default::default(),
                }),
                entry_point: "main",
                buffers: &[
                    v0::Vertex::wgpu_attr(), // 0 vertex buffer
                    v0::VertexInstance::wgpu_attr(), //1 vertex buffer
                ],
            },
            primitive: wgpu::PrimitiveState {
                polygon_mode: wgpu::PolygonMode::Fill,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                //culling
                cull_mode: wgpu::CullMode::Back,
                topology: wgpu::PrimitiveTopology::TriangleList,
            },
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
                    label: Some("one.frag.spv"),
                    source: wgpu::util::make_spirv(file_system::read_file(frag_path).as_slice()),
                    flags: Default::default(),
                }),
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    alpha_blend: Default::default(),
                    color_blend: Default::default(),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
        });
        (pipeline, uniform_bind_group)
    }

    pub fn new(
        device: &mut wgpu::Device,
        memory_manager: &mut MemoryManager,
        sc_desc: &wgpu::SwapChainDescriptor,
        receiver: mpsc::Receiver<DrawCmd>,
    ) -> Self {
        let (pipeline, uniform_bind_group) = Self::pipeline(device, sc_desc, memory_manager);
        PipelineV0 {
            pipeline,
            receiver,
            uniform_bind_group,
        }
    }

    fn prepare_instances(&mut self,
                         queue: &wgpu::Queue,
                         buffer: &wgpu::Buffer,
    ) -> Vec<InstanceDraw> {
        let receiver = &mut self.receiver;
        let mut instances_offset: u32 = 0;
        let grouped_queue = receiver
            .try_iter()
            .into_iter()
            .sorted_by(|(l_ptr, ..), (r_ptr, ..)| {
                l_ptr.base_vertex.partial_cmp(&r_ptr.base_vertex).unwrap()
            })
            .group_by(|ptr| ptr.0.clone());

        let mut render_calls = Vec::new();
        let data: Vec<VertexInstance> = grouped_queue.into_iter().flat_map(|(ptr, list)| {
            let ptr_instances: Vec<VertexInstance> = list.map(|e| {
                e.into()
            }).collect_vec();
            let current_count = ptr_instances.len() as u32;
            render_calls.push(InstanceDraw {
                indices: ptr.indices.clone(),
                base_vertex: ptr.base_vertex,
                instances: instances_offset..instances_offset + current_count,
            });
            instances_offset += current_count;
            ptr_instances
        }).collect_vec();

        queue.write_buffer(
            buffer,
            0,
            bytemuck::cast_slice(&data),
        );
        render_calls
    }
}


struct InstanceDraw {
    indices: Range<u32>,
    base_vertex: i32,
    instances: Range<u32>,
}

impl Pipeline for PipelineV0 {
    fn process(&mut self, state: FrameState) {
        let FrameState { target_texture: _, frame, encoder, depth_texture, mem, queue, .. } = state;
        encoder.push_debug_group("pipeline_v0");
        let mut draw_cmds = self.prepare_instances(queue, &mem.instanced_buffer);
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("PipelineV0: renderpass"),
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    }
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.push_debug_group("PipelineV0: renderpass");
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_index_buffer(
                mem.idx_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.set_vertex_buffer(
                0,
                mem.mesh_buffer.slice(..),
            );
            render_pass.set_vertex_buffer(
                1,
                mem.instanced_buffer.slice(..),
            );

            draw_cmds.drain(..).for_each(|cmd| {
                render_pass.draw_indexed(
                    cmd.indices,
                    cmd.base_vertex,
                    cmd.instances,
                )
            });
            render_pass.pop_debug_group();
        }
        encoder.pop_debug_group();
    }
}