use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rx::glm;

use crate::glm::Vec2;

#[derive(Debug)]
pub struct Grid2D {
    step: f32,
    dimensions: u32,
    depth: u32,

    dimensions_x: u32,
    dimensions_y: u32,
    bounds: Option<(u32, u32)>,
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct Grid2DCell {
    x: u32,
    y: u32,
}

impl Grid2D {
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
        if let Some(bounds) = self.bounds {
            let cells_per = self.cells_per(level);
            let mut max_x = bounds.0 / cells_per.0;
            let mut max_y = bounds.1 / cells_per.1;
            if max_x == 0 { max_x = 1 };
            if max_y == 0 { max_y = 1 };

            return x >= -(max_x as i32) && x < max_x as i32
                && y >= -(max_y as i32) && y < max_y as i32;
        }
        true
    }

    pub fn get_cell_at(&self, origin: &glm::Vec2, point: &glm::Vec2, level: u32) -> Option<glm::IVec2> {
        let relative_point: glm::Vec2 = point.borrow() - origin.borrow();
        let (x_level_step, y_level_step) = self.get_level_step(level);
        let mut x = (relative_point.x / x_level_step);
        let mut y = (relative_point.y / y_level_step);

        if x < 0. { x=x.floor()};
        if y < 0. { y=y.floor()};

        if self.is_in_bounds((x as i32, y as i32), level) {
            return Some(glm::vec2(x as i32, y as i32));
        }
        None
    }

    ///returns center of cell
    pub fn pos_for_cell_at(&self, origin: &glm::Vec2, (x, y): (i32, i32), level: u32) -> Option<glm::Vec2> {
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
    pub fn pos_for_cell_start(&self, origin: &glm::Vec2, (x, y): (i32, i32), level: u32) -> Option<glm::Vec2> {
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

impl Grid2D {
    pub fn new_symmetrical(step: f32, dimensions: u32, depth: u32) -> Self {
        Self {
            step,
            dimensions,
            depth,
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
            dimensions_x: dimensions.0,
            dimensions_y: dimensions.1,
            bounds,
        }
    }
}

#[test]
fn test() {
    crate::init_log();
    let mut d: Grid2D = Grid2D::new_asymmetrical(2., (5, 5), 1, Some(((5,5), 0)));

    d.get_cell_at(&glm::vec2(0., 0.), &glm::vec2(-22.6, -2.5), 0)
        .map(|p| info!("0 {:?}", p));

    d.pos_for_cell_at(&glm::vec2(0., 0.), (-1, -1), 0)
        .map(|p| info!("0 {:?}", p));

    d.pos_for_cell_start(&glm::vec2(0., 0.), (-1, -1), 0)
        .map(|p| info!("0 {:?}", p));
}
