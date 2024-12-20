use crate::{cell::Cell, utils};

use bevy::prelude::*;
use bevy_rapier3d::{plugin::*, prelude::*};

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

                if let Some(_entity) = hit {
                    // increase cost now that cell exists
                    grid.grid[y as usize][x as usize].increase_cost(255);
                }
            }
        }

        grid
    }

    pub fn get_cell_from_world_position(&self, world_pos: Vec3) -> Cell {
        let cell = utils::get_cell_from_world_position_helper(
            world_pos,
            self.size,
            self.cell_diameter,
            &self.grid,
        );

        return cell;
    }

    pub fn reset_selected_unit_costs(&mut self, selected_units: Vec<(Vec3, (f32, f32))>) {
        for (unit_pos, unit_size) in selected_units.iter() {
            let hw = unit_size.0;
            let hh = unit_size.1;

            let min_world = Vec3::new(unit_pos.x - hw, 0.0, unit_pos.y - hh);
            let max_world = Vec3::new(unit_pos.x + hw, 0.0, unit_pos.y + hh);

            let min_cell = self.get_cell_from_world_position(min_world);
            let max_cell = self.get_cell_from_world_position(max_world);

            let min_x = min_cell.idx.x.clamp(0, self.size.x as i32 - 1);
            let max_x = max_cell.idx.x.clamp(0, self.size.x as i32 - 1);
            let min_y = min_cell.idx.y.clamp(0, self.size.y as i32 - 1);
            let max_y = max_cell.idx.y.clamp(0, self.size.y as i32 - 1);

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    self.grid[y as usize][x as usize].cost = 1;
                }
            }
        }
    }
}
