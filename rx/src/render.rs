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
use crate::graphics::wrapper::ApiWrapper;
use crate::hal::buffer::{IndexBufferView, SubRange};
use crate::hal::IndexType;
use crate::window::WinitState;
use crate::graphics_api::{DrawCmd, RenderCommand};


pub trait Pipeline {
    fn process(&mut self);
}

pub struct Renderer {
    pub(crate) api: ApiWrapper<back::Backend>,
    pub(crate) storage: AssetsStorage,
    pub(crate) loader: AssetsLoader,
    resize_flag: Option<PhysicalSize<u32>>,

    sender: Sender<DrawCmd>,
    receiver: Receiver<DrawCmd>,

    cmd_s: Sender<RenderCommand>,
    cmd_r: Receiver<RenderCommand>,

    pipelines: Vec<Box<dyn Pipeline>>,
}

impl Renderer {
    pub fn new(window: &mut WinitState) -> Result<Self, &str> {
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
        match self.resize_flag {
            None => (),
            Some(size) => {
                info!("Req size: {:?}", size);
                self.api.reset_swapchain(size)
                    .expect("cannot recreate swapchain");
                self.resize_flag = None;
            }
        };

        let ex = self.api.swapchain.current_extent();
        let next_frame = self.api.next_frame();
        match next_frame {
            Ok(fr) => {
                for _pipe in self.pipelines.iter_mut() {}


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
                        (storage.mesh_bundle.buffer.deref(), SubRange {
                            offset: 0,
                            size: None,
                        }),
                        (storage.instanced_bundle.buffer.deref(), SubRange {
                            offset: instanced_offset.start as u64,
                            size: None,
                        })
                    ].into();
                    buffer.bind_vertex_buffers(0, buffers);
                    buffer.bind_index_buffer(IndexBufferView {
                        buffer: &storage.idx_bundle.buffer,
                        range: SubRange::default(),
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
                            RenderCommand::PushView(_mtx) => {
                                // use hal::pso::ShaderStageFlags;
//                                buffer.push_graphics_constants(
//                                    &pipeline.pipeline_layout,
//                                    ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT,
//                                    0,
//                                    cast_slice::<f32, u32>(&mtx.as_slice())
//                                        .expect("this cast never fails for same-aligned same-size data"),
//                                );
                            }
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
                    ).unwrap();
                    storage.instanced_bundle.unmap(&state.device).unwrap();
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
    pub fn typed(st: &mut WinitState) -> Result<Self, &str> {
        let wb = st.window_builder.take().unwrap();
        let window = wb.build(&st.events_loop).unwrap();
        let instance =
            back::Instance::create("gfx-rs quad", 1).expect("Failed to create an instance!");
        info!("{:?}", instance);
        let surface = unsafe {
            instance
                .create_surface(&window)
                .expect("Failed to create a surface!")
        };
        info!("{:?}", surface);
        let adapters = instance.enumerate_adapters();

        let wrap = ApiWrapper::new(&window, Some(instance), surface, adapters);
        st.window = Some(window);
        wrap
    }
}

#[cfg(feature = "gl")]
impl ApiWrapper<back::Backend> {
    pub fn typed(st: &mut WinitState) -> Result<Self, &str> {
        #[cfg(not(target_arch = "wasm32"))]
            let (window, surface) = {
            let builder =
                back::config_context(back::glutin::ContextBuilder::new(), hal::format::Format::Rgba8Srgb, None)
                    .with_vsync(false);
            let wb = st.window_builder.take();
            let windowed_context = builder.build_windowed(wb.unwrap(), &st.events_loop).unwrap();
            let (context, window) = unsafe {
                windowed_context
                    .make_current()
                    .expect("Unable to make context current")
                    .split()
            };
            let surface = back::Surface::from_context(context);
            (window, surface)
        };
        #[cfg(target_arch = "wasm32")]
            let (window, surface) = {
            extern crate web_sys;
            let wb = st.window_builder.take();
            let window = wb.unwrap().build(&st.events_loop).unwrap();
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .body()
                .unwrap()
                .append_child(&winit::platform::web::WindowExtWebSys::canvas(&window));
            let surface = back::Surface::from_raw_handle(&window);
            (window, surface)
        };
        info!("{:?}", surface);
        let adapters = surface.enumerate_adapters();

        let wrap = ApiWrapper::new(&window, None, surface, adapters);
        st.window = Some(window);
        wrap
    }
}
