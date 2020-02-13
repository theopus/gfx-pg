extern crate env_logger;
extern crate log;

use std::fs;
use std::sync::mpsc::Sender;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rx;
use rx::ecs::layer::EcsInit;
use rx::render::DrawCmd;
use rx::specs::{Dispatcher, System, World, ReadStorage, WriteStorage, DispatcherBuilder};
use rx::specs::Builder;
use rx::specs::WorldExt;
use rx::ecs::{Transformation, Render, WinitEvents};

mod systems;



fn main() {
    env_logger::from_env(env_logger::Env::default().default_filter_or(
        "\
         info,\
         winit::platform_impl::platform::event_loop::runner=error,\
         gfx_backend_vulkan=warn\
         ",
    ))
    .init();

    let mut eng = rx::run::Engine::default();

    let render_sys = systems::RenderSubmitSystem::new(eng.renderer().queue());
    let input_sys = systems::InputTestSystem { should_affect: false };
    let transform_sys = systems::TransformationSystem;

    let mut ecs_layer = rx::ecs::layer::EcsLayer::new(move |mut world: World, mut dispatcher: DispatcherBuilder<'static, 'static>| {
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
            .build();

        let cam_entity = world.create_entity()
            .with(TargetCamera::default())
            .build();


        world.insert(ActiveCamera(Some(cam_entity)));
        world.insert(CameraTarget(Some(entity)));
        world.insert(WinitEvents::default());

        dispatcher = dispatcher
            .with(input_sys, "in_tst_sys", &[])
            .with(transform_sys, "tsm_sys", &[])
            .with_thread_local(render_sys);
        return (world, dispatcher);
    });


    {
        use rx::ecs::TargetCamera;
        use rx::glm;
        let cam = TargetCamera::default();

        cam.target_at(&glm::vec3(0., 0., 0.), &glm::vec3(0., 0., 0.))
    };

//    unimplemented!();
    eng.push_layer(ecs_layer);
    eng.run();
}
