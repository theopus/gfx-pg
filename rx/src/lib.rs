#[cfg(feature = "dx12")]
pub extern crate gfx_backend_dx12 as back;
#[cfg(feature = "gl")] //TODO: figureout later
pub extern crate gfx_backend_gl as back;
#[cfg(feature = "metal")]
pub extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
pub extern crate gfx_backend_vulkan as back;
pub extern crate gfx_hal as hal;
pub extern crate nalgebra as na;
pub extern crate nalgebra_glm as glm;
#[cfg(target_arch = "wasm32")]
pub extern crate shaderc;
pub extern crate specs;
pub extern crate winit;
pub extern crate wgpu;

pub mod window;
pub mod utils;
#[cfg(feature = "hal")]
pub mod graphics;
pub mod run;
pub mod ecs;
#[cfg(feature = "hal")]
pub mod render;
pub mod assets;
pub mod events;
pub mod wgpu_graphics;
pub mod render_w;
pub mod graphics_api;

