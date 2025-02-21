use bevy::{prelude::*, render::primitives::Aabb};

use crate::{
    cell::Cell,
    components::{RtsDynamicObj, RtsObjSize, RtsStaticObj},
    events::UpdateCostEv,
    utils,
};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Grid>()
            .add_systems(PostStartup, initialize_costfield)
            .add_systems(Update, update_costfield_on_add)
            .add_observer(update_costfield_on_remove);
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Grid {
    pub size: IVec2, // 'x' represents rows, 'y' represents columns
    pub cell_radius: f32,
    pub cell_diameter: f32,
    pub grid: Vec<Vec<Cell>>,
}

impl Grid {
    // creates the grid and the costfield
    // all flowfields will share the same costfield
    pub fn new(size: IVec2, cell_diameter: f32) -> Self {
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

    pub fn update_cell_costs(&mut self, unit_transform: &Transform, unit_size: &RtsObjSize) {
        self.for_each_cell_in_unit(unit_transform, unit_size, |grid, pos| {
            grid.update_cell_cost_helper(pos);
        });
    }

    pub fn reset_cell_costs(&mut self, unit_transform: &Transform, unit_size: &RtsObjSize) {
        self.for_each_cell_in_unit(unit_transform, unit_size, |grid, pos| {
            grid.reset_cell_cost_helper(pos);
        });
    }

    // Iterates over all grid cell positions that intersect with the unit’s AABB.
    fn for_each_cell_in_unit<F>(
        &mut self,
        unit_transform: &Transform,
        unit_size: &RtsObjSize,
        mut callback: F,
    ) where
        F: FnMut(&mut Self, Vec3),
    {
        let cell_size = self.cell_diameter;
        let grid_offset_x = -self.size.x as f32 * cell_size / 2.0;
        let grid_offset_y = -self.size.y as f32 * cell_size / 2.0;

        let unit_pos = unit_transform.translation;
        let half_extent = unit_size.0 / 2.0;
        let aabb = Aabb::from_min_max(
            Vec3::new(
                unit_pos.x - half_extent.x,
                unit_pos.y - half_extent.y,
                unit_pos.z - half_extent.y,
            ),
            Vec3::new(
                unit_pos.x + half_extent.x,
                unit_pos.y + half_extent.y,
                unit_pos.z + half_extent.y,
            ),
        );

        let grid_min_x = ((aabb.min().x - grid_offset_x) / cell_size).floor() as isize;
        let grid_max_x = ((aabb.max().x - grid_offset_x) / cell_size).floor() as isize;
        let grid_min_y = ((aabb.min().z - grid_offset_y) / cell_size).floor() as isize;
        let grid_max_y = ((aabb.max().z - grid_offset_y) / cell_size).floor() as isize;

        for y in grid_min_y..=grid_max_y {
            for x in grid_min_x..=grid_max_x {
                if x >= 0 && x < self.size.x as isize && y >= 0 && y < self.size.y as isize {
                    let cell_pos = Vec3::new(
                        x as f32 * cell_size + grid_offset_x,
                        0.0,
                        y as f32 * cell_size + grid_offset_y,
                    );
                    callback(self, cell_pos);
                }
            }
        }
    }

    fn update_cell_cost_helper(&mut self, position: Vec3) -> Cell {
        let cell = self.get_cell_from_world_position(position);
        if cell.idx.y < self.grid.len() as i32
            && cell.idx.x < self.grid[cell.idx.y as usize].len() as i32
        {
            self.grid[cell.idx.y as usize][cell.idx.x as usize].cost = 255;
        }
        cell
    }

    // TODO: Will eventually need rework. This is setting the cell cost back to 1. What if the cost was originally
    // something else? Like different terrain (mud, snow)? Maybe we need to store the original costfield in a hashmap or something
    fn reset_cell_cost_helper(&mut self, position: Vec3) -> Cell {
        let cell = self.get_cell_from_world_position(position);
        self.grid[cell.idx.y as usize][cell.idx.x as usize].cost = 1;
        cell
    }
}

// update this so that it gets the aabb of the entity and checks if it intersects with the cell
fn initialize_costfield(
    mut grid: ResMut<Grid>,
    q_objects: Query<(&Transform, &RtsObjSize), With<RtsStaticObj>>,
) {
    let objects = q_objects.iter().collect::<Vec<_>>();

    for (transform, size) in objects {
        grid.update_cell_costs(transform, size);
    }
}

// detects if a new static object has been added and updates the costfield
fn update_costfield_on_add(
    mut cmds: Commands,
    mut grid: ResMut<Grid>,
    q_objects: Query<(&Transform, &RtsObjSize), Added<RtsStaticObj>>,
) {
    let objects = q_objects.iter().collect::<Vec<_>>();
    if objects.is_empty() {
        return;
    }

    for (transform, size) in objects.iter() {
        grid.update_cell_costs(transform, size);
    }

    cmds.trigger(UpdateCostEv);
}

fn update_costfield_on_remove(
    trigger: Trigger<OnRemove, RtsStaticObj>,
    mut cmds: Commands,
    mut grid: ResMut<Grid>,
    q_transform: Query<(&Transform, &RtsObjSize)>,
) {
    let ent = trigger.entity();
    if let Ok((transform, size)) = q_transform.get(ent) {
        grid.reset_cell_costs(transform, size);
    } else {
        return;
    }

    cmds.trigger(UpdateCostEv);
}

// TODO: remove?
// update this so that it gets the aabb of the entity and checks if it intersects with the cell
fn update_costfield_og(
    mut cmds: Commands,
    grid: Res<Grid>,
    q: Query<(&Transform, Entity), Added<RtsDynamicObj>>,
) {
    // For each newly added dynamic object, compute an AABB in the XZ plane.
    for (transform, _entity) in q.iter() {
        println!("Checking");
        // Assume a default half-extent for the entity's AABB; adjust as needed.
        let half_extent = 0.5;
        let entity_min = Vec2::new(
            transform.translation.x - half_extent,
            transform.translation.z - half_extent,
        );
        let entity_max = Vec2::new(
            transform.translation.x + half_extent,
            transform.translation.z + half_extent,
        );

        // Iterate through all grid cells.
        // Each cell is assumed to be a square centered at its world_pos,
        // with half size grid.cell_radius.
        for row in grid.grid.iter() {
            for cell in row.iter() {
                println!("checking cell");
                let cell_min = Vec2::new(
                    cell.world_pos.x - grid.cell_radius,
                    cell.world_pos.z - grid.cell_radius,
                );
                let cell_max = Vec2::new(
                    cell.world_pos.x + grid.cell_radius,
                    cell.world_pos.z + grid.cell_radius,
                );

                // Check if the entity's AABB intersects with the cell's AABB.
                if entity_min.x <= cell_max.x
                    && entity_max.x >= cell_min.x
                    && entity_min.y <= cell_max.y
                    && entity_max.y >= cell_min.y
                {
                    println!("Updating cost for cell {:?}", cell.idx);
                    // cmds.trigger(UpdateCostEv::new(*cell));
                }
            }
        }
    }
}
