use crate::{
    cell::*, grid::Grid, grid_direction::GridDirection, utils, ActiveDebugFlowfield, GameCamera,
    InitializeFlowFieldEv, MapBase, Selected, SetActiveFlowfieldEv,
};
use bevy::{prelude::*, window::PrimaryWindow};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};

// Global atomic counter
static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct FlowfieldPlugin;

impl Plugin for FlowfieldPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Update, set_active_dbg_flowfield)
        app.add_observer(initialize_flowfield);
    }
}

#[derive(Component, Clone, Default, PartialEq)]
pub struct FlowField {
    pub cell_radius: f32,
    pub cell_diameter: f32,
    pub destination_cell: Cell,
    pub grid: Vec<Vec<Cell>>,
    pub idx: usize,
    pub size: IVec2,
    pub units: Vec<Entity>,
}

impl FlowField {
    pub fn new(cell_radius: f32, grid_size: IVec2, selected_units: Vec<Entity>) -> Self {
        FlowField {
            cell_radius,
            cell_diameter: cell_radius * 2.,
            destination_cell: Cell::default(),
            grid: Vec::default(),
            idx: create_idx(),
            size: grid_size,
            units: selected_units,
        }
    }

    pub fn create_integration_field(&mut self, grid: Res<Grid>, destination_cell: Cell) {
        // println!("Start Integration Field Create");

        self.grid = grid.grid.clone();

        // Initialize the destination cell in the grid
        let dest_idx = destination_cell.idx;
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
                    && neighbor_idx.x < self.size.x
                    && neighbor_idx.y >= 0
                    && neighbor_idx.y < self.size.y
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

        // println!("End Integration Field Create");
    }

    pub fn create_flowfield(&mut self) {
        // println!("Start Flowfield Create");

        let grid_size_y = self.size.y as usize;
        let grid_size_x = self.size.x as usize;

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
        // println!("End Flowfield Create");
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
}

fn initialize_flowfield(
    _trigger: Trigger<InitializeFlowFieldEv>,
    mut cmds: Commands,
    // mut active_dbg_flowfield: ResMut<ActiveDebugFlowfield>,
    grid: Res<Grid>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_cam: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    q_map_base: Query<&GlobalTransform, With<MapBase>>,
    q_selected: Query<Entity, With<Selected>>,
    // q_flowfield: Query<Entity, With<FlowField>>,
) {
    // println!("Start Initialize Flowfield");

    let Some(mouse_pos) = q_windows.single().cursor_position() else {
        return;
    };

    let Ok(cam) = q_cam.get_single() else {
        return;
    };

    let Ok(map_base) = q_map_base.get_single() else {
        return;
    };

    let selected_units: Vec<Entity> = q_selected.iter().collect();
    let world_mouse_pos = utils::get_world_pos(map_base, cam.1, cam.0, mouse_pos);
    let destination_cell = grid.get_cell_from_world_position(world_mouse_pos);

    let mut flowfield = FlowField::new(grid.cell_radius, grid.size, selected_units);
    flowfield.create_integration_field(grid, destination_cell);
    flowfield.create_flowfield();

    // active_dbg_flowfield.0 = Some(flowfield.clone());

    cmds.trigger(SetActiveFlowfieldEv(Some(flowfield.clone())));
    cmds.spawn(flowfield);

    // println!("End Initialize Flowfield");
}

fn create_idx() -> usize {
    COUNTER.fetch_add(1, Ordering::SeqCst)
}
