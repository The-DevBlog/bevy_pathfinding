use bevy::prelude::*;

use crate::*;

#[derive(Resource)]
pub struct Grid {
    pub rows: usize,
    pub columns: usize,
    pub width: f32,
    pub depth: f32,
    pub cell_size: f32,
    pub colors: GridColors,
}

impl Grid {
    pub fn new(rows: usize, columns: usize, cell_size: f32) -> Self {
        Self {
            rows,
            columns,
            width: cell_size * rows as f32,
            depth: cell_size * columns as f32,
            cell_size,
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
pub struct TargetCell(pub Option<(usize, usize)>);

impl Default for TargetCell {
    fn default() -> Self {
        TargetCell(None)
    }
}
