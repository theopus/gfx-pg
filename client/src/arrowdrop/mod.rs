#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use rand_distr::num_traits::Pow;
use rand_distr::num_traits::real::Real;

use rx::{egui, glm, Render, specs, specs::{Builder, Component, Join, VecStorage, WorldExt}, winit, winit::event::{ElementState, MouseButton}};
use rx::ecs::base_systems::to_radians;
use crate::gui_sys::{EcsUiWidget, EcsUiSystem, EcsUiWidgetSystem};

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct RectFromVec2 {
    first: glm::Vec3,
    second: glm::Vec3,
}


#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Grid {
    cells: Vec<Vec<bool>>,
    cells_e: Vec<Vec<specs::Entity>>,
    step: f32,
    x_len: u32,
    y_len: u32,
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

impl Grid {
    fn reset_all(&mut self, value: bool) {
        for y in 0..self.y_len as usize {
            for x in 0..self.x_len as usize {
                self.cells[y][x] = value;
            }
        }
    }
    pub fn new(step: f32, x_len: u32, y_len: u32, entities: Vec<Vec<specs::Entity>>) -> Self {
        let mut cells = Vec::with_capacity(x_len as usize);
        for y in 0..y_len as usize {
            let mut row = Vec::with_capacity(y_len as usize);
            for x in 0..x_len as usize {
                row.push(false);
            }
            cells.push(row)
        }

        Grid { cells, cells_e: entities, step, x_len, y_len }
    }
}

#[derive(Default)]
struct GridSystem {
    reader: rx::EventReader<()>,
}

#[derive(Default)]
pub struct GridUiSys;

impl EcsUiWidgetSystem for GridUiSys {
    fn name() -> &'static str {
        "GridUiSys"
    }
}

impl<'a> specs::System<'a> for GridUiSys {
    type SystemData = (
        specs::ReadStorage<'a, EcsUiWidget>,
        specs::Read<'a, rx::EguiCtx>,
        specs::WriteStorage<'a, Grid>,
        specs::WriteStorage<'a, rx::Render>,
    );

    fn run(&mut self, (widgets, gui, mut grid_st, mut render_st): Self::SystemData) {
        if Self::should_draw(&widgets) {
            for g in (&mut grid_st).join() {
                let grid: &mut Grid = g;
                if let Some(gui_ctx) = gui.as_ref() {
                    egui::Window::new("Grid_debug").show(gui_ctx, |ui| {
                        egui::Grid::new("grid_grid").striped(true)
                            .spacing([1.,1.])
                            .show(ui, |ui| {
                            for y in grid.cells.iter() {
                                for x in y.iter() {
                                    ui.label(if *x {
                                        "o"
                                    } else {
                                        "x"
                                    });
                                }
                                ui.end_row();
                            }
                        });
                        if ui.button("reset").clicked() {
                            grid.reset_all(false);
                            grid.cells_e.iter().flatten().for_each(|e| {
                                if let Some(render) = render_st.get_mut(*e) {
                                    render.hidden = false
                                }
                            })
                        }
                    }).unwrap().id;
                }
            }
        }
    }
}

impl<'a> specs::System<'a> for GridSystem {
    type SystemData = (
        specs::Read<'a, rx::EventChannelReader<()>>,
        specs::WriteStorage<'a, Grid>,
        specs::WriteStorage<'a, rx::Render>,
        specs::ReadStorage<'a, RectFromVec2>,
        specs::ReadStorage<'a, rx::Position>,
        specs::ReadStorage<'a, rx::Rotation>,
    );

    fn run(&mut self, (events, mut grid_st, mut render_st, rect_st, pos_st, rot_st): Self::SystemData) {
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

            const STEP_LEN: f32 = 2.5;
            const STEP_N: u32 = 4;

            for (mut grid, rect, pos, rot, ) in (&mut grid_st, &rect_st, &pos_st, &rot_st).join() {
                for (cam_vec, cam_pos) in clicks.iter() {
                    let new_rect = rect.rotate(rot);
                    let intrsect = crate::maths::intersection(&new_rect.normal(), &pos.as_vec3(), cam_vec, cam_pos);
                    if let Some(location) = &intrsect {
                        let p0 = pos.as_vec3();
                        let relational = (&p0 - location) / (grid.step as f32);
                        let truncated = glm::trunc(&relational);
                        let x_axis: glm::IVec3 = -1 * glm::vec3(
                            (truncated.x * new_rect.first.x) as i32,
                            (truncated.y * new_rect.first.y) as i32,
                            (truncated.z * new_rect.first.z) as i32,
                        );
                        let y_axis: glm::IVec3 = -1 * glm::vec3(
                            (truncated.x * new_rect.second.x) as i32,
                            (truncated.y * new_rect.second.y) as i32,
                            (truncated.z * new_rect.second.z) as i32,
                        );

                        let x: i32 = if x_axis.x != 0 {
                            x_axis.x
                        } else if x_axis.y != 0 {
                            x_axis.y
                        } else {
                            x_axis.z
                        };

                        let y: i32 = if y_axis.x != 0 {
                            y_axis.x
                        } else if y_axis.y != 0 {
                            y_axis.y
                        } else {
                            y_axis.z
                        };

                        if (x >= 0 && x < grid.y_len as i32) && (y >= 0 && y < grid.x_len as i32) {
                            grid.cells[x as usize][y as usize] = true;
                            if let Some(rend) = render_st.get_mut(grid.cells_e[y as usize][x as usize]) {
                                info!("x,y {:?}",[y,x]);
                                rend.hidden = true;
                            }
                        }
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

pub fn new_grid(
    world: &mut specs::World,
    mesh: rx::MeshPtr,
    pos: &glm::Vec3,
    rot: &glm::Vec3,
    step: f32,
    dim: (u32, u32),
    rect: RectFromVec2
) {
    let first = rect.rotate(&rx::Rotation::from_vec3(rot)).first.clone();
    let second = rect.rotate(&rx::Rotation::from_vec3(rot)).second.clone();


    let mut entities: Vec<Vec<specs::Entity>> = Vec::with_capacity(dim.0 as usize);
    for y in 0..dim.0 {
        let y_pos = pos + second * step * y as f32 + (second); //B
        let mut row = Vec::with_capacity(dim.1 as usize);
        for x in 0..dim.1 {
            let x_pos = pos + first * step * x as f32 + (first); //C
            //  find 4th rect point
            let final_pos = (y_pos + x_pos - pos); //D=A+(B-A)-(C-A)=B+C-A
            let entity = world
                .create_entity()
                .with(rx::Rotation::default())
                .with(rx::Position::from_vec3(&final_pos))
                .with(rx::Transformation::default())
                .with(rx::Render::new(mesh.clone()))
                .with(rx::Culling::default())
                .build();
            row.push(entity);
        }
        entities.push(row);
    }
    let grid = Grid::new(step, dim.0, dim.1, entities);
    world.create_entity()
        .with(rx::Position::from_vec3(pos))
        .with(rx::Rotation::from_vec3(rot))
        .with(rect)
        .with(grid)
        .build();
}

pub fn create((mut world, rated, constant): rx::EcsInitTuple, mesh_ptr: rx::MeshPtr) {
    world.register::<RectFromVec2>();
    world.register::<Grid>();

    // rated.add(GridSystem::default(), "grid_sys", &[]);

    new_grid(
        world,
        mesh_ptr,
        &glm::vec3(-13.5, 26., 0.),
        &glm::vec3(0., 0., 0.),
        2.5,
        (10, 10),
        RectFromVec2::new(
            glm::vec3(0.,-1.,0.),
            glm::vec3(1.,0.,0.)
        ).unwrap()
    )


    // world.create_entity()
    //     .with(rx::Position {
    //         x: 0.0,
    //         y: 16.0,
    //         z: 0.0,
    //     })
    //     .with(rx::Rotation {
    //         x: 0.0,
    //         y: 0.0,
    //         z: -90.0,
    //     })
    //     .with(RectFromVec2::default())
    //     .with(Grid {
    //         cells: vec![
    //             vec![false, false, false, false],
    //             vec![false, false, false, false],
    //             vec![false, false, false, false],
    //             vec![false, false, false, false],
    //         ],
    //         step: 2.5,
    //         x_len: 4,
    //         y_len: 4
    //     })
    //     .build();
    //
    // for v in 0..4 {
    //     for h in 0..4 {
    //         world.
    //             create_entity()
    //             .with(rx::Rotation {
    //                 x: 0.0,
    //                 y: 0.0,
    //                 z: 0.0,
    //             })
    //             .with(rx::Position {
    //                 x: 0.0,
    //                 y: 15.0 - (v as f32 * 2.5),
    //                 z: -1.-(h as f32 * 2.5),
    //             })
    //             .with(rx::Transformation::default())
    //             .with(rx::Render {
    //                 mesh: mesh_ptr.clone(),
    //             })
    //             .build();
    //     }
    // }
}