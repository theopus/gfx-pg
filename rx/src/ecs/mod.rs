use glm::Vec3;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use specs::{Component, Entity, VecStorage, shrev::EventChannel};

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

use crate::{events, RxEvent};
use winit::event_loop::EventLoopProxy;

pub mod base_systems;

pub mod layer;


#[derive(Default, Debug)]
pub struct WinitEvents<T: 'static + Clone + Send>(pub Option<Vec<events::WinitEvent<T>>>);
pub struct EventSender<T: 'static + Clone + Send>(pub EventLoopProxy<RxEvent<T>>);

#[derive(Default)]
pub struct Egui(pub Option<egui::CtxRef>);

#[derive(Default)]
pub struct SelectedEntity(pub Option<Entity>);

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

