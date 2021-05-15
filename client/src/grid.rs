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

    dimensions_x: u32,
    dimensions_y: u32,
    bounds: Option<(u32, u32)>,
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct Grid2DCell {
    x: u32,
    y: u32,
}

impl<V> Grid2D<V> {
    fn get_level_step(&self, level: u32) -> (f32, f32) {
        return if level == 0 {
            (self.step, self.step)
        } else {
            (
                self.step * self.dimensions_x as f32 * level as f32,
                self.step * self.dimensions_y as f32 * level as f32,
            )
        };
    }

    pub fn put_data(&mut self, level0: (u32, u32), offset: (u32, u32, u32), data: V) {
        let (cell_per_level_x, cell_per_level_y) = self.cells_per(offset.2);
        self.put_data_0(
            (level0.0 + offset.0 * cell_per_level_x,
             level0.1 + offset.1 * cell_per_level_y),
            data,
        )
    }

    fn put_data_0(&mut self, (x, y): (u32, u32), data: V) {
        info!("put to: {:?}", (x, y));
        self.data.insert(Grid2DCell { x, y }, data);
    }

    pub fn cells_per(&self, level: u32) -> (u32, u32) {
        (
            self.dimensions_x.pow(level),
            self.dimensions_y.pow(level),
        )
    }

    pub fn max_cell_at(&self, level: u32) -> (u32, u32) {
        if let Some(bounds) = self.bounds {
            let cells_per = self.cells_per(level);
            let mut x = bounds.0 / cells_per.0;
            let mut y = bounds.1 / cells_per.1;
            if x == 0 { x = 1 };
            if y == 0 { y = 1 };
            (x, y)
        } else {
            (
                self.dimensions_x.pow(self.depth + 1 - level),
                self.dimensions_y.pow(self.depth + 1 - level),
            )
        }
    }

    fn is_in_bounds(&self, (x, y): (i32, i32), level: u32) -> bool {
        let (max_x, max_y) = self.max_cell_at(level);
        x >= 0 && x < max_x as i32
            && y >= 0 && y < max_y as i32
    }

    pub fn get_cell_at(&self, origin: &glm::Vec2, point: &glm::Vec2, level: u32) -> Option<glm::UVec2> {
        let relative_point: glm::Vec2 = point.borrow() - origin.borrow();
        let (x_level_step, y_level_step) = self.get_level_step(level);
        let x = (relative_point.x / x_level_step) as i32;
        let y = (relative_point.y / y_level_step) as i32;

        if self.is_in_bounds((x, y), level) {
            return Some(glm::vec2(x as u32, y as u32));
        }
        None
    }

    ///returns center of cell
    pub fn pos_for_cell_at(&self, origin: &glm::Vec2, (x, y): (u32, u32), level: u32) -> Option<glm::Vec2> {
        if !self.is_in_bounds((x as i32, y as i32), level) {
            return None;
        }
        let (level_step_x, level_step_y) = self.get_level_step(level);
        let x = x as f32 * level_step_x;
        let y = y as f32 * level_step_y;
        let mut target: glm::Vec2 = (glm::vec2(x, y) + origin.borrow());
        target.x += level_step_x / 2.;
        target.y += level_step_y / 2.;
        Some(target)
    }

    ///returns top left of cell
    pub fn pos_for_cell_start(&self, origin: &glm::Vec2, (x, y): (u32, u32), level: u32) -> Option<glm::Vec2> {
        if !self.is_in_bounds((x as i32, y as i32), level) {
            return None;
        }
        let (level_step_x, level_step_y) = self.get_level_step(level);
        let x = x as f32 * level_step_x;
        let y = y as f32 * level_step_y;
        let mut target: glm::Vec2 = (glm::vec2(x, y) + origin.borrow());
        Some(target)
    }
}

impl<V> Grid2D<V> {
    pub fn new_symmetrical(step: f32, dimensions: u32, depth: u32) -> Self {
        Self {
            step,
            dimensions,
            depth,
            data: HashMap::with_capacity(dimensions.pow(2) as usize),
            dimensions_x: dimensions,
            dimensions_y: dimensions,
            bounds: None,
        }
    }
    pub fn new_asymmetrical(step: f32, dimensions: (u32, u32), depth: u32, bounds: Option<((u32, u32), u32)>) -> Self {
        let max = dimensions.0.max(dimensions.1);

        let bounds = bounds.map(|((x, y), level)| {
            (dimensions.0.pow(level) * (x),
             dimensions.1.pow(level) * (y))
        });

        Self {
            step,
            dimensions: max,
            depth,
            data: HashMap::with_capacity(max.pow(2) as usize),
            dimensions_x: dimensions.0,
            dimensions_y: dimensions.1,
            bounds,
        }
    }
}

#[test]
fn test() {
    crate::init_log();
    let mut d: Grid2D<u32> = Grid2D::new_asymmetrical(2., (5, 10), 2, Some(((1, 10_000), 1)));

    d.get_cell_at(&glm::vec2(0., 0.), &glm::vec2(9.0, 2.5), 0)
        .map(|p| info!("0 {:?}", p));
    d.get_cell_at(&glm::vec2(0., 0.), &glm::vec2(9.0, 2.5), 2)
        .map(|p| info!("2 {:?}", p));
    d.get_cell_at(&glm::vec2(0., 0.), &glm::vec2(9.0, 2.5), 1)
        .map(|p| info!("1 {:?}", p));
    info!("max for cell:");

    for i in 0..3 {
        info!("l: {:?} = {:?}", i, &d.max_cell_at(i))
    }
}
