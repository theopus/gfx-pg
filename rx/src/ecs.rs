use std::time::Duration;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use specs::{Dispatcher, World, WorldExt};
use winit::event::Event;

use crate::run::Layer;

pub struct EcsLayer<'a> {
    world: specs::World,
    dispatcher: specs::Dispatcher<'a, 'a>,
    lag: Duration
}

const UPD_60_PER_SEC_NANOS: u64 = 16600000;
const DURATION_PER_UPD: Duration = Duration::from_nanos(UPD_60_PER_SEC_NANOS);

impl<'a> Layer for EcsLayer<'a> {
    fn on_update(&mut self, events: &Vec<Event<()>>, elapsed: Duration) {
        self.lag += elapsed;
        while self.lag >= DURATION_PER_UPD {
            self.dispatcher.dispatch(&self.world);
            self.lag -= DURATION_PER_UPD;
        }
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
        Self {
            world,
            dispatcher,
            lag: Duration::new(0, 0)
        }
    }
}
