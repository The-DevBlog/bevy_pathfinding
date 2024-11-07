use bevy::{color::palettes::css::*, prelude::*};

#[derive(Component)]
pub struct MapBase;

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component)]
pub struct Unit;

#[derive(Component, Default)]
pub struct Destination {
    pub endpoint: Option<Vec3>,
    pub waypoints: Vec<Cell>,
}

#[derive(Component, Debug)]
pub struct Grid {
    pub cells: Vec<Vec<Cell>>,               // 2D matrix of cells
    pub occupied_cells: Vec<(usize, usize)>, // Store occupied cells as (row, column)
    pub rows: usize,
    pub columns: usize,
    pub width: f32,
    pub height: f32,
    pub cell_width: f32,
    pub cell_height: f32,
}

#[derive(Component, Debug)]
pub struct GridColors {
    pub path_finding: Srgba,
    pub path: Srgba,
    pub occupied: Srgba,
    pub grid: Srgba,
}

impl Default for GridColors {
    fn default() -> Self {
        Self {
            path_finding: YELLOW,
            path: LIGHT_STEEL_BLUE,
            occupied: RED,
            grid: GRAY,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub position: Vec2,
    pub occupied: bool,
    pub row: usize,
    pub column: usize,
}

impl Grid {
    pub fn new(
        rows: usize,
        columns: usize,
        width: f32,
        height: f32,
        cell_width: f32,
        cell_height: f32,
    ) -> Self {
        let mut cells = Vec::new();

        for row in 0..rows {
            let mut row_cells = Vec::new();
            for column in 0..columns {
                // Calculate the center position of each cell
                let position = Vec2::new(
                    -width / 2.0 + column as f32 * cell_width + cell_width / 2.0,
                    -height / 2.0 + row as f32 * cell_height + cell_height / 2.0,
                );

                row_cells.push(Cell {
                    position,
                    occupied: false,
                    row: row as usize,
                    column: column as usize,
                });
            }
            cells.push(row_cells);
        }

        Grid {
            cells,
            occupied_cells: Vec::default(),
            rows,
            columns,
            width,
            height,
            cell_width,
            cell_height,
        }
    }
}
