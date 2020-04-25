extern crate env_logger;
extern crate log;


#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

pub use rx;
use rx::ecs::{Render, Transformation, WinitEvents};
use rx::ecs::layer::EcsInitTuple;
use rx::specs::Builder;
use rx::specs::WorldExt;

mod systems;
mod map;



pub fn start() {
    env_logger::from_env(env_logger::Env::default().default_filter_or(
        "\
         info,\
         winit::platform_impl::platform::event_loop::runner=error,\
         gfx_backend_vulkan=warn\
         ",
    ))
    .init();

    let mut eng = rx::run::Engine::default();

    let ico_mesh = {
        let (api, loader, storage) = eng.loader();
        let obj = loader.load_obj("ico-sphere").expect("");
        storage.load_mesh(api, obj).expect("")
    };

    let tetrahedron_mesh = {
        let (api, loader, storage) = eng.loader();
        let obj = loader.load_obj("tetrahedron").expect("");
        storage.load_mesh(api, obj).expect("")
    };

    let map_mesh_ptr = {
        let (api, loader, storage) = eng.loader();
        let mesh = map::generate2d();
        storage.load_mesh(api, mesh).expect("")
    };
    let (draw, redner) = eng.renderer().queue();

    let render_sys = systems::RenderSubmitSystem::new(draw, redner);
    let input_sys = systems::InputTestSystem::default();
    let transform_sys = systems::TransformationSystem;

    let ecs_layer = rx::ecs::layer::EcsLayer::new(move |(mut world, mut r_dispatcher, mut c_dispatcher): EcsInitTuple<'static>| {
        use rx::ecs::{
            TargetCamera,
            Position,
            Rotation,
            ActiveCamera,
            CameraTarget,
        };


        world.register::<Render>();
        world.register::<Rotation>();
        world.register::<Position>();
        world.register::<Transformation>();
        world.register::<TargetCamera>();

        let entity = world.create_entity()
            .with(Rotation::default())
            .with(Position::default())
            .with(Transformation::default())
            .with(Render {
                mesh: ico_mesh.clone()
            })
            .build();
        world.create_entity()
            .with(Rotation::default())
            .with(Position {
                x: -100.,
                y: -5.,
                z: -100.
            })

            .with(Transformation::default())
            .with(Render {
                mesh: map_mesh_ptr.clone()
            })
            .build();

        for e in 1..20 {
            let _ = world.create_entity()
                .with(Rotation::default())
                .with(Position {
                    x: e as f32 * 10. *  {if e % 2 == 0 { -1. } else { 1. } },
                    y: 0.0,
                    z: 0.0,
                })
                .with(Transformation::default())
            .with(Render {
                mesh: {
                    if e % 2 == 0 {
                        ico_mesh.clone()
                    } else {
                        tetrahedron_mesh.clone()
                    }
                }
            })
            .build();
        }

        let cam_entity = world.create_entity()
            .with(TargetCamera::default())
            .build();


        world.insert(ActiveCamera(Some(cam_entity)));
        world.insert(CameraTarget(Some(entity)));
        world.insert(WinitEvents::default());

        r_dispatcher = r_dispatcher
            .with(input_sys, "in_tst_sys", &[])
            .with(transform_sys, "tsm_sys", &[]);
        c_dispatcher = c_dispatcher
            .with_thread_local(render_sys);
        return (world, r_dispatcher, c_dispatcher);
    });



    eng.push_layer(ecs_layer);
    eng.run();
}
