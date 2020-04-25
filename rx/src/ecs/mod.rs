use glm::{
    Mat4,
    radians,
    Vec3,
};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use specs::{Component, Entity, VecStorage};

use crate::assets::MeshPtr;
use crate::events::MyEvent;

pub mod layer;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Transformation {
    pub mvp: Mat4,
    pub model: Mat4,
}

impl Default for Transformation {
    fn default() -> Self {
        Self {
            mvp: glm::identity(),
            model: glm::identity(),
        }
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Render {
    pub mesh: MeshPtr
}

#[derive(Default, Debug)]
pub struct WinitEvents(pub Vec<MyEvent>);

#[derive(Default)]
pub struct CameraTarget(pub Option<Entity>);

#[derive(Default)]
pub struct ActiveCamera(pub Option<Entity>);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Rotation {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct TargetCamera {
    projection: Mat4,
    pub view: Mat4,
    fov: f32,
    //
    offset_y: f32,
    pub distance: f32,
    //angle around y
    pub yaw: f32,
    //angle around x
    pub pitch: f32,
}

impl Default for TargetCamera {
    fn default() -> Self {
        let aspect_ratio = 6. / 4.;

        Self {
            projection: glm::perspective(aspect_ratio, glm::radians(&glm::vec1(60.)).x, 0.1, 1000.),
            view: glm::identity(),
            fov: 60.,
            offset_y: 0.,
            distance: 100.,
            yaw: 180.,
            pitch: 115.,
        }
    }
}


impl TargetCamera {
    pub fn update_aspect(&mut self, aspect_ratio: f32) {
        self.projection = glm::perspective(aspect_ratio, glm::radians(&glm::vec1(self.fov)).x, 0.1, 1000.);
    }

    pub fn target_at(&mut self, position: &Vec3, _rotation: &Vec3) -> Mat4 {
        let (x, y, z) = (position.x, position.y, position.z);
        let theta = radians(&glm::vec1(self.yaw));
        let pitch_rad = radians(&glm::vec1(self.pitch));

        let (horiz, vert) = (
            self.distance * glm::cos(&pitch_rad),
            self.distance * glm::sin(&pitch_rad)
        );

        let (offset_x, offset_z) = (
            (horiz * glm::sin(&theta)).x,
            (horiz * glm::cos(&theta)).x,
        );

        let cam_pos: Vec3 = glm::vec3(
            -(x - offset_x),
            -(y + vert.x + self.offset_y),
            -(z - offset_z),
        );
        let cam_rot: Vec3 = glm::vec3(
            self.pitch,
            180_f32 - self.yaw,
            0_f32,
        );
        self.view = Self::get_view(&cam_pos, &cam_rot);
        &self.projection  * self.view.clone()
    }

    pub fn get_view(pos: &Vec3, rot: &Vec3) -> Mat4 {
        let mut mtx: Mat4 = glm::identity();
        //camera rot
        mtx = glm::rotate(
            &mtx,
            glm::radians(&glm::vec1(rot.x)).x,
            &glm::vec3(1., 0., 0.),
        );
        mtx = glm::rotate(
            &mtx,
            glm::radians(&glm::vec1(rot.y)).x,
            &glm::vec3(0., 1., 0.),
        );
        mtx = glm::rotate(
            &mtx,
            glm::radians(&glm::vec1(rot.z)).x,
            &glm::vec3(0., 0., 1.),
        );
        // camera translate
        mtx = glm::translate(&mtx, &glm::vec3(pos.x, pos.y, pos.z));
        mtx
    }
}