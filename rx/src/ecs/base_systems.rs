pub mod world3d {
    use std::ops::Deref;
    use std::sync::mpsc::Sender;
    use std::time::Instant;

    use glm;
    #[allow(unused_imports)]
    use log::{debug, error, info, trace, warn};
    use specs::{
        Builder, Component, Entity, Join, Read, ReadStorage, System,
        VecStorage, World, WorldExt, Write, WriteStorage,
    };

    use crate::assets::MeshPtr;
    use crate::ecs::base_systems::camera3d::{
        ActiveCamera, CameraTarget, init as init_cam, TargetedCamera, ViewProjection,
    };
    use crate::ecs::base_systems::to_radians;
    use crate::glm::Vec3;
    use crate::graphics_api::{DrawCmd, RenderCommand};

    #[derive(Component, Debug)]
    #[storage(VecStorage)]
    pub struct Render {
        pub mesh: MeshPtr,
    }

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
                .send(RenderCommand::PushView(cam.view.clone() as glm::Mat4)).unwrap();
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


    ///
    ///                  camera  system
    pub type WorldInit = (Entity, TransformationSystem);

    pub fn init(world: &mut World, camera_at: &glm::Vec3) -> WorldInit {
        info!("Init world3d_system");
        world.register::<Render>();
        world.register::<Rotation>();
        world.register::<Position>();
        world.register::<Transformation>();

        let target = world
            .create_entity()
            .with(Rotation::default())
            .with(Position {
                x: camera_at.x,
                y: camera_at.y,
                z: camera_at.z,
            })
            .build();

        (init_cam(world, target), TransformationSystem)
    }

    pub struct TransformationSystem;

    impl<'a> System<'a> for TransformationSystem {
        type SystemData = (
            Read<'a, ActiveCamera>,
            Read<'a, CameraTarget>,
            WriteStorage<'a, TargetedCamera>,
            ReadStorage<'a, Rotation>,
            ReadStorage<'a, Position>,
            WriteStorage<'a, Transformation>,
            Write<'a, ViewProjection>,
        );

        fn run(&mut self, data: Self::SystemData) {
            let (active_camera, camera_target, mut camera, rot, pos, mut tsm, mut vp_e) = data;

            let target_pos = pos.get(camera_target.0.unwrap()).unwrap();
            let target_rot = rot.get(camera_target.0.unwrap()).unwrap();
            let cam = camera.get_mut(active_camera.0.unwrap()).unwrap();

            let vp = cam.target_at(
                &glm::vec3(target_pos.x, target_pos.y, target_pos.z),
                &glm::vec3(target_rot.x, target_rot.y, target_rot.z),
            );

            //set current V+P
            vp_e.view = cam.view.clone() as glm::Mat4;
            vp_e.proj = cam.projection.clone() as glm::Mat4;

            //bottleneck
            {
                let start = Instant::now();
                for (pos, rot, tsm) in (&pos, &rot, &mut tsm).join() {
                    tsm.model = {
                        let mut mtx: glm::Mat4 = glm::identity();
                        mtx = glm::translate(&mut mtx, &glm::vec3(pos.x, pos.y, pos.z));
                        // if rot.x != 0.0 || rot.x != 0.0 || rot.x != 0.0 {
                        //     mtx = mtx * Rotation3::from_euler_angles(rot.x, rot.y, rot.z);
                        // }
                        // if rot.x != 0.0 || rot.y != 0.0 || rot.z != 0.0 {
                        //     mtx = mtx * glm::rotate_vec3();
                        // }
                        if rot.x != 0.0 {
                            mtx = glm::rotate(&mut mtx, glm::radians(&glm::vec1(rot.x)).x, &glm::vec3(1., 0., 0.));
                        }
                        if rot.y != 0.0 {
                            mtx = glm::rotate(&mut mtx, glm::radians(&glm::vec1(rot.y)).x, &glm::vec3(0., 1., 0.));
                        }
                        if rot.z != 0.0 {
                            mtx = glm::rotate(&mut mtx, glm::radians(&glm::vec1(rot.z)).x, &glm::vec3(0., 0., 1.));
                        }
                        mtx
                    };
                    tsm.mvp = &vp * &tsm.model
                }
                debug!("matrix took {:?}", Instant::now() - start);
            }
        }
    }

    #[derive(Component, Debug)]
    #[storage(VecStorage)]
    pub struct Transformation {
        pub mvp: glm::Mat4,
        pub model: glm::Mat4,
    }

    #[derive(Component, Debug)]
    #[storage(VecStorage)]
    pub struct Position {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    #[derive(Component, Debug)]
    #[storage(VecStorage)]
    pub struct Rotation {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    impl Default for Transformation {
        fn default() -> Self {
            Self {
                mvp: glm::identity() as glm::Mat4,
                model: glm::identity() as glm::Mat4,
            }
        }
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

    impl Default for Rotation {
        fn default() -> Self {
            Self {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }
        }
    }

    impl Position {
        pub fn as_vec3(&self) -> glm::Vec3 {
            glm::vec3(self.x, self.y, self.z)
        }
    }

    impl Rotation {
        pub fn as_vec3(&self) -> glm::Vec3 {
            glm::vec3(self.x, self.y, self.z)
        }

        pub fn rotate_vec3(&self, vec: &glm::Vec3) -> glm::Vec3 {
            let mut result = glm::rotate_vec3(&vec, to_radians(self.x), &glm::vec3(1.0, 0., 0.0));
            result = glm::rotate_vec3(&result, to_radians(self.y), &glm::vec3(0.0, 1.0, 0.0));
            result = glm::rotate_vec3(&result, to_radians(self.z), &glm::vec3(0.0, 0.0, 1.0));
            result
        }
    }
}

pub fn to_radians(degree: f32) -> f32 {
    glm::radians(&glm::vec1(degree)).x
}

pub mod camera3d {
    use glm;
    #[allow(unused_imports)]
    use log::{debug, error, info, trace, warn};
    use specs::{Builder, Component, Entity, VecStorage, World, WorldExt};

    ///
            /// creates targeted camera, places camera to active
            /// @return Camera Entity
            ///
    pub fn init(world: &mut World, cam_target: Entity) -> Entity {
        info!("Init camera3d_system");
        world.register::<TargetedCamera>();
        let cam_entity = world
            .create_entity()
            .with(TargetedCamera::default())
            .build();
        world.insert(ActiveCamera(Some(cam_entity)));
        world.insert(CameraTarget(Some(cam_target)));
        world.insert(ViewProjection::default());
        cam_entity
    }

    pub struct ViewProjection {
        pub view: glm::Mat4,
        pub proj: glm::Mat4,
    }

    //current camera Id
    #[derive(Default)]
    pub struct ActiveCamera(pub Option<Entity>);

    //camera target for ```TargetedCamera```
    #[derive(Default)]
    pub struct CameraTarget(pub Option<Entity>);

    #[derive(Component, Debug)]
    #[storage(VecStorage)]
    pub struct TargetedCamera {
        pub projection: glm::Mat4,
        pub view: glm::Mat4,
        fov: f32,
        //
        offset_y: f32,
        pub distance: f32,
        //angle around y
        pub yaw: f32,
        //angle around x
        pub pitch: f32,

        pub cam_pos: glm::Vec3,
    }

    impl Default for ViewProjection {
        fn default() -> Self {
            Self {
                view: glm::identity() as glm::Mat4,
                proj: glm::identity() as glm::Mat4,
            }
        }
    }

    impl Default for TargetedCamera {
        fn default() -> Self {
            let aspect_ratio = 6. / 4.;

            Self {
                projection: glm::perspective(
                    aspect_ratio,
                    glm::radians(&glm::vec1(60.)).x,
                    0.1,
                    1000.,
                ),
                view: glm::identity() as glm::Mat4,
                fov: 60.,
                offset_y: 0.,
                distance: 100.,
                yaw: 180.,
                pitch: 90.,
                cam_pos: glm::vec3(0., 0., 0.),
            }
        }
    }

    impl TargetedCamera {
        pub fn update_aspect(&mut self, aspect_ratio: f32) {
            self.projection = glm::perspective(
                aspect_ratio,
                glm::radians(&glm::vec1(self.fov)).x,
                0.1,
                1000.,
            );
        }

        pub fn target_at(&mut self, position: &glm::Vec3, _rotation: &glm::Vec3) -> glm::Mat4 {
            let (x, y, z) = (position.x, position.y, position.z);
            let theta = glm::radians(&glm::vec1(self.yaw));
            let pitch_rad = glm::radians(&glm::vec1(self.pitch));

            let (horiz, vert) = (
                self.distance * glm::cos(&pitch_rad),
                self.distance * glm::sin(&pitch_rad),
            );

            let (offset_x, offset_z) = ((horiz * glm::sin(&theta)).x, (horiz * glm::cos(&theta)).x);

            let cam_pos: glm::Vec3 = glm::vec3(
                -(x - offset_x),
                -(y + vert.x + self.offset_y),
                -(z - offset_z),
            );
            let cam_rot: glm::Vec3 = glm::vec3(self.pitch, 180_f32 - self.yaw, 0_f32);
            self.cam_pos = -cam_pos;
            self.view = Self::get_view(&cam_pos, &cam_rot);
            &self.projection * self.view.clone()
        }

        pub fn get_view(pos: &glm::Vec3, rot: &glm::Vec3) -> glm::Mat4 {
            let mut mtx = glm::identity() as glm::Mat4;
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
}
