use std::mem::size_of;
use std::ops::Range;
use std::sync::mpsc;

use futures::executor::block_on;
use futures::StreamExt;
use itertools::Itertools;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::event::WindowEvent;
use winit::window::Window;

use crate::graphics_api;
use crate::graphics_api::v0;
use crate::graphics_api::v0::VertexInstance;
use crate::utils::file_system;
use crate::wgpu_graphics::memory::{MemoryManager, MemoryManagerConfig};

pub mod memory;
pub mod texture;

pub struct State {
    surface: wgpu::Surface,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    pub(crate) memory_manager: memory::MemoryManager,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::VULKAN);
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();

        let (mut device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::NON_FILL_POLYGON_MODE | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
                limits: wgpu::Limits::default(),
                label: None,
            },
            None, // Trace path
        ).await.unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        let pipeline = Self::pipeline(&mut device, &sc_desc);
        let mm = MemoryManager::new(&mut device, MemoryManagerConfig {
            mesh_buffer_size: 1_000_000,
            idx_buffer_size: 1_000_000,
            instanced_buffer_size: (64 * 2) * 50_000,
        });
        let depth_texture = texture::Texture::create_depth_texture(&device, &sc_desc, "depth_texture");
        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            pipeline,
            depth_texture,
            memory_manager: mm,
        }
    }

    fn pipeline(device: &mut wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> wgpu::RenderPipeline {
        let vert_path = &["shaders", "one.vert.spv"];
        let frag_path = &["shaders", "one.frag.spv"];
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("v0 render_pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("v0 render_pipeline_layout"),
                bind_group_layouts: &[],
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
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture")
    }

    fn prepare_instances(&mut self, receiver: &mut mpsc::Receiver<graphics_api::DrawCmd>) -> Vec<InstanceDraw> {
        let mut instances_offset: u32 = 0;
        let mut grouped_queue = receiver
            .try_iter()
            .into_iter()
            .sorted_by(|(l_ptr, ..), (r_ptr, ..)| {
                l_ptr.base_vertex.partial_cmp(&r_ptr.base_vertex).unwrap()
            })
            .group_by(|ptr| ptr.0.clone());

        let mut render_calls = Vec::new();
        let mut data: Vec<VertexInstance> = grouped_queue.into_iter().flat_map(|(ptr, list)| {
            let ptr_instances: Vec<VertexInstance> = list.map(|e| {
                e.into()
            }).collect_vec();
            let mut current_count = ptr_instances.len() as u32;
            render_calls.push(InstanceDraw {
                indices: ptr.indices.clone(),
                base_vertex: ptr.base_vertex,
                instances: instances_offset..instances_offset + current_count,
            });
            instances_offset += current_count;
            ptr_instances
        }).collect_vec();

        self.queue.write_buffer(
            &self.memory_manager.instanced_buffer,
            0,
            bytemuck::cast_slice(&data),
        );
        render_calls
    }

    pub fn render(
        &mut self,
        recevier: &mut mpsc::Receiver<graphics_api::DrawCmd>,
    ) -> Result<(), wgpu::SwapChainError> {
        let mut draw_cmds = self.prepare_instances(recevier);

        let frame = self
            .swap_chain
            .get_current_frame()?
            .output;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
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
                    attachment: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_index_buffer(
                self.memory_manager.idx_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.set_vertex_buffer(
                0,
                self.memory_manager.mesh_buffer.slice(..),
            );
            render_pass.set_vertex_buffer(
                1,
                self.memory_manager.instanced_buffer.slice(..),
            );

            draw_cmds.drain(..).for_each(|cmd| {
                render_pass.draw_indexed(
                    cmd.indices,
                    cmd.base_vertex,
                    cmd.instances,
                )
            })
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}

struct InstanceDraw {
    indices: Range<u32>,
    base_vertex: i32,
    instances: Range<u32>,
}