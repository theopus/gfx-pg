pub mod test {
    #[allow(unused_imports)]
    use log::{debug, error, info, trace, warn};

    use rx::{EventWriter, glm, RxEvent};
    use rx::ecs::{
        Position, SelectedEntity,
        Velocity, WinitEvents,
    };
    use rx::ecs::base_systems::camera3d::{
        ActiveCamera, CameraTarget, TargetedCamera, ViewProjection,
    };
    use rx::events::RxEvent::EcsEvent;
    use rx::glm::{Vec2, Vec3};
    use rx::specs::{Entity, Join, Read, ReadStorage, System, Write, WriteStorage};
    use rx::specs::Component;
    use rx::specs::shrev::EventChannel;
    use rx::specs::storage::VecStorage;
    use rx::winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode};

    use crate::egui::CtxRef;
    use crate::maths;
    use crate::specs::World;

    #[derive(Default)]
    pub struct MoveClickSystem {
        x: f32,
        y: f32,
        w: u32,
        h: u32,

        pressed: bool,
        reader: rx::EventReader<()>,
    }

    #[derive(Default)]
    pub struct ScreenClickSystem {
        reader: rx::EventReader<()>,
        writer: rx::EventWriter<()>,

        pressed: bool,

        x: f64,
        y: f64,
        w: u32,
        h: u32,
    }

    impl<'a> System<'a> for ScreenClickSystem {
        type SystemData = (
            Read<'a, EventChannel<RxEvent<()>>>,
            Read<'a, ViewProjection>
        );

        fn run(&mut self, (events, vp): Self::SystemData) {
            use rx::winit::event::{Event, WindowEvent};

            if let Some(reader_id) = &mut self.reader {
                for rx_e in &mut events.read(reader_id) {
                    match rx_e {
                        rx::RxEvent::WinitEvent(e) => match e {
                            Event::WindowEvent {
                                event: WindowEvent::CursorMoved { position, .. },
                                ..
                            } => {
                                self.x = position.x;
                                self.y = position.y;
                            }
                            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                                self.w = size.width;
                                self.h = size.height;
                            }
                            Event::WindowEvent {
                                event: WindowEvent::MouseInput { state, button, .. },
                                ..
                            } => match state {
                                ElementState::Pressed => self.writer.iter().for_each(|w| {
                                    w.send(rx::ScreenClickEvent {
                                        screen_pos: (self.x, self.y),
                                        world_vec: maths::screen2world((self.x as f32, self.y as f32), (self.w, self.h), &vp.view, &vp.proj),
                                        mouse_button: button.clone(),
                                        state: state.clone(),
                                    }.into());
                                }),
                                _ => {}
                            }
                            _ => {}
                        }
                        _ => {}
                    }
                }
            }
        }

        fn setup(&mut self, world: &mut World) {
            use rx::{
                specs::SystemData,
                specs::shrev::EventChannel,
            };
            Self::SystemData::setup(world);
            self.reader = Some(world.fetch_mut::<EventChannel<RxEvent<()>>>().register_reader());
            self.writer = world.fetch_mut::<EventWriter<()>>().clone();
        }
    }

    impl<'a> System<'a> for MoveClickSystem {
        type SystemData = (
            Read<'a, ViewProjection>,
            Write<'a, EventChannel<RxEvent<()>>>,
            Read<'a, ActiveCamera>,
            ReadStorage<'a, TargetedCamera>,
            Read<'a, CameraTarget>,
            WriteStorage<'a, Position>,
            WriteStorage<'a, Velocity>,
            Read<'a, SelectedEntity>,
        );


        fn run(&mut self, data: Self::SystemData) {
            let (vp, mut event_channel, active_cam, camera, target, mut pos, mut vel, selected) = data;

            let cam = camera.get(active_cam.0.unwrap()).unwrap();
            let mut _posit = pos.get_mut(target.0.unwrap()).unwrap();
            // let mut velos: &mut Velocity = vel.get_mut(target.0.unwrap()).unwrap();
            let sel: &mut Position = pos.get_mut(selected.0.unwrap()).unwrap();
            let mut sel_vel: &mut Velocity = vel.get_mut(selected.0.unwrap()).unwrap();

            use rx::winit::event::{Event, WindowEvent};
            if let Some(reader_id) = &mut self.reader {
                for rx_e in &mut event_channel.read(reader_id) {
                    match rx_e {
                        rx::RxEvent::WinitEvent(e) => match e {
                            Event::WindowEvent {
                                event: WindowEvent::CursorMoved { position, .. },
                                ..
                            } => {
                                self.x = position.x as f32;
                                self.y = position.y as f32;
                            }
                            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                                self.w = size.width;
                                self.h = size.height;
                            }
                            Event::WindowEvent {
                                event: WindowEvent::MouseInput { state, button: MouseButton::Middle, .. },
                                ..
                            } => match state {
                                ElementState::Pressed => self.pressed = true,
                                ElementState::Released => self.pressed = false,
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }


            if self.pressed {
                let vec =
                    maths::screen2world((self.x, self.y), (self.w, self.h), &vp.view, &vp.proj);
                let mut intersect = maths::intersection(
                    &glm::vec3(0., 1., 0.),
                    &glm::vec3(0., 0., 0.),
                    &vec,
                    &cam.cam_pos,
                )
                    .unwrap();
                info!("intersection: {:?}", vec);
                intersect.y = 0.;

                let dir = &intersect - &sel.as_vec3();
                sel_vel.v = glm::normalize(&dir) as glm::Vec3;
            }
        }

        fn setup(&mut self, world: &mut World) {
            use rx::{
                specs::SystemData,
                specs::shrev::EventChannel,
            };
            Self::SystemData::setup(world);
            self.reader = Some(world.fetch_mut::<EventChannel<RxEvent<()>>>().register_reader());
        }
    }


    #[derive(Default, Debug)]
    pub struct MovePad {
        pub up: bool,
        pub down: bool,
        pub right: bool,
        pub left: bool,
    }

    impl MovePad {
        pub fn is_active(&self) -> bool {
            self.up || self.down || self.right || self.left
        }

        pub fn as_vec2(&self, invert_up: bool) -> Vec2 {
            let y: f32 = if !(self.up ^ self.down) {
                0.
            } else {
                if self.up {
                    if invert_up {
                        -1.
                    } else {
                        1.
                    }
                } else {
                    if invert_up {
                        1.
                    } else {
                        -1.
                    }
                }
            };
            let x: f32 = if !(self.right ^ self.left) {
                0.
            } else {
                if self.right {
                    1.
                } else {
                    -1.
                }
            };

            if !(y == 0. && x == 0.) {
                glm::normalize(&glm::vec2(y, x)) as glm::Vec2
            } else {
                glm::vec2(y, x) as glm::Vec2
            }
        }
    }

    pub struct MoveSystem;

    impl<'a> System<'a> for MoveSystem {
        type SystemData = (WriteStorage<'a, Position>, WriteStorage<'a, Velocity>);

        fn run(&mut self, (mut pos, mut vel): Self::SystemData) {
            for (p, v) in (&mut pos, &mut vel).join() {
                let velocity: Vec3 = v.v;
                const SLOW: f32 = 0.05;
                p.x += velocity.x;
                p.y += velocity.y;
                p.z += velocity.z;

                let inversed: Vec3 = -1. * &velocity;
                let oposite_vec = if !(inversed.x == 0. && inversed.y == 0. && inversed.z == 0.) {
                    glm::normalize(&inversed)
                } else {
                    inversed
                };

                v.v = velocity + oposite_vec * SLOW;
                if v.v.x >= -1.3 && v.v.x <= 1.3 {
                    v.v.x = 0.;
                }
                if v.v.z >= -1.3 && v.v.z <= 1.3 {
                    v.v.z = 0.;
                }
            }
        }
    }

    #[derive(Component, Debug)]
    #[storage(VecStorage)]
    pub struct Follower {
        pub lead: Entity,
    }

    pub struct FollowingSystem;

    impl<'a> System<'a> for FollowingSystem {
        type SystemData = (
            ReadStorage<'a, Follower>,
            ReadStorage<'a, Position>,
            WriteStorage<'a, Velocity>,
        );

        fn run(&mut self, (fol, pos, mut vel): Self::SystemData) {
            for (f, p, v) in (&fol, &pos, &mut vel).join() {
                let lp = pos.get(f.lead).unwrap();
                v.v =
                    glm::normalize(&(glm::vec3(lp.x, lp.y, lp.z) - glm::vec3(p.x, p.y, p.z))) * 0.3;
            }
        }
    }
}