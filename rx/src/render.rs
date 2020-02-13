use std::hash::Hasher;
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
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::dpi::PhysicalSize;

use crate::assets::{AssetsLoader, AssetsStorage, MeshPtr};
use crate::graphics::api::HalState;
use crate::graphics::memory::MemoryManager;
use crate::graphics::state::HalStateV2;
use crate::graphics::wrapper::ApiWrapper;
use crate::hal::buffer::IndexBufferView;
use crate::hal::IndexType;
use crate::utils::{Camera, cast_slice};

pub type DrawCmd = (MeshPtr, glm::Mat4);

pub struct Renderer {
    api: ApiWrapper<back::Backend>,
    storage: AssetsStorage,
    loader: AssetsLoader,
    pub(crate)  _cam: Camera,
    cube_mesh_ptr: MeshPtr,
    tetra_mesh_ptr: MeshPtr,
    resize_flag: Option<PhysicalSize<u32>>,
    sender: Sender<DrawCmd>,
    receiver: Receiver<DrawCmd>,
}

impl Renderer {
    pub fn new(window: &winit::window::Window) -> Result<Self, &str> {
        let api = ApiWrapper::typed(window)?;

        let loader = AssetsLoader::new("/home/otkachov/projects/cg/gfx-pg/assets")?;
        let mut storage = AssetsStorage::new()?;
        let (send, recv) = channel();

//
        let tetra_mesh = loader.load_obj("tetrahedron")?;
        let tetra_mesh_ptr = storage.load_mesh(&api, tetra_mesh)?;

        let cube_mesh = loader.load_obj("cube")?;
        let cube_mesh_ptr = storage.load_mesh(&api, cube_mesh)?;

        Ok(Self {
            api,
            storage,
            loader,
            _cam: Default::default(),
            cube_mesh_ptr: cube_mesh_ptr,
            tetra_mesh_ptr: tetra_mesh_ptr,
            resize_flag: None,
            sender: send,
            receiver: recv
        })
    }

    pub fn reset_swapchain(&mut self, size: PhysicalSize<u32>) {
        self.resize_flag = Some(size)
    }

    pub fn queue(&self) -> Sender<DrawCmd> {
        self.sender.clone()
    }

    pub fn render(&mut self) {
        match self.resize_flag {
            None => (),
            Some(size) => {
                info!("Req size: {:?}", size);
                self.api.reset_swapchain()
                    .expect("cannot recreate swapchain");
                self.resize_flag = None;
            },
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
                pipeline
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

                let buffers: ArrayVec<[_; 1]> = [
                    (storage.mesh_bundle.buffer.deref(), 0)
                ].into();
                buffer.set_viewports(0, &[viewport]);
                buffer.set_scissors(0, &[render_area]);

                buffer.bind_graphics_pipeline(&pipeline.graphics_pipeline);


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

                buffer.begin_render_pass(
                    &render_pass,
                    &fb,
                    render_area,
                    CLEAR.iter(),
                    command::SubpassContents::Inline,
                );
                use hal::pso::ShaderStageFlags;
                buffer.push_graphics_constants(
                    &pipeline.pipeline_layout,
                    ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT,
                    0,
                    cast_slice::<f32, u32>(&self._cam.view_projection().as_slice())
                        .expect("this cast never fails for same-aligned same-size data"),
                );
                buffer.draw_indexed(
                    self.cube_mesh_ptr.indices.clone(),
                    self.cube_mesh_ptr.base_vertex,
                    0..1,
                );
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
        let mut surface = unsafe {
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

fn do_the_render(
    hal_state: &mut HalState<back::Backend>,
    cam: &Camera,
) -> Result<(), &'static str> {
    let mtx = glm::translate(&glm::identity(), &glm::vec3(0., 0., -30.));
    hal_state.draw_quad_frame(
        crate::utils::Quad {
            x: -0.5 as f32,
            y: -0.5 as f32,
            w: 1 as f32,
            h: 1 as f32,
        },
        cam,
        &mtx,
    )
}
