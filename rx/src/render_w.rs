use std::sync::mpsc::{channel, Receiver, Sender};

use futures::executor::block_on;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::dpi::PhysicalSize;

use crate::{gui, wgpu_graphics};
use crate::assets::{AssetsLoader, AssetsStorage};
use crate::graphics_api::{DrawCmd, RenderCommand, v0};
use crate::utils::file_system;
use crate::wgpu_graphics::{FrameState, pipeline, pipeline_test};
use crate::wgpu_graphics::pipeline::Pipeline;

pub struct Renderer {
    pub(crate) wpgu_state: wgpu_graphics::State,
    pub(crate) storage: AssetsStorage,
    pub(crate) loader: AssetsLoader,

    sender: Sender<DrawCmd>,
    // receiver: Receiver<DrawCmd>,

    cmd_s: Sender<RenderCommand>,
    cmd_r: Receiver<RenderCommand>,

    pipeline_v0: pipeline::PipelineV0,
    pipeline_grid: pipeline_test::GridPipeline,

    pipelines: Vec<Box<dyn Pipeline>>,
    egui_pipeline: gui::EguiPipeline,
}

impl Renderer {
    pub fn new(
        window: &winit::window::Window
    ) -> Result<Self, &'static str> {
        let mut wpgu_state = block_on(wgpu_graphics::State::new(window));

        let buf = file_system::path_from_root(&["assets"]);
        let loader = AssetsLoader::new(buf)?;
        let storage = AssetsStorage::new()?;
        let (send, recv) = channel();
        let (r_send, r_recv) = channel();

        let egui_pipeline = gui::EguiPipeline::new(&wpgu_state.device, false);
        let pipeline = wgpu_graphics::pipeline::PipelineV0::new(&mut wpgu_state.device, &mut wpgu_state.memory_manager, &wpgu_state.sc_desc, recv);
        let pipeline_grid = wgpu_graphics::pipeline_test::GridPipeline::new(&mut wpgu_state.device, &mut wpgu_state.memory_manager, &wpgu_state.sc_desc);

        wpgu_state.queue.write_buffer(&wpgu_state.memory_manager.uniform_buffer, 0,
                                           bytemuck::cast_slice(
                                               &[v0::Uniforms {
                                                   proj: (glm::identity() as glm::Mat4).into(),
                                                   view: (glm::identity() as glm::Mat4).into(),
                                                   light_pos: glm::vec4(0., 100., 30., 1.).into(),
                                                   light_intensity: glm::vec4(1., 1., 1., 1.).into(),
                                               }]
                                           ));
        Ok(Self {
            storage,
            loader,
            wpgu_state,
            sender: send,
            cmd_s: r_send,
            cmd_r: r_recv,
            pipeline_v0: pipeline,
            pipeline_grid,
            pipelines: Vec::new(),
            egui_pipeline,
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
        for cmd in self.cmd_r.try_iter() {
            match cmd {
                RenderCommand::PushProjView((proj,view)) => {
                    self.wpgu_state.queue.write_buffer(
                        &self.wpgu_state.memory_manager.uniform_buffer,
                        v0::Uniforms::VIEW_PROJ_OFFSET,
                        bytemuck::cast_slice(&[
                            v0::ProjViewMtx {
                                proj: proj.into(),
                                view: view.into()
                            }
                        ])
                    );
                }
                _ => {}
            }
        }

        let next_frame = self.wpgu_state.start_frame();
        match next_frame {
            Ok(frame) => {
                let mut encoder = self.wpgu_state.create_encode();
                self.pipeline_v0.process(FrameState::of(&frame, &mut encoder, &mut self.wpgu_state));
                // for p in self.pipelines.iter_mut() {
                //     p.process(FrameState::of(&frame, &mut encoder, &mut self.wpgu_state))
                // }
                self.pipeline_grid.process(FrameState::of(&frame, &mut encoder, &mut self.wpgu_state));
                self.egui_pipeline.process(FrameState::of(&frame, &mut encoder, &mut self.wpgu_state), ctx, egui_state);
                self.wpgu_state.end_frame(frame, encoder)
            }
            Err(err) => {
                error!("{:?}", err)
            }
        };
    }
}
