use specs::{Dispatcher, World, WorldExt};

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

pub trait EcsInit<'a, 'b: 'a> {
    fn init(&mut self, world: World, dispatcher: Dispatcher<'a, 'a>) -> (World, Dispatcher<'b, 'b>);
}
//lmao bruh
impl<'a, 'b, F> EcsInit<'a, 'b> for F where F: FnMut(World, Dispatcher<'a, 'a>) -> (World, Dispatcher<'b, 'b>), 'b: 'a {
    fn init(&mut self, world: World, dispatcher: Dispatcher<'a, 'a>) -> (World, Dispatcher<'b, 'b>) {
        self(world, dispatcher)
    }
}

impl<'a> Default for EcsLayer<'a> {
    fn default() -> Self {
        EcsLayer::new(|w, d| (w, d))
    }
}


impl<'a> EcsLayer<'a> {
    pub fn new<I: 'a>(mut i: I) -> Self where I: EcsInit<'a, 'a> {
        let mut world: specs::World = specs::WorldExt::new();
        let mut dispatcher: Dispatcher = specs::DispatcherBuilder::new().build();
        let (world, dispatcher) = i.init(world, dispatcher);
        Self { world, dispatcher }
    }
}
