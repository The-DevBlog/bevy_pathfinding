use std::{cell, cmp::min, collections::VecDeque};

use bevy::prelude::*;
use bevy_rapier3d::{plugin::RapierContext, prelude::*};

use crate::{cell::*, grid_direction::GridDirection};

#[derive(Component, Clone, Default)]
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

        self.grid = (0..self.grid_size.x)
            .map(|x| {
                (0..self.grid_size.y)
                    .map(|y| {
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

    // pub fn create_integration_field(&mut self, destination_cell: Cell) {
    //     let mut tmp_destination_cell = destination_cell.clone();
    //     tmp_destination_cell.cost = 0;
    //     tmp_destination_cell.best_cost = 0;
    //     self.destination_cell = Some(tmp_destination_cell);

    //     let mut cells_to_check = VecDeque::new();
    //     let destination_cell = self.destination_cell.unwrap().clone();
    //     cells_to_check.push_front(destination_cell);

    //     while cells_to_check.len() > 0 {
    //         let cur_cell = cells_to_check.pop_front();

    //         if let Some(cur_cell) = cur_cell {
    //             let mut cur_neighbors = self
    //                 .get_neighbor_cells(cur_cell.grid_idx, GridDirection::cardinal_directions());

    //             for cur_neighbor in cur_neighbors.iter_mut() {
    //                 if cur_neighbor.cost == u8::MAX {
    //                     continue;
    //                 }

    //                 if cur_neighbor.cost as u16 + cur_cell.best_cost < cur_neighbor.best_cost {
    //                     cur_neighbor.best_cost = cur_neighbor.cost as u16 + cur_cell.best_cost;
    //                     cells_to_check.push_front(cur_neighbor.clone());
    //                 }
    //             }
    //         }
    //     }
    // }

    // fn get_neighbor_cells(&self, node_idx: IVec2, directions: Vec<GridDirection>) -> Vec<Cell> {
    //     let mut neighbor_cells = Vec::new();

    //     for cur_direction in directions.iter() {
    //         let new_neighbor = self.get_cell_at_relative_position(node_idx, cur_direction.vector());

    //         if let Some(new_neighbor) = new_neighbor {
    //             neighbor_cells.push(new_neighbor);
    //         }
    //     }

    //     return neighbor_cells;
    // }

    // fn get_cell_at_relative_position(
    //     &self,
    //     origin_position: IVec2,
    //     relative_position: IVec2,
    // ) -> Option<Cell> {
    //     let final_position = origin_position + relative_position;

    //     if final_position.x < 0
    //         || final_position.x >= self.grid_size.x
    //         || final_position.y < 0
    //         || final_position.y >= self.grid_size.y
    //     {
    //         return None;
    //     }

    //     return Some(self.grid[final_position.x as usize][final_position.y as usize]);
    // }

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
                    self.grid[neighbor_index.x as usize][neighbor_index.y as usize] = cur_neighbor;
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

                        let best_direction =
                            GridDirection::from_vector2(cur_neighbor.grid_idx - cell.grid_idx);

                        if let Some(best_direction) = best_direction {
                            cell.best_direction = best_direction;
                        }
                    }
                }
            }
        }
    }

    // pub fn create_flowfield(&self) {
    //     for cell_row in &self.grid {
    //         for cell in cell_row.iter() {
    //             let mut cur_neighbors =
    //                 self.get_neighbor_cells(cell.grid_idx, GridDirection::all_directions());

    //             let mut best_cost = cell.best_cost;

    //             for cur_neighbor in cur_neighbors.iter() {
    //                 if cur_neighbor.best_cost < best_cost {
    //                     best_cost = cur_neighbor.best_cost;
    //                     let best_direction =
    //                         GridDirection::from_vector2(cur_neighbor.grid_idx - cell.grid_idx);

    //                     if let Some(best_direction) = best_direction {
    //                         cell.best_direction = best_direction;
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

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
            Some(self.grid[final_pos.x as usize][final_pos.y as usize])
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

        // Return the cell at the calculated indices
        // why am I returning [y][x] instead of [x][y]?
        // The columns run across the Y axis (Z axis in 3D, but I am treating the plane as a 2D space)
        // The rows run across the X axis
        self.grid[y][x]
    }
}

// // Phase 3
// fn detect_colliders(
//     _trigger: Trigger<DetectCollidersEv>,
//     mut cmds: Commands,
//     mut flowfield_q: Query<&mut FlowField>,
//     rapier_context: Res<RapierContext>,
//     grid: Res<Grid>,
//     selected_q: Query<Entity, With<Selected>>,
// ) {
//     // println!("PHASE 3: Detect Colliders");
//     let selected_entities: HashSet<Entity> = selected_q.iter().collect();

//     for mut flowfield in flowfield_q.iter_mut() {
//         for x in 0..grid.rows {
//             for z in 0..grid.columns {
//                 let cell = &mut flowfield.cells[x][z];
//                 cell.occupied = false; // Reset obstacle status

//                 let cell_size = grid.cell_size / 2.0;
//                 let cell_shape = Collider::cuboid(cell_size, cell_size, cell_size);
//                 let mut cell_occupied = false;

//                 // Capture selected_entities by reference for use in the closure
//                 let selected_entities = &selected_entities;

//                 rapier_context.intersections_with_shape(
//                     cell.world_position,
//                     Quat::IDENTITY, // No rotation
//                     &cell_shape,
//                     QueryFilter::default().exclude_sensors(),
//                     |collider_entity| {
//                         if !selected_entities.contains(&collider_entity) {
//                             // Collider is overlapping the cell and is not a selected unit
//                             cell_occupied = true;
//                             false
//                         } else {
//                             // Collider is a selected unit, ignore it
//                             true
//                         }
//                     },
//                 );

//                 cell.occupied = cell_occupied;
//             }
//         }
//     }

//     // println!("PHASE 3: Detect Colliders Done");
//     cmds.trigger(CalculateFlowFieldEv);
// }
