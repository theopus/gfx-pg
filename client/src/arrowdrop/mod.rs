#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rx::{
    egui,
    glm,
    specs,
    specs::{Builder, Component, Join, VecStorage, WorldExt},
    winit,
    winit::event::{ElementState, MouseButton},
};
use rand_distr::num_traits::Pow;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct RectFromVec2 {
    first: glm::Vec3,
    second: glm::Vec3,
}

impl RectFromVec2 {
    pub fn new(first: glm::Vec3, second: glm::Vec3) -> Option<Self> {
        if first.dot(&second) == 0.0 {
            return Some(RectFromVec2 { first, second });
        }
        None
    }
    pub fn normal(&self) -> glm::Vec3 {
        self.first.cross(&self.second)
    }

    pub fn rotate(&self, rot: &rx::Rotation) -> RectFromVec2 {
        RectFromVec2 {
            first: rot.rotate_vec3(&self.first),
            second: rot.rotate_vec3(&self.second),
        }
    }
}

impl Default for RectFromVec2 {
    fn default() -> Self {
        Self { first: glm::vec3(1., 0., 0.), second: glm::vec3(0., 0., -1.) }
    }
}


#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Grid {
    cells: Vec<Vec<bool>>,
}

#[derive(Default)]
struct GridSystem {
    reader: rx::EventReader<()>,
}

#[derive(Default)]
struct GridUiSys {
    reader: rx::EventReader<()>,
}

impl<'a> specs::System<'a> for GridUiSys {
    type SystemData = (
        specs::Read<'a, rx::EventChannelReader<()>>,
        specs::Read<'a, rx::EguiCtx>,
        specs::WriteStorage<'a, Grid>,
    );

    fn run(&mut self, (events, gui, mut grid_st): Self::SystemData) {
        ;
        for g in (&mut grid_st).join() {
            let grid: &mut Grid = g;
            if let Some(gui_ctx) = gui.as_ref() {
                egui::Window::new("Grid_debug").show(gui_ctx, |ui| {
                    egui::Grid::new("grid_grid").striped(true).show(ui, |ui| {
                        for y in grid.cells.iter() {
                            for x in y.iter() {
                                ui.label(format!("{:?}", x));
                            }
                            ui.end_row();
                        }
                    });
                });
            }
        }
    }

    fn setup(&mut self, world: &mut specs::World) {
        use rx::{
            specs::SystemData,
            specs::shrev::EventChannel,
        };
        Self::SystemData::setup(world);
        self.reader = Some(world.fetch_mut::<EventChannel<rx::RxEvent<()>>>().register_reader());
    }
}

impl<'a> specs::System<'a> for GridSystem {
    type SystemData = (
        specs::Read<'a, rx::EventChannelReader<()>>,
        specs::WriteStorage<'a, Grid>,
        specs::ReadStorage<'a, RectFromVec2>,
        specs::ReadStorage<'a, rx::Position>,
        specs::ReadStorage<'a, rx::Rotation>
    );

    fn run(&mut self, (events, mut grid_st, rect_st, pos_st, rot_st): Self::SystemData) {
        if let Some(reader_id) = &mut self.reader {
            let clicks: Vec<(glm::Vec3, glm::Vec3)> = events.read(reader_id).map(|rx_event| {
                match rx_event {
                    rx::RxEvent::EcsEvent(
                        rx::EcsEvent::ScreenClick(
                            rx::ScreenClickEvent {
                                state: ElementState::Pressed,
                                mouse_button: MouseButton::Left,
                                world_vec,
                                cam_pos,
                                ..
                            })) => Some((world_vec.clone(), cam_pos.clone())),
                    _ => None
                }
            }).flatten().collect();

            const STEP_LEN: f32 = 3.0;
            const STEP_N: u32 = 4;

            for (mut grid, rect, pos, rot, ) in (&mut grid_st, &rect_st, &pos_st, &rot_st).join() {
                for (cam_vec, cam_pos) in clicks.iter() {


                    let new_rect = rect.rotate(rot);
                    let intrsect = crate::maths::intersection(&new_rect.normal(), &pos.as_vec3(), cam_vec, cam_pos);
                    if let Some(location) = &intrsect {
                        let p0 = pos.as_vec3();
                        // let p1 = pos.as_vec3() + (new_rect.first) * (STEP_LEN * STEP_N as f32);
                        // let p2 = pos.as_vec3() + (new_rect.second) * (STEP_LEN * STEP_N as f32);
                        // let p3 = (p1 - p0) + p2;
                        let relational = (&p0 - location) / (STEP_LEN as f32);
                        let truncated = glm::vec3(relational.x.trunc(),relational.y.trunc(),relational.z.trunc());
                        info!("trunc: {:?}", truncated);
                        info!("axis x: {},{},{}", truncated.x * new_rect.first.x, truncated.y* new_rect.first.y, truncated.z* new_rect.first.z);
                        info!("axis y: {},{},{}", truncated.x * new_rect.second.x, truncated.y* new_rect.second.y, truncated.z* new_rect.second.z);
                    }
                }
            }
        }
    }

    fn setup(&mut self, world: &mut specs::World) {
        use rx::{specs::SystemData, specs::shrev::EventChannel};
        Self::SystemData::setup(world);
        self.reader = Some(world.fetch_mut::<EventChannel<rx::RxEvent<()>>>().register_reader());
    }
}

pub fn create((mut world, rated, constant): rx::EcsInitTuple, mesh_ptr: rx::MeshPtr) {
    world.register::<RectFromVec2>();
    world.register::<Grid>();

    rated.add(GridSystem::default(), "grid_sys", &[]);
    constant.add_thread_local(GridUiSys::default());
    world.create_entity()
        .with(rx::Position {
            x: 0.0,
            y: 15.0,
            z: 0.0,
        })
        .with(rx::Rotation {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        })
        .with(RectFromVec2::default())
        .with(Grid {
            cells: vec![
                vec![false, false, false, false],
                vec![false, false, false, false],
                vec![false, false, false, false],
                vec![false, false, false, false],
            ]
        })
        .build();

    for v in 0..4 {
        for h in 0..4 {
            world.
                create_entity()
                .with(rx::Rotation {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                })
                .with(rx::Position {
                    x: 0.0,
                    y: 15.0 - (v as f32 * 3.0),
                    z: -h as f32 * 3.0,
                })
                .with(rx::Transformation::default())
                .with(rx::Render {
                    mesh: mesh_ptr.clone(),
                })
                .build();
        }
    }
}