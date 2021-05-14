use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rx::glm;

use crate::glm::Vec2;

struct Grid2D<V> {
    step: f32,
    dimensions: u32,
    depth: u32,
    data: HashMap<Grid2DCell, V>,
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct Grid2DCell {
    x: u32,
    y: u32,
}

impl<V> Grid2D<V> {
    fn get_level_step(&self, level: u32) -> f32 {
        return if level == 0 {
            self.step
        } else {
            self.step * self.dimensions as f32 * level as f32
        };
    }

    pub fn put_data(&mut self, level0: (u32, u32), offset: (u32,u32,u32), data: V) {
        let cell_per_level = self.cells_per(offset.2);
        self.put_data_0(
            (level0.0 + offset.0  * cell_per_level,
            level0.1 + offset.1  * cell_per_level),
            data
        )
    }

    fn put_data_0(&mut self, (x, y): (u32, u32), data: V) {
        info!("put to: {:?}", (x,y));
        self.data.insert(Grid2DCell {x, y}, data);
    }

    pub fn cells_per(&self, level: u32) -> u32 {
        self.dimensions.pow(level)
    }

    pub fn max_cell_at(&self, level: u32) -> u32 {
        self.dimensions.pow(self.depth + 1 - level)
    }

    fn is_in_bounds(&self, (x, y): (i32, i32), level: u32) -> bool {
        let max = self.max_cell_at(level) as i32;
        x >= 0 && x < max
            && y >= 0 && y < max
    }

    pub fn get_cell_at<T: Borrow<glm::Vec2>>(&self, origin: T, point: T, level: u32) -> Option<glm::UVec2> {
        let relative_point: glm::Vec2 = point.borrow() - origin.borrow();
        let level_step = self.get_level_step(level);
        let x = (relative_point.x / level_step) as i32;
        let y = (relative_point.y / level_step) as i32;

        if self.is_in_bounds((x, y), level) {
            return Some(glm::vec2(x as u32, y as u32));
        }
        None
    }

    ///returns center of cell
    pub fn pos_for_cell_at<T: Borrow<glm::Vec2>>(&self, origin: T, (x, y): (u32, u32), level: u32) -> Option<glm::Vec2> {
        let level_step = self.get_level_step(level);
        let x = x as f32 * level_step;
        let y = y as f32 * level_step;
        let mut target: glm::Vec2 = (glm::vec2(x, y) + origin.borrow());
        target.x += level_step / 2.;
        target.y += level_step / 2.;
        Some(target)
    }
}

fn new_grid<V>(step: f32, dimensions: u32, depth: u32) -> Grid2D<V> {
    Grid2D {
        step,
        dimensions,
        depth,
        data: HashMap::with_capacity(dimensions.pow(2) as usize),
    }
}

#[test]
fn test() {
    crate::init_log();
    let mut d: Grid2D<u32> = new_grid(2., 5, 2);

    d.get_cell_at(glm::vec2(0., 0.), glm::vec2(49.9, 2.5), 0)
        .map(|p| info!("{:?}", p));
    d.get_cell_at(glm::vec2(0., 0.), glm::vec2(49.9, 2.5), 1)
        .map(|p| info!("{:?}", p));
    d.pos_for_cell_at(glm::vec2(0., 0.), (0, 0), 0)
        .map(|p| info!("{:?}", p));

    d.put_data((0,0), (1,1,0), 999);
    for i in 0..3 {
        info!("{:?}", d.max_cell_at(i))
    }
}
