use rx::assets::MeshPtr;
use rx::ecs::{Position, Render, Rotation, Transformation, Velocity};
use rx::specs::Builder;
use rx::specs::World;
use rx::specs::WorldExt;

pub fn create(world: &mut World, mesh_ptr: MeshPtr) {
    for v in 0..1000 {
        for h in 0..10 {
            world.
                create_entity()
                .with(Rotation {
                    x: 45.0,
                    y: 0.0,
                    z: 45.0
                })
                .with(Position {
                    x: h as f32 * 3.0,
                    y: 50.0,
                    z: -50.0 - (v as f32 * 3.0),
                })
                .with(Transformation::default())
                .with(Render {
                    mesh: mesh_ptr.clone(),
                })
                .build();
        }
    }
}