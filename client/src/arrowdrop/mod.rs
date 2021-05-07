use rx::glm;
use rx::specs::{Component, prelude::*};

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

impl Grid {
    // pub fn get_hit(
    //     self,
    //     point_vec: glm::Vec3,
    //     plane_normal: glm::Vec3
    // ) -> (usize, usize) {
    //
    // }
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