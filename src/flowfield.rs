use std::{cell, cmp::min, collections::VecDeque};

use bevy::prelude::*;
use bevy_rapier3d::{plugin::RapierContext, prelude::*};

use crate::{cell::*, grid_direction::GridDirection};

#[derive(Clone, Default)]
pub struct FlowField {
    pub cell_radius: f32,
    pub cell_diameter: f32,
    pub destination_cell: Option<Cell>,
    pub grid: Vec<Vec<Cell>>,
    pub grid_size: IVec2,
}

impl FlowField {
    pub fn new(cell_radius: f32, grid_size: IVec2) -> Self {
        FlowField {
            cell_radius,
            cell_diameter: cell_radius * 2.,
            destination_cell: None,
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
        let mut tmp_destination_cell = destination_cell.clone();
        tmp_destination_cell.cost = 0;
        tmp_destination_cell.best_cost = 0;
        self.destination_cell = Some(tmp_destination_cell);

        let mut cells_to_check: VecDeque<Cell> = VecDeque::new();
        let destination_cell = self.destination_cell.unwrap().clone();
        cells_to_check.push_back(destination_cell);

        while let Some(cur_cell) = cells_to_check.pop_front() {
            let cur_neighbors =
                self.get_neighbor_cells(cur_cell.grid_idx, GridDirection::cardinal_directions());

            for mut cur_neighbor in cur_neighbors {
                if cur_neighbor.cost == u8::MAX {
                    continue;
                }

                if cur_neighbor.cost as u16 + cur_cell.best_cost < cur_neighbor.best_cost {
                    let neighbor_index = cur_neighbor.grid_idx;
                    cur_neighbor.best_cost = cur_neighbor.cost as u16 + cur_cell.best_cost;
                    // self.grid[neighbor_index.x as usize][neighbor_index.y as usize] = cur_neighbor;
                    self.grid[neighbor_index.y as usize][neighbor_index.x as usize] = cur_neighbor;
                    cells_to_check.push_back(cur_neighbor);
                }
            }
        }
    }

    pub fn create_flowfield(&mut self) {
        // Collect all neighbors' data before making mutable changes
        let neighbors_data: Vec<Vec<Vec<Cell>>> = self
            .grid
            .iter()
            .map(|cell_row| {
                cell_row
                    .iter()
                    .map(|cell| {
                        self.get_neighbor_cells(cell.grid_idx, GridDirection::all_directions())
                    })
                    .collect()
            })
            .collect();

        // Iterate over the grid and update cells' best_direction
        for (row_idx, cell_row) in self.grid.iter_mut().enumerate() {
            for (col_idx, cell) in cell_row.iter_mut().enumerate() {
                let cur_neighbors = &neighbors_data[row_idx][col_idx];
                let mut best_cost = cell.best_cost;

                for cur_neighbor in cur_neighbors.iter() {
                    if cur_neighbor.best_cost < best_cost {
                        best_cost = cur_neighbor.best_cost;

                        // Calculate the difference in grid indices
                        let delta_x = cur_neighbor.grid_idx.x - cell.grid_idx.x; // Columns (X-axis)
                        let delta_y = cur_neighbor.grid_idx.y - cell.grid_idx.y; // Rows (Y-axis)

                        // Create the direction vector
                        let direction_vector = IVec2::new(delta_x, delta_y);

                        // Get the corresponding GridDirection
                        let best_direction = GridDirection::from_vector2(direction_vector);

                        if let Some(best_direction) = best_direction {
                            cell.best_direction = best_direction;
                        }
                    }
                }
            }
        }
    }

    fn get_neighbor_cells(&self, node_index: IVec2, directions: Vec<GridDirection>) -> Vec<Cell> {
        let mut neighbor_cells = Vec::new();

        for direction in directions {
            if let Some(new_neighbor) = self.get_cell_at_relative_pos(node_index, direction) {
                neighbor_cells.push(new_neighbor);
            }
        }
        neighbor_cells
    }

    fn get_cell_at_relative_pos(
        &self,
        origin_pos: IVec2,
        direction: GridDirection,
    ) -> Option<Cell> {
        let relative_pos = direction.vector();
        let final_pos = origin_pos + relative_pos;

        if final_pos.x < 0
            || final_pos.x >= self.grid_size.x
            || final_pos.y < 0
            || final_pos.y >= self.grid_size.y
        {
            None
        } else {
            Some(self.grid[final_pos.y as usize][final_pos.x as usize]) // Note the swap of y and x
        }
    }

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

        let x = min(x, self.grid_size.x as usize - 1);
        let y = min(y, self.grid_size.y as usize - 1);

        self.grid[y][x] // Swap x and y
    }
}
