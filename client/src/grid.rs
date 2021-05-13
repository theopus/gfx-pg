use std::borrow::Borrow;
use std::ops::{Deref, DerefMut};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rx::glm;

use crate::glm::Vec2;
use std::collections::HashMap;

struct Grid2D<V> {
    step: f32,
    scale: f32,
    dimensions: usize,
    depth: usize,
    data: HashMap<Grid2DCell, V>
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct Grid2DCell {
    level: usize,
    x: u32,
    y: u32,
}

impl<V> Grid2D<V> {
    pub fn get_cell_at<T: Borrow<glm::Vec2>>(&self, origin: T, point: T) -> Option<glm::UVec2> {
        self.get_cell_at_level(origin, point, 0)
    }

    pub fn get_cell_at_level<T: Borrow<glm::Vec2>>(&self, origin: T, point: T, level: u32) -> Option<glm::UVec2> {
        let relative_point: glm::Vec2 = point.borrow() - origin.borrow();
        let level_step = (self.step * (level as f32 + 1.));
        let x = (relative_point.x / level_step) as i32;
        let y = (relative_point.y / level_step) as i32;

        if x >= 0 && x < self.dimensions as i32
            && y >= 0 && y < self.dimensions as i32 {
            return Some(glm::vec2(x as u32, y as u32));
        }
        None
    }

    pub fn pos_for_cell<T: Borrow<glm::Vec2>>(&self, origin: T, (x, y): (u32, u32)) -> Option<glm::Vec2>{
        self.pos_for_cell_at_level(origin, (x, y), 0)
    }

    pub fn pos_for_cell_at_level<T: Borrow<glm::Vec2>>(&self, origin: T, (x, y): (u32, u32), level: u32) -> Option<glm::Vec2>{
        let level_step = self.step * (level + 1) as f32;
        let x = x as f32 * level_step;
        let y = y as f32 * level_step;
        let mut target: glm::Vec2 = (glm::vec2(x, y) + origin.borrow());
        target.x += level_step / 2.;
        target.y += level_step / 2.;
        Some(target)
    }
}


fn new_grid<V>(step: f32, scale: f32, dimensions: usize, depth: usize) -> Grid2D<V> {
    Grid2D {
        step,
        scale,
        dimensions,
        depth,
        data: HashMap::with_capacity(dimensions.pow(2))
    }
}

#[test]
fn test() {
    crate::init_log();
    let d: Grid2D<u32> = new_grid(2.5, 10., 10, 0);
    d.get_cell_at(glm::vec2(9., 0.), glm::vec2(9.5, 2.5))
        .map(|p| info!("{:?}", p));
    d.pos_for_cell(glm::vec2(0.,0.), (0, 0))
        .map(|p| info!("{:?}", p));
}
