use std::ops::Deref;
use std::sync::mpsc::{channel, Receiver, Sender};

use arrayvec::ArrayVec;
use gfx_hal::Instance;
use hal::{
    command,
    command::ClearValue,
    command::CommandBuffer,
    pso::{Rect, Viewport},
};
use itertools::Itertools;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::dpi::PhysicalSize;

use back;

use crate::assets::{AssetsLoader, AssetsStorage, MeshPtr};
use crate::hal::buffer::{IndexBufferView, SubRange};
use crate::hal::IndexType;
use crate::window::WinitState;
use crate::wgpu_graphics;
use futures::executor::block_on;
use crate::graphics_api::{DrawCmd, RenderCommand};


pub trait Pipeline {
    fn process(&mut self);
}

pub struct Renderer {
    wpgu_state: wgpu_graphics::State,
    resize_flag: Option<PhysicalSize<u32>>,

    sender: Sender<DrawCmd>,
    receiver: Receiver<DrawCmd>,

    cmd_s: Sender<RenderCommand>,
    cmd_r: Receiver<RenderCommand>,

    pipelines: Vec<Box<dyn Pipeline>>,
}

impl Renderer {
    pub fn new(window: &mut WinitState) -> Result<Self, &str> {
        let wpgu_state = block_on(wgpu_graphics::State::new(window.window.as_ref().unwrap()));

        let (send, recv) = channel();
        let (r_send, r_recv) = channel();


        Ok(Self {
            // api,
            // storage,
            // loader,
            wpgu_state,
            resize_flag: None,
            sender: send,
            receiver: recv,
            cmd_s: r_send,
            cmd_r: r_recv,
            pipelines: vec![],
        })
    }

    pub fn reset_swapchain(&mut self, size: PhysicalSize<u32>) {
        self.resize_flag = Some(size)
    }

    pub fn queue(&self) -> (Sender<DrawCmd>, Sender<RenderCommand>) {
        (self.sender.clone(), self.cmd_s.clone())
    }

    pub fn render(&mut self) {
        self.wpgu_state.render();
    }
}
