use bevy::{prelude::*, window::PrimaryWindow};
use ops::FloatPow;
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::components::*;
use crate::events::*;
use crate::{cell::*, grid::Grid, grid_direction::GridDirection, utils};

pub struct FlowfieldPlugin;

impl Plugin for FlowfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, destination_movement_system);

        app.add_systems(
            Update,
            (
                transfer_to_destination_flowfield.run_if(resource_exists::<Grid>),
                remove_flowfields,
            ),
        )
        .add_systems(Update, count)
        .add_observer(initialize_flowfield)
        .add_observer(initialize_destination_flowfields);
    }
}

fn count(q_ff: Query<&FlowField>, q_d: Query<&Destination>) {
    // println!("Destinations: {}", q_d.iter().len());

    let mut dest_ffs = 0;
    for ff in q_ff.iter() {
        dest_ffs += ff.destination_flowfields.len();
    }

    // println!(
    //     "FF Count: {}, Dest FF Count: {}",
    //     q_ff.iter().count(),
    //     dest_ffs
    // );
}

// TODO: Remove. This is just for visualizing the destination radius
#[derive(Component)]
pub struct DestinationRadius(pub u32);

#[derive(Clone, Default, PartialEq)]
pub struct FlowFieldProps {
    pub cell_radius: f32,
    pub cell_diameter: f32,
    pub grid: Vec<Vec<Cell>>,
    pub offset: Vec3,
    pub size: IVec2,
    pub steering_map: HashMap<Entity, Vec3>,
    pub units: Vec<Entity>,
}

impl FlowFieldProps {
    pub fn create_flowfield(&mut self) {
        // println!("Start Flowfield Create");

        let grid_size_y = self.size.y as usize;
        let grid_size_x = self.size.x as usize;

        for y in 0..grid_size_y {
            for x in 0..grid_size_x {
                let cell = &self.grid[y][x]; // Immutable borrow to get best_cost
                let mut best_cost = cell.best_cost;
                let mut best_direction = GridDirection::None;

                // Get all possible directions
                for direction in GridDirection::all_directions() {
                    let delta = direction.vector();
                    let nx = x as isize + delta.x as isize;
                    let ny = y as isize + delta.y as isize;

                    if nx >= 0 && nx < grid_size_x as isize && ny >= 0 && ny < grid_size_y as isize
                    {
                        let neighbor = &self.grid[ny as usize][nx as usize];
                        if neighbor.best_cost < best_cost {
                            best_cost = neighbor.best_cost;
                            best_direction = direction;
                        }
                    }
                }

                // Now, set the best_direction for the cell
                self.grid[y][x].best_direction = best_direction;
            }
        }
    }

    pub fn add_unit(&mut self, unit: Entity) {
        self.units.push(unit);
    }

    pub fn remove_unit(&mut self, unit: Entity) {
        self.units.retain(|&u| u != unit);
        self.steering_map.retain(|&u, _| u != unit);
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct DestinationFlowField {
    pub destination_cell: Cell,
    pub flowfield_props: FlowFieldProps,
}

impl DestinationFlowField {
    pub fn new(cell_diameter: f32, grid_size: IVec2, offset: Vec3, grid: Vec<Vec<Cell>>) -> Self {
        let ff_props = FlowFieldProps {
            cell_radius: cell_diameter / 2.0,
            cell_diameter,
            grid,
            offset,
            size: grid_size,
            steering_map: HashMap::new(),
            units: Vec::new(),
        };

        DestinationFlowField {
            destination_cell: Cell::default(),
            flowfield_props: ff_props,
        }
    }

    /// When querying a cell from world position, use the offset if this is a mini flowfield.
    pub fn get_cell_from_world_position(&self, mut position: Vec3) -> Cell {
        let cell_diameter = self.flowfield_props.cell_diameter;
        let size = self.flowfield_props.size;
        position = position - self.flowfield_props.offset;

        utils::get_cell_from_world_position_helper(
            position,
            size,
            cell_diameter,
            &self.flowfield_props.grid,
            None,
        )
    }

    pub fn create_integration_field(&mut self, destination_idx: IVec2) {
        // println!("Start Integration Field Create");

        // Initialize the destination cell in the grid
        let dest_cell =
            &mut self.flowfield_props.grid[destination_idx.y as usize][destination_idx.x as usize];
        dest_cell.cost = 0;
        dest_cell.best_cost = 0;
        self.destination_cell = dest_cell.clone();

        let mut cells_to_check: VecDeque<IVec2> = VecDeque::new();
        cells_to_check.push_back(destination_idx);

        while let Some(cur_idx) = cells_to_check.pop_front() {
            let cur_x = cur_idx.x as usize;
            let cur_y = cur_idx.y as usize;

            let cur_cell_best_cost = self.flowfield_props.grid[cur_y][cur_x].best_cost;

            // Iterate over cardinal directions
            for direction in GridDirection::cardinal_directions() {
                let delta = direction.vector();
                let neighbor_idx = cur_idx + delta;

                if neighbor_idx.x >= 0
                    && neighbor_idx.x < self.flowfield_props.size.x
                    && neighbor_idx.y >= 0
                    && neighbor_idx.y < self.flowfield_props.size.y
                {
                    let neighbor_x = neighbor_idx.x as usize;
                    let neighbor_y = neighbor_idx.y as usize;

                    let neighbor_cell = &mut self.flowfield_props.grid[neighbor_y][neighbor_x];

                    if neighbor_cell.cost == u8::MAX {
                        continue;
                    }

                    let tentative_best_cost = neighbor_cell.cost as u16 + cur_cell_best_cost;
                    if tentative_best_cost < neighbor_cell.best_cost {
                        neighbor_cell.best_cost = tentative_best_cost;
                        cells_to_check.push_back(neighbor_idx);
                    }
                }
            }
        }

        // println!("End Integration Field Create");
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum AvgDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component, Clone, Default, PartialEq)]
pub struct FlowField {
    pub destination_flowfields: Vec<DestinationFlowField>,
    pub destination_grid_size: IVec2,
    pub destinations: Vec<IVec2>,
    pub destination_cell: Cell,
    pub destination_radius: f32,
    pub flowfield_props: FlowFieldProps,
}

impl FlowField {
    pub fn new(
        cell_diameter: f32,
        grid_size: IVec2,
        units: Vec<Entity>,
        unit_size: f32,
        offset: Vec3,
    ) -> Self {
        let steering_map: HashMap<Entity, Vec3> =
            units.iter().map(|&unit| (unit, Vec3::ZERO)).collect();

        let ff_props = FlowFieldProps {
            cell_radius: cell_diameter / 2.0,
            cell_diameter,
            grid: Vec::default(),
            offset,
            size: grid_size,
            steering_map,
            units: units.clone(),
        };

        let mut destination_ff = DestinationFlowField {
            flowfield_props: ff_props.clone(),
            ..default()
        };
        destination_ff.flowfield_props.units = Vec::new();

        FlowField {
            destinations: Vec::new(),
            destination_grid_size: IVec2::ZERO,
            destination_cell: Cell::default(),
            destination_radius: (units.len() as f32 * unit_size).sqrt() * 5.0,
            destination_flowfields: Vec::new(),
            flowfield_props: ff_props,
        }
    }

    fn _print_idx(&self) {
        for (i, ff) in self.destination_flowfields.iter().enumerate() {
            println!("\nIndex Field: {}", i);
            for y in ff.flowfield_props.grid.iter() {
                for x in y.iter() {
                    print!("[{},{}], ", x.idx.y, x.idx.x);
                }

                println!();
                println!();
            }

            println!();
        }
    }

    fn _print_integration_fields(&self) {
        for (i, ff) in self.destination_flowfields.iter().enumerate() {
            println!("\nIntegration Field: {}", i);
            for y in ff.flowfield_props.grid.iter() {
                for x in y.iter() {
                    print!("{:>2}, ", x.best_cost);
                }

                println!();
            }

            println!();
        }
    }

    fn _print_costfields(&self) {
        for (i, ff) in self.destination_flowfields.iter().enumerate() {
            println!("\nCost Field: {}", i);
            for y in ff.flowfield_props.grid.iter() {
                for x in y.iter() {
                    print!("{:>2}, ", x.cost);
                }

                println!();
            }

            println!();
        }
    }

    fn _print_flowfields(&self) {
        for (i, ff) in self.destination_flowfields.iter().enumerate() {
            println!("\nFlow Field: {}", i);
            for y in ff.flowfield_props.grid.iter() {
                for x in y.iter() {
                    x.best_direction.print_short();
                }

                println!();
            }

            println!();
        }
    }

    /// Get the average direction of all units relative to the parent destination.
    /// Eg. If most units are to the right of the destination, the average direction will be Right.
    fn get_avg_direction_from_destination(&self, q_transform: Query<&Transform>) -> AvgDirection {
        // We'll accumulate the offset of each unit from the parent destination.
        let mut sum_x = 0.0;
        let mut sum_z = 0.0;
        let mut count = 0;
        let parent_dest = self.destination_cell.world_pos;
        for &unit in &self.flowfield_props.units {
            if let Ok(transform) = q_transform.get(unit) {
                let unit_pos = transform.translation;
                // Compute the offset along x and z relative to the destination.
                let offset_x = unit_pos.x - parent_dest.x;
                let offset_z = unit_pos.z - parent_dest.z;
                sum_x += offset_x;
                sum_z += offset_z;
                count += 1;
            }
        }

        // Compute the average offset.
        let avg_x = sum_x / count as f32;
        let avg_z = sum_z / count as f32;

        // Compare the absolute averages to determine which axis is dominant.
        if avg_x.abs() > avg_z.abs() {
            // Horizontal dominance: decide between left or right.
            if avg_x < 0.0 {
                AvgDirection::Left
            } else {
                AvgDirection::Right
            }
        } else {
            // Vertical (z axis) dominance: decide between up or down.
            if avg_z < 0.0 {
                AvgDirection::Down
            } else {
                AvgDirection::Up
            }
        }
    }

    fn assign_unit_to_destination_flowfield(&mut self, unit: Entity) {
        for ff in self.destination_flowfields.iter_mut() {
            // Occupied. Only 1 unit per destination flowfield
            if ff.flowfield_props.units.len() > 0 {
                continue;
            }

            ff.flowfield_props.add_unit(unit);
            break;
        }

        self.flowfield_props.remove_unit(unit);
    }

    fn create_destinations(&mut self, avg_direction: AvgDirection) {
        let mut unit_count = self.flowfield_props.units.len();
        for ff in self.destination_flowfields.iter() {
            unit_count += ff.flowfield_props.units.len();
        }

        let destinations =
            utils::build_destinations(unit_count, self.destination_grid_size, avg_direction);
        self.destinations = destinations.clone();
    }

    /// Gets the Cell at the given world position.
    pub fn get_cell_from_world_position(&self, position: Vec3) -> Cell {
        let pos = position;
        let cell_diameter = self.flowfield_props.cell_diameter;
        let size = self.flowfield_props.size;

        // Calculate the offset for the grid's top-left corner
        let adjusted_x = pos.x - (-size.x as f32 * cell_diameter / 2.0);
        let adjusted_y = pos.z - (-size.y as f32 * cell_diameter / 2.0);

        // Calculate percentages within the grid
        let percent_x = adjusted_x / (size.x as f32 * cell_diameter);
        let percent_y = adjusted_y / (size.y as f32 * cell_diameter);

        let offset = Some(Vec2::new(percent_x, percent_y));

        utils::get_cell_from_world_position_helper(
            pos,
            size,
            cell_diameter,
            &self.flowfield_props.grid,
            offset,
        )
    }

    fn create_integration_field(&mut self, grid: Vec<Vec<Cell>>, destination_idx: IVec2) {
        // println!("Start Integration Field Create");

        self.flowfield_props.grid = grid;

        // Initialize the destination cell in the grid
        let dest_cell =
            &mut self.flowfield_props.grid[destination_idx.y as usize][destination_idx.x as usize];
        dest_cell.cost = 0;
        dest_cell.best_cost = 0;
        self.destination_cell = dest_cell.clone();

        let mut cells_to_check: VecDeque<IVec2> = VecDeque::new();
        cells_to_check.push_back(destination_idx);

        while let Some(cur_idx) = cells_to_check.pop_front() {
            let cur_x = cur_idx.x as usize;
            let cur_y = cur_idx.y as usize;

            let cur_cell_best_cost = self.flowfield_props.grid[cur_y][cur_x].best_cost;

            // Iterate over cardinal directions
            for direction in GridDirection::cardinal_directions() {
                let delta = direction.vector();
                let neighbor_idx = cur_idx + delta;

                if neighbor_idx.x >= 0
                    && neighbor_idx.x < self.flowfield_props.size.x
                    && neighbor_idx.y >= 0
                    && neighbor_idx.y < self.flowfield_props.size.y
                {
                    let neighbor_x = neighbor_idx.x as usize;
                    let neighbor_y = neighbor_idx.y as usize;

                    let neighbor_cell = &mut self.flowfield_props.grid[neighbor_y][neighbor_x];

                    if neighbor_cell.cost == u8::MAX {
                        continue;
                    }

                    let tentative_best_cost = neighbor_cell.cost as u16 + cur_cell_best_cost;
                    if tentative_best_cost < neighbor_cell.best_cost {
                        neighbor_cell.best_cost = tentative_best_cost;
                        cells_to_check.push_back(neighbor_idx);
                    }
                }
            }
        }

        // println!("End Integration Field Create");
    }
}

fn transfer_to_destination_flowfield(
    mut q_ff: Query<&mut FlowField>,
    q_transform: Query<&Transform>,
) {
    for mut ff in q_ff.iter_mut() {
        let destination_pos = ff.destination_cell.world_pos;
        let radius_squared = ff.destination_radius.squared();

        let mut units_to_transfer: Vec<Entity> = Vec::new();
        // Identify units that need to be moved to the destination flowfield
        for &mut unit_ent in ff.flowfield_props.units.iter_mut() {
            if let Ok(unit_transform) = q_transform.get(unit_ent) {
                let unit_pos = unit_transform.translation;

                // If unit is within destination radius, store the unit for FF transfer
                let distance_squared = (destination_pos - unit_pos).length_squared(); // squared for performance
                if distance_squared < radius_squared {
                    units_to_transfer.push(unit_ent);
                }
            }
        }

        for unit in units_to_transfer {
            ff.assign_unit_to_destination_flowfield(unit);
        }
    }
}

fn remove_flowfields(
    mut cmds: Commands,
    mut q_ff: Query<(Entity, &mut FlowField)>,
    q_transform: Query<&Transform>,
) {
    for (ff_ent, mut ff) in q_ff.iter_mut() {
        let mut dest_ff_to_remove = Vec::new();
        for (dest_ff_idx, dest_ff) in ff.destination_flowfields.iter_mut().enumerate() {
            if dest_ff.flowfield_props.units.len() == 0 {
                continue;
            }

            let unit_ent = dest_ff.flowfield_props.units[0];
            if let Ok(unit_transform) = q_transform.get(unit_ent) {
                let unit_pos = unit_transform.translation;
                let cell_diamter_squared = dest_ff.flowfield_props.cell_diameter.powi(2);
                let distance_squared =
                    (dest_ff.destination_cell.world_pos - unit_pos).length_squared();

                if distance_squared < cell_diamter_squared {
                    println!("Removing unit from dest FF");
                    cmds.entity(unit_ent).remove::<Destination>();
                    dest_ff_to_remove.push(dest_ff_idx);
                }
            }
        }

        // Remove destiantion flowfields that are marked from above
        for idx in dest_ff_to_remove.iter() {
            ff.destination_flowfields.remove(*idx);
        }

        // if the flowfield has no destination flowfields, remove it
        if ff.destination_flowfields.is_empty() {
            println!("Revmoing FlowField");
            cmds.entity(ff_ent).despawn_recursive();
        }
    }
}

fn get_min_max(radius: f32, center: Vec3, grid: &Grid) -> (IVec2, IVec2) {
    let tl = center + Vec3::new(-radius, 0.0, -radius); // top left
    let tr = center + Vec3::new(radius, 0.0, -radius); // top right
    let bl = center + Vec3::new(-radius, 0.0, radius); // bottom left
    let br = center + Vec3::new(radius, 0.0, radius); // bottom right

    // find cell positions
    let tl = grid.get_cell_from_world_position(tl);
    let tr = grid.get_cell_from_world_position(tr);
    let bl = grid.get_cell_from_world_position(bl);
    let br = grid.get_cell_from_world_position(br);

    // find the min and max x and y values
    let min_x = tl.idx.x.min(tr.idx.x).min(bl.idx.x).min(br.idx.x);
    let max_x = tl.idx.x.max(tr.idx.x).max(bl.idx.x).max(br.idx.x);
    let min_y = tl.idx.y.min(tr.idx.y).min(bl.idx.y).min(br.idx.y);
    let max_y = tl.idx.y.max(tr.idx.y).max(bl.idx.y).max(br.idx.y);

    let min = IVec2::new(min_x, min_y);
    let max = IVec2::new(max_x, max_y);

    (min, max)
}

fn build_destination_grid(
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
    grid: &Grid,
) -> Vec<Vec<Cell>> {
    let mut mini_grid = Vec::new();
    for y in min_y..max_y {
        let mut row = Vec::new();
        for x in min_x..max_x {
            let mut cell = grid.grid[y as usize][x as usize].clone();
            cell.idx = IVec2::new(x - min_x, y - min_y);
            row.push(cell);
        }
        mini_grid.push(row);
    }
    mini_grid
}

fn initialize_flowfield(
    trigger: Trigger<InitializeFlowFieldEv>,
    mut cmds: Commands,
    grid: ResMut<Grid>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_cam: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    q_map_base: Query<&GlobalTransform, With<MapBase>>,
    q_unit_info: Query<(&Transform, &UnitSize)>,
    mut q_ff: Query<(Entity, &mut FlowField)>,
    mut meshes: ResMut<Assets<Mesh>>,                // TODO: Remove
    mut materials: ResMut<Assets<StandardMaterial>>, // TODO: Remove
    q_destination_radius: Query<(Entity, &DestinationRadius)>, // TODO: Remove
) {
    let Some(mouse_pos) = q_windows.single().cursor_position() else {
        return;
    };

    let Ok(cam) = q_cam.get_single() else {
        return;
    };

    let Ok(map_base) = q_map_base.get_single() else {
        return;
    };

    let units = trigger.event().0.clone();
    if units.is_empty() {
        return;
    }

    // Remove existing flowfields that contain any of the units
    for (ff_ent, mut ff) in q_ff.iter_mut() {
        // 1) Filter out any units from `flowfield.units` that are in `units`
        //    i.e. the ones that are about to be added to the new flowfield.
        ff.flowfield_props.units.retain(|ent| !units.contains(ent));

        ff.flowfield_props
            .steering_map
            .retain(|ent, _| !units.contains(ent));

        // 2) If after removal, the flowfield is now empty, *then* despawn it.
        if ff.flowfield_props.units.is_empty() {
            cmds.entity(ff_ent).despawn_recursive();

            // Also remove any "destination radius" entity that references this flowfield
            // TODO: Remove
            for (ent, d) in q_destination_radius.iter() {
                if d.0 == ff_ent.index() {
                    cmds.entity(ent).despawn_recursive();
                }
            }
        }
    }

    // Gather unit positions and sizes
    let mut unit_positions = Vec::new();
    for &unit in &units {
        if let Ok((transform, size)) = q_unit_info.get(unit) {
            unit_positions.push((transform.translation, size.0));
        }
    }

    let world_mouse_pos = utils::get_world_pos(map_base, cam.1, cam.0, mouse_pos);
    let destination_cell = grid.get_cell_from_world_position(world_mouse_pos);

    let mut ff = FlowField::new(
        grid.cell_diameter,
        grid.size,
        units.clone(),
        unit_positions[0].1.x,
        Vec3::ZERO,
    );

    ff.create_integration_field(grid.grid.clone(), destination_cell.idx);
    ff.flowfield_props.create_flowfield();

    // Spawn the new flowfield
    // cmds.spawn(flowfield.clone()); // TODO: Uncomment
    let ff_ent = cmds.spawn((ff.clone(), Name::new("ParentFlowField"))).id();
    cmds.trigger(InitializeDestinationFlowFieldsEv(ff_ent));

    // TODO: Remove
    let mesh = Mesh3d(meshes.add(Cylinder::new(ff.destination_radius, 2.0)));
    let material = MeshMaterial3d(materials.add(Color::srgba(1.0, 1.0, 0.33, 0.85)));
    cmds.spawn((
        DestinationRadius(ff_ent.index()),
        mesh,
        material,
        Transform::from_translation(ff.destination_cell.world_pos),
    ));

    cmds.trigger(SetActiveFlowfieldEv(Some(ff)));
}

fn initialize_destination_flowfields(
    trigger: Trigger<InitializeDestinationFlowFieldsEv>,
    grid: Res<Grid>,
    mut q_parent_ff: Query<&mut FlowField>,
    q_transform: Query<&Transform>,
) {
    println!("\ninitialize_destination_flowfields() start");

    let parent_ff_ent = trigger.event().0.clone();
    let Ok(mut parent_ff) = q_parent_ff.get_mut(parent_ff_ent) else {
        return;
    };

    let (min, max) = get_min_max(
        parent_ff.destination_radius,
        parent_ff.destination_cell.world_pos,
        &grid,
    );

    let dest_ff_grid = build_destination_grid(min.x, min.y, max.x, max.y, &grid);
    let dest_ff_size = IVec2::new(dest_ff_grid[0].len() as i32, dest_ff_grid.len() as i32);
    let dest_ff_offset = grid.grid[min.y as usize][min.x as usize].world_pos;

    parent_ff.destination_grid_size = dest_ff_size;
    let avg_direction = parent_ff.get_avg_direction_from_destination(q_transform);
    parent_ff.create_destinations(avg_direction);

    let mut dest_ffs = Vec::new();
    for dest_idx in parent_ff.destinations.iter() {
        let mut dest_ff = DestinationFlowField::new(
            grid.cell_diameter,
            dest_ff_size,
            dest_ff_offset,
            dest_ff_grid.clone(),
        );

        dest_ff.create_integration_field(*dest_idx);
        dest_ff.flowfield_props.create_flowfield();
        dest_ffs.push(dest_ff);
    }

    parent_ff.destination_flowfields = dest_ffs;

    // Debugging Purposes
    // parent_ff._print_integration_fields();
    // parent_ff._print_costfields();
    // parent_ff._print_flowfields();
    // parent_ff._print_idx();

    println!("initialize_destination_flowfields() end");
}
