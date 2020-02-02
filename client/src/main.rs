extern crate env_logger;
extern crate log;

use std::fs;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rx;

fn main() {
    env_logger::from_env(
        env_logger::Env::default()
            .default_filter_or("\
            info,\
            winit::platform_impl::platform::event_loop::runner=error,\
            gfx_backend_vulkan=debug\
            "))
        .init();
    let mut ecs_layer = rx::ecs::EcsLayer::default();
    let mut eng = rx::run::Engine::default();

    eng.push_layer(ecs_layer);
    eng.run();
}
