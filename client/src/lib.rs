extern crate env_logger;
extern crate itertools;
extern crate log;
extern crate rand;
extern crate rand_distr;
extern crate rand_pcg;
extern crate rand_seeder;
extern crate serde_json;


#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

pub use rx;
pub use rx::*;
pub use rx::{glm, run};
use rx::ecs::base_systems::world3d::init;
use rx::ecs::layer::EcsInitTuple;
use rx::specs::Builder;
use rx::specs::WorldExt;
use rx::winit::dpi::PhysicalSize;

use crate::gui_sys::{EcsUiWidget, EcsUiWidgetSystem};
use crate::systems::test::Follower;
use crate::winit::event::Event;
use rx::ecs::systems::frustum;
use rx::ecs::systems::frustum::Culling;


use rx::maths;

mod flowchart;
mod generatin;
mod map;
mod systems;
mod arrowdrop;
mod input_sys;
mod gui_sys;
mod info_layer;

pub fn init_log() {
    env_logger::from_env(env_logger::Env::default().default_filter_or(
        "\
         info,\
         winit::platform_impl::platform::event_loop::runner=error,\
         gfx_backend_vulkan=warn\
         ",
    )).init();
}

pub fn start() {
    init_log();
    let mut eng: rx::run::Engine<()> = rx::run::Engine::default();


    let _cube_mesh = {
        let (api, loader, storage) = eng.loader();
        let obj = loader.load_obj("cube").expect("");
        storage.load_mesh(api, obj).expect("")
    };
    let arrow_01 = {
        let (api, loader, storage) = eng.loader();
        let obj = loader.load_obj("arrow-01").expect("");
        storage.load_mesh(api, obj).expect("")
    };
    let arrow_02 = {
        let (api, loader, storage) = eng.loader();
        let obj = loader.load_obj("arrow-02").expect("");
        storage.load_mesh(api, obj).expect("")
    };

    let _tetrahedron_mesh = {
        let (api, loader, storage) = eng.loader();
        let obj = loader.load_obj("tetrahedron").expect("");
        storage.load_mesh(api, obj).expect("")
    };

    let map_mesh_ptr = {
        let (api, _loader, storage) = eng.loader();
        let mesh = map::generate2d();
        storage.load_mesh(api, mesh).expect("")
    };

    let (draw, redner) = eng.renderer().queue();

    let input_sys = input_sys::InputTestSystem::default();
    let move_sys = systems::test::MoveSystem;
    let mouse_sys = systems::test::MoveClickSystem::default();

    let ecs_layer = rx::ecs::layer::EcsLayer::new(
        Box::new(move |(mut world, mut r_dispatcher, mut c_dispatcher): EcsInitTuple| {
            use rx::ecs::{CameraTarget, Position, Rotation};
            world.register::<Velocity>();
            world.register::<Follower>();
            world.register::<rx::Culling>();

            let (mut cam, cam_sys, transform_sys) = init::<()>(&mut world, &glm::vec3(0., 0., 0.));

            {
                world.system_data::<specs::WriteStorage<rx::Camera>>()
                    .get_mut(cam)
                    .map(|cam| {
                        match cam {
                            Camera::Targeted(cam) => {
                                cam.yaw = 180.;
                                cam.pitch = 0.;
                                cam.distance = 30.;
                                cam.fov = 30.
                            }
                            Camera::Free => {}
                        }
                    });
            }

            let player = world
                .create_entity()
                .with(Culling::default())
                .with(Rotation::new(0., 90., -90.))
                .with(Position::new(0., 30., 0.))
                .with(Transformation::default())
                .with(Velocity::default())
                .with(Render::new(arrow_02.clone()))
                .build();

            world
                .create_entity()
                .with(Rotation::new(180., 0., 0.))
                .with(Position::new(0.,10.,0.))
                .with(Transformation::default())
                .with(Render::new(map_mesh_ptr.clone()))
                .with(Culling::never())
                .build();
            world.insert(WinitEvents::default() as WinitEvents<()>);
            world.insert(CameraTarget::new(player));

            // r_dispatcher.add(systems::test::FollowingSystem, "follow_sys", &[]);
            r_dispatcher.add(systems::test::ScreenClickSystem::default(), "screen_click_sys", &[]);
            //
            r_dispatcher.add(input_sys, "in_tst_sys", &[]);
            r_dispatcher.add(move_sys, "move_sys", &[]);
            r_dispatcher.add(cam_sys, "cam_sys", &[]);

            world.register::<EcsUiWidget>();
            // gui_sys::EcsUiSystem.register_widget(c_dispatcher, world);
            // gui_sys::CameraUiSystem.register_widget(c_dispatcher, world);
            // gui_sys::ScreenClickUiSystem::default().register_widget(c_dispatcher, world);
            // arrowdrop::GridUiSys.register_widget(c_dispatcher, world);

            arrowdrop::create((world, r_dispatcher, c_dispatcher), _cube_mesh.clone());


            c_dispatcher.add(frustum::CullingSystem, "cull_sys", &[]);
            c_dispatcher.add(transform_sys, "tsm_sys", &["cull_sys"]);
            c_dispatcher.add_thread_local(rx::RenderSubmitSystem::new(draw, redner));
        })
    );

    eng.push_layer(ecs_layer);
    eng.push_layer(info_layer::InfoLayer::default());
    eng.run();
}

