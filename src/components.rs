use bevy::prelude::*;

#[derive(Component)]
pub struct MapBase;

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct Destination;

#[derive(Component)]
pub struct Selected;

#[derive(Component, Clone)]
pub struct FlowField {
    // pub cells: Vec<Vec<Cell>>,
    // pub rows: usize,
    // pub columns: usize,
    // pub destination: (usize, usize),
    // pub entities: Vec<Entity>,
    pub cell_radius: f32,
    pub cell_diameter: f32,
    pub grid: Vec<Vec<Cell>>,
    pub grid_size: (usize, usize),
}

impl FlowField {
    fn new(cell_radius: f32, grid_size: (usize, usize)) -> Self {
        FlowField {
            cell_radius,
            cell_diameter: cell_radius * 2.,
            grid: Vec::default(),
            grid_size,
        }
    }

    fn create_grid(&mut self) {
        self.grid = 
    }

    // pub fn new(
    //     rows: usize,
    //     columns: usize,
    //     target_row: usize,
    //     target_column: usize,
    //     cell_size: f32,
    // ) -> Self {
    //     let mut grid: Vec<Vec<Cell>> = vec![];

    //     // Calculate the offset to center the grid at (0, 0, 0)
    //     let grid_width = rows as f32 * cell_size;
    //     let grid_depth = columns as f32 * cell_size;
    //     let offset_x = grid_width / 2.0 - cell_size / 2.0;
    //     let offset_z = grid_depth / 2.0 - cell_size / 2.0;

    //     for x in 0..rows {
    //         let mut row: Vec<Cell> = vec![];
    //         for z in 0..columns {
    //             let world_x = x as f32 * cell_size - offset_x;
    //             let world_z = z as f32 * cell_size - offset_z;

    //             row.push(Cell::new(Vec3::new(world_x, 0.0, world_z), (x, z)));
    //         }

    //         grid.push(row);
    //     }

    //     grid[target_row][target_column].cost = 0.0;

    //     FlowField {
    //         cells: grid,
    //         rows,
    //         columns,
    //         destination: (target_row, target_column),
    //         entities: Vec::new(),
    //     }
    // }

    fn create_cost_field() {
        // let cell_half_extents = Vec3::ONE *
    }
}

#[derive(Clone, Default)]
pub struct Cell {
    pub world_position: Vec3,
    // pub flow_vector: Vec3,
    pub grid_idx: (usize, usize),
    pub cost: u8,
    // pub cost: f32,
    // pub occupied: bool,
}

impl Cell {
    // fn new(world_position: Vec3, grid_idx: (usize, usize)) -> Self {
    //     Cell {
    //         world_position,
    //         grid_idx,
    //         flow_vector: Vec3::ZERO,
    //         // cost: f32::INFINITY,
    //         // cost: u8,
    //         occupied: false,
    //     }
    // }

    fn new(world_position: Vec3, grid_idx: (usize, usize)) -> Self {
        Cell {
            world_position,
            grid_idx,
            cost: 1,
        }
    }

    fn increase_cost(&mut self, amount: u8) {
        if self.cost == u8::MAX {
            return;
        }

        if amount + self.cost >= u8::MAX {
            self.cost = u8::MAX;
        } else {
            self.cost += amount;
        }
    }
}
