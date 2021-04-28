pub mod test {
    #[allow(unused_imports)]
    use log::{debug, error, info, trace, warn};

    use rx::ecs::{
        Position, SelectedEntity,
        Velocity, WinitEvents,
    };
    use rx::ecs::base_systems::camera3d::{
        ActiveCamera, CameraTarget, TargetedCamera, ViewProjection,
    };
    use rx::events::MyEvent;
    use rx::glm;
    use rx::glm::{Vec2, Vec3};
    use rx::specs::{Entity, Join, Read, ReadStorage, System, WriteStorage};
    use rx::specs::Component;
    use rx::specs::storage::VecStorage;
    use rx::winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode};

    use crate::maths;

    #[derive(Default)]
    pub struct MoveClickSystem {
        x: f32,
        y: f32,
        w: u32,
        h: u32,

        pressed: bool,
    }

    impl<'a> System<'a> for MoveClickSystem {
        type SystemData = (
            Read<'a, ViewProjection>,
            Read<'a, WinitEvents>,
            Read<'a, ActiveCamera>,
            ReadStorage<'a, TargetedCamera>,
            Read<'a, CameraTarget>,
            WriteStorage<'a, Position>,
            WriteStorage<'a, Velocity>,
            Read<'a, SelectedEntity>,
        );

        fn run(&mut self, data: Self::SystemData) {
            let (vp, events, active_cam, camera, target, mut pos, mut vel, selected) = data;

            let cam = camera.get(active_cam.0.unwrap()).unwrap();
            let mut _posit = pos.get_mut(target.0.unwrap()).unwrap();
            // let mut velos: &mut Velocity = vel.get_mut(target.0.unwrap()).unwrap();
            let sel: &mut Position = pos.get_mut(selected.0.unwrap()).unwrap();
            let mut sel_vel: &mut Velocity = vel.get_mut(selected.0.unwrap()).unwrap();

            for e in &events.0 {
                match e {
                    MyEvent::CursorMoved { position, .. } => {
                        self.x = position.x as f32;
                        self.y = position.y as f32;
                    }
                    MyEvent::Resized(w, h) => {
                        self.w = *w;
                        self.h = *h;
                    }
                    MyEvent::MouseInput {
                        state,
                        button: MouseButton::Middle,
                        ..
                    } => match state {
                        ElementState::Pressed => self.pressed = true,
                        ElementState::Released => self.pressed = false,
                    },
                    _ => {}
                }
            }

            if self.pressed {
                let vec =
                    maths::screen2world((self.x, self.y), (self.w, self.h), &vp.view, &vp.proj);
                let intersect = maths::intersection(
                    &glm::vec3(0., 1., 0.),
                    &glm::vec3(0., 0., 0.),
                    &vec,
                    &cam.cam_pos,
                )
                    .unwrap();
                info!("{:?}", &intersect);

                let dir = &intersect - &sel.as_vec3();
                sel_vel.v = glm::normalize(&dir);
            }
        }
    }

    #[derive(Default)]
    pub struct InputTestSystem {
        pub should_affect_angle: bool,
        pub should_affect_distance: bool,
        pub speed: f32,
        pub vert: f32,
        pub hor: f32,
        pad: MovePad,
    }

    #[derive(Default, Debug)]
    struct MovePad {
        pub up: bool,
        pub down: bool,
        pub right: bool,
        pub left: bool,
    }

    impl MovePad {
        pub fn is_active(&self) -> bool {
            self.up || self.down || self.right || self.left
        }

        pub fn as_vec2(&self) -> Vec2 {
            let y: f32 = if !(self.up ^ self.down) {
                0.
            } else {
                if self.up {
                    1.
                } else {
                    -1.
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
                glm::normalize(&glm::vec2(y, x))
            } else {
                glm::vec2(y, x)
            }
        }
    }

    impl<'a> System<'a> for InputTestSystem {
        type SystemData = (
            Read<'a, WinitEvents>,
            Read<'a, ActiveCamera>,
            Read<'a, CameraTarget>,
            WriteStorage<'a, Position>,
            //        Write<'a, CameraTarget>,
            WriteStorage<'a, TargetedCamera>,
            WriteStorage<'a, Velocity>,
        );

        fn run(&mut self, data: Self::SystemData) {
            let (events, active, target, mut position, mut camera, mut velocity) = data;

            let events = &events.0;
            let cam = camera.get_mut(active.0.unwrap()).unwrap();
            let pos = position.get_mut(target.0.unwrap()).unwrap();

            let mut accum_delta = (0.0, 0.0);
            let mut accum_dist = 0_f32;
            for event in events {
                match event {
                    MyEvent::MouseMotion { delta } => {
                        if self.should_affect_angle {
                            accum_delta.0 += delta.0;
                            accum_delta.1 += delta.1;
                        }
                        if self.should_affect_distance {
                            accum_dist += delta.1 as f32;
                        }
                    }
                    MyEvent::MouseInput {
                        state,
                        button: MouseButton::Left,
                        ..
                    } => match state {
                        ElementState::Pressed => self.should_affect_angle = true,
                        ElementState::Released => self.should_affect_angle = false,
                    },
                    MyEvent::MouseInput {
                        state,
                        button: MouseButton::Right,
                        ..
                    } => match state {
                        ElementState::Pressed => self.should_affect_distance = true,
                        ElementState::Released => self.should_affect_distance = false,
                    },
                    //move
                    MyEvent::KeyboardInput {
                        input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            ..
                        },
                        ..
                    } => pos.y += 1.,
                    MyEvent::KeyboardInput {
                        input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::C),
                            ..
                        },
                        ..
                    } => pos.y -= 1.,
                    MyEvent::KeyboardInput {
                        input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(VirtualKeyCode::W),
                            ..
                        },
                        ..
                    } => {
                        self.pad.up = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        }
                    }
                    MyEvent::KeyboardInput {
                        input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(VirtualKeyCode::S),
                            ..
                        },
                        ..
                    } => {
                        self.pad.down = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        }
                    }
                    MyEvent::KeyboardInput {
                        input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(VirtualKeyCode::A),
                            ..
                        },
                        ..
                    } => {
                        self.pad.left = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        }
                    }
                    MyEvent::KeyboardInput {
                        input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(VirtualKeyCode::D),
                            ..
                        },
                        ..
                    } => {
                        self.pad.right = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        }
                    }
                    MyEvent::Resized(w, h) => cam.update_aspect(*w as f32 / *h as f32),
                    _ => (),
                };
            }

            cam.distance += 0.4 * accum_dist;
            cam.yaw += 0.4 * accum_delta.0 as f32;
            cam.pitch -= 0.4 * accum_delta.1 as f32;

            let degree = cam.yaw - 180.;

            let d_vec: Vec2 = self.pad.as_vec2();
            if self.pad.is_active() {
                self.speed = 0.5;

                let d_vec: Vec2 = glm::rotate_vec2(&d_vec, glm::radians(&glm::vec1(degree)).x);
                let move_vec = self.speed * d_vec;

                let mut v: &mut Velocity = velocity.get_mut(target.0.unwrap()).unwrap();
                v.v = glm::vec3(move_vec.y, 0., move_vec.x);
            };
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

pub mod generic {
    use std::sync::mpsc::Sender;

    #[allow(unused_imports)]
    use log::{debug, error, info, trace, warn};

    use rx::ecs::{
        Render, Transformation,
    };
    use rx::ecs::base_systems::camera3d::{
        ActiveCamera, TargetedCamera,
    };
    use rx::glm;
    use rx::render::{DrawCmd, RenderCommand};
    use rx::specs::{Join, Read, ReadStorage, System, WriteStorage};

    pub struct RenderSubmitSystem {
        send_draw: Sender<DrawCmd>,
        send_render: Sender<RenderCommand>,
    }

    impl RenderSubmitSystem {
        pub fn new(send_draw: Sender<DrawCmd>, send_render: Sender<RenderCommand>) -> Self {
            Self {
                send_draw,
                send_render,
            }
        }
    }

    impl<'a> System<'a> for RenderSubmitSystem {
        type SystemData = (
            Read<'a, ActiveCamera>,
            ReadStorage<'a, TargetedCamera>,
            ReadStorage<'a, Transformation>,
            WriteStorage<'a, Render>,
        );

        fn run(&mut self, (active, camera, transformation, mut render): Self::SystemData) {
            let cam = camera.get(active.0.unwrap()).unwrap();
            self.send_render
                .send(RenderCommand::PushView(cam.view.clone())).unwrap();
            for (transformation, render) in (&transformation, &mut render).join() {
                self.send_draw
                    .send((
                        render.mesh.clone(),
                        transformation.mvp.clone() as glm::Mat4,
                        transformation.model.clone() as glm::Mat4,
                    ))
                    .expect("not able to submit");
            }
        }
    }
}
