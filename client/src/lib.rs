extern crate env_logger;
extern crate itertools;
extern crate log;
extern crate rand;
extern crate rand_distr;
extern crate rand_pcg;
extern crate rand_seeder;
extern crate serde_json;

use std::time::{Duration, Instant};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

pub use rx;
use rx::{glm, run};
use rx::ecs::{Render, SelectedEntity, Transformation, Velocity, WinitEvents};
use rx::ecs::base_systems::world3d::init;
use rx::ecs::layer::EcsInitTuple;
use rx::specs::Builder;
use rx::specs::WorldExt;

use crate::systems::test::{Follower, MoveClickSystem};

mod flowchart;
mod generatin;
mod map;
mod maths;
mod systems;
mod arrowdrop;
mod input_sys;

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


    let cube_mesh = {
        let (api, loader, storage) = eng.loader();
        let obj = loader.load_obj("cube").expect("");
        storage.load_mesh(api, obj).expect("")
    };
    let ico_mesh = {
        let (api, loader, storage) = eng.loader();
        let obj = loader.load_obj("arrow-01").expect("");
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

    let render_sys = systems::generic::RenderSubmitSystem::new(draw, redner);
    let input_sys = input_sys::InputTestSystem::default();
    let move_sys = systems::test::MoveSystem;
    let mouse_sys = systems::test::MoveClickSystem::default();

    let ecs_layer = rx::ecs::layer::EcsLayer::new(
        move |(mut world, mut r_dispatcher, mut c_dispatcher): EcsInitTuple<'static>| {
            use rx::ecs::{CameraTarget, Position, Rotation};
            world.register::<Render>();
            world.register::<Velocity>();
            world.register::<Follower>();

            let (.., transform_sys) = init(&mut world, &glm::vec3(0., 0., 0.));

            let player = world
                .create_entity()
                .with(Rotation::default())
                .with(Position::default())
                .with(Transformation::default())
                .with(Velocity::default())
                .with(Render {
                    mesh: ico_mesh.clone(),
                })
                .build();

            let selected = world
                .create_entity()
                .with(Rotation::default())
                .with(Position::default())
                .with(Transformation::default())
                .with(Velocity::default())
                .with(Render {
                    mesh: ico_mesh.clone(),
                })
                .build();
            world
                .create_entity()
                .with(Rotation {
                    x: 180.0,
                    y: 0.0,
                    z: 0.0,
                })
                .with(Position {
                    x: 0.,
                    y: -10.,
                    z: 0.,
                })
                .with(Transformation::default())
                .with(Render {
                    mesh: map_mesh_ptr.clone(),
                })
                .build();
            //
            world.insert(SelectedEntity(Some(selected)));
            world.insert(WinitEvents::default() as WinitEvents<()>);
            world.insert(CameraTarget(Some(player)));

            arrowdrop::create(&mut world, cube_mesh.clone());
            r_dispatcher = r_dispatcher
                .with(systems::test::FollowingSystem, "follow_sys", &[])
                //
                .with(input_sys, "in_tst_sys", &[])
                .with(move_sys, "move_sys", &[])
                .with(mouse_sys, "mouse_sys", &[]);

            c_dispatcher = c_dispatcher
                .with(transform_sys, "tsm_sys", &[])
                .with_thread_local(render_sys);
            return (world, r_dispatcher, c_dispatcher);
        },
    );

    eng.push_layer(ecs_layer);


    let mut cnt = 0;
    eng.push_layer(move |upd: run::FrameUpdate<()>| {
        use rx::egui;
        if cnt < 1 {
            egui::Window::new("Info layer: ").show(&upd.egui_ctx, |ui| {
                ui.label(format!("Frame time: {} ms", upd.elapsed.as_millis()));
                ui.label(format!("Frames: {:.2} /sec", 1000.0 / upd.elapsed.as_millis() as f32));
            });
            cnt = 0;
        } else {
            cnt += 1;
        }
    });
    eng.run();
}
