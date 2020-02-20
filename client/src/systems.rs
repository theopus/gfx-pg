use std::sync::mpsc::Sender;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rx::ecs::{ActiveCamera, CameraTarget, Position, Render, Rotation, TargetCamera, Transformation, WinitEvents};
use rx::events::MyEvent;
use rx::glm;
use rx::render::DrawCmd;
use rx::specs::{Join, Read, ReadStorage, System, Write, WriteStorage};
use rx::winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode};

pub struct InputTestSystem {
    pub should_affect: bool
}


impl<'a> System<'a> for InputTestSystem {
    type SystemData = (
        Read<'a, WinitEvents>,
        Read<'a, ActiveCamera>,
        Read<'a, CameraTarget>,
        WriteStorage<'a, Position>,
//        Write<'a, CameraTarget>,
        WriteStorage<'a, TargetCamera>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            events,
            active,
            target,
            mut position,
            mut camera
        ) = data;

        let events = &events.0;
        let cam = camera.get_mut(active.0.unwrap()).unwrap();
        let pos = position.get_mut(target.0.unwrap()).unwrap();

        let mut accum_delta = (0.0, 0.0);
        for event in events {
            match event {
                MyEvent::MouseMotion {
                    delta
                } => {
                    if self.should_affect {
                        accum_delta.0 += delta.0;
                        accum_delta.1 += delta.1;
                    }
                }
                MyEvent::MouseInput {
                    state,
                    button: MouseButton::Left,
                    ..
                } => match state {
                    ElementState::Pressed => self.should_affect = true,
                    ElementState::Released => self.should_affect = false,
                },
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
                MyEvent::Resized(w, h) => cam.update_aspect(*w as f32 / *h as f32),
                _ => ()
            };
        }

        cam.yaw += 0.4 * accum_delta.0 as f32;
        cam.pitch -= 0.4 * accum_delta.1 as f32;
    }
}


pub struct RenderSubmitSystem {
    sender: Sender<DrawCmd>
}

impl RenderSubmitSystem {
    pub fn new(send: Sender<DrawCmd>) -> Self {
        Self {
            sender: send
        }
    }
}

impl<'a> System<'a> for RenderSubmitSystem {
    type SystemData = (
        ReadStorage<'a, Transformation>,
        WriteStorage<'a, Render>
    );

    fn run(&mut self, (transformation, mut render): Self::SystemData) {
        for (transformation, render) in (&transformation, &mut render).join() {
            self.sender.send((render.mesh.clone(), transformation.mvp))
                .expect("not able to submit");
        }
    }
}

pub struct TransformationSystem;

impl<'a> System<'a> for TransformationSystem {
    type SystemData = (
        Read<'a, ActiveCamera>,
        Read<'a, CameraTarget>,
        ReadStorage<'a, TargetCamera>,
        ReadStorage<'a, Rotation>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Transformation>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            active_camera,
            camera_target,
            camera,
            rot,
            pos,
            mut tsm
        ) = data;

        let target_pos = pos.get(camera_target.0.unwrap()).unwrap();
        let target_rot = rot.get(camera_target.0.unwrap()).unwrap();
        let cam = camera.get(active_camera.0.unwrap()).unwrap();

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