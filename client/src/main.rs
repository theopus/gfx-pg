use rx;
use rx::env_logger;
use rx::run::Layer;
#[allow(unused_imports)]
use rx::log::{debug, error, info, trace, warn};


fn main() {
    env_logger::from_env(
        env_logger::Env::default()
            .default_filter_or("info,winit::platform_impl::platform::event_loop::runner=error,gfx_backend_vulkan=info"))
        .init();

    let mut eng = rx::run::Engine::default();
    eng.run();
}
