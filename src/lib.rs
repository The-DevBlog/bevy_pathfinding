use bevy::{
    color::palettes::{css::*, tailwind::CYAN_100},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_rapier3d::{plugin::RapierContext, prelude::*};
use std::collections::VecDeque;

pub mod components;
pub mod events;
pub mod resources;

use components::*;
use events::*;
use resources::*;

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
        app.init_resource::<InitializeGrid>()
            .add_systems(
                Update,
                (
                    tick_grid_init_timer,
                    calculate_flow_field,
                    calculate_flow_vectors,
                    draw_flow_field,
                    draw_grid,
                ),
            )
            .observe(detect_colliders)
            .observe(set_target_cell);
    }
}

fn tick_grid_init_timer(
    mut cmds: Commands,
    mut grid_init: ResMut<InitializeGrid>,
    time: Res<Time>,
) {
    if grid_init.delay.just_finished() {
        cmds.trigger(DetectCollidersEv);
    } else {
        grid_init.delay.tick(time.delta());
    }
}

fn calculate_flow_field(mut grid: ResMut<Grid>, target: Res<TargetCell>) {
    // Reset costs
    for row in grid.cells.iter_mut() {
        for cell in row.iter_mut() {
            cell.cost = f32::INFINITY;
        }
    }

    // Set the cost of the target cell to zero
    grid.cells[target.row][target.column].cost = 0.0;

    let mut queue = VecDeque::new();
    queue.push_back((target.row, target.column));

    while let Some((x, z)) = queue.pop_front() {
        let current_cost = grid.cells[x][z].cost;

        for &(dx, dz) in &NEIGHBOR_OFFSETS {
            let nx = x as isize + dx;
            let nz = z as isize + dz;

            if nx >= 0 && nx < grid.cell_rows as isize && nz >= 0 && nz < grid.cell_columns as isize
            {
                let nx = nx as usize;
                let nz = nz as usize;

                let neighbor = &mut grid.cells[nx][nz];

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

fn calculate_flow_vectors(mut grid: ResMut<Grid>) {
    for x in 0..grid.cell_rows {
        for z in 0..grid.cell_columns {
            if grid.cells[x][z].occupied {
                continue;
            }

            let mut min_cost = grid.cells[x][z].cost;
            let mut min_direction = Vec3::ZERO;

            for (dx, dz) in &NEIGHBOR_OFFSETS {
                let nx = x as isize + dx;
                let nz = z as isize + dz;

                if nx >= 0
                    && nx < grid.cell_rows as isize
                    && nz >= 0
                    && nz < grid.cell_columns as isize
                {
                    let nx = nx as usize;
                    let nz = nz as usize;

                    let neighbor = &grid.cells[nx][nz];

                    if neighbor.cost < min_cost {
                        min_cost = neighbor.cost;
                        min_direction = (neighbor.position - grid.cells[x][z].position).normalize();
                    }
                }
            }

            grid.cells[x][z].flow_vector = min_direction;
        }
    }
}

fn detect_colliders(
    _trigger: Trigger<DetectCollidersEv>,
    mut grid: ResMut<Grid>,
    rapier_context: Res<RapierContext>,
    mut grid_init: ResMut<InitializeGrid>,
) {
    if grid_init.done && grid_init.delay.finished() {
        return;
    }

    for x in 0..grid.cell_rows {
        for z in 0..grid.cell_columns {
            let cell = &mut grid.cells[x][z];
            cell.occupied = false; // Reset obstacle status

            let cell_shape = Collider::cuboid(CELL_SIZE / 2.0, CELL_SIZE / 2.0, CELL_SIZE / 2.0);
            rapier_context.intersections_with_shape(
                cell.position,
                Quat::IDENTITY, // no rotation
                &cell_shape,
                QueryFilter::default().exclude_sensors(),
                |_collider_entity| {
                    // A collider is found overlapping the cell
                    cell.occupied = true;
                    false // Return false to stop after finding one collider
                },
            );
        }
    }

    grid_init.done = true;
}

fn draw_flow_field(grid: Res<Grid>, mut gizmos: Gizmos) {
    let arrow_len = CELL_SIZE * 0.75 / 2.0;

    for x in 0..grid.cell_rows {
        for z in 0..grid.cell_columns {
            let cell = &grid.cells[x][z];

            if cell.occupied || cell.flow_vector == Vec3::ZERO {
                // Draw an 'X' for each occupied cell
                let top_left = cell.position + Vec3::new(-arrow_len, 0.0, -arrow_len);
                let top_right = cell.position + Vec3::new(arrow_len, 0.0, -arrow_len);
                let bottom_left = cell.position + Vec3::new(-arrow_len, 0.0, arrow_len);
                let bottom_right = cell.position + Vec3::new(arrow_len, 0.0, arrow_len);

                gizmos.line(top_left, bottom_right, RED);
                gizmos.line(top_right, bottom_left, RED);

                continue;
            }

            // Normalize the flow vector
            let flow_direction = cell.flow_vector.normalize();

            // Calculate start and end points
            let start = cell.position - flow_direction * arrow_len;
            let end = cell.position + flow_direction * arrow_len;

            gizmos.arrow(start, end, COLOR_ARROWS);
        }
    }
}

fn draw_grid(mut gizmos: Gizmos, grid: Res<Grid>) {
    gizmos.grid(
        Vec3::ZERO,
        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        UVec2::new(grid.cell_rows as u32, grid.cell_columns as u32),
        Vec2::new(CELL_SIZE, CELL_SIZE),
        COLOR_GRID,
    );
}

fn set_target_cell(
    _trigger: Trigger<SetTargetCellEv>,
    grid: Res<Grid>,
    mut cmds: Commands,
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
    let grid_origin_x = -grid.width / 2.0;
    let grid_origin_z = -grid.depth / 2.0;
    let adjusted_x = coords.x - grid_origin_x; // Shift origin to (0, 0)
    let adjusted_z = coords.z - grid_origin_z;

    // Calculate the column and row indices
    let cell_width = grid.width / grid.cell_rows as f32;
    let cell_depth = grid.depth / grid.cell_columns as f32;
    let row = (adjusted_x / cell_width).floor() as u32;
    let column = (adjusted_z / cell_depth).floor() as u32;

    // Check if indices are within the grid bounds
    if row < grid.width as u32 && column < grid.depth as u32 {
        // println!("Mouse is over cell at row {}, column {}, position {:?}", cell.row, cell.column, cell.position);
        target_cell.row = row as usize;
        target_cell.column = column as usize;
        cmds.trigger(DetectCollidersEv);
    }
}

pub fn get_world_coords(
    map_base_trans: &GlobalTransform,
    cam_transform: &GlobalTransform,
    cam: &Camera,
    cursor_pos: Vec2,
) -> Vec3 {
    let plane_origin = map_base_trans.translation();
    let plane = InfinitePlane3d::new(map_base_trans.up());
    let ray = cam.viewport_to_world(cam_transform, cursor_pos).unwrap();
    let distance = ray.intersect_plane(plane_origin, plane).unwrap();
    return ray.get_point(distance);
}
