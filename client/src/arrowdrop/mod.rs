use rx::{
    glm,
    specs,
    specs::{Component, prelude::*}
};
use crate::specs::shred::DynamicSystemData;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Normal {
    vec: glm::Vec3,
}

impl Default for Normal {
    fn default() -> Self {
        Self { vec: glm::vec3(0., 0., 0.) }
    }
}


#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Grid {
    cells: Vec<Vec<bool>>,
}

struct GridSystem;

impl<'a> specs::System<'a> for GridSystem {
    type SystemData = (
        WriteStorage<'a, Grid>,
        ReadStorage<'a, Normal>,
        ReadStorage<'a, rx::Rotation>
    );

    fn run(&mut self, (mut grid_st, normal_st, rot_st): Self::SystemData) {
        for i in (&mut grid_st, &normal_st, &rot_st).join() {

        }
    }
}

pub fn create((mut world, r, c): rx::EcsInitTuple, mesh_ptr: rx::MeshPtr){
    world.register::<Normal>();
    world.register::<Grid>();


    world.create_entity()
        .with(rx::Position::default())
        .with(rx::Rotation::default())
        .with(Normal::default())
        // .with(Grid::default())
        .build();

    for v in 0..7 {
        for h in 0..7 {
            world.
                create_entity()
                .with(rx::Rotation {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                })
                .with(rx::Position {
                    x: h as f32 * 3.0,
                    y: 5.0,
                    z: -50.0 - (v as f32 * 3.0),
                })
                .with(rx::Transformation::default())
                .with(rx::Render {
                    mesh: mesh_ptr.clone(),
                })
                .build();
        }
    }
}