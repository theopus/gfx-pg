use std::convert::identity;
use std::time::{Duration, Instant};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use specs::{DispatcherBuilder, shrev::EventChannel, World, WorldExt};

use crate::ecs::WinitEvents;
use crate::glm::e;
use crate::run::{FrameUpdate, Layer};
use crate::RxEvent;
use crate::winit::event_loop::EventLoopProxy;

pub struct EcsLayer<'a> {
    world: specs::World,
    rated_dispatcher: specs::Dispatcher<'a, 'a>,
    constant_dispatcher: specs::Dispatcher<'a, 'a>,
    lag: Duration,
}

const UPD_60_PER_SEC_NANOS: u64 = 16600000;
const DURATION_PER_UPD: Duration = Duration::from_nanos(UPD_60_PER_SEC_NANOS);

impl<'a, T: 'static + Clone + Send + Sync> Layer<T> for EcsLayer<'a> {
    fn on_update(&mut self, frame: FrameUpdate<T>) {
        self.lag += frame.elapsed;

        {
            self.world.write_resource::<EventChannel<RxEvent<T>>>()
                .iter_write(frame.events
                    .into_iter()
                    .map(|e| { e.clone() }))
        }
        //rated
        {
            let start = Instant::now();
            let mut count = 0;
            while self.lag >= DURATION_PER_UPD {
                self.rated_dispatcher.dispatch(&self.world);
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

    fn setup(&mut self) {
        self.world.insert(specs::shrev::EventChannel::new() as specs::shrev::EventChannel<RxEvent<T>>);
        self.rated_dispatcher.setup(&mut self.world);
        self.constant_dispatcher.setup(&mut self.world);
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
