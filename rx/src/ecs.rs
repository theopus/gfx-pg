use specs::{Dispatcher, WorldExt};

use crate::run::Layer;

pub struct EcsLayer<'a> {
    world: specs::World,
    dispatcher: specs::Dispatcher<'a, 'a>,
}

impl<'a> Layer for EcsLayer<'a> {
    fn on_update(&mut self) {
        self.dispatcher.dispatch(&self.world);
    }
}

impl<'a> EcsLayer<'a> {
    pub fn new() -> Self {
        let mut world: specs::World = specs::WorldExt::new();
        let mut dispatcher: Dispatcher = specs::DispatcherBuilder::new().build();
        EcsLayer { world, dispatcher }
    }
}
