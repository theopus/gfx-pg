use std::sync::mpsc::Sender;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rx::ecs::{ActiveCamera, CameraTarget, Position, Render, Rotation, TargetCamera, Transformation, WinitEvents};
use rx::events::MyEvent;
use rx::glm;
use rx::render::DrawCmd;
use rx::specs::{Join, Read, ReadStorage, System, WriteStorage};

pub struct InputTestSystem;

impl<'a> System<'a> for InputTestSystem {
    type SystemData = (
        Read<'a, WinitEvents>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let mut events = data.0;
        let events = &events.0;
        
        for event in events {

            match event {
                MyEvent::MouseMotion {
                    delta
                } => info!("{:?}", delta),
                _ => ()
            };
        }
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
            self.sender.send((render.mesh.clone(), transformation.mvp));
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
            pos,
            rot,
            mut tsm
        ) = data;

        let target_pos = pos.get(camera_target.0.unwrap()).unwrap();
        let target_rot = pos.get(camera_target.0.unwrap()).unwrap();
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