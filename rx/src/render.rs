use std::ops::Deref;
use std::sync::mpsc::{channel, Receiver, Sender};

use arrayvec::ArrayVec;
use back;
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

use crate::assets::{AssetsLoader, AssetsStorage, MeshPtr};
use crate::glm::Mat4;
use crate::graphics::wrapper::ApiWrapper;
use crate::hal::buffer::IndexBufferView;
use crate::hal::IndexType;
use crate::utils::cast_slice;

pub type DrawCmd = (MeshPtr, glm::Mat4, glm::Mat4);

pub enum RenderCommand {
    PushView(glm::Mat4),
    PushLight(glm::Vec3)
}

pub struct Renderer {
    pub(crate)api: ApiWrapper<back::Backend>,
    pub(crate)storage: AssetsStorage,
    pub(crate)loader: AssetsLoader,
    resize_flag: Option<PhysicalSize<u32>>,

    sender: Sender<DrawCmd>,
    receiver: Receiver<DrawCmd>,

    cmd_s: Sender<RenderCommand>,
    cmd_r: Receiver<RenderCommand>,
}

impl Renderer {
    pub fn new(window: &winit::window::Window) -> Result<Self, &str> {
        let api = ApiWrapper::typed(window)?;

        let loader = AssetsLoader::new("assets")?;
        let storage = AssetsStorage::new()?;
        let (send, recv) = channel();
        let (r_send, r_recv) = channel();


        Ok(Self {
            api,
            storage,
            loader,
            resize_flag: None,
            sender: send,
            receiver: recv,
            cmd_s: r_send,
            cmd_r: r_recv
        })
    }

    pub fn reset_swapchain(&mut self, size: PhysicalSize<u32>) {
        self.resize_flag = Some(size)
    }

    pub fn queue(&self) -> (Sender<DrawCmd>, Sender<RenderCommand>) {
        (self.sender.clone(), self.cmd_s.clone())
    }

    pub fn render(&mut self) {
        match self.resize_flag {
            None => (),
            Some(size) => {
                info!("Req size: {:?}", size);
                self.api.reset_swapchain()
                    .expect("cannot recreate swapchain");
                self.resize_flag = None;
            }
        };

        let ex = self.api.swapchain.current_extent();
        let next_frame = self.api.next_frame();
        match next_frame {
            Ok(fr) => {
                let (
                    frame,
                    buffer,
                    fb,
                    render_pass,
                    storage,
                    pipeline,
                    state,
                ) = fr;
                //lmao dude move this outta my eyes
                unsafe {
                    const CLEAR: [ClearValue; 2] = [
                        command::ClearValue {
                            color: command::ClearColor {
                                float32: [0.1, 0.2, 0.3, 1.0],
                            },
                        },
                        command::ClearValue {
                            depth_stencil: command::ClearDepthStencil {
                                depth: 1.0,
                                stencil: 0,
                            },
                        },
                    ];

                    let render_area = Rect {
                        x: 0,
                        y: 0,
                        w: ex.width as i16,
                        h: ex.height as i16,
                    };
                    let viewport = Viewport {
                        rect: render_area,
                        depth: (0.0..1.0),
                    };

                    buffer.begin_primary(command::CommandBufferFlags::empty());
                    let instanced_offset = storage.instanced_offset(frame);
                    buffer.set_viewports(0, &[viewport]);
                    buffer.set_scissors(0, &[render_area]);



                    buffer.bind_graphics_pipeline(&pipeline.graphics_pipeline);


                    let buffers: ArrayVec<[_; 2]> = [
                        (storage.mesh_bundle.buffer.deref(), 0),
                        (storage.instanced_bundle.buffer.deref(), instanced_offset.start as u64)
                    ].into();
                    buffer.bind_vertex_buffers(0, buffers);
                    buffer.bind_index_buffer(IndexBufferView {
                        buffer: &storage.idx_bundle.buffer,
                        offset: 0,
                        index_type: IndexType::U32,
                    });

//                buffer.bind_graphics_descriptor_sets(
//                    &pipeline.pipeline_layout,
//                    0,
//                    Some(pipeline.descriptor_set.deref()),
//                    &[],
//                );

                    for cmd in self.cmd_r.try_iter() {
                        match cmd {
                            RenderCommand::PushView(mtx) => {
                                use hal::pso::ShaderStageFlags;
                                buffer.push_graphics_constants(
                                    &pipeline.pipeline_layout,
                                    ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT,
                                    0,
                                    cast_slice::<f32, u32>(&mtx.as_slice())
                                        .expect("this cast never fails for same-aligned same-size data"),
                                );
                            },
                            _ => ()
                        }
                    }

                    buffer.begin_render_pass(
                        &render_pass,
                        &fb,
                        render_area,
                        CLEAR.iter(),
                        command::SubpassContents::Inline,
                    );


                    let instanced_ptr = storage.instanced_bundle.map_mem_range(
                        &state.device,
                        instanced_offset.start as u64..instanced_offset.end as u64,
                    ).expect("");

                    let mut instances_offset: u32 = 0;
                    let mut data_offset = 0;
                    let grouped_queue = self.receiver
                        .try_iter()
                        .into_iter()
                        .sorted_by(|(l_ptr, ..), (r_ptr, ..)| {
                            l_ptr.base_vertex.partial_cmp(&r_ptr.base_vertex).unwrap()
                        })
                        .group_by(|ptr| ptr.0.clone());

                    for (ptr, list) in &grouped_queue {

                        let mut current_count = 0;

                        let data: Vec<_> = list.flat_map(|(_, mvp, model)| {
                            current_count += 1;
                            let mut base = mvp.as_slice().to_owned();
                            base.append(&mut model.as_slice().to_owned());
                            base
                        }).collect::<Vec<f32>>();

                        let data_len = data.len() * 4;

                        use std::ptr;
                        ptr::copy(
                            data.as_slice().as_ptr() as *const u8,
                            instanced_ptr.offset(data_offset),
                            data_len,
                        );

                        data_offset += data_len as isize;


                        buffer.draw_indexed(
                            ptr.indices.clone(),
                            ptr.base_vertex,
                            instances_offset..instances_offset + current_count,
                        );
                        instances_offset += current_count
                    };

                    storage.instanced_bundle.flush_mem_range(
                        &state.device,
                        instanced_offset.start as u64..instanced_offset.end as u64,
                    );
                    storage.instanced_bundle.unmap(&state.device);
                    //
                    buffer.end_render_pass();
                    buffer.finish();
                }
                self.api.present_buffer(frame).expect("");
            }
            Err(e) => {
                error!("{:?}", e);
                self.resize_flag = Some(PhysicalSize { width: 100, height: 100 });
            }
        };
    }
}

#[cfg(not(feature = "gl"))]
impl ApiWrapper<back::Backend> {
    pub fn typed(window: &winit::window::Window) -> Result<Self, &str> {
        let instance =
            back::Instance::create("gfx-rs quad", 1).expect("Failed to create an instance!");
        info!("{:?}", instance);
        let surface = unsafe {
            instance
                .create_surface(window)
                .expect("Failed to create a surface!")
        };
        info!("{:?}", surface);
        ApiWrapper::new(window, instance, surface)
    }
}

#[cfg(feature = "gl")]
impl ApiWrapper<back::Backend> {
    pub fn typed(window: &winit::window::Window) -> Result<Self, &str> {
        let builder =
            back::config_context(back::glutin::ContextBuilder::new(), ColorFormat::SELF, None)
                .with_vsync(true);
        builder.build_kek();
        let surface = back::Surface::from_context(context);
        info!("{:?}", instance);
        info!("{:?}", surface);
        ApiWrapper::new(window, instance, surface)
    }
}
