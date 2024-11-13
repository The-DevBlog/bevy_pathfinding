use bevy::color::palettes::{css::*, tailwind::*};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::{plugin::RapierContext, prelude::*};
use std::collections::HashSet;
use std::collections::VecDeque;

use crate::components::*;
use crate::events::*;
use crate::resources::*;

pub mod components;
pub mod debug;
pub mod events;
pub mod resources;
pub mod utils;

const COLOR_GRID: Srgba = GRAY;
const COLOR_ARROWS: Srgba = CYAN_100;
const COLOR_OCCUPIED_CELL: Srgba = RED;
const CELL_SIZE: f32 = 10.0;
const NEIGHBOR_OFFSETS: [(isize, isize); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (-1, 1),
    (1, -1),
    (-1, -1),
];

pub struct BevyRtsPathFindingPlugin;

impl Plugin for BevyRtsPathFindingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TargetCell>()
            .add_systems(Update, remove_flowfield)
            .observe(set_target_cell)
            .observe(set_flow_field)
            .observe(detect_colliders)
            .observe(calculate_flowfield)
            .observe(calculate_flowfield_vectors);
    }
}

// Phase 1
fn set_target_cell(
    _trigger: Trigger<SetTargetCellEv>,
    mut cmds: Commands,
    mut target_cell: ResMut<TargetCell>,
    cam_q: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    map_base_q: Query<&GlobalTransform, With<MapBase>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    grid: Res<Grid>,
) {
    println!("");
    println!("");
    println!("");
    println!("PHASE 1: Setting target cell");
    let map_base = match map_base_q.get_single() {
        Ok(value) => value,
        Err(_) => return,
    };

    let cam = match cam_q.get_single() {
        Ok(value) => value,
        Err(_) => return,
    };

    let Some(viewport_cursor) = window_q.single().cursor_position() else {
        return;
    };

    let coords = utils::get_world_coords(map_base, &cam.1, &cam.0, viewport_cursor);
    let cell = utils::get_cell(&grid, &coords);

    // Check if indices are within the grid bounds
    if cell.0 < grid.width as u32 && cell.1 < grid.depth as u32 {
        // println!("Mouse is over cell at row {}, column {}, position {:?}", cell.row, cell.column, cell.position);
        target_cell.0 = Some((cell.0 as usize, cell.1 as usize));
        cmds.trigger(SetFlowFieldEv);
    }
    println!("PHASE 1: Target cell set");
}

// Phase 2
fn set_flow_field(
    _trigger: Trigger<SetFlowFieldEv>,
    mut cmds: Commands,
    mut unit_q: Query<Entity, With<Selected>>,
    mut flowfield_q: Query<&mut FlowField>,
    target_cell: Res<TargetCell>,
    grid: Res<Grid>,
) {
    println!("PHASE 2: Setting flow field");

    if target_cell.0.is_none() {
        return;
    }

    let target_cell = target_cell.0.unwrap();
    let mut new_flowfield = FlowField::new(grid.rows, grid.columns, target_cell.0, target_cell.1);

    for unit_entity in unit_q.iter_mut() {
        // remove units index that may be currently subscribed to existing flowfield
        for mut flowfield in flowfield_q.iter_mut() {
            flowfield.entities.retain(|&e| e != unit_entity);
        }

        new_flowfield.entities.push(unit_entity);
    }

    cmds.spawn(new_flowfield);
    cmds.trigger(DetectCollidersEv);
    println!("PHASE 2: Flow field set");
}

// Phase 3
fn detect_colliders(
    _trigger: Trigger<DetectCollidersEv>,
    mut cmds: Commands,
    mut flowfield_q: Query<&mut FlowField>,
    rapier_context: Res<RapierContext>,
    grid: Res<Grid>,
    selected_q: Query<Entity, With<Selected>>,
) {
    println!("PHASE 3: Detect Colliders");

    let selected_entities: HashSet<Entity> = selected_q.iter().collect();

    for mut flowfield in flowfield_q.iter_mut() {
        for x in 0..grid.rows {
            for z in 0..grid.columns {
                let cell = &mut flowfield.cells[x][z];
                cell.occupied = false; // Reset obstacle status

                let cell_size = CELL_SIZE / 2.0;
                let cell_shape = Collider::cuboid(cell_size, cell_size, cell_size);
                let mut cell_occupied = false;

                // Capture selected_entities by reference for use in the closure
                let selected_entities = &selected_entities;

                rapier_context.intersections_with_shape(
                    cell.position,
                    Quat::IDENTITY, // No rotation
                    &cell_shape,
                    QueryFilter::default().exclude_sensors(),
                    |collider_entity| {
                        if !selected_entities.contains(&collider_entity) {
                            // Collider is overlapping the cell and is not a selected unit
                            cell_occupied = true;
                            false
                        } else {
                            // Collider is a selected unit, ignore it
                            true
                        }
                    },
                );

                cell.occupied = cell_occupied;
            }
        }
    }

    println!("PHASE 3: Detect Colliders Done");
    cmds.trigger(CalculateFlowFieldEv);
}

// Phase 4
fn calculate_flowfield(
    _trigger: Trigger<CalculateFlowFieldEv>,
    mut flowfield_q: Query<&mut FlowField>,
    mut cmds: Commands,
    grid: Res<Grid>,
) {
    for mut flowfield in flowfield_q.iter_mut() {
        println!("PHASE 4: Calcing flowfield");

        // Reset costs
        for row in flowfield.cells.iter_mut() {
            for cell in row.iter_mut() {
                cell.cost = f32::INFINITY;
            }
        }

        // Set the cost of the target cell to zero
        let target_cell = flowfield.destination.clone();
        flowfield.cells[target_cell.0][target_cell.1].cost = 0.0;

        let mut queue = VecDeque::new();
        queue.push_back((flowfield.destination.0, flowfield.destination.1));

        while let Some((x, z)) = queue.pop_front() {
            let current_cost = flowfield.cells[x][z].cost;

            for &(dx, dz) in &NEIGHBOR_OFFSETS {
                let nx = x as isize + dx;
                let nz = z as isize + dz;

                if nx >= 0 && nx < grid.rows as isize && nz >= 0 && nz < grid.columns as isize {
                    let nx = nx as usize;
                    let nz = nz as usize;

                    let neighbor = &mut flowfield.cells[nx][nz];

                    if neighbor.occupied {
                        continue;
                    }

                    let new_cost = current_cost + 1.0; // Assuming uniform cost

                    if new_cost < neighbor.cost {
                        neighbor.cost = new_cost;
                        queue.push_back((nx, nz));
                    }
                }
            }
        }
    }

    println!("PHASE 4: Done calcing flowfield");
    cmds.trigger(CalculateFlowVectorsEv);
}

// Phase 5
fn calculate_flowfield_vectors(
    _trigger: Trigger<CalculateFlowVectorsEv>,
    mut flowfield_q: Query<&mut FlowField>,
    grid: Res<Grid>,
) {
    for mut flowfield in flowfield_q.iter_mut() {
        println!("PHASE 5: calcing flowfield vectors");
        for x in 0..grid.rows {
            for z in 0..grid.columns {
                if flowfield.cells[x][z].occupied {
                    continue;
                }

                // Initialize min_cost to infinity
                let mut min_cost = f32::INFINITY;
                let mut min_direction = Vec3::ZERO;

                for &(dx, dz) in &NEIGHBOR_OFFSETS {
                    let nx = x as isize + dx;
                    let nz = z as isize + dz;

                    if nx >= 0 && nx < grid.rows as isize && nz >= 0 && nz < grid.columns as isize {
                        let nx = nx as usize;
                        let nz = nz as usize;

                        let neighbor = &flowfield.cells[nx][nz];

                        if neighbor.cost < min_cost {
                            min_cost = neighbor.cost;
                            min_direction =
                                (neighbor.position - flowfield.cells[x][z].position).normalize();
                        }
                    }
                }

                flowfield.cells[x][z].flow_vector = min_direction;
            }
        }
    }

    println!("PHASE 5: done flowfield vectors");
}

// remove any flow field that has no entities attached
fn remove_flowfield(mut cmds: Commands, flowfield_q: Query<(Entity, &FlowField)>) {
    for (flowfield_entity, flowfield) in flowfield_q.iter() {
        if flowfield.entities.len() == 0 {
            cmds.entity(flowfield_entity).despawn();
        }
    }
}
