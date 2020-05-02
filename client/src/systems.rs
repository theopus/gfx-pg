pub mod test {
    use std::sync::mpsc::Sender;

    #[allow(unused_imports)]
    use log::{debug, error, info, trace, warn};

    use rx::ecs::{ActiveCamera, CameraTarget, Position, Render, Rotation, TargetCamera, Transformation, Velocity, ViewProjection, WinitEvents};
    use rx::events::MyEvent;
    use rx::glm;
    use rx::glm::{Vec2, Vec3};
    use rx::na::Vector3;
    use rx::render::{DrawCmd, RenderCommand};
    use rx::specs::{Entity, Join, Read, ReadStorage, System, Write, WriteStorage};
    use rx::specs::Component;
    use rx::specs::storage::VecStorage;
    use rx::winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode};

    #[derive(Default)]
    pub struct MoveClickSystem {
        x: f32,
        y: f32,
        w: u32,
        h: u32,
    }

    impl<'a> System<'a> for MoveClickSystem {
        type SystemData = (
            Read<'a, ViewProjection>,
            Read<'a, WinitEvents>,
        );

        fn run(&mut self, data: Self::SystemData) {
            let (vp, events) = data;

            for e in &events.0 {
                match e {
                    MyEvent::CursorMoved { position, .. } => {
                        self.x = position.x as f32;
                        self.y = position.y as f32;
//                        info!("Mouse x:{}, y:{}", self.x, self.y);
                    },
                    MyEvent::Resized(w, h) => {
                        self.w = *w;
                        self.h = *h;
                        info!("Resized w:{}, h:{}", self.w, self.h);
                    },
                    _ => {}
                }
            }

            let i_view = glm::inverse(&vp.view);
            let i_proj = glm::inverse(&vp.proj);

            let d_x = (2. * self.x) / self.w as f32 - 1.;
            let d_y = (2. * self.y) / self.h as f32 - 1.;
            let d_z = 1.;
            let ray_ndc = glm::vec3(d_x, d_y, d_z);
            let ray_clip = glm::vec4(ray_ndc.x, ray_ndc.y, -1., 1.);

            let mut ray_eye: glm::Vec4 = &i_proj * ray_clip;
            let ray_eye = glm::vec4(ray_eye.x, ray_eye.y, -1.0, 0.0);

            let mut ray_wor = &i_view * ray_eye;
            let mut ray_wor = glm::vec4_to_vec3(&ray_wor);
            let ray_wor = glm::normalize(&ray_wor);

//            float x = (2.0f * mouse_x) / width - 1.0f;
//            float y = 1.0f - (2.0f * mouse_y) / height;
//            float z = 1.0f;

//            let mut near = glm::vec4(d_x, d_y, -1., 1.);
//            let mut far = glm::vec4(d_x, d_y, 1., 1.);
//
//            let mut near: glm::Vec4 = &i_vp * near;
//            let mut far: glm::Vec4 = &i_vp * far;
//
//            let mut near: glm::Vec3 = glm::vec3(near.x, near.y, near.z) / near.w;
//            let mut far: glm::Vec3 = glm::vec3(far.x, far.y, far.z) / far.w;
//
//            let dir: glm::Vec3 = glm::normalize(&(far - near));
//
//            info!("x: {:?}", d_x);
//            info!("y: {:?}", d_y);
        }
    }

//    mat4 invMat= inverse(m_glData.getPerspective()*m_glData.getView());
//    vec4 near = vec4((pos.x - Constants::m_halfScreenWidth) / Constants::m_halfScreenWidth, -1*(pos.y - Constants::m_halfScreenHeight) / Constants::m_halfScreenHeight, -1, 1.0);
//    vec4 far = vec4((pos.x - Constants::m_halfScreenWidth) / Constants::m_halfScreenWidth, -1*(pos.y - Constants::m_halfScreenHeight) / Constants::m_halfScreenHeight, 1, 1.0);
//    vec4 nearResult = invMat*near;
//    vec4 farResult = invMat*far;
//    nearResult /= nearResult.w;
//    farResult /= farResult.w;
//    vec3 dir = vec3(farResult - nearResult );
//    return normalize(dir);

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
            let y: f32 = if !(self.up ^ self.down) { 0. } else { if self.up { 1. } else { -1. } };
            let x: f32 = if !(self.right ^ self.left) { 0. } else { if self.right { 1. } else { -1. } };

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
            WriteStorage<'a, TargetCamera>,
            WriteStorage<'a, Velocity>
        );

        fn run(&mut self, data: Self::SystemData) {
            let (
                events,
                active,
                target,
                mut position,
                mut camera,
                mut velocity,
            ) = data;

            let events = &events.0;
            let cam = camera.get_mut(active.0.unwrap()).unwrap();
            let pos = position.get_mut(target.0.unwrap()).unwrap();

            let mut accum_delta = (0.0, 0.0);
            let mut accum_dist = 0_f32;
            for event in events {
                match event {
                    MyEvent::MouseMotion {
                        delta
                    } => {
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
                        input: KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            ..
                        },
                        ..
                    } => pos.y += 1.,
                    MyEvent::KeyboardInput {
                        input: KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::C),
                            ..
                        },
                        ..
                    } => pos.y -= 1.,
                    MyEvent::KeyboardInput {
                        input: KeyboardInput {
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
                    },
                    MyEvent::KeyboardInput {
                        input: KeyboardInput {
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
                    },
                    MyEvent::KeyboardInput {
                        input: KeyboardInput {
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
                    },
                    MyEvent::KeyboardInput {
                        input: KeyboardInput {
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
                    },
                    MyEvent::Resized(w, h) => cam.update_aspect(*w as f32 / *h as f32),
                    _ => ()
                };
            }


            cam.distance += 0.4 * accum_dist;
            cam.yaw += 0.4 * accum_delta.0 as f32;
            cam.pitch -= 0.4 * accum_delta.1 as f32;


            let mut degree = cam.yaw - 180.;

            let mut d_vec: Vec2 = self.pad.as_vec2();
            if self.pad.is_active() {
                self.speed = 0.5;

                let mut d_vec: Vec2 = glm::rotate_vec2(&d_vec, glm::radians(&glm::vec1(degree)).x);
                let move_vec = self.speed * d_vec;

                let mut v: &mut Velocity = velocity.get_mut(target.0.unwrap()).unwrap();
                v.v = glm::vec3(move_vec.y, 0., move_vec.x);
            };
        }
    }

    pub struct MoveSystem;

    impl<'a> System<'a> for MoveSystem {
        type SystemData = (
            WriteStorage<'a, Position>,
            WriteStorage<'a, Velocity>
        );

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
            }
        }
    }

    #[derive(Component, Debug)]
    #[storage(VecStorage)]
    pub struct Follower {
        pub lead: Entity
    }

    pub struct FollowingSystem;


    impl<'a> System<'a> for FollowingSystem {
        type SystemData = (
            ReadStorage<'a, Follower>,
            ReadStorage<'a, Position>,
            WriteStorage<'a, Velocity>
        );

        fn run(&mut self, (fol, pos, mut vel): Self::SystemData) {
            for (f, p, v) in (&fol, &pos, &mut vel).join() {
                let lp = pos.get(f.lead).unwrap();
                v.v = glm::normalize(&(glm::vec3(lp.x, lp.y, lp.z) - glm::vec3(p.x, p.y, p.z))) * 0.3;
            }
        }
    }
}

pub mod generic {
    use std::sync::mpsc::Sender;

    use rx::ecs::{ActiveCamera, CameraTarget, Position, Render, Rotation, TargetCamera, Transformation, ViewProjection};
    use rx::glm;
    use rx::render::{DrawCmd, RenderCommand};
    use rx::specs::{Join, Read, ReadStorage, System, Write, WriteStorage};

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
            ReadStorage<'a, TargetCamera>,
            ReadStorage<'a, Transformation>,
            WriteStorage<'a, Render>
        );


        fn run(&mut self, (active, camera, transformation, mut render): Self::SystemData) {
            let cam = camera.get(active.0.unwrap()).unwrap();
            self.send_render.send(RenderCommand::PushView(cam.view.clone()));

            for (transformation, render) in (&transformation, &mut render).join() {
                self.send_draw.send((render.mesh.clone(), transformation.mvp, transformation.model))
                    .expect("not able to submit");
            }
        }
    }

    pub struct TransformationSystem;

    impl<'a> System<'a> for TransformationSystem {
        type SystemData = (
            Read<'a, ActiveCamera>,
            Read<'a, CameraTarget>,
            WriteStorage<'a, TargetCamera>,
            ReadStorage<'a, Rotation>,
            ReadStorage<'a, Position>,
            WriteStorage<'a, Transformation>,
            Write<'a, ViewProjection>,
        );

        fn run(&mut self, data: Self::SystemData) {
            let (
                active_camera,
                camera_target,
                mut camera,
                rot,
                pos,
                mut tsm,
                mut vp_e
            ) = data;

            let target_pos = pos.get(camera_target.0.unwrap()).unwrap();
            let target_rot = rot.get(camera_target.0.unwrap()).unwrap();
            let cam = camera.get_mut(active_camera.0.unwrap()).unwrap();

            let vp = cam.target_at(
                &glm::vec3(
                    target_pos.x,
                    target_pos.y,
                    target_pos.z,
                ),
                &glm::vec3(
                    target_rot.x,
                    target_rot.y,
                    target_rot.z,
                ),
            );

            vp_e.view = cam.view.clone();
            vp_e.proj = cam.projection.clone();

            for (pos, rot, tsm) in (&pos, &rot, &mut tsm).join() {
                tsm.model = {
                    let mut mtx = glm::identity();
                    glm::rotate(&mut mtx, rot.x, &glm::vec3(1., 0., 0.)) *
                        glm::rotate(&mut mtx, rot.y, &glm::vec3(0., 1., 0.)) *
                        glm::rotate(&mut mtx, rot.z, &glm::vec3(0., 0., 1.)) *
                        glm::translate(&mut mtx, &glm::vec3(pos.x, pos.y, pos.z))
                };
                tsm.mvp = &vp * tsm.model
            }
        }
    }
}