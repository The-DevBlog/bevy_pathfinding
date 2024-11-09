use bevy::prelude::*;
use bevy_rapier3d::prelude::ExternalImpulse;
use components::*;

mod components;

pub struct BevyRtsPathFindingPlugin;

impl Plugin for BevyRtsPathFindingPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Resource, Debug, Clone)]
pub struct FlowField {
    pub cost_field: Vec<Vec<usize>>,        // Movement cost for each cell
    pub integration_field: Vec<Vec<usize>>, // Cumulative cost to reach each cell from the target
    pub direction_field: Vec<Vec<Vec2>>,    // Direction vector for each cell
    pub target_cell: (u32, u32),            // Target cell position for the flow field
}

impl FlowField {
    pub fn new(grid_width: usize, grid_height: usize, target_cell: (u32, u32)) -> Self {
        FlowField {
            cost_field: vec![vec![1; grid_width]; grid_height],
            integration_field: vec![vec![usize::MAX; grid_width]; grid_height],
            direction_field: vec![vec![Vec2::ZERO; grid_width]; grid_height],
            target_cell,
        }
    }
}

fn build_cost_field(flow_field: &mut FlowField, grid: &Grid) {
    for (row, cells) in grid.cells.iter().enumerate() {
        for (col, cell) in cells.iter().enumerate() {
            flow_field.cost_field[row][col] = if cell.occupied { usize::MAX } else { 1 };
        }
    }
}

use std::collections::VecDeque;

fn build_integration_field(flow_field: &mut FlowField) {
    let mut queue = VecDeque::new();
    let (target_row, target_col) = flow_field.target_cell;

    // Set the target cell's integration cost to zero
    flow_field.integration_field[target_row as usize][target_col as usize] = 0;
    queue.push_back((target_row, target_col));

    while let Some((row, col)) = queue.pop_front() {
        let current_cost = flow_field.integration_field[row as usize][col as usize];

        for (d_row, d_col) in [(-1, 0), (1, 0), (0, -1), (0, 1)].iter() {
            let new_row = row as i32 + d_row;
            let new_col = col as i32 + d_col;

            if new_row >= 0
                && new_row < flow_field.integration_field.len() as i32
                && new_col >= 0
                && new_col < flow_field.integration_field[0].len() as i32
            {
                let new_pos = (new_row as usize, new_col as usize);
                let cost = flow_field.cost_field[new_pos.0][new_pos.1];

                if cost != usize::MAX {
                    let new_cost = current_cost + cost;
                    if new_cost < flow_field.integration_field[new_pos.0][new_pos.1] {
                        flow_field.integration_field[new_pos.0][new_pos.1] = new_cost;
                        queue.push_back((new_row as u32, new_col as u32));
                    }
                }
            }
        }
    }
}

fn build_direction_field(flow_field: &mut FlowField) {
    let directions = [
        (-1, 0, Vec2::new(0.0, -1.0)),
        (1, 0, Vec2::new(0.0, 1.0)),
        (0, -1, Vec2::new(-1.0, 0.0)),
        (0, 1, Vec2::new(1.0, 0.0)),
    ];

    for row in 0..flow_field.integration_field.len() {
        for col in 0..flow_field.integration_field[0].len() {
            let mut best_direction = Vec2::ZERO;
            let mut lowest_cost = usize::MAX;

            for (d_row, d_col, direction) in directions.iter() {
                let new_row = row as i32 + d_row;
                let new_col = col as i32 + d_col;

                if new_row >= 0
                    && new_row < flow_field.integration_field.len() as i32
                    && new_col >= 0
                    && new_col < flow_field.integration_field[0].len() as i32
                {
                    let cost = flow_field.integration_field[new_row as usize][new_col as usize];
                    if cost < lowest_cost {
                        lowest_cost = cost;
                        best_direction = *direction;
                    }
                }
            }

            flow_field.direction_field[row][col] = best_direction;
        }
    }
}

fn move_units_along_flow_field(
    time: Res<Time>,
    flow_field: Res<FlowField>,
    mut unit_q: Query<(&mut Transform, &Speed, &mut ExternalImpulse)>,
    grid_q: Query<&Grid>,
) {
    let grid = grid_q.single();

    for (mut transform, speed, mut ext_impulse) in unit_q.iter_mut() {
        let (row, col) = get_unit_cell_row_and_column(&grid, &transform);

        // Ensure we're within bounds
        if row < flow_field.direction_field.len() && col < flow_field.direction_field[0].len() {
            let direction = flow_field.direction_field[row][col];
            let movement =
                Vec3::new(direction.x, 0.0, direction.y) * speed.0 * time.delta_seconds();

            ext_impulse.impulse += movement;
            rotate_towards(&mut transform, movement.normalize_or_zero());
        }
    }
}

fn set_target_and_generate_flow_field(
    mut commands: Commands,
    grid_q: Query<&Grid>,
    target_cell: Res<TargetCell>,
) {
    let grid = grid_q.single();

    if let (Some(row), Some(col)) = (target_cell.row, target_cell.column) {
        let mut flow_field = FlowField::new(grid.width as usize, grid.height as usize, (row, col));
        build_cost_field(&mut flow_field, &grid);
        build_integration_field(&mut flow_field);
        build_direction_field(&mut flow_field);

        commands.insert_resource(flow_field);
    }
}

pub fn get_unit_cell_row_and_column(grid: &Grid, transform: &Transform) -> (u32, u32) {
    let unit_pos = transform.translation;
    let grid_origin_x = -grid.width / 2.0;
    let grid_origin_y = -grid.height / 2.0;
    let adjusted_x = unit_pos.x - grid_origin_x;
    let adjusted_z = unit_pos.z - grid_origin_y;

    let column = (adjusted_x / grid.cell_width).floor() as u32;
    let row = (adjusted_z / grid.cell_height).floor() as u32;

    (row, column)
}

pub fn rotate_towards(trans: &mut Transform, direction: Vec3) {
    let target_yaw = direction.x.atan2(direction.z);
    trans.rotation = Quat::from_rotation_y(target_yaw);
}
