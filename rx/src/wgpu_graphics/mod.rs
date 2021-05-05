use std::mem::size_of;
use std::ops::Range;
use std::sync::{mpsc, Arc};

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
use wgpu::SwapChainTexture;
use crate::wgpu::CommandEncoder;
use std::rc::Rc;

pub mod memory;
pub mod texture;
pub mod pipeline;

pub struct State {
    surface: wgpu::Surface,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    // pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    pub(crate) memory_manager: memory::MemoryManager,
}

impl State {
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
                features: wgpu::Features::NON_FILL_POLYGON_MODE | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
                limits: wgpu::Limits::default(),
                label: None,
            },
            None, // Trace path
        ).await.unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        // let pipeline = Self::pipeline(&mut device, &sc_desc);
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
            // pipeline,
            depth_texture,
            memory_manager: mm,
        }
    }


    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture")
    }

    pub fn start_frame(&mut self) -> Result<SwapChainTexture, wgpu::SwapChainError> {
        let frame = self
            .swap_chain
            .get_current_frame()?
            .output;
        Ok(frame)
    }

    pub fn end_frame(&self, _frame: SwapChainTexture, encoder: wgpu::CommandEncoder) {
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn create_encode(&self) -> CommandEncoder {
        self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        })
    }

    pub fn render(
        &mut self,
        recevier: &mut mpsc::Receiver<graphics_api::DrawCmd>,
    ) -> Result<(), wgpu::SwapChainError> {
        // let mut draw_cmds = self.prepare_instances(recevier);
        //
        // let frame = self
        //     .swap_chain
        //     .get_current_frame()?
        //     .output;
        // let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        //     label: Some("Render Encoder"),
        // });
        //
        // // submit will accept anything that implements IntoIter
        // self.queue.submit(std::iter::once(encoder.finish()));
        //
        // Ok(())
        Ok(())
    }
}

// pub type FrameState<'a> = (wgpu::SwapChainTexture, wgpu::CommandEncoder, &'a mut MemoryManager, &'a texture::Texture);
pub struct FrameState<'a> {
    pub frame: &'a wgpu::SwapChainTexture,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub mem: &'a mut MemoryManager,
    pub depth_texture: &'a texture::Texture,
    pub queue: &'a wgpu::Queue,
    pub device: &'a wgpu::Device,
    pub sc_desc: &'a wgpu::SwapChainDescriptor
}

impl<'a> FrameState<'a> {
    pub fn of(frame: &'a wgpu::SwapChainTexture, encoder: &'a mut wgpu::CommandEncoder, state: &'a mut State) -> FrameState<'a> {
        Self {
            frame: frame,
            encoder: encoder,
            mem: &mut state.memory_manager,
            depth_texture: &state.depth_texture,
            queue: &state.queue,
            device: &state.device,
            sc_desc: &state.sc_desc
        }
    }
    pub fn of_ui(
        frame: &'a wgpu::SwapChainTexture,
        encoder: &'a mut wgpu::CommandEncoder,
        state: &'a mut State
    ) -> FrameState<'a> {
        Self {
            frame: frame,
            encoder: encoder,
            mem: &mut state.memory_manager,
            depth_texture: &state.depth_texture,
            queue: &state.queue,
            device: &state.device,
            sc_desc: &state.sc_desc
        }
    }
}

