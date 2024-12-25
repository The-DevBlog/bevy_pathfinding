use crate::{cell::Cell, components::Destination, utils, UpdateCellEv};

use bevy::prelude::*;
use bevy_rapier3d::{plugin::*, prelude::*};
use std::collections::HashMap;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Grid>()
            .add_event::<UpdateCellEv>()
            .add_systems(Update, update_costs);
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Grid {
    pub size: IVec2,
    pub cell_radius: f32,
    pub cell_diameter: f32,
    pub grid: Vec<Vec<Cell>>,
    pub cost_entities: HashMap<IVec2, Entity>,
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
            cost_entities: HashMap::new(),
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

                    // Associate the cell index with the entity
                    let cell_idx = IVec2::new(x, y);
                    grid.cost_entities.insert(cell_idx, entity);
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

    pub fn reset_costs(&mut self, units: Vec<(Vec3, Vec2)>) {
        for (unit_pos, unit_size) in units.iter() {
            let hw = unit_size.x;
            let hh = unit_size.y;

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

    pub fn update_unit_cell_costs(&mut self, unit_pos: Vec3) -> Cell {
        // Determine which cell the unit occupies
        let cell = self.get_cell_from_world_position(unit_pos);

        // Set the cost of the cell to 255
        if cell.idx.y < self.grid.len() as i32
            && cell.idx.x < self.grid[cell.idx.y as usize].len() as i32
        {
            self.grid[cell.idx.y as usize][cell.idx.x as usize].cost = 255;
        }

        return cell;
    }

    // Example helper to associate a cell with a Cost entity
    pub fn set_cost_entity(&mut self, cell_idx: IVec2, entity: Entity) {
        self.cost_entities.insert(cell_idx, entity);
    }

    // Example helper to get the Cost entity at a cell
    pub fn get_cost_entity(&self, cell_idx: IVec2) -> Option<Entity> {
        self.cost_entities.get(&cell_idx).copied()
    }
}

pub fn update_costs(
    mut grid: ResMut<Grid>,
    mut events: EventWriter<UpdateCellEv>,
    q_units: Query<(Entity, &Transform), With<Destination>>,
) {
    for (entity, transform) in q_units.iter() {
        let cell = grid.update_unit_cell_costs(transform.translation);
        events.send(UpdateCellEv::new(cell, entity));
    }
}
