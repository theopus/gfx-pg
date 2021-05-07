use std::convert::identity;
use std::time::{Duration, Instant};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use specs::{DispatcherBuilder, World, WorldExt};

use crate::ecs::WinitEvents;
use crate::run::{FrameUpdate, Layer};

pub struct EcsLayer<'a> {
    world: specs::World,
    rated_dispatcher: specs::Dispatcher<'a, 'a>,
    constant_dispatcher: specs::Dispatcher<'a, 'a>,
    lag: Duration,
}

const UPD_60_PER_SEC_NANOS: u64 = 16600000;
const DURATION_PER_UPD: Duration = Duration::from_nanos(UPD_60_PER_SEC_NANOS);

impl<'a, T: Clone + Send + Sync> Layer<T> for EcsLayer<'a> {
    fn on_update(&mut self, frame: FrameUpdate<T>) {
        self.lag += frame.elapsed;
        {
            let mut events_resource = self.world.write_resource::<WinitEvents<T>>();
            for e in frame.events.iter() {
                events_resource.0.push((*e).clone());
            }
        }

        {
            let start = Instant::now();
            let mut count = 0;
            while self.lag >= DURATION_PER_UPD {
                self.rated_dispatcher.dispatch(&self.world);
                let mut events_resource = self.world.write_resource::<WinitEvents<T>>();
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

impl<'a> Default for EcsLayer<'a> {
    fn default() -> Self {
        EcsLayer::new(Box::new(|_| {}))
    }
}

pub type EcsInitTuple<'a, 'r> = (&'r mut World, &'r mut DispatcherBuilder<'a, 'a>, &'r mut DispatcherBuilder<'a, 'a>);
pub type EcsInit<'a> = Box<dyn FnOnce(EcsInitTuple)>;

impl<'a> EcsLayer<'a> {
    pub fn new<'b>(i: EcsInit<'a>) -> Self {
        let mut world: specs::World = specs::WorldExt::new();
        let mut rated_dispatcher: DispatcherBuilder<'a, 'a> = specs::DispatcherBuilder::new();
        let mut constant_dispatcher: DispatcherBuilder<'a, 'a> = specs::DispatcherBuilder::new();
        i((&mut world, &mut rated_dispatcher, &mut constant_dispatcher));
        Self {
            world,
            rated_dispatcher: rated_dispatcher.build(),
            constant_dispatcher: constant_dispatcher.build(),
            lag: Duration::new(0, 0),
        }
    }
}
