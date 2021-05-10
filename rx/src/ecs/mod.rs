use glm::Vec3;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use specs::{Component, Entity, shrev::EventChannel, VecStorage};
use winit::event_loop::EventLoopProxy;

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

pub mod base_systems;

pub mod layer;
pub mod systems;


pub type EventReader<T> = Option<specs::shrev::ReaderId<RxEvent<T>>>;
pub type EventChannelReader<T> = EventChannel<RxEvent<T>>;
pub type EventWriter<T> = Option<crossbeam_channel::Sender<RxEvent<T>>>;

#[derive(Default, Debug)]
pub struct WinitEvents<T: 'static + Clone + Send>(pub Option<Vec<events::WinitEvent<T>>>);

#[derive(Debug, Clone)]
pub enum EcsEvent {
    ScreenClick(ScreenClickEvent)
}

#[derive(Debug, Clone)]
pub struct ScreenClickEvent {
    pub screen_pos: (f64, f64),
    pub world_vec: glm::Vec3,
    pub cam_pos: glm::Vec3,
    pub mouse_button: winit::event::MouseButton,
    pub state: winit::event::ElementState,
}

impl<T: 'static + Send + Clone> Into<RxEvent<T>> for ScreenClickEvent {
    fn into(self) -> RxEvent<T> {
        RxEvent::EcsEvent(EcsEvent::ScreenClick(self))
    }
}

pub type EguiCtx = Option<egui::CtxRef>;

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

