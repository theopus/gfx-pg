use std::sync::mpsc::Sender;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rx::ecs::{ActiveCamera, CameraTarget, Position, Render, Rotation, TargetCamera, Transformation, WinitEvents};
use rx::events::MyEvent;
use rx::glm;
use rx::render::{DrawCmd, RenderCommand};
use rx::specs::{Join, Read, ReadStorage, System, Write, WriteStorage};
use rx::winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode};

#[derive(Default)]
pub struct InputTestSystem {
    pub should_affect_angle: bool,
    pub should_affect_distance: bool,
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


        cam.distance += 0.4 * accum_dist;
        cam.yaw += 0.4 * accum_delta.0 as f32;
        cam.pitch -= 0.4 * accum_delta.1 as f32;
    }
}


pub struct RenderSubmitSystem {
    send_draw: Sender<DrawCmd>,
    send_render: Sender<RenderCommand>
}

impl RenderSubmitSystem {
    pub fn new(send_draw: Sender<DrawCmd>, send_render: Sender<RenderCommand>) -> Self {
        Self {
            send_draw,
            send_render
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
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            active_camera,
            camera_target,
            mut camera,
            rot,
            pos,
            mut tsm
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