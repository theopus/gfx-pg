
use crate::{ViewProjection, Position, maths, ActiveCamera, Camera};
use specs::{
    Component,
    VecStorage,
    System,
    Join
};
use std::time::Instant;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};


#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Culling {
    culled: bool,
    sphere_radius: f32
}

impl Default for Culling {
    fn default() -> Self {
        Self {
            culled: false,
            sphere_radius: 5.0
        }
    }
}

impl Culling {
    pub fn new(sphere_radius: f32) -> Self {
        Culling { culled: false, sphere_radius }
    }
    pub fn never() -> Self {
        Culling { culled: false, sphere_radius: f32::MAX }
    }

    pub fn is_culled(&self) -> bool {
        self.culled
    }
}

pub struct CullingSystem;

impl<'a> specs::System<'a> for CullingSystem {
    type SystemData = (
        specs::Read<'a, ActiveCamera>,
        specs::ReadStorage<'a, Camera>,
        specs::ReadStorage<'a, Position>,
        specs::WriteStorage<'a, Culling>
    );

    fn run(&mut self, (active_cam, cam, pos_st, mut culling_st): Self::SystemData) {
        let start = Instant::now();

        let mut cnt = 0;

        active_cam.camera(&cam).map(|cam|{
            let planes = maths::frustum_planes(&cam.vp());
            let should_cull = |pos: &glm::Vec3, radius: f32| {
                if radius == f32::MAX {
                    return false;
                }

                for p in planes.iter() {
                    let dist = glm::dot(&glm::vec4_to_vec3(p),&pos) + p.w + radius; //1.0 -radius
                    if dist < 0. {
                        return true;
                    }
                }
                return false;
            };

            for (pos, cull) in (&pos_st, &mut culling_st).join(){
                cull.culled = should_cull(&pos.as_vec3(), cull.sphere_radius);
                cnt += 1;
            }
        });

        debug!("frustum took {:?}, instances {:?}", Instant::now() - start, cnt);
    }
}


