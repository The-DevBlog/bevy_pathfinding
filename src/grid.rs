use bevy::{prelude::*, render::primitives::Aabb};

use crate::{
    cell::Cell,
    components::{RtsDynamicObj, RtsObjSize, RtsStaticObj},
    events::{DrawAllEv, UpdateCostEv},
    utils,
};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Grid>()
            // .add_event::<UpdateCostEv>()
            .add_systems(PostStartup, initialize_costfield)
            .add_systems(Update, update_costfield);
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

    // pub fn get_cell_from_world_position(&self, world_pos: Vec3, offset: Option<Vec2>) -> Cell {
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

    pub fn update_unit_cell_costs(&mut self, position: Vec3) -> Cell {
        // Determine which cell the unit occupies
        let cell = self.get_cell_from_world_position(position);

        // Set the cost of the cell to 255
        if cell.idx.y < self.grid.len() as i32
            && cell.idx.x < self.grid[cell.idx.y as usize].len() as i32
        {
            self.grid[cell.idx.y as usize][cell.idx.x as usize].cost = 255;
        }

        return cell;
    }

    fn update_cell_costs(&mut self, objects: Vec<(&Transform, &RtsObjSize)>) {
        // Grid cell size (assumed uniform square grid)
        let cell_size = self.cell_diameter;

        // Calculate the grid offset (world position of the grid's origin)
        let grid_offset_x = -self.size.x as f32 * cell_size / 2.0;
        let grid_offset_y = -self.size.y as f32 * cell_size / 2.0;

        // Mark cells occupied by units
        for (unit_transform, unit_size) in objects.iter() {
            let unit_pos = unit_transform.translation;

            // Construct an Aabb for the unit
            let half_extent = unit_size.0 / 2.0; // Half size of the unit
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

            // Map AABB to grid coordinates
            let grid_min_x = ((aabb.min().x - grid_offset_x) / cell_size).floor() as isize;
            let grid_max_x = ((aabb.max().x - grid_offset_x) / cell_size).floor() as isize;
            let grid_min_y = ((aabb.min().z - grid_offset_y) / cell_size).floor() as isize;
            let grid_max_y = ((aabb.max().z - grid_offset_y) / cell_size).floor() as isize;

            // Iterate over all cells the unit intersects
            for y in grid_min_y..=grid_max_y {
                for x in grid_min_x..=grid_max_x {
                    if x >= 0 && x < self.size.x as isize && y >= 0 && y < self.size.y as isize {
                        self.update_unit_cell_costs(Vec3::new(
                            x as f32 * cell_size + grid_offset_x,
                            0.0,
                            y as f32 * cell_size + grid_offset_y,
                        ));
                        // let cell = self.update_unit_cell_costs(Vec3::new(
                        //     x as f32 * cell_size + grid_offset_x,
                        //     0.0,
                        //     y as f32 * cell_size + grid_offset_y,
                        // ));

                        // cmds.trigger(UpdateCostEv::new(cell)); // TODO: This should only need to be called once
                    }
                }
            }
        }
    }
}

// update this so that it gets the aabb of the entity and checks if it intersects with the cell
fn initialize_costfield(
    mut grid: ResMut<Grid>,
    q_objects: Query<(&Transform, &RtsObjSize), With<RtsStaticObj>>,
) {
    let objects = q_objects.iter().collect::<Vec<_>>();
    grid.update_cell_costs(objects);
}

// detects if a new static object has been added and updates the costfield
fn update_costfield(
    mut cmds: Commands,
    mut grid: ResMut<Grid>,
    q_objects: Query<(&Transform, &RtsObjSize), Added<RtsStaticObj>>,
) {
    let objects = q_objects.iter().collect::<Vec<_>>();
    if objects.is_empty() {
        return;
    }

    grid.update_cell_costs(objects);
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
// fn update_costfield(q: Query<Entity, Added<RtsDynamicObj>>) {
//     for _e in q.iter() {
//         println!("component added");
//     }
// }

// TODO: This is not precise. It does not capture 'every' cell that a unit is currenlty intersecting with.
// pub fn update_costs(
//     mut occupied_cells: ResMut<OccupiedCells>,
//     mut grid: ResMut<Grid>,
//     mut cmds: Commands,
//     q_units: Query<(&Transform, &RtsObjSize), With<Unit>>,
//     costmap: Res<CostMap>,
// ) {
//     if q_units.is_empty() {
//         return;
//     }

//     let mut current_occupied = HashSet::new();

//     // Grid cell size (assumed uniform square grid)
//     let cell_size = grid.cell_diameter;

//     // Calculate the grid offset (world position of the grid's origin)
//     let grid_offset_x = -grid.size.x as f32 * cell_size / 2.0;
//     let grid_offset_y = -grid.size.y as f32 * cell_size / 2.0;

//     // Mark cells occupied by units
//     for (unit_transform, unit_size) in q_units.iter() {
//         let unit_pos = unit_transform.translation;

//         // Construct an Aabb for the unit
//         let half_extent = unit_size.0 / 2.0; // Half size of the unit
//         let aabb = Aabb::from_min_max(
//             Vec3::new(
//                 unit_pos.x - half_extent.x,
//                 unit_pos.y - half_extent.y,
//                 unit_pos.z - half_extent.y,
//             ),
//             Vec3::new(
//                 unit_pos.x + half_extent.x,
//                 unit_pos.y + half_extent.y,
//                 unit_pos.z + half_extent.y,
//             ),
//         );

//         // Map AABB to grid coordinates
//         let grid_min_x = ((aabb.min().x - grid_offset_x) / cell_size).floor() as isize;
//         let grid_max_x = ((aabb.max().x - grid_offset_x) / cell_size).floor() as isize;
//         let grid_min_y = ((aabb.min().z - grid_offset_y) / cell_size).floor() as isize;
//         let grid_max_y = ((aabb.max().z - grid_offset_y) / cell_size).floor() as isize;

//         // Iterate over all cells the unit intersects
//         // let mut idxs = Vec::new();
//         for y in grid_min_y..=grid_max_y {
//             for x in grid_min_x..=grid_max_x {
//                 if x >= 0 && x < grid.size.x as isize && y >= 0 && y < grid.size.y as isize {
//                     let cell = grid.update_unit_cell_costs(Vec3::new(
//                         x as f32 * cell_size + grid_offset_x,
//                         0.0,
//                         y as f32 * cell_size + grid_offset_y,
//                     ));

//                     // idxs.push(cell.idx);

//                     cmds.trigger(UpdateCostEv::new(cell));
//                     current_occupied.insert(cell.idx);
//                 }
//             }
//         }

//         // print!("Unit occupying cells:");
//         // for i in idxs.iter() {
//         //     print!(" {},{} -", i.y, i.x);
//         // }
//         // println!();
//     }

//     // Reset previously occupied cells that are no longer occupied
//     let columns = grid.grid.len();
//     for idx in occupied_cells.0.difference(&current_occupied) {
//         if idx.y >= 0 && idx.y < grid.size.y && idx.x >= 0 && idx.x < grid.size.x {
//             let cell = &mut grid.grid[idx.y as usize][idx.x as usize];
//             if let Some(cost) = costmap.0.get(&cell.idx_to_id(columns)) {
//                 cell.cost = *cost;
//             }

//             cmds.trigger(UpdateCostEv::new(*cell));
//         }
//     }

//     // Update the occupied cells set
//     occupied_cells.0 = current_occupied;
// }
