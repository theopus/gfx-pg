use std::time::Duration;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use specs::{Dispatcher, DispatcherBuilder, World, WorldExt};
use winit::event::Event;

use crate::ecs::WinitEvents;
use crate::events::MyEvent;
use crate::run::Layer;

pub struct EcsLayer<'a> {
    world: specs::World,
    dispatcher: specs::Dispatcher<'a, 'a>,
    lag: Duration
}

const UPD_60_PER_SEC_NANOS: u64 = 16600000;
const DURATION_PER_UPD: Duration = Duration::from_nanos(UPD_60_PER_SEC_NANOS);

impl<'a> Layer for EcsLayer<'a> {
    fn on_update(&mut self, events: &Vec<MyEvent>, elapsed: Duration) {
        self.lag += elapsed;
        {
            let mut events_resource = self.world.write_resource::<WinitEvents>();
            for e in events.iter() {
                events_resource.0.push((*e).clone());
            }
        }

        while self.lag >= DURATION_PER_UPD {
            self.dispatcher.dispatch(&self.world);
            {
                let mut events_resource = self.world.write_resource::<WinitEvents>();
                events_resource.0.clear();
            }
            self.lag -= DURATION_PER_UPD;
        }
    }
}

pub trait EcsInit<'a> {
    fn init(mut self, world: World, dispatcher: DispatcherBuilder<'a, 'a>) -> (World, DispatcherBuilder<'a, 'a>);
}
//lmao bruh
impl<'a, F> EcsInit<'a> for F where F: FnOnce(World, DispatcherBuilder<'a, 'a>) -> (World, DispatcherBuilder<'a, 'a>) {
    fn init(mut self, world: World, dispatcher: DispatcherBuilder<'a, 'a>) -> (World, DispatcherBuilder<'a, 'a>) {
        self(world, dispatcher)
    }
}

impl<'a> Default for EcsLayer<'a> {
    fn default() -> Self {
        EcsLayer::new(|w, d| (w, d))
    }
}


impl<'a> EcsLayer<'a> {
    pub fn new<I>(mut i: I) -> Self where I: EcsInit<'a> {
        let mut world: specs::World = specs::WorldExt::new();
        let mut dispatcher = specs::DispatcherBuilder::new();
        let (world, dispatcher) = i.init(world, dispatcher);
        Self {
            world,
            dispatcher: dispatcher.build(),
            lag: Duration::new(0, 0)
        }
    }
}
