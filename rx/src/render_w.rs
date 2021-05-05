use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};

use arrayvec::ArrayVec;
use futures::executor::block_on;
use itertools::Itertools;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::dpi::PhysicalSize;

use crate::assets::{AssetsLoader, AssetsStorage, MeshPtr};
use crate::graphics_api::{DrawCmd, RenderCommand};
use crate::utils::file_system;
use crate::wgpu::SwapChainError;
use crate::wgpu_graphics::{FrameState, pipeline};
use crate::wgpu_graphics::pipeline::Pipeline;
use crate::window::WinitState;
use crate::{wgpu_graphics, gui};

pub struct Renderer {
    pub(crate) wpgu_state: wgpu_graphics::State,
    pub(crate) storage: AssetsStorage,
    pub(crate) loader: AssetsLoader,

    sender: Sender<DrawCmd>,
    // receiver: Receiver<DrawCmd>,

    cmd_s: Sender<RenderCommand>,
    cmd_r: Receiver<RenderCommand>,

    pipeline_v0: pipeline::PipelineV0,

    pipelines: Vec<Box<dyn Pipeline>>,
    egui_pipeline: gui::EguiPipeline
}
impl Renderer {
    pub fn new(
        window: &mut WinitState
    ) -> Result<Self, &'static str> {
        let mut wpgu_state = block_on(wgpu_graphics::State::new(window.window.as_ref().unwrap()));

        let buf = file_system::path_from_root(&["assets"]);
        let loader = AssetsLoader::new(buf)?;
        let storage = AssetsStorage::new()?;
        let (send, recv) = channel();
        let (r_send, r_recv) = channel();

        let egui_pipeline = gui::EguiPipeline::new(&wpgu_state.device, false);
        let pipeline = wgpu_graphics::pipeline::PipelineV0::new(&mut wpgu_state.device, &wpgu_state.sc_desc, recv);
        Ok(Self {
            storage,
            loader,
            wpgu_state,
            sender: send,
            cmd_s: r_send,
            cmd_r: r_recv,
            pipeline_v0: pipeline,
            pipelines: Vec::new(),
            egui_pipeline
        })
    }

    pub fn reset_swapchain(&mut self, size: PhysicalSize<u32>) {
        self.wpgu_state.resize(size);
    }

    pub fn queue(&self) -> (Sender<DrawCmd>, Sender<RenderCommand>) {
        (self.sender.clone(), self.cmd_s.clone())
    }

    pub fn push_pipelines<P>(&mut self, pipeline: P) where P: Pipeline + 'static {
        self.pipelines.push(Box::new(pipeline));
    }

    pub fn render(&mut self, ctx: egui::CtxRef, egui_state: &mut gui::EguiState) {
        let next_frame = self.wpgu_state.start_frame();
        match next_frame {
            Ok(mut frame) => {
                let mut encoder = self.wpgu_state.create_encode();

                self.pipeline_v0.process(FrameState::of(&frame, &mut encoder, &mut self.wpgu_state));

                for p in self.pipelines.iter_mut() {
                    p.process(FrameState::of(&frame, &mut encoder, &mut self.wpgu_state))
                }
                self.egui_pipeline.process(FrameState::of(&frame, &mut encoder, &mut self.wpgu_state), ctx, egui_state);
                self.wpgu_state.end_frame(frame, encoder)
            }
            Err(err) => {
                error!("{:?}", err)
            }
        };
    }
}
