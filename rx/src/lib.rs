extern crate bytemuck;
pub extern crate egui;
pub extern crate nalgebra as na;
pub extern crate nalgebra_glm as glm;
pub extern crate specs;
pub extern crate wgpu;
pub extern crate winit;

pub mod window;
pub mod utils;
pub mod run;
pub mod ecs;
pub mod assets;
pub mod events;
pub mod wgpu_graphics;
pub mod render_w;
pub mod graphics_api;
mod gui;


pub use ecs::{
    layer::{
        EcsInit,
        EcsInitTuple
    },
    base_systems::world3d::{
        Render,
        TransformationSystem,
        RenderSubmitSystem,
    },
    base_systems::camera3d::{
        Camera,
        TargetedCamera,
        CameraTarget,
        ActiveCamera,
    },
    Position,
    Rotation,
    Velocity,
    Transformation,
    SelectedEntity,
    WinitEvents,
    EventReader,
    EventWriter,
    EventChannelReader,
    EcsEvent,
    ScreenClickEvent,
    EguiCtx,
};

pub use assets::{
    MeshPtr
};

pub use events::{
    WinitEvent,
    RxEvent
};

pub use run::{
    Layer,
};
