/// 1. make grid
/// 2. set values for grid hardness
///
/// 3. on click calculate routes starting from destination
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rx::glm;

mod pathfinding {
    #[allow(unused_imports)]
    use log::{debug, error, info, trace, warn};

    use rx::glm;

    const DEFAULT_INTEGRATION_COST: u32 = u32::MAX - 1000;
    const BLOCKED_COST: u32 = u32::MAX;
    pub const BLOCKED: u32 = u32::MAX;

    #[derive(Debug)]
    pub struct FlowGridCell {
        integration: u32,
        cost: u32,
        visited: bool,
    }

    pub struct FlowGrid {
        cell_width: f32,
        n_width: usize,
        n_height: usize,
        cells: Vec<Vec<FlowGridCell>>,
    }

    pub struct FlowField {
        cell_width: f32,
        n_width: usize,
        n_height: usize,
        cells: Vec<Vec<glm::IVec2>>,
    }

    fn cell_at(
        cell_width: f32,
        n_width: usize,
        n_height: usize,
        origin: &glm::Vec2,
        coords: &glm::Vec2,
    ) -> Option<(usize, usize)> {
        let x = ((coords.x + (-1. * origin.x)) / cell_width) as i32;
        let y = n_height as i32 - ((-coords.y + origin.y) / cell_width) as i32 - 1;

        if x >= 0 && x < n_width as i32 && y >= 0 && y < n_height as i32 {
            return Some((x as usize, y as usize));
        }
        return None;
    }

    impl FlowField {
        #[allow(dead_code)]
        pub fn print_flowfield(&self) {
            let flow = self.to_char_flowfield(&self.cells);
            for line in flow.iter().rev() {
                let mut string_vec = Vec::with_capacity(self.n_width);
                for (_i, c) in line.iter().enumerate() {
                    string_vec.push(c.to_string());
                }
                info!("{:?}", string_vec);
            }
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
    }


    ///public api
    impl FlowGrid {
        pub fn flow_field_at(&mut self, origin: &glm::Vec2, target: &glm::Vec2) -> Option<FlowField> {
            let cell = cell_at(self.cell_width, self.n_width, self.n_height, origin, target)?;
            info!("{:?}", cell);
            self.update_integration_field(cell)?;
            let field = self.flow_field();
            // self.reset_field();
            Some(field)
        }
    }

    ///integration upd & flowfield
    impl FlowGrid {
        fn reset_field(&mut self) {
            self.cells
                .iter_mut()
                .flat_map(|a| a.iter_mut())
                .for_each(|c| {
                    c.visited = false;
                    c.integration = 0;
                });
        }

        #[allow(dead_code)]
        pub fn print_integration(&self) {
            for (_y, line) in self.cells.iter().rev().enumerate() {
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

        fn update_integration_field(&mut self, cell: (usize, usize)) -> Option<()> {
            let selected_cell = glm::vec2(cell.0 as u32, cell.1 as u32);
            let size = self.n_height.clone() * self.n_width.clone();
            let cell = self.cell_mut(selected_cell.x, selected_cell.y);
            let cell_cost = cell.cost.clone();
            cell.cost = 0;
            cell.integration = 0;

            let mut open_list: Vec<glm::UVec2> = Vec::with_capacity(size);

            open_list.push(selected_cell);

            while !open_list.is_empty() {
                let cell_xy = open_list.pop().unwrap();
                let neighbors = self.neighbors_cross(cell_xy.x, cell_xy.y);
                self.cell_mut(cell_xy.x, cell_xy.y).visited = true;

                for n in neighbors.iter() {
                    let cost = self.cell(cell_xy.x, cell_xy.y).cost;
                    let integration = self.cell(cell_xy.x, cell_xy.y).integration;
                    let n_integration = self.cell(n.x, n.y).integration;

                    if !self.cell(n.x, n.y).visited {
                        open_list.push(n.clone_owned());
                    }
                    if n_integration == BLOCKED_COST {
                        continue;
                    }
                    if cost == BLOCKED_COST {
                        self.cell_mut(cell_xy.x, cell_xy.y).integration = BLOCKED_COST;
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

                if self.cell(cell_xy.x, cell_xy.y).integration == DEFAULT_INTEGRATION_COST {
                    self.cell_mut(cell_xy.x, cell_xy.y).integration = BLOCKED_COST;
                }
            }
            let cell2 = self.cell_mut(selected_cell.x, selected_cell.y);
            cell2.cost = cell_cost;
            Some(())
        }

        pub fn flow_field(&self) -> FlowField {
            let mut response = Vec::with_capacity(self.n_height);
            for (y, line) in self.cells.iter().enumerate() {
                let mut c_line = Vec::with_capacity(self.n_width);
                for (x, c) in line.iter().enumerate() {
                    let int = c.integration;
                    let mut character = glm::vec2(0, 0);
                    let mut min = BLOCKED;
                    if int == BLOCKED {
                        character = glm::vec2(i32::MAX, i32::MAX);
                        c_line.push(character);
                        continue;
                    }
                    for n in self.neighbors_1grid(x as u32, y as u32).iter() {
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
            FlowField {
                cell_width: self.cell_width,
                n_width: self.n_width,
                n_height: self.n_height,
                cells: response,
            }
        }
    }

    ///cell accessors
    impl FlowGrid {
        #[allow(dead_code)]
        pub fn set_cost(&mut self, x: u32, y: u32, cost: u32) {
            self.cells[self.n_height - 1 - y as usize][x as usize].cost = cost;
        }

        pub fn set_cost_all(&mut self, cost: u32) {
            self.cells.iter_mut().flat_map(|v| { v.iter_mut() }).for_each(|c| {
                c.cost = cost
            })
        }

        #[allow(dead_code)]
        fn cell(&self, x: u32, y: u32) -> &FlowGridCell {
            &self.cells[y as usize][x as usize]
        }

        #[allow(dead_code)]
        fn cell_mut(&mut self, x: u32, y: u32) -> &mut FlowGridCell {
            &mut self.cells[y as usize][x as usize]
        }
    }

    ///neighbours
    impl FlowGrid {
        fn neighbors_cross(&self, x: u32, y: u32) -> Vec<glm::UVec2> {
            let mut neighbours = Vec::new();
            let n = glm::vec2(x, y + 1);
            if n.y <= (self.n_height - 1) as u32 {
                neighbours.push(n)
            }
            if y >= 1 {
                neighbours.push(glm::vec2(x, y - 1))
            }
            let e = glm::vec2(x + 1, y);
            if e.x <= (self.n_width - 1) as u32 {
                neighbours.push(e)
            }
            if x >= 1 {
                neighbours.push(glm::vec2(x - 1, y))
            }
            neighbours
        }

        fn neighbors_1grid(&self, x: u32, y: u32) -> Vec<glm::UVec2> {
            let mut neighbours = Vec::new();

            if y < self.n_height as u32 - 1 {
                //n
                neighbours.push(glm::vec2(x, y + 1));
                //nw
                if x >= 1 {
                    neighbours.push(glm::vec2(x - 1, y + 1))
                }
                //ne
                if x < self.n_width as u32 - 1 {
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
                if x < self.n_width as u32 - 1 {
                    neighbours.push(glm::vec2(x + 1, y - 1))
                }
            }

            //w
            if x >= 1 {
                neighbours.push(glm::vec2(x - 1, y))
            }
            //e
            if x < self.n_width as u32 - 1 {
                neighbours.push(glm::vec2(x + 1, y))
            }
            neighbours
        }
    }

    impl FlowGrid {
        pub fn new(cell_width: f32, n_width: usize, n_height: usize) -> Self {
            let mut v_vec = Vec::with_capacity(n_height);
            for _ in 0..n_height {
                let mut h_vec = Vec::with_capacity(n_width);
                for _ in 0..n_width {
                    h_vec.push(FlowGridCell {
                        integration: DEFAULT_INTEGRATION_COST,
                        cost: 1,
                        visited: false,
                    })
                }
                v_vec.push(h_vec);
            }
            FlowGrid { cell_width, n_width, n_height, cells: v_vec }
        }
    }
}

#[test]
fn test() {
    crate::init_log();
    // let mut grid = Grid::new(&glm::vec2(-13.5, 26.), 2.5, 11);
    // {
    //     grid.set_cost_all(fields::BLOCKED);
    // }
    // grid.integration_field(&glm::vec2(0., 26.));
    // info!("{:?}", grid.flow_field().get_dir(&glm::vec2(-13., 26.)));
    // grid.print_integration();
    // grid.print_flowfield();

    let mut field = pathfinding::FlowGrid::new(2.5, 50, 11);
    // field.set_cost_all(pathfinding::BLOCKED);
    field.set_cost(0, 0, 1);
    field.set_cost(0, 1, 1);
    field.set_cost(0, 2, 1);
    field.set_cost(0, 3, 1);
    field.set_cost(1, 3, 1);
    field.set_cost(0, 4, 1);
    field.set_cost(1, 4, 1);
    field.set_cost(2, 4, 1);
    field.set_cost(3, 4, 1);
    field.set_cost(4, 4, 1);
    let flow_field = field
        .flow_field_at(
            &glm::vec2(-13.5, 26.),
            &glm::vec2(-13.5, 26.),
        );
    field.print_integration();
    flow_field.map(|f| {
        f.print_flowfield();
    });
}
