use bevy::prelude::*;

use crate::*;

#[derive(Component)]
pub struct MapBase;

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct Grid {
    pub cells: Vec<Vec<GridCell>>,
    pub cell_rows: usize,
    pub cell_columns: usize,
    pub colors: GridColors,
    pub width: f32,
    pub depth: f32,
}

impl Grid {
    pub fn new(cell_rows: usize, cell_columns: usize, width: f32, depth: f32) -> Self {
        let mut grid = vec![
            vec![
                GridCell {
                    position: Vec3::ZERO,
                    cost: f32::INFINITY,
                    flow_vector: Vec3::ZERO,
                    occupied: false,
                };
                cell_rows
            ];
            cell_columns
        ];

        // Calculate the offset to center the grid at (0, 0, 0)
        let grid_width = cell_rows as f32 * CELL_SIZE;
        let grid_depth = cell_columns as f32 * CELL_SIZE;
        let half_grid_width = grid_width / 2.0;
        let half_grid_depth = grid_depth / 2.0;

        for x in 0..cell_rows {
            for z in 0..cell_columns {
                let world_x = x as f32 * CELL_SIZE - half_grid_width + CELL_SIZE / 2.0;
                let world_z = z as f32 * CELL_SIZE - half_grid_depth + CELL_SIZE / 2.0;

                grid[x][z].position = Vec3::new(world_x, 0.0, world_z);
            }
        }

        let target = TargetCell::new(cell_rows, cell_columns);
        grid[target.row][target.column].cost = 0.0;

        Grid {
            cells: grid,
            colors: GridColors::default(),
            cell_rows,
            cell_columns,
            width,
            depth,
        }
    }
}

#[derive(Clone)]
pub struct GridCell {
    pub position: Vec3,
    pub cost: f32,
    pub flow_vector: Vec3,
    pub occupied: bool,
}

pub struct GridColors {
    pub grid: Srgba,
    pub arrows: Srgba,
    pub occupied_cells: Srgba,
}

impl Default for GridColors {
    fn default() -> Self {
        Self {
            grid: COLOR_GRID,
            arrows: COLOR_ARROWS,
            occupied_cells: COLOR_OCCUPIED_CELL,
        }
    }
}
