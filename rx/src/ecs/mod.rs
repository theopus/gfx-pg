use glm::Vec3;
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

use crate::events;
use crate::events::RxEvent;

pub mod base_systems;

pub mod layer;


#[derive(Default, Debug)]
pub struct WinitEvents(pub Vec<events::WinitEvent>);

#[derive(Default)]
pub struct Egui(pub Option<egui::CtxRef>);

pub type EventSender<T> = Option<crossbeam_channel::Sender<RxEvent<T>>>;
pub type EventReceiver<T> = Option<crossbeam_channel::Receiver<RxEvent<T>>>;

#[derive(Default)]
pub struct EventChannel<T: 'static + Send + Clone> {
    pub s: EventSender<T>,
    pub r: EventReceiver<T>,
}

/*
    Events setup

    fn setup(&mut self, world: &mut specs::World) {
        use rx::specs::SystemData;
        Self::SystemData::setup(world);
        let rs = rx::ecs::fetch_events_channel::<()>(world);
    }
 */

pub fn fetch_events_channel<T: 'static + Send + Clone>(world: &mut specs::World) -> (EventSender<T>, EventReceiver<T>) {
    if let Some(channel) = world.try_fetch::<EventChannel<T>>() {
        return (channel.s.clone(), channel.r.clone());
    }
    (None, None)
}


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

