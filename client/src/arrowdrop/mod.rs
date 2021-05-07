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

pub fn create((mut world, r, c): rx::EcsInitTuple){
    world.register::<Normal>();
    world.register::<Grid>();

    world.create_entity()
        .with(rx::Position::default())
        .with(rx::Rotation::default())
        .with(Normal::default())
        .build();
}