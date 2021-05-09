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

use crate::systems::test::Follower;
use crate::winit::event::Event;

mod flowchart;
mod generatin;
mod map;
mod maths;
mod systems;
mod arrowdrop;
mod input_sys;
mod gui_sys;

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

            let (.., transform_sys) = init(&mut world, &glm::vec3(0., 0., 0.));

            let player = world
                .create_entity()
                .with(Rotation::default())
                .with(Position::default())
                .with(Transformation::default())
                .with(Velocity::default())
                .with(Render::new(arrow_02.clone()))
                .build();

            let selected = world
                .create_entity()
                .with(Rotation::default())
                .with(Position::default())
                .with(Transformation::default())
                .with(Velocity::default())
                .with(Render::new(arrow_01.clone()))
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
                .with(Render::new(map_mesh_ptr.clone()))
                .build();

            world.insert(SelectedEntity(Some(selected)));
            world.insert(WinitEvents::default() as WinitEvents<()>);
            world.insert(CameraTarget(Some(player)));

            r_dispatcher.add(systems::test::FollowingSystem, "follow_sys", &[]);
            r_dispatcher.add(systems::test::ScreenClickSystem::default(), "screen_click_sys", &[]);
            //
            r_dispatcher.add(input_sys, "in_tst_sys", &[]);
            r_dispatcher.add(move_sys, "move_sys", &[]);
            r_dispatcher.add(mouse_sys, "mouse_sys", &[]);
            r_dispatcher.add(transform_sys, "tsm_sys", &[]);

            c_dispatcher.add_thread_local(gui_sys::GuiSystem::default());
            arrowdrop::create((world, r_dispatcher, c_dispatcher), _cube_mesh.clone());
            c_dispatcher.add_thread_local(rx::RenderSubmitSystem::new(draw, redner));
        })
    );

    eng.push_layer(ecs_layer);

    let mut frames = 0;
    let mut frame_rate = 0.0;
    let mut elapsed = std::time::Duration::from_millis(0);
    let mut size_d: PhysicalSize<u32> = PhysicalSize { width: 0, height: 0 };
    let mut cursor_pos: rx::winit::dpi::PhysicalPosition<f64> = rx::winit::dpi::PhysicalPosition { x: 0., y: 0. };
    eng.push_layer(move |upd: run::FrameUpdate<()>| {
        use rx::egui;
        use rx::winit::event;
        for rx_e in upd.events {
            match rx_e {
                RxEvent::WinitEvent(event) => match event {
                    event::Event::WindowEvent { event: event::WindowEvent::Resized(size), .. } => {
                        size_d = *size
                    }
                    event::Event::WindowEvent { event: event::WindowEvent::CursorMoved { position, .. }, .. } => {
                        cursor_pos = position.clone();
                    }
                    _ => {}
                }
                _ => {}
            }
        }
        elapsed += upd.elapsed;
        frames += 1;
        if elapsed >= std::time::Duration::from_millis(100) {
            frame_rate = frames as f32 * 0.5 + frame_rate * 0.5;
            frames = 0;
            elapsed -= std::time::Duration::from_millis(100)
        }

        egui::Window::new("info")
            .collapsible(false)
            .default_pos((size_d.width as f32 - 190. , 0.))
            .resizable(false)
            .show(&upd.egui_ctx, |ui| {
                egui::Grid::new("info_grid").min_col_width(180.).striped(true).show(ui, |ui| {
                    ui.label(format!("Frame time: {} ms", upd.elapsed.as_millis()));
                    ui.end_row();
                    ui.label(format!("Frames: {:.2} /sec", frame_rate * 10.));
                    ui.end_row();
                    ui.label(format!("Size: {}x{}", size_d.width, size_d.height));
                    ui.end_row();
                    ui.label(format!("Cursor: x: {:.2} y: {:.2}", cursor_pos.x, cursor_pos.y));
                    ui.end_row();
                });
            }).unwrap();
    });
    eng.run();
}

