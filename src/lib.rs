use bevy::color::palettes::css::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::plugin::RapierContext;
use bevy_rapier3d::prelude::{Collider, ExternalImpulse, QueryFilter};

pub mod components;
mod resources;

use components::*;
use resources::*;

const MAP_SIZE: f32 = 800.0;
const MAP_GRID_SIZE: u32 = 60;
const MAP_CELL_SIZE: f32 = MAP_SIZE / MAP_GRID_SIZE as f32;
const COLOR_PATH_FINDING: Srgba = YELLOW;
const COLOR_PATH: Srgba = LIGHT_STEEL_BLUE;
const COLOR_OCCUPIED_CELL: Srgba = RED;
const COLOR_GRID: Srgba = GRAY;

pub struct BevyRtsPathFindingPlugin;

impl Plugin for BevyRtsPathFindingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Grid>()
            .init_resource::<TargetCell>()
            .init_resource::<SetGridOccupantsOnce>()
            .init_resource::<DelayedRunTimer>()
            .add_systems(
                Update,
                (
                    draw_grid,
                    set_grid_occupants,
                    update_grid_occupants,
                    move_units_along_path,
                    draw_line_to_destination,
                    set_target_cell,
                    set_destination_path,
                ),
            );
    }
}

// runs once at Update
fn set_grid_occupants(
    mut grid: ResMut<Grid>,
    rapier_context: Res<RapierContext>,
    mut track: ResMut<SetGridOccupantsOnce>,
    time: Res<Time>,
    mut timer: ResMut<DelayedRunTimer>,
) {
    // Wait until the delay timer finishes, then run the system
    if !track.0 && timer.0.tick(time.delta()).finished() {
        let half_size = MAP_CELL_SIZE / 2.0;

        let mut occupied_cells = Vec::new();

        // Loop through each cell in the grid
        for (row_idx, cell_row) in grid.cells.iter_mut().enumerate() {
            for (column_idx, cell) in cell_row.iter_mut().enumerate() {
                // Define the cell's bounding box as a Rapier cuboid (half extents of the cell)
                let cell_center = Vec3::new(cell.position.x, 0.0, cell.position.y);
                let cell_shape = Collider::cuboid(half_size, 1.0, half_size);

                if let Some(_) = rapier_context.intersection_with_shape(
                    cell_center,
                    Quat::IDENTITY,
                    &cell_shape,
                    QueryFilter::default().exclude_sensors(),
                ) {
                    occupied_cells.push((row_idx, column_idx));
                    cell.occupied = true;
                }
            }
        }

        grid.occupied_cells = occupied_cells;
        track.0 = true;
    }
}

fn update_grid_occupants(
    mut grid: ResMut<Grid>,
    rapier_context: Res<RapierContext>,
    collider_q: Query<&Transform, With<Collider>>,
) {
    let half_size = MAP_CELL_SIZE / 2.0;

    // Create a new vector to hold indices of cells that are still occupied
    let mut still_occupied_cells = Vec::new();

    // Clone the occupied_cells list to iterate over to avoid borrowing issues
    let occupied_cells_snapshot = grid.occupied_cells.clone();

    // First pass: Check currently occupied cells and mark them as unoccupied if necessary
    for (row, column) in occupied_cells_snapshot.iter() {
        let cell = grid.cells[*row][*column];
        let cell_center = Vec3::new(cell.position.x, 0.0, cell.position.y);
        let cell_shape = Collider::cuboid(half_size, 1.0, half_size);

        // If cell is no longer occupied, mark it as unoccupied
        if rapier_context
            .intersection_with_shape(
                cell_center,
                Quat::IDENTITY,
                &cell_shape,
                QueryFilter::default().exclude_sensors(),
            )
            .is_none()
        {
            grid.cells[*row][*column].occupied = false;
        } else {
            // If still occupied, add it to the new list
            still_occupied_cells.push((*row, *column));
        }
    }

    // Second pass: Check each collider to detect new occupied cells
    for transform in collider_q.iter() {
        let collider_position = transform.translation;

        // Calculate the grid cell row and column based on collider's position
        let normalized_x = (collider_position.x + MAP_SIZE / 2.0) / MAP_CELL_SIZE;
        let normalized_y = (collider_position.z + MAP_SIZE / 2.0) / MAP_CELL_SIZE;
        let row = normalized_y.floor() as usize;
        let column = normalized_x.floor() as usize;

        // Ensure the calculated row and column are within bounds
        if row < grid.cells.len() && column < grid.cells[row].len() {
            // Access the cell and check if it needs to be marked as occupied
            let cell = &grid.cells[row][column];
            let cell_center = Vec3::new(cell.position.x, 0.0, cell.position.y);
            let cell_shape = Collider::cuboid(half_size, 1.0, half_size);

            if let Some(_) = rapier_context.intersection_with_shape(
                cell_center,
                Quat::IDENTITY,
                &cell_shape,
                QueryFilter::default().exclude_sensors(),
            ) {
                // Mark the cell as occupied if it's not already
                if !grid.cells[row][column].occupied {
                    grid.cells[row][column].occupied = true;
                    still_occupied_cells.push((row, column));
                }
            }
        }
    }

    // Update the grid's occupied cells list
    grid.occupied_cells = still_occupied_cells;
}

fn draw_grid(
    mut gizmos: Gizmos,
    mut unit_q: Query<(&Transform, &Selected), With<Selected>>,
    target_cell: Res<TargetCell>,
    grid: Res<Grid>,
) {
    // draw grid
    gizmos.grid(
        Vec3::ZERO,
        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        UVec2::new(MAP_GRID_SIZE, MAP_GRID_SIZE),
        Vec2::new(MAP_CELL_SIZE, MAP_CELL_SIZE),
        COLOR_GRID,
    );

    // highlight unit paths
    for (unit_trans, selected) in unit_q.iter_mut() {
        if !selected.0 {
            continue;
        }
        if let (Some(goal_row), Some(goal_column)) = (target_cell.row, target_cell.column) {
            // Get the unit's current cell
            let (start_row, start_column) = get_unit_cell_row_and_column(&unit_trans);

            // Compute the path, ensuring only non-occupied cells are included
            if let Some(path) = find_path(&grid, (start_row, start_column), (goal_row, goal_column))
            {
                // Highlight the path
                for &(row, column) in &path {
                    // Draw a rectangle for each cell in the path
                    let cell = grid.cells[row as usize][column as usize];
                    let position = Vec3::new(cell.position.x, 0.1, cell.position.y);
                    let rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
                    let size = Vec2::splat(MAP_CELL_SIZE);
                    let color = COLOR_PATH;

                    gizmos.rect(position, rotation, size, color);
                }
            }
        }
    }

    // highlight occupied cells
    for (row, column) in &grid.occupied_cells {
        let cell = grid.cells[*row][*column];
        let position = Vec3::new(cell.position.x, 0.1, cell.position.y);
        let rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        let size = Vec2::splat(MAP_CELL_SIZE);
        gizmos.rect(position, rotation, size, COLOR_OCCUPIED_CELL);
    }
}

fn draw_line_to_destination(
    unit_q: Query<(&Destination, &Transform), With<Unit>>,
    mut gizmos: Gizmos,
) {
    for (destination, unit_trans) in unit_q.iter() {
        if let Some(_) = destination.endpoint {
            let mut current = unit_trans.translation;

            for cell in destination.waypoints.iter() {
                let next = Vec3::new(cell.position.x, 0.1, cell.position.y);
                gizmos.line(current, next, COLOR_PATH_FINDING);
                current = next;
            }
        }
    }
}

fn move_units_along_path(
    time: Res<Time>,
    mut unit_q: Query<(
        &mut Transform,
        &mut Destination,
        &Speed,
        &mut ExternalImpulse,
    )>,
) {
    for (mut unit_trans, mut destination, speed, mut ext_impulse) in unit_q.iter_mut() {
        // Check if we've reached the end of the path
        if destination.waypoints.len() == 0 {
            destination.endpoint = None;
            *destination = Destination::default();
            continue;
        }

        // Get the current waypoint
        let cell = &destination.waypoints[0];
        let target_pos = Vec3::new(
            cell.position.x,
            unit_trans.translation.y, // Keep current y to avoid vertical movement
            cell.position.y,
        );

        // Calculate the direction and distance to the target position
        let direction = target_pos - unit_trans.translation;
        let distance_sq = direction.length_squared();

        let threshold = 5.0;
        if distance_sq < threshold {
            destination.waypoints.remove(0); // reach waypoint, remove it
        } else {
            // Move towards the waypoint
            let direction_normalized = Vec3::new(direction.x, 0.0, direction.z).normalize();
            rotate_towards(&mut unit_trans, direction_normalized);
            ext_impulse.impulse += direction_normalized * speed.0 * time.delta_seconds();
        }
    }
}

fn set_destination_path(
    grid: Res<Grid>,
    mut unit_q: Query<(&Transform, &Selected, &mut Destination), With<Selected>>,
    target_cell: Res<TargetCell>,
    input: Res<ButtonInput<MouseButton>>,
) {
    for (transform, selected, mut destination) in unit_q.iter_mut() {
        // println!("MADE IT");
        if !selected.0 {
            continue;
        }

        if let (Some(goal_row), Some(goal_column)) = (target_cell.row, target_cell.column) {
            // Get the unit's current cell
            let (start_row, start_column) = get_unit_cell_row_and_column(&transform);

            // Compute the path, ensuring only non-occupied cells are included
            if let Some(path) = find_path(&grid, (start_row, start_column), (goal_row, goal_column))
            {
                let mut waypoints: Vec<Cell> = Vec::new();

                // Highlight the path
                for &(row, column) in &path {
                    let cell = grid.cells[row as usize][column as usize];
                    waypoints.push(cell.clone());
                }

                // If a left mouse click is detected, assign the computed path
                if input.just_pressed(MouseButton::Left) {
                    destination.waypoints = waypoints;
                }
            }
        }
    }
}

fn set_target_cell(
    mut target_cell: ResMut<TargetCell>,
    cam_q: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    map_base_q: Query<&GlobalTransform, With<MapBase>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
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

    let coords = get_world_coords(map_base, &cam.1, &cam.0, viewport_cursor);

    // Adjust mouse coordinates to the grid's coordinate system
    let grid_origin = -MAP_SIZE / 2.0;
    let adjusted_x = coords.x - grid_origin; // Shift origin to (0, 0)
    let adjusted_z = coords.z - grid_origin;

    // Calculate the column and row indices
    let column = (adjusted_x / MAP_CELL_SIZE).floor() as u32;
    let row = (adjusted_z / MAP_CELL_SIZE).floor() as u32;

    // Check if indices are within the grid bounds
    if column < MAP_GRID_SIZE && row < MAP_GRID_SIZE {
        // println!("Mouse is over cell at row {}, column {}, position {:?}", cell.row, cell.column, cell.position);
        target_cell.row = Some(row);
        target_cell.column = Some(column);
    } else {
        target_cell.row = None;
        target_cell.column = None;
    }
}

pub fn find_path(grid: &Grid, start: (u32, u32), goal: (u32, u32)) -> Option<Vec<(u32, u32)>> {
    pathfinding::prelude::astar(
        &start,
        |&(row, column)| successors(&grid, row, column),
        |&(row, column)| heuristic(row, column, goal.0, goal.1),
        |&pos| pos == goal,
    )
    .map(|(path, _cost)| path)
}

pub fn successors(grid: &Grid, row: u32, column: u32) -> Vec<((u32, u32), usize)> {
    let mut neighbors = Vec::new();
    let directions = [
        (-1, 0),  // Up
        (1, 0),   // Down
        (0, -1),  // Left
        (0, 1),   // Right
        (-1, -1), // Up-Left (diagonal)
        (-1, 1),  // Up-Right (diagonal)
        (1, -1),  // Down-Left (diagonal)
        (1, 1),   // Down-Right (diagonal)
    ];

    for (d_row, d_col) in directions.iter() {
        let new_row = row as i32 + d_row;
        let new_col = column as i32 + d_col;

        if new_row >= 0
            && new_row < MAP_GRID_SIZE as i32
            && new_col >= 0
            && new_col < MAP_GRID_SIZE as i32
        {
            let neighbor_cell = grid.cells[new_row as usize][new_col as usize];

            // Only add the neighbor if it is not occupied
            if !neighbor_cell.occupied {
                neighbors.push(((new_row as u32, new_col as u32), 1)); // Cost is 1 per move
            }
        }
    }

    neighbors
}

pub fn heuristic(row: u32, column: u32, goal_row: u32, goal_column: u32) -> usize {
    let dx = (column as i32 - goal_column as i32).abs();
    let dy = (row as i32 - goal_row as i32).abs();
    (dx + dy) as usize // Manhattan distance
}

pub fn get_unit_cell_row_and_column(transform: &Transform) -> (u32, u32) {
    // Get the unit's current cell
    let unit_pos = transform.translation;
    let grid_origin = -MAP_SIZE / 2.0;
    let adjusted_x = unit_pos.x - grid_origin;
    let adjusted_z = unit_pos.z - grid_origin;

    let column = (adjusted_x / MAP_CELL_SIZE).floor() as u32;
    let row = (adjusted_z / MAP_CELL_SIZE).floor() as u32;

    (row, column)
}

pub fn rotate_towards(trans: &mut Transform, direction: Vec3) {
    let target_yaw = direction.x.atan2(direction.z);
    trans.rotation = Quat::from_rotation_y(target_yaw);
}

pub fn get_world_coords(
    map_base_trans: &GlobalTransform,
    cam_trans: &GlobalTransform,
    cam: &Camera,
    cursor_pos: Vec2,
) -> Vec3 {
    let plane_origin = map_base_trans.translation();
    let plane = InfinitePlane3d::new(map_base_trans.up());
    let ray = cam.viewport_to_world(cam_trans, cursor_pos).unwrap();
    let distance = ray.intersect_plane(plane_origin, plane).unwrap();
    return ray.get_point(distance);
}
