use std::mem::size_of;

use winit::event::WindowEvent;
use winit::window::Window;
use crate::graphics_api::v0;
use crate::utils::file_system;

mod memory;

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    pipeline: wgpu::RenderPipeline,
    // memory_manager: memory::MemoryManager
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();

        let (mut device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
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
        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            pipeline
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
                    flags: Default::default()
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
                front_face: wgpu::FrontFace::Cw,
                //culling
                cull_mode: wgpu::CullMode::None,
                topology: wgpu::PrimitiveTopology::TriangleList,
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: Some("one.frag.spv"),
                    source: wgpu::util::make_spirv(file_system::read_file(frag_path).as_slice()),
                    flags: Default::default()
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

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }


    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        todo!()
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        // self.swap_chain.
        let frame = self
            .swap_chain
            .get_current_frame()?
            .output;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                depth_stencil_attachment: None,
            });
        }


        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
