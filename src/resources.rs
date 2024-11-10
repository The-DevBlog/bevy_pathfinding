use bevy::prelude::*;

use crate::*;

#[derive(Resource)]
pub struct Grid {
    pub rows: usize,
    pub columns: usize,
    pub width: f32,
    pub depth: f32,
    pub colors: GridColors,
}

impl Grid {
    pub fn new(rows: usize, columns: usize, width: f32, depth: f32) -> Self {
        Self {
            rows,
            columns,
            width,
            depth,
            colors: GridColors::default(),
        }
    }
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

#[derive(Resource)]
pub struct InitializeGrid {
    pub done: bool,
    pub delay: Timer,
}

impl Default for InitializeGrid {
    fn default() -> Self {
        Self {
            done: false,
            delay: Timer::from_seconds(0.05, TimerMode::Once),
        }
    }
}

#[derive(Resource)]
pub struct TargetCell {
    pub row: usize,
    pub column: usize,
}

impl TargetCell {
    pub fn new(cells_width: usize, cells_depth: usize) -> Self {
        let target = TargetCell {
            row: cells_width - 1,
            column: cells_depth - 1,
        };

        target
    }
}
