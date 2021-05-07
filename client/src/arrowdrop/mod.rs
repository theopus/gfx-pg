use rx::{
    crossbeam_channel,
    glm,
    specs,
    specs::{Accessor, Builder, Component, EntityBuilder, Join, VecStorage, WorldExt},
};
use rx::events::RxEvent;

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

struct GridSystem {
    sender: Option<crossbeam_channel::Sender<RxEvent<()>>>,
    receiver: Option<crossbeam_channel::Receiver<RxEvent<()>>>,
}


impl<'a> specs::System<'a> for GridSystem {
    type SystemData = (
        specs::WriteStorage<'a, Grid>,
        specs::ReadStorage<'a, Normal>,
        specs::ReadStorage<'a, rx::Rotation>
    );

    fn run(&mut self, (mut grid_st, normal_st, rot_st): Self::SystemData) {
        for i in (&mut grid_st, &normal_st, &rot_st).join() {}
    }

    fn setup(&mut self, world: &mut specs::World) {
        use rx::specs::SystemData;
        Self::SystemData::setup(world);
        let rs = rx::ecs::fetch_events_channel::<()>(world);
    }
}

pub fn create((mut world, r, c): rx::EcsInitTuple) {
    world.register::<Normal>();
    world.register::<Grid>();

    world.create_entity()
        .with(rx::Position::default())
        .with(rx::Rotation::default())
        .with(Normal::default())
        .build();
}