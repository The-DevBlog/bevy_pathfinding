use bevy::{color::palettes::tailwind::CYAN_100, prelude::*};
use rand::Rng;
use std::collections::VecDeque;

pub const GRID_WIDTH: f32 = GRID_CELLS_X as f32 * CELL_SIZE as f32;
pub const GRID_DEPTH: f32 = GRID_CELLS_Z as f32 * CELL_SIZE as f32;
const GRID_CELLS_X: usize = 40;
const GRID_CELLS_Z: usize = 40;
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
        app.init_resource::<Grid>()
            .init_resource::<TargetCell>()
            .add_systems(
                Update,
                (
                    calculate_flow_field,
                    calculate_flow_vectors,
                    draw_flow_field,
                ),
            );
    }
}

#[derive(Resource)]
struct TargetCell {
    x: usize,
    z: usize,
}

impl Default for TargetCell {
    fn default() -> Self {
        let target = TargetCell {
            x: GRID_CELLS_X - 1,
            z: GRID_CELLS_Z - 1,
        };

        target
    }
}

#[derive(Clone)]
struct GridCell {
    position: Vec3,
    cost: f32,
    flow_vector: Vec3,
    is_obstacle: bool,
}

#[derive(Resource)]
struct Grid {
    cells: Vec<Vec<GridCell>>,
}

impl Default for Grid {
    fn default() -> Self {
        let mut grid = vec![
            vec![
                GridCell {
                    position: Vec3::ZERO,
                    cost: f32::INFINITY,
                    flow_vector: Vec3::ZERO,
                    is_obstacle: false,
                };
                GRID_CELLS_Z
            ];
            GRID_CELLS_X
        ];

        // for x in 0..GRID_WIDTH {
        //     for z in 0..GRID_DEPTH {
        //         grid[x][z].position = Vec3::new(x as f32 * CELL_SIZE, 0.0, z as f32 * CELL_SIZE);
        //     }
        // }

        let mut rng = rand::thread_rng();
        for x in 0..GRID_CELLS_X {
            for z in 0..GRID_CELLS_Z {
                grid[x][z].position = Vec3::new(x as f32 * CELL_SIZE, 0.0, z as f32 * CELL_SIZE);

                // Randomly set some cells as obstacles
                if rng.gen_bool(0.1) {
                    // 10% chance to be an obstacle
                    grid[x][z].is_obstacle = true;
                }
            }
        }

        let target = TargetCell::default();
        grid[target.x][target.z].cost = 0.0;

        Grid { cells: grid }
    }
}

fn calculate_flow_field(mut grid: ResMut<Grid>, target: Res<TargetCell>) {
    let mut queue = VecDeque::new();
    queue.push_back((target.x, target.z));

    while let Some((x, z)) = queue.pop_front() {
        let current_cost = grid.cells[x][z].cost;

        for (dx, dz) in &NEIGHBOR_OFFSETS {
            let nx = x as isize + dx;
            let nz = z as isize + dz;

            if nx >= 0 && nx < GRID_CELLS_X as isize && nz >= 0 && nz < GRID_CELLS_Z as isize {
                let nx = nx as usize;
                let nz = nz as usize;

                let neighbor = &mut grid.cells[nx][nz];

                if neighbor.is_obstacle {
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
    for x in 0..GRID_CELLS_X {
        for z in 0..GRID_CELLS_Z {
            if grid.cells[x][z].is_obstacle {
                continue;
            }

            let mut min_cost = grid.cells[x][z].cost;
            let mut min_direction = Vec3::ZERO;

            for (dx, dz) in &NEIGHBOR_OFFSETS {
                let nx = x as isize + dx;
                let nz = z as isize + dz;

                if nx >= 0 && nx < GRID_CELLS_X as isize && nz >= 0 && nz < GRID_CELLS_Z as isize {
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

fn draw_flow_field(grid: Res<Grid>, mut gizmos: Gizmos) {
    for x in 0..GRID_CELLS_X {
        for z in 0..GRID_CELLS_Z {
            let cell = &grid.cells[x][z];

            if cell.is_obstacle || cell.flow_vector == Vec3::ZERO {
                continue;
            }

            gizmos.line(
                cell.position,
                cell.position + cell.flow_vector * 4.0,
                CYAN_100,
            );
        }
    }
}

fn draw_grid(mut gizmos: Gizmos) {
    gizmos.grid()
}

// GAME LOGIC
// #[derive(Component)]
// struct Agent;

// fn spawn_agents(mut commands: Commands) {
//     let agent = ();
//     for _ in 0..10 {
//         commands
//             .spawn()
//             .insert(Agent)
//             .insert(Transform::from_translation(Vec3::new(
//                 0.0,
//                 GRID_HEIGHT as f32 * CELL_SIZE / 2.0,
//                 0.0,
//             )))
//             .insert(GlobalTransform::default());
//     }
// }
