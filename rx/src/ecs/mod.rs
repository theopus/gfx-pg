use glm::{Vec3};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use specs::{Component, Entity, VecStorage};

pub use base_systems::{
    camera3d::ActiveCamera,
    camera3d::CameraTarget,
    camera3d::ViewProjection,
};
pub use base_systems::world3d::{
    Position,
    Rotation,
    Transformation,
};

use crate::assets::MeshPtr;
use crate::events::MyEvent;
use std::sync::{Arc, Mutex, Weak, mpsc};
use crate::utils::functions::RawFunc;

pub mod base_systems;

pub mod layer;


#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Render {
    pub mesh: MeshPtr,
}

#[derive(Default, Debug)]
pub struct WinitEvents(pub Vec<MyEvent>);

#[derive(Default)]
pub struct SelectedEntity(pub Option<Entity>);


// #[derive(Default)]
pub struct UiDraw(pub mpsc::Sender<RawFunc>);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Velocity {
    pub v: Vec3,
}

impl Default for Velocity {
    fn default() -> Self {
        Self {
            v: glm::vec3(0., 0., 0.),
        }
    }
}

