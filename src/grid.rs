use bevy::prelude::*;
use bevy_rapier3d::{plugin::*, prelude::*};
use std::cmp::min;

use crate::cell::Cell;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Grid>();
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Grid {
    pub size: IVec2,
    pub cell_radius: f32,
    pub cell_diameter: f32,
    pub grid: Vec<Vec<Cell>>,
}

impl Grid {
    // creates the grid and the costfield
    // all flowfields will share the same costfield
    pub fn new(size: IVec2, cell_diameter: f32, rapier_ctx: &RapierContext) -> Self {
        println!("Start Grid Create");
        println!("Grid Size: {}, {}", size.x, size.y);

        let mut grid = Grid {
            size,
            cell_diameter,
            cell_radius: cell_diameter / 2.0,
            grid: Vec::default(),
        };

        // Calculate offsets for top-left alignment
        let offset_x = -(grid.size.x as f32 * grid.cell_diameter) / 2.;
        let offset_y = -(grid.size.y as f32 * grid.cell_diameter) / 2.;

        // Initialize Grid
        grid.grid = (0..grid.size.y)
            .map(|y| {
                (0..grid.size.x)
                    .map(|x| {
                        let x_pos = grid.cell_diameter * x as f32 + grid.cell_radius + offset_x;
                        let y_pos = grid.cell_diameter * y as f32 + grid.cell_radius + offset_y;
                        let world_pos = Vec3::new(x_pos, 0.0, y_pos);
                        Cell::new(world_pos, IVec2::new(x, y))
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // Create Costfield
        for y in 0..grid.size.y {
            for x in 0..grid.size.x {
                let world_pos = grid.grid[y as usize][x as usize].world_pos;
                let hit = rapier_ctx.intersection_with_shape(
                    world_pos,
                    Quat::IDENTITY,
                    &Collider::cuboid(grid.cell_radius, grid.cell_radius, grid.cell_radius),
                    QueryFilter::default().exclude_sensors(),
                );

                if let Some(entity) = hit {
                    // increase cost now that cell exists
                    grid.grid[y as usize][x as usize].increase_cost(255);
                }
            }
        }

        println!("End Grid Create");

        grid
    }

    pub fn get_cell_from_world_position(&self, world_pos: Vec3) -> Cell {
        // Adjust world position relative to the grid's top-left corner
        let adjusted_x = world_pos.x - (-self.size.x as f32 * self.cell_diameter / 2.0);
        let adjusted_y = world_pos.z - (-self.size.y as f32 * self.cell_diameter / 2.0);

        // Calculate percentages within the grid
        let mut percent_x = adjusted_x / (self.size.x as f32 * self.cell_diameter);
        let mut percent_y = adjusted_y / (self.size.y as f32 * self.cell_diameter);

        // Clamp percentages to ensure they're within [0.0, 1.0]
        percent_x = percent_x.clamp(0.0, 1.0);
        percent_y = percent_y.clamp(0.0, 1.0);

        // Calculate grid indices
        let x = ((self.size.x as f32) * percent_x).floor() as usize;
        let y = ((self.size.y as f32) * percent_y).floor() as usize;

        let x = min(x, self.size.x as usize - 1);
        let y = min(y, self.size.y as usize - 1);

        self.grid[y][x] // Swap x and y
    }
}
