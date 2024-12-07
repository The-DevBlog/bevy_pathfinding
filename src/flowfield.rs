use crate::{cell::*, grid_direction::GridDirection};
use bevy::prelude::*;
use bevy_rapier3d::{plugin::RapierContext, prelude::*};
use std::{cmp::min, collections::VecDeque};

#[derive(Clone, Default, PartialEq)]
pub struct FlowField {
    pub cell_radius: f32,
    pub cell_diameter: f32,
    pub destination_cell: Cell,
    pub grid: Vec<Vec<Cell>>,
    pub grid_size: IVec2,
}

impl FlowField {
    pub fn new(cell_radius: f32, grid_size: IVec2) -> Self {
        FlowField {
            cell_radius,
            cell_diameter: cell_radius * 2.,
            destination_cell: Cell::default(),
            grid: Vec::default(),
            grid_size,
        }
    }

    pub fn create_grid(&mut self) {
        // Calculate offsets for top-left alignment
        let offset_x = -(self.grid_size.x as f32 * self.cell_diameter) / 2.;
        let offset_y = -(self.grid_size.y as f32 * self.cell_diameter) / 2.;

        self.grid = (0..self.grid_size.y)
            .map(|y| {
                (0..self.grid_size.x)
                    .map(|x| {
                        let x_pos = self.cell_diameter * x as f32 + self.cell_radius + offset_x;
                        let y_pos = self.cell_diameter * y as f32 + self.cell_radius + offset_y;
                        let world_pos = Vec3::new(x_pos, 0.0, y_pos);
                        Cell::new(world_pos, IVec2::new(x, y))
                    })
                    .collect()
            })
            .collect();
    }

    pub fn create_costfield(&mut self, rapier_ctx: &RapierContext) {
        for cell_row in self.grid.iter_mut() {
            for cell in cell_row.iter_mut() {
                let hit = rapier_ctx.intersection_with_shape(
                    cell.world_position,
                    Quat::IDENTITY,
                    &Collider::cuboid(self.cell_radius, self.cell_radius, self.cell_radius),
                    QueryFilter::default().exclude_sensors(),
                );

                if let Some(_) = hit {
                    cell.increase_cost(255);
                }
            }
        }
    }

    pub fn create_integration_field(&mut self, destination_cell: Cell) {
        // Initialize the destination cell in the grid
        let dest_idx = destination_cell.grid_idx;
        let dest_cell = &mut self.grid[dest_idx.y as usize][dest_idx.x as usize];
        dest_cell.cost = 0;
        dest_cell.best_cost = 0;
        self.destination_cell = dest_cell.clone();

        let mut cells_to_check: VecDeque<IVec2> = VecDeque::new();
        cells_to_check.push_back(dest_idx);

        while let Some(cur_idx) = cells_to_check.pop_front() {
            let cur_x = cur_idx.x as usize;
            let cur_y = cur_idx.y as usize;

            let cur_cell_best_cost = self.grid[cur_y][cur_x].best_cost;

            // Iterate over cardinal directions
            for direction in GridDirection::cardinal_directions() {
                let delta = direction.vector();
                let neighbor_idx = cur_idx + delta;

                if neighbor_idx.x >= 0
                    && neighbor_idx.x < self.grid_size.x
                    && neighbor_idx.y >= 0
                    && neighbor_idx.y < self.grid_size.y
                {
                    let neighbor_x = neighbor_idx.x as usize;
                    let neighbor_y = neighbor_idx.y as usize;

                    let neighbor_cell = &mut self.grid[neighbor_y][neighbor_x];

                    if neighbor_cell.cost == u8::MAX {
                        continue;
                    }

                    let tentative_best_cost = neighbor_cell.cost as u16 + cur_cell_best_cost;
                    if tentative_best_cost < neighbor_cell.best_cost {
                        neighbor_cell.best_cost = tentative_best_cost;
                        cells_to_check.push_back(neighbor_idx);
                    }
                }
            }
        }
    }

    pub fn create_flowfield(&mut self) {
        let grid_size_y = self.grid_size.y as usize;
        let grid_size_x = self.grid_size.x as usize;

        for y in 0..grid_size_y {
            for x in 0..grid_size_x {
                let cell = &self.grid[y][x]; // Immutable borrow to get best_cost
                let mut best_cost = cell.best_cost;
                let mut best_direction = GridDirection::None;

                // Get all possible directions
                for direction in GridDirection::all_directions() {
                    let delta = direction.vector();
                    let nx = x as isize + delta.x as isize;
                    let ny = y as isize + delta.y as isize;

                    if nx >= 0 && nx < grid_size_x as isize && ny >= 0 && ny < grid_size_y as isize
                    {
                        let neighbor = &self.grid[ny as usize][nx as usize];
                        if neighbor.best_cost < best_cost {
                            best_cost = neighbor.best_cost;
                            best_direction = direction;
                        }
                    }
                }

                // Now, set the best_direction for the cell
                self.grid[y][x].best_direction = best_direction;
            }
        }

        // TODO: This was from the original tutorial. Do I need to do it this way?
        // for y in 0..grid_size_y {
        //     for x in 0..grid_size_x {
        //         let cur_cell = &mut self.grid[y][x].clone();

        //         let cur_neighbors =
        //             self.get_neighbor_cells(cur_cell.grid_idx, GridDirection::all_directions());

        //         let mut best_cost = cur_cell.best_cost;

        //         for cur_neighbor in cur_neighbors.iter() {
        //             if cur_neighbor.best_cost < best_cost {
        //                 best_cost = cur_neighbor.best_cost;
        //                 let best_direction =
        //                     GridDirection::from_vector2(cur_neighbor.grid_idx - cur_cell.grid_idx);

        //                 if let Some(best_direction) = best_direction {
        //                     cur_cell.best_direction = best_direction;
        //                 }

        //                 self.grid[y][x] = *cur_cell;
        //             }
        //         }
        //     }
        // }
    }

    // TODO: This was from the original tutorial. Do I need to do it this way?
    // TODO: use this in create_integration field above
    // fn get_neighbor_cells(&self, node_index: IVec2, directions: Vec<GridDirection>) -> Vec<Cell> {
    //     let mut neighbor_cells = Vec::new();

    //     for direction in directions {
    //         if let Some(new_neighbor) = self.get_cell_at_relative_pos(node_index, direction) {
    //             neighbor_cells.push(new_neighbor);
    //         }
    //     }
    //     neighbor_cells
    // }

    // TODO: This was from the original tutorial. Do I need to do it this way?
    // TODO: use this in create_integration field above
    // fn get_cell_at_relative_pos(
    //     &self,
    //     origin_pos: IVec2,
    //     direction: GridDirection,
    // ) -> Option<Cell> {
    //     let relative_pos = direction.vector();
    //     let final_pos = origin_pos + relative_pos;

    //     if final_pos.x < 0
    //         || final_pos.x >= self.grid_size.x
    //         || final_pos.y < 0
    //         || final_pos.y >= self.grid_size.y
    //     {
    //         None
    //     } else {
    //         Some(self.grid[final_pos.y as usize][final_pos.x as usize]) // Note the swap of y and x
    //     }
    // }

    pub fn get_cell_from_world_position(&self, world_pos: Vec3) -> Cell {
        // Adjust world position relative to the grid's top-left corner
        let adjusted_x = world_pos.x - (-self.grid_size.x as f32 * self.cell_diameter / 2.0);
        let adjusted_y = world_pos.z - (-self.grid_size.y as f32 * self.cell_diameter / 2.0);

        // Calculate percentages within the grid
        let mut percent_x = adjusted_x / (self.grid_size.x as f32 * self.cell_diameter);
        let mut percent_y = adjusted_y / (self.grid_size.y as f32 * self.cell_diameter);

        // Clamp percentages to ensure they're within [0.0, 1.0]
        percent_x = percent_x.clamp(0.0, 1.0);
        percent_y = percent_y.clamp(0.0, 1.0);

        // Calculate grid indices
        let x = ((self.grid_size.x as f32) * percent_x).floor() as usize;
        let y = ((self.grid_size.y as f32) * percent_y).floor() as usize;

        println!("x: {}, grid size x: {}", x, self.grid_size.x);
        let x = min(x, self.grid_size.x as usize - 1);
        let y = min(y, self.grid_size.y as usize - 1);

        self.grid[y][x] // Swap x and y
    }
}
