pub mod world3d {
    use std::ops::{Deref, DerefMut};
    use std::sync::mpsc::Sender;
    use std::time::Instant;

    use glm;
    #[allow(unused_imports)]
    use log::{debug, error, info, trace, warn};
    use specs::{
        Builder, Component, Entity, Join, Read, ReadStorage, System,
        VecStorage, World, WorldExt, Write, WriteStorage,
    };

    use crate::{EventChannelReader, EventReader, RxEvent};
    use crate::assets::MeshPtr;
    use crate::ecs::base_systems::camera3d::{ActiveCamera, Camera, CameraTarget, init as init_cam, TargetedCamera, ViewProjection};
    use crate::ecs::base_systems::to_radians;
    use crate::glm::{e, Vec3};
    use crate::graphics_api::{DrawCmd, RenderCommand};
    use crate::specs::RunningTime;
    use crate::ecs::systems::frustum::Culling;

    #[derive(Component, Debug)]
    #[storage(VecStorage)]
    pub struct Render {
        pub mesh: MeshPtr,
        pub hidden: bool,
    }

    impl Render {
        pub fn new(mesh: MeshPtr) -> Self {
            Render { mesh, hidden: false }
        }
        pub fn new_hidden(mesh: MeshPtr) -> Self {
            Render { mesh, hidden: true }
        }
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
            Read<'a, ViewProjection>,
            ReadStorage<'a, Transformation>,
            ReadStorage<'a, Render>,
            ReadStorage<'a, Culling>,
        );

        fn run(&mut self, (vp, transformation, render, culling_st): Self::SystemData) {
            self.send_render
                .send(RenderCommand::PushView(vp.view.clone() as glm::Mat4)).unwrap();
            let start = Instant::now();
            for (transformation, render, cull) in (&transformation, &render, &culling_st).join() {
                if !render.hidden && !cull.is_culled() {
                    self.send_draw
                        .send((
                            render.mesh.clone(),
                            transformation.mvp.clone() as glm::Mat4,
                            transformation.model.clone() as glm::Mat4,
                        ))
                        .expect("not able to submit");
                }
            }
            debug!("render_submin took {:?}", Instant::now() - start);
        }
    }


    ///
    ///                  camera  system
    pub type WorldInit<T> = (Entity, CameraSystem<T>, TransformationSystem);

    pub fn init<T: 'static + Send + Clone>(world: &mut World, camera_at: &glm::Vec3) -> WorldInit<T> {
        info!("Init world3d_system");
        world.register::<Render>();
        world.register::<Rotation>();
        world.register::<Position>();
        world.register::<Transformation>();

        let target = world
            .create_entity()
            .with(Rotation::default())
            .with(Position::new(camera_at.x,camera_at.y,camera_at.z))
            .build();

        let (cam, cam_sys) = init_cam(world, target);
        (cam, cam_sys, TransformationSystem)
    }

    pub struct TransformationSystem;

    pub struct CameraSystem<T: 'static + Send + Clone> {
        reader: EventReader<T>,
    }

    impl<T: 'static + Send + Clone> Default for CameraSystem<T> {
        fn default() -> Self {
            Self {
                reader: None
            }
        }
    }

    impl<'a, T: 'static + Send + Clone + Sync> System<'a> for CameraSystem<T> {
        type SystemData = (
            WriteStorage<'a, Position>,
            WriteStorage<'a, Camera>,
            Read<'a, CameraTarget>,
            Read<'a, EventChannelReader<T>>,
        );

        fn run(&mut self, (mut pos_st, mut cam_st, cam_tg, events): Self::SystemData) {
            let start = Instant::now();
            // if let Some(reader) = ) {
            let size = self.reader.as_mut().map(|mut reader| {
                events
                    .read(reader)
                    .filter_map(|rx_event| {
                        match rx_event {
                            RxEvent::WinitEvent(winit::event::Event::WindowEvent { event: winit::event::WindowEvent::Resized(size), .. }) => {
                                Some(size)
                            }
                            _ => None
                        }
                    }).last()
            }).flatten();
            // }


            let cam_target_pos = cam_tg
                .target_pos_mut(&mut pos_st)
                .map(|e| { e.as_vec3() });
            for (pos, cam) in (&mut pos_st, &mut cam_st).join() {
                match cam {
                    Camera::Targeted(cam) => {
                        if let Some(t_pos) = cam_target_pos.as_ref() {
                            cam.target_at(t_pos, &glm::vec3(0., 0., 0.));
                            pos.upd_from_vec3(&cam.cam_pos)
                        }
                    }
                    Camera::Free => {}
                }
                if let Some(size) = size{
                    cam.update_aspect(size.width as f32 / size.height as f32)
                }
            }
            debug!("cam_sys took {:?}", Instant::now() - start);
        }

        fn setup(&mut self, world: &mut World) {
            use specs::SystemData;
            use specs::shrev::EventChannel;
            Self::SystemData::setup(world);
            self.reader = Some(world.fetch_mut::<EventChannelReader<T>>().register_reader());
        }
    }

    impl<'a> System<'a> for TransformationSystem {
        type SystemData = (
            Read<'a, ActiveCamera>,
            ReadStorage<'a, Camera>,
            WriteStorage<'a, Rotation>,
            WriteStorage<'a, Position>,
            ReadStorage<'a, Culling>,
            WriteStorage<'a, Transformation>,
            Write<'a, ViewProjection>,
        );



        fn run(&mut self, data: Self::SystemData) {
            let (
                active_camera,
                cam_st,
                mut rot,
                mut pos,
                culling_st,
                mut tsm,
                mut vp_e
            ) = data;

            let start = Instant::now();
            let cam = match active_camera.camera(&cam_st) {
                None => return,
                Some(e) => e
            };

            //set current V+P
            let vp = cam.vp();
            vp_e.view = cam.view();
            vp_e.proj = cam.projection();

            //bottleneck
            {

                for (cull, pos, rot, tsm) in (&culling_st, &mut pos, &mut rot, &mut tsm).join() {
                    if cull.is_culled() {
                        continue;
                    }
                    if pos.did_change || rot.did_change {
                        tsm.model = {
                            let mut mtx = glm::identity() as glm::Mat4;
                            mtx = glm::translate(&mut mtx, &pos);
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
                        pos.did_change = false;
                        rot.did_change = false;
                    }
                    tsm.mvp = &vp * &tsm.model
                }
            }
            info!("transform_sys took {:?}", Instant::now() - start);
        }

        fn running_time(&self) ->  specs::RunningTime {
            specs::RunningTime::VeryLong
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
        vec: glm::Vec3,
        did_change: bool
    }

    #[derive(Component, Debug)]
    #[storage(VecStorage)]
    pub struct Rotation {
        vec: glm::Vec3,
        did_change: bool
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
                vec: glm::zero::<glm::Vec3>() as glm::Vec3,
                did_change: true
            }
        }
    }

    impl Default for Rotation {
        fn default() -> Self {
            Self {
                vec: glm::zero::<glm::Vec3>(),
                did_change: true
            }
        }
    }

    impl Deref for Position {
        type Target = glm::Vec3;

        fn deref(&self) -> &Self::Target {
            &self.vec
        }
    }

    impl DerefMut for Position {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.did_change = true;
            &mut self.vec
        }
    }

    impl Into<Position> for glm::Vec3 {
        fn into(self) -> Position {
            Position {
                vec: self,
                did_change: true
            }
        }
    }
    impl Position {
        pub fn x(&self) -> f32 {
            self.vec.x
        }
        pub fn y(&self) -> f32 {
            self.vec.y
        }
        pub fn z(&self) -> f32 {
            self.vec.z
        }
        pub fn as_vec3(&self) -> glm::Vec3 {
            glm::vec3(self.x, self.y, self.z)
        }

        pub fn from_vec3(from: &glm::Vec3) -> Self {
            Self {
                vec: from.clone() as glm::Vec3,
                did_change: true
            }
        }
        pub fn upd_from_vec3(&mut self, from: &glm::Vec3) {
            self.did_change = true;
            self.vec = from.clone() as glm::Vec3
        }
        pub fn new(x: f32,y: f32,z: f32) -> Self {
            Position { vec: glm::vec3(x,y,z), did_change: true }
        }
    }
    impl Rotation {
        pub fn x(&self) -> f32 {
            self.vec.x
        }
        pub fn y(&self) -> f32 {
            self.vec.y
        }
        pub fn z(&self) -> f32 {
            self.vec.z
        }

        pub fn upd_from_vec3(&mut self, from: &glm::Vec3) {
            self.did_change = true;
            self.vec = from.clone() as glm::Vec3
        }
        pub fn new(x: f32,y: f32,z: f32) -> Self {
            Self { vec: glm::vec3(x,y,z), did_change: true }
        }
    }

    impl Deref for Rotation {
        type Target = glm::Vec3;

        fn deref(&self) -> &Self::Target {
            &self.vec
        }
    }

    impl DerefMut for Rotation {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.did_change = true;
            &mut self.vec
        }
    }

    impl Into<Rotation> for glm::Vec3 {
        fn into(self) -> Rotation {
            Rotation {
                vec: self,
                did_change: true
            }
        }
    }

    impl Rotation {

        pub fn as_vec3(&self) -> glm::Vec3 {
            glm::vec3(self.x, self.y, self.z)
        }

        pub fn from_vec3(from: &glm::Vec3) -> Self {
            Self {
                vec: from.clone() as glm::Vec3,
                did_change: true
            }
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
    use specs::{Builder, Component, Entity, ReadStorage, VecStorage, World, WorldExt, WriteStorage};

    use crate::{Position, Velocity};
    use crate::ecs::base_systems::world3d::CameraSystem;

    ///
                                            /// creates targeted camera, places camera to active
                                            /// @return Camera Entity
                                            ///
    pub fn init<T: 'static + Send + Clone>(world: &mut World, cam_target: Entity) -> (Entity, CameraSystem<T>) {
        info!("Init camera3d_system");
        world.register::<Camera>();
        let cam_entity = world
            .create_entity()
            .with(Position::default())
            .with(Camera::Targeted(TargetedCamera::default()))
            .build();
        world.insert(ActiveCamera(Some(cam_entity)));
        world.insert(CameraTarget(Some(cam_target)));
        world.insert(ViewProjection::default());
        (cam_entity, CameraSystem::default())
    }

    pub struct ViewProjection {
        pub view: glm::Mat4,
        pub proj: glm::Mat4,
    }

    //current camera Id
    #[derive(Default)]
    pub struct ActiveCamera(Option<Entity>);

    impl ActiveCamera {
        pub fn camera<'a>(&self, cam_st: &'a ReadStorage<Camera>) -> Option<&'a Camera> {
            self.0.map(|e| { cam_st.get(e) }).flatten()
        }
        pub fn camera_pos<'a>(&self, pos_st: &'a ReadStorage<Position>) -> Option<&'a Position> {
            self.0.map(|e| { pos_st.get(e) }).flatten()
        }
        pub fn camera_pos_mut<'a>(&self, pos_st: &'a mut WriteStorage<Position>) -> Option<&'a mut Position> {
            self.0.map(move |e| { pos_st.get_mut(e) }).flatten()
        }
        pub fn camera_mut<'a>(&self, cam_st: &'a mut WriteStorage<Camera>) -> Option<&'a mut Camera> {
            self.0.map(move |e| { cam_st.get_mut(e) }).flatten()
        }
    }

    //camera target for ```TargetedCamera```
    #[derive(Default)]
    pub struct CameraTarget(Option<Entity>);

    impl CameraTarget {
        pub fn target_pos<'a>(&self, pos_st: &'a ReadStorage<Position>) -> Option<&'a Position> {
            self.0.map(|e| { pos_st.get(e) }).flatten()
        }
        pub fn target_pos_mut<'a>(&self, pos_st: &'a mut WriteStorage<Position>) -> Option<&'a mut Position> {
            self.0.map(move |e| { pos_st.get_mut(e) }).flatten()
        }
        pub fn target_vel_mut<'a>(&self, pos_st: &'a mut WriteStorage<Velocity>) -> Option<&'a mut Velocity> {
            self.0.map(move |e| { pos_st.get_mut(e) }).flatten()
        }
        pub fn new(target: Entity) -> Self {
            CameraTarget(Some(target))
        }
    }

    #[derive(Component, Debug)]
    #[storage(VecStorage)]
    pub enum Camera {
        Targeted(TargetedCamera),
        Free,
    }

    impl Camera {
        pub fn update_aspect(&mut self, aspect: f32) {
            match self {
                Camera::Targeted(t) => t.update_aspect(aspect),
                Camera::Free => {}
            }
        }
        pub fn view(&self) -> glm::Mat4 {
            (match self {
                Camera::Targeted(t) => t.view.clone(),
                Camera::Free => glm::identity()
            }) as glm::Mat4
        }
        pub fn projection(&self) -> glm::Mat4 {
            (match self {
                Camera::Targeted(t) => t.projection.clone(),
                Camera::Free => glm::identity()
            }) as glm::Mat4
        }
        pub fn vp(&self) -> glm::Mat4 {
            (match self {
                Camera::Targeted(t) => &t.projection * &t.view,
                Camera::Free => glm::identity()
            }) as glm::Mat4
        }
    }

    #[derive(Debug)]
    pub struct TargetedCamera {
        pub projection: glm::Mat4,
        pub view: glm::Mat4,
        pub fov: f32,
        offset_y: f32,
        pub distance: f32,
        pub yaw: f32,
        pub pitch: f32,
        pub aspect_ratio: f32,

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
                aspect_ratio: 6. / 4.,
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
            self.aspect_ratio = aspect_ratio;
            self.projection = glm::perspective(
                self.aspect_ratio,
                glm::radians(&glm::vec1(self.fov)).x,
                0.1,
                1000.,
            );
        }
        pub fn update_fov(&mut self, fov: f32) {
            self.fov = fov;
            self.projection = glm::perspective(
                self.aspect_ratio,
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
