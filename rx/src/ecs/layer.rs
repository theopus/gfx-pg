use std::convert::identity;
use std::sync::{Arc, mpsc, Mutex, Weak};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use specs::{DispatcherBuilder, World, WorldExt};

use crate::ecs::{WinitEvents};
use crate::events::MyEvent;
use crate::run::Layer;

pub struct EcsLayer<'a> {
    world: specs::World,
    rated_dispatcher: specs::Dispatcher<'a, 'a>,
    constant_dispatcher: specs::Dispatcher<'a, 'a>,
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

        {
            let start = Instant::now();
            let mut count = 0;
            while self.lag >= DURATION_PER_UPD {
                self.rated_dispatcher.dispatch(&self.world);
                let mut events_resource = self.world.write_resource::<WinitEvents>();
                events_resource.0.clear();
                self.lag -= DURATION_PER_UPD;
                count += 1;
            }
            debug!("rated dispatch took {:?}, {:?} times", Instant::now() - start, count);
        }

        {
            let start = Instant::now();
            self.constant_dispatcher.dispatch(&self.world);
            debug!("constant dispatcher took {:?}", Instant::now() - start);
        }
    }

    fn name(&self) -> &'static str {
        "layer_ecs"
    }
}


pub type EcsInitTuple<'a> = (World, DispatcherBuilder<'a, 'a>, DispatcherBuilder<'a, 'a>);

pub trait EcsInit<'a> {
    fn init(self, tuple: EcsInitTuple<'a>) -> EcsInitTuple<'a>;
}

impl<'a, F> EcsInit<'a> for F where F: FnOnce(EcsInitTuple<'a>) -> EcsInitTuple<'a> {
    fn init(self, tuple: EcsInitTuple<'a>) -> EcsInitTuple<'a> {
        self(tuple)
    }
}

impl<'a> Default for EcsLayer<'a> {
    fn default() -> Self {
        EcsLayer::new(identity)
    }
}


impl<'a> EcsLayer<'a> {
    pub fn new<I>(i: I) -> Self where I: EcsInit<'a> {
        let world: specs::World = specs::WorldExt::new();
        let rated_dispatcher = specs::DispatcherBuilder::new();
        let constant_dispatcher = specs::DispatcherBuilder::new();
        let (mut world, rated_dispatcher, constant_dispatcher) = i.init((world, rated_dispatcher, constant_dispatcher));
        Self {
            world,
            rated_dispatcher: rated_dispatcher.build(),
            constant_dispatcher: constant_dispatcher.build(),
            lag: Duration::new(0, 0)
        }
    }
}
