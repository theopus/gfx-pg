use std::convert::identity;
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use specs::{DispatcherBuilder, shrev::EventChannel, World, WorldExt, AccessorCow};

use crate::{EventWriter, RxEvent};
use crate::ecs::{WinitEvents, EguiCtx};
use crate::glm::e;
use crate::run::{FrameUpdate, Layer};
use crate::winit::event_loop::EventLoopProxy;
use crossbeam_channel;
use std::ops::{Deref, DerefMut};
use crate::egui::CtxRef;

pub struct EcsLayer<'a, T: 'static + Send + Clone> {
    world: specs::World,
    rated_dispatcher: specs::Dispatcher<'a, 'a>,
    constant_dispatcher: specs::Dispatcher<'a, 'a>,
    lag: Duration,
    channel: Option<(crossbeam_channel::Sender<RxEvent<T>>, crossbeam_channel::Receiver<RxEvent<T>>)>,
}


const UPD_60_PER_SEC_NANOS: u64 = 16600000;
const UPD_60_PER_SEC_NANOS_LAG: u64 = 19600000;
const DURATION_PER_UPD: Duration = Duration::from_nanos(UPD_60_PER_SEC_NANOS);
const DURATION_PER_UPD_LAG: Duration = Duration::from_nanos(UPD_60_PER_SEC_NANOS_LAG);

impl<'a, T: 'static + Clone + Send + Sync> Layer<T> for EcsLayer<'a, T> {
    fn on_update(&mut self, frame: FrameUpdate<T>) {
        self.lag += frame.elapsed;


        {
            let start = Instant::now();
            self.world.write_resource::<EguiCtx>().replace(frame.egui_ctx.clone());
            let mut event_channel = self.world.write_resource::<EventChannel<RxEvent<T>>>();
            event_channel
                .iter_write(frame.events
                    .into_iter()
                    .map(|e| { e.clone() }));
            if let Some((_, reader)) = &self.channel {
                for e in reader.try_iter() {
                    event_channel.single_write(e)
                }
            }
            debug!("event_propogate took {:?}", Instant::now() - start);
        }
        //rated


        {

            let start = Instant::now();
            let mut count = 0;
            while self.lag >= DURATION_PER_UPD {
                let start = Instant::now();
                self.rated_dispatcher.dispatch(&self.world);
                self.lag -= DURATION_PER_UPD;
                let dur = Instant::now() - start;
                if dur > DURATION_PER_UPD_LAG {
                    warn!("Last upd is {:?}. dropping lag...", dur);
                    self.lag = Duration::from_secs(0);
                }
                count += 1;
            }
            info!("rated_dispatch took {:?}, excuted {:?} times", Instant::now() - start, count);
        }
        {
            let start = Instant::now();
            self.constant_dispatcher.dispatch(&self.world);
            info!("constant dispatcher took {:?}", Instant::now() - start);
        }
    }

    fn setup(&mut self) {
        self.channel = Some(crossbeam_channel::unbounded());
        self.world.insert(specs::shrev::EventChannel::new() as specs::shrev::EventChannel<RxEvent<T>>);
        self.world.insert(self.channel.as_ref()
            .and_then(|e| { Some(e.0.clone()) }));
        self.rated_dispatcher.setup(&mut self.world);
        self.constant_dispatcher.setup(&mut self.world);
    }

    fn name(&self) -> &'static str {
        "layer_ecs"
    }
}

impl<'a, T: 'static + Send + Clone> Default for EcsLayer<'a, T> {
    fn default() -> Self {
        EcsLayer::new(Box::new(|_| {}))
    }
}

pub type EcsInitTuple<'a, 'r> = (&'r mut World, &'r mut DispatcherBuilder<'a, 'a>, &'r mut DispatcherBuilder<'a, 'a>);
pub type EcsInit<'a> = Box<dyn FnOnce(EcsInitTuple)>;

impl<'a, T: 'static + Send + Clone> EcsLayer<'a, T> {
    pub fn new<'b>(i: EcsInit<'a>) -> Self {
        let mut world: specs::World = specs::WorldExt::new();
        let mut rated_dispatcher = specs::DispatcherBuilder::new();
        let mut constant_dispatcher = specs::DispatcherBuilder::new();
        i((&mut world, &mut rated_dispatcher, &mut constant_dispatcher));
        world.insert(None as EguiCtx);
        Self {
            world,
            rated_dispatcher: rated_dispatcher.build(),
            constant_dispatcher: constant_dispatcher.build(),
            lag: Duration::new(0, 0),
            channel: None
        }
    }
}
