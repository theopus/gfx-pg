/// 1. make grid
/// 2. set values for grid hardness
///
/// 3. on click calculate routes starting from destination
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rx::glm;
use rx::glm::UVec2;

#[derive(Debug)]
struct Cell {
    cost: u32,
    integration: u32,
    visited: bool,
    ch: char,
}

#[allow(dead_code)]
struct Grid {
    array: Vec<Vec<Cell>>,
    n_cells: usize,
    cells_width: f32,
    origin: glm::Vec2,
}

#[allow(dead_code)]
const BLOCKED: u32 = u32::max_value();
#[allow(dead_code)]
const DEFAULT: u32 = u32::max_value() - 1000;

impl Grid {
    #[allow(dead_code)]
    fn new(a: &glm::Vec2, cells_width: f32, n_cells: usize) -> Self {
        use std::iter;
        let grid_array = iter::repeat_with(|| {
            iter::repeat_with(|| Cell {
                cost: 1,
                integration: DEFAULT,
                visited: false,
                ch: '*',
            })
                .take(n_cells)
                .collect::<Vec<Cell>>()
        })
            .take(n_cells)
            .collect::<Vec<Vec<Cell>>>();

        Self {
            array: grid_array,
            n_cells,
            cells_width,
            origin: a.clone_owned(),
        }
    }
    #[allow(dead_code)]
    fn cell_coords(&self, coords: &glm::Vec2) -> glm::UVec2 {
        let x = ((coords.x + (-1. * self.origin.x)) / self.cells_width) as u32;
        let y = ((coords.y + (-1. * self.origin.y)) / self.cells_width) as u32;
        glm::vec2(x, y)
    }
    #[allow(dead_code)]
    pub fn neighbors(&self, x: u32, y: u32) -> Vec<glm::UVec2> {
        let mut neighbours = Vec::new();
        let n = glm::vec2(x, y + 1);
        if n.y <= (self.n_cells - 1) as u32 {
            neighbours.push(n)
        }
        if y >= 1 {
            neighbours.push(glm::vec2(x, y - 1))
        }
        let e = glm::vec2(x + 1, y);
        if e.x <= (self.n_cells - 1) as u32 {
            neighbours.push(e)
        }
        if x >= 1 {
            neighbours.push(glm::vec2(x - 1, y))
        }
        neighbours
    }

    #[allow(dead_code)]
    pub fn neighbors_1(&self, x: u32, y: u32) -> Vec<glm::UVec2> {
        let mut neighbours = Vec::new();

        if y < self.n_cells as u32 - 1 {
            //n
            neighbours.push(glm::vec2(x, y + 1));
            //nw
            if x >= 1 {
                neighbours.push(glm::vec2(x - 1, y + 1))
            }
            //ne
            if x < self.n_cells as u32 - 1 {
                neighbours.push(glm::vec2(x + 1, y + 1))
            }
        }

        if y >= 1 {
            //s
            neighbours.push(glm::vec2(x, y - 1));
            //sw
            if x >= 1 {
                neighbours.push(glm::vec2(x - 1, y - 1))
            }
            //se
            if x < self.n_cells as u32 - 1 {
                neighbours.push(glm::vec2(x + 1, y - 1))
            }
        }

        //w
        if x >= 1 {
            neighbours.push(glm::vec2(x - 1, y))
        }
        //e
        if x < self.n_cells as u32 - 1 {
            neighbours.push(glm::vec2(x + 1, y))
        }
        neighbours
    }

    #[allow(dead_code)]
    pub fn print_integration(&self) {
        for (_y, line) in self.array.iter().rev().enumerate() {
            for (_x, c) in line.iter().enumerate() {
                if c.integration == BLOCKED {
                    print!("-X-");
                    continue;
                };

                if c.integration >= 10 {
                    print!("{:?}-", c.integration);
                } else {
                    print!("-{:?}-", c.integration);
                }
            }
            println!("");
        }
    }

    #[allow(dead_code)]
    pub fn print_flowfield(&self) {
        let flow = self.to_char_flowfield(&self.flow_field());
        for line in flow.iter().rev() {
            let mut string_vec = Vec::with_capacity(self.n_cells);
            for (_i, c) in line.iter().enumerate() {
                string_vec.push(c.to_string());
            }
            info!("{:?}", string_vec);
        }
    }

    #[allow(dead_code)]
    pub fn cell(&self, x: u32, y: u32) -> &Cell {
        // maybe wrong?
        &self.array[y as usize][x as usize]
    }

    #[allow(dead_code)]
    pub fn cell_mut(&mut self, x: u32, y: u32) -> &mut Cell {
        // maybe wrong?
        &mut self.array[y as usize][x as usize]
    }

    pub fn flow_field(&self) -> Vec<Vec<glm::IVec2>> {
        let mut response = Vec::with_capacity(self.n_cells);
        for (y, line) in self.array.iter().enumerate() {
            let mut c_line = Vec::with_capacity(self.n_cells);
            for (x, c) in line.iter().enumerate() {
                let int = c.integration;
                let mut character = glm::vec2(0, 0);
                let mut min = u32::max_value();
                if int == BLOCKED {
                    character = glm::vec2(i32::max_value(), i32::max_value());
                    c_line.push(character);
                    continue;
                }
                for n in self.neighbors_1(x as u32, y as u32).iter() {
                    let n_int = self.cell(n.x, n.y).integration;

                    if (n_int < int) && n_int < min {
                        min = n_int;
                        character =
                            &glm::vec2(n.x as i32, n.y as i32) - &glm::vec2(x as i32, y as i32);
                    }
                }
                c_line.push(character);
            }
            response.push(c_line);
        }
        response
    }

    #[allow(dead_code)]
    pub fn to_char_flowfield(&self, field: &Vec<Vec<glm::IVec2>>) -> Vec<Vec<char>> {
        let mut response = Vec::with_capacity(field.len());
        for (_y, line) in field.iter().enumerate() {
            let mut c_line = Vec::with_capacity(line.len());
            for (_x, dir) in line.iter().enumerate() {
                let mut character = '↯';
                if dir.x == i32::max_value() && dir.y == i32::max_value() {
                    character = 'x';
                } else {
                    if dir.x > 0 && dir.y > 0 {
                        character = '↗';
                    }
                    if dir.x < 0 && dir.y < 0 {
                        character = '↙';
                    }
                    if dir.x < 0 && dir.y > 0 {
                        character = '↖';
                    }
                    if dir.x > 0 && dir.y < 0 {
                        character = '↘';
                    }
                    if dir.x > 0 && dir.y == 0 {
                        character = '→';
                    }
                    if dir.x < 0 && dir.y == 0 {
                        character = '←';
                    }
                    if dir.y > 0 && dir.x == 0 {
                        character = '↑';
                    }
                    if dir.y < 0 && dir.x == 0 {
                        character = '↓';
                    }
                }
                c_line.push(character);
            }
            response.push(c_line);
        }
        response
    }

    #[allow(dead_code)]
    pub fn integration_field(&mut self, coords: &glm::Vec2) {
        let selected_cell = self.cell_coords(coords);
        info!("{:?}", selected_cell);
        let cell = &mut self.cell_mut(selected_cell.x, selected_cell.y);
        cell.cost = 0;
        cell.integration = 0;

        let mut open_list: Vec<UVec2> = Vec::with_capacity(self.n_cells * self.n_cells);
        open_list.push(selected_cell);

        while !open_list.is_empty() {
            let cell_xy = open_list.pop().unwrap();
            let neighbors = self.neighbors(cell_xy.x, cell_xy.y);
            self.cell_mut(cell_xy.x, cell_xy.y).visited = true;

            for n in neighbors.iter() {
                let cost = self.cell(cell_xy.x, cell_xy.y).cost;
                let integration = self.cell(cell_xy.x, cell_xy.y).integration;
                let n_integration = self.cell(n.x, n.y).integration;

                if !self.cell(n.x, n.y).visited {
                    open_list.push(n.clone_owned());
                }
                if n_integration == BLOCKED {
                    continue;
                }
                if cost == BLOCKED {
                    self.cell_mut(cell_xy.x, cell_xy.y).integration = BLOCKED;
                    continue;
                }

                let integration_cost = if self.cell(n.x, n.y).visited {
                    n_integration + cost
                } else {
                    n_integration
                };
                if integration_cost < integration {
                    self.cell_mut(cell_xy.x, cell_xy.y).integration = integration_cost;
                }
            }

            if self.cell(cell_xy.x, cell_xy.y).integration == DEFAULT {
                self.cell_mut(cell_xy.x, cell_xy.y).integration = BLOCKED;
            }
        }
    }
}

#[test]
fn test() {
    crate::init_log();
    let mut grid = Grid::new(&glm::vec2(-100., -100.), 20., 11);
    {
        grid.cell_mut(4, 6).cost = BLOCKED;
        grid.cell_mut(5, 6).cost = BLOCKED;
        grid.cell_mut(6, 6).cost = BLOCKED;
        grid.cell_mut(6, 5).cost = BLOCKED;
        grid.cell_mut(6, 4).cost = BLOCKED;
        grid.cell_mut(5, 4).cost = BLOCKED;
        grid.cell_mut(4, 4).cost = BLOCKED;
        grid.cell_mut(4, 5).cost = BLOCKED;
    }
    grid.integration_field(&glm::vec2(119., 119.));
    grid.print_integration();
    grid.print_flowfield();
}
