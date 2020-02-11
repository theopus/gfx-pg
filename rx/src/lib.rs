#[cfg(feature = "dx12")]
pub extern crate gfx_backend_dx12 as back;
#[cfg(feature = "gl")] //TODO: figureout later
pub extern crate gfx_backend_gl as back;
#[cfg(feature = "metal")]
pub extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
pub extern crate gfx_backend_vulkan as back;
pub extern crate gfx_hal as hal;
pub extern crate nalgebra_glm as glm;
pub extern crate specs;
pub extern crate specs_derive;
pub extern crate winit;

pub mod window;
mod local;
mod utils;
mod graphics;
pub mod run;
pub mod ecs;
pub mod render;
pub mod assets;
