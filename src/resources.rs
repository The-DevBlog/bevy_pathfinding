use bevy::prelude::*;

use crate::*;

#[derive(Resource, Default)]
pub struct SetGridOccupantsOnce(pub bool);

#[derive(Resource)]
pub struct DelayedRunTimer(pub Timer);

impl Default for DelayedRunTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.5, TimerMode::Once)) // 0.5 seconds delay
    }
}

#[derive(Resource, Default)]
pub struct TargetCell {
    pub row: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Resource, Debug)]
pub struct Grid {
    // pub cell_size: u32,
    pub cells: Vec<Vec<Cell>>, // 2D matrix of cells
    // pub grid_size: u32,
    pub occupied_cells: Vec<(usize, usize)>, // Store occupied cells as (row, column)
                                             // pub size: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub row: usize,
    pub column: usize,
    pub position: Vec2,
    pub occupied: bool,
}

impl Grid {
    fn new() -> Self {
        let mut cells = Vec::new();

        for row in 0..MAP_GRID_SIZE {
            let mut row_cells = Vec::new();
            for column in 0..MAP_GRID_SIZE {
                // Calculate the center position of each cell
                let position = Vec2::new(
                    -MAP_SIZE / 2.0 + column as f32 * MAP_CELL_SIZE + MAP_CELL_SIZE / 2.0,
                    -MAP_SIZE / 2.0 + row as f32 * MAP_CELL_SIZE + MAP_CELL_SIZE / 2.0,
                );

                row_cells.push(Cell {
                    position,
                    row: row as usize,
                    column: column as usize,
                    occupied: false,
                });
            }
            cells.push(row_cells);
        }

        Grid {
            cells,
            occupied_cells: Vec::default(),
        }
    }
}

impl Default for Grid {
    fn default() -> Self {
        let mut cells = Vec::new();

        for row in 0..MAP_GRID_SIZE {
            let mut row_cells = Vec::new();
            for column in 0..MAP_GRID_SIZE {
                // Calculate the center position of each cell
                let position = Vec2::new(
                    -MAP_SIZE / 2.0 + column as f32 * MAP_CELL_SIZE + MAP_CELL_SIZE / 2.0,
                    -MAP_SIZE / 2.0 + row as f32 * MAP_CELL_SIZE + MAP_CELL_SIZE / 2.0,
                );

                row_cells.push(Cell {
                    position,
                    row: row as usize,
                    column: column as usize,
                    occupied: false,
                });
            }
            cells.push(row_cells);
        }

        Grid {
            cells,
            occupied_cells: Vec::default(),
        }
    }
}
