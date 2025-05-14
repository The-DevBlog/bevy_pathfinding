use bevy::prelude::*;
use std::collections::HashMap;

use crate::{cell::Cell, components::RtsObj, events::UpdateCostEv, utils};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Grid>().add_systems(
            Update,
            (update_costfield_on_add, update_costfield_on_remove),
        );
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Grid {
    pub cell_radius: f32,
    pub cell_diameter: f32,
    pub grid: Vec<Vec<Cell>>,
    pub size: IVec2, // 'x' represents rows, 'y' represents columns
    pub occupied_cells: HashMap<u32, Vec<IVec2>>,
}

impl Grid {
    // creates the grid and the costfield
    // all flowfields will share the same costfield
    pub fn new(size: IVec2, cell_diameter: f32) -> Self {
        let mut grid = Grid {
            cell_diameter,
            cell_radius: cell_diameter / 2.0,
            grid: Vec::default(),
            size,
            occupied_cells: HashMap::default(),
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

        grid
    }

    pub fn get_cell_from_world_position(&self, world_pos: Vec3) -> Cell {
        // Calculate the offset for the grid's top-left corner
        let adjusted_x = world_pos.x - (-self.size.x as f32 * self.cell_diameter / 2.0);
        let adjusted_y = world_pos.z - (-self.size.y as f32 * self.cell_diameter / 2.0);

        // Calculate percentages within the grid
        let percent_x = adjusted_x / (self.size.x as f32 * self.cell_diameter);
        let percent_y = adjusted_y / (self.size.y as f32 * self.cell_diameter);

        let offset = Some(Vec2::new(percent_x, percent_y));

        utils::get_cell_from_world_position_helper(
            world_pos,
            self.size,
            self.cell_diameter,
            &self.grid,
            offset,
        )
    }

    pub fn update_cell_costs(
        &mut self,
        entity_id: u32,
        obj_transform: &Transform,
        obj_size: &RtsObj,
    ) {
        let cell_size = self.cell_diameter;
        let grid_offset_x = -self.size.x as f32 * cell_size / 2.0;
        let grid_offset_y = -self.size.y as f32 * cell_size / 2.0;

        let obj_pos = obj_transform.translation;
        let obj_pos = Vec2::new(obj_pos.x, obj_pos.z);
        let half_extent = obj_size.0 / 2.0;

        // Obtain the rotation matrix from the object's rotation.
        let rotation = Mat3::from_quat(obj_transform.rotation);

        // Compute the absolute value of each column of the rotation matrix.
        // This effectively gives the scaling of the half extents in world space.
        let abs_rotation = Mat2::from_cols(rotation.x_axis.xy(), rotation.y_axis.xy());

        // Compute the world-space half extents by multiplying with the local half extents.
        let world_half_extent = abs_rotation * half_extent;

        // Compute the axis aligned bounding box that encloses the rotated object.
        let aabb_min = obj_pos - world_half_extent;
        let aabb_max = obj_pos + world_half_extent;

        // Calculate grid cell indices based on the object's AABB in the xz-plane.
        let min_x = ((aabb_min.x - grid_offset_x) / cell_size).floor() as isize;
        let max_x = ((aabb_max.x - grid_offset_x) / cell_size).floor() as isize;
        let min_y = ((aabb_min.y - grid_offset_y) / cell_size).floor() as isize;
        let max_y = ((aabb_max.y - grid_offset_y) / cell_size).floor() as isize;

        let mut occupied_cells = Vec::new();
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if x >= 0 && x < self.size.x as isize && y >= 0 && y < self.size.y as isize {
                    occupied_cells.push(IVec2::new(x as i32, y as i32));
                    self.grid[y as usize][x as usize].cost = 255;
                }
            }
        }

        self.occupied_cells
            .entry(entity_id)
            .and_modify(|cells| cells.extend(occupied_cells.iter().cloned()))
            .or_insert(occupied_cells);
    }

    // TODO: Will eventually need rework. This is setting the cell cost back to 1. What if the cost was originally
    // something different like rough terrain?
    pub fn reset_cell_costs(&mut self, entities: Vec<Entity>) {
        for ent in entities.iter() {
            if let Some(occupied_cells) = self.occupied_cells.remove(&ent.index()) {
                for cell in occupied_cells.iter() {
                    self.grid[cell.y as usize][cell.x as usize].cost = 1;
                }
            }
        }
    }
}

// detects if a new static object has been added and updates the costfield
fn update_costfield_on_add(
    mut cmds: Commands,
    mut grid: ResMut<Grid>,
    q_objects: Query<(Entity, &Transform, &RtsObj), Added<RtsObj>>,
) {
    let objects = q_objects.iter().collect::<Vec<_>>();
    if objects.is_empty() {
        return;
    }

    for (ent, transform, size) in objects.iter() {
        grid.update_cell_costs(ent.index(), transform, size);
    }

    cmds.trigger(UpdateCostEv);
}

// detects if a static object has been removed and updates the costfield
fn update_costfield_on_remove(
    mut cmds: Commands,
    mut grid: ResMut<Grid>,
    mut removed: RemovedComponents<RtsObj>,
) {
    let objs: Vec<Entity> = removed.read().collect();

    if !objs.is_empty() {
        grid.reset_cell_costs(objs);
        cmds.trigger(UpdateCostEv);
    }
}
