use crate::components::*;
use crate::events::*;
use crate::{cell::*, grid::Grid, grid_direction::GridDirection, utils};

use bevy::utils::HashSet;
use bevy::{prelude::*, window::PrimaryWindow};
use std::collections::VecDeque;

pub struct FlowfieldPlugin;

impl Plugin for FlowfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (count, remove_flowfield))
            .add_observer(initialize_flowfield);
    }
}

fn count(q: Query<&FlowField>, q2: Query<&Destination>) {
    // println!("Destinations: {}", q2.iter().len());
    // println!("Flowfields: {}", q.iter().len());
    // println!(
    //     "Destinations: {}, Flowfields: {}",
    //     q2.iter().len(),
    //     q.iter().len()
    // );
}

#[derive(Component, Clone, Default, PartialEq)]
pub struct FlowField {
    pub cell_radius: f32,
    pub cell_diameter: f32,
    pub destination_cell: Cell,
    pub grid: Vec<Vec<Cell>>,
    pub size: IVec2,
    pub units: Vec<Entity>,
}

impl FlowField {
    pub fn new(cell_radius: f32, grid_size: IVec2, units: Vec<Entity>) -> Self {
        FlowField {
            cell_radius,
            cell_diameter: cell_radius * 2.,
            destination_cell: Cell::default(),
            grid: Vec::default(),
            size: grid_size,
            units,
        }
    }

    pub fn create_integration_field(&mut self, grid: ResMut<Grid>, destination_cell: Cell) {
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
    }

    pub fn get_cell_from_world_position(&self, world_pos: Vec3) -> Cell {
        let cell = utils::get_cell_from_world_position_helper(
            world_pos,
            self.size,
            self.cell_diameter,
            &self.grid,
        );

        return cell;
    }

    pub fn remove_unit(&mut self, unit: Entity, cmds: &mut Commands) {
        // Remove the unit from the units list
        self.units.retain(|&u| u != unit);

        cmds.entity(unit).remove::<Destination>();
    }
}

fn remove_flowfield(
    mut cmds: Commands,
    q_flowfield: Query<(Entity, &FlowField)>,
    // q_flowfield_entity: Query<Entity, With<FlowFieldEntity>>,
) {
    for (entity, flowfield) in q_flowfield.iter() {
        if flowfield.units.is_empty() {
            // if let Ok(flowfield_entity) = q_flowfield_entity.get(entity) {
            //     println!("Despawning Flowfield Entity");
            //     cmds.entity(flowfield_entity).despawn_recursive();
            // }

            println!("Despawning Flowfield");
            cmds.entity(entity).despawn_recursive();
        }
    }
}

fn initialize_flowfield(
    trigger: Trigger<InitializeFlowFieldEv>,
    mut cmds: Commands,
    mut grid: ResMut<Grid>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_cam: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    q_map_base: Query<&GlobalTransform, With<MapBase>>,
    q_unit_info: Query<(&Transform, &UnitSize)>,
    q_flowfield_entity: Query<&FlowFieldEntity>,
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

    let units = trigger.event().0.clone();
    if units.is_empty() {
        return;
    }

    let mut unit_positions = Vec::new();
    let mut flowfields_to_despawn = HashSet::new();
    for unit in &units {
        // despawn any existing flowfields associated with the current units
        if let Ok(flowfield) = q_flowfield_entity.get(*unit) {
            cmds.entity(*unit).remove::<FlowFieldEntity>();

            if flowfields_to_despawn.insert(flowfield.0.index()) {
                cmds.entity(flowfield.0).despawn_recursive();
            }
        }

        if let Ok((transform, size)) = q_unit_info.get(*unit) {
            unit_positions.push((transform.translation, size.0));
        }
    }

    // reset the cell costs of the units in this new flowfield
    grid.reset_costs(unit_positions);

    let world_mouse_pos = utils::get_world_pos(map_base, cam.1, cam.0, mouse_pos);
    let destination_cell = grid.get_cell_from_world_position(world_mouse_pos);

    let mut flowfield = FlowField::new(grid.cell_radius, grid.size, units.clone());
    flowfield.create_integration_field(grid, destination_cell);
    flowfield.create_flowfield();

    cmds.trigger(SetActiveFlowfieldEv(Some(flowfield.clone())));

    // Spawn the new flowfield entity
    let flowfield_entity = cmds.spawn(flowfield).id();

    // Insert a FlowFieldEntity component that points to the new flowfield
    for &unit in units.iter() {
        cmds.entity(unit).insert(FlowFieldEntity(flowfield_entity));
    }

    // println!("End Initialize Flowfield");
}
