use crate::components::*;
use crate::events::*;
use crate::grid;
use crate::{cell::*, grid::Grid, grid_direction::GridDirection, utils};

use bevy::{prelude::*, window::PrimaryWindow};
use ops::FloatPow;
use std::collections::HashMap;
use std::collections::VecDeque;

pub struct FlowfieldPlugin;

impl Plugin for FlowfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_flowfields.run_if(resource_exists::<Grid>))
            .add_systems(Update, print_ff_count)
            .add_observer(initialize_flowfield);
    }
}

//TODO: remove
fn print_ff_count(_q: Query<&FlowField>, _qd: Query<&Destination>) {
    // println!("Destinationc count: {}", _qd.iter().len());
    // println!("FF count: {}", _q.iter().len());
    // for f in _q.iter() {
    //     println!("FF unit count: {}", f.units.len());
    // }
}

// TODO: Remove. This is just for visualizing the destination radius
#[derive(Component)]
pub struct DestinationRadius(pub u32);

#[derive(Component, Clone, Default, PartialEq)]
pub struct FlowField {
    pub cell_radius: f32,
    pub cell_diameter: f32,
    pub destination_cell: Cell,
    pub destination_radius: f32, // TODO: remove (or put into dbg only logic)
    pub grid: Vec<Vec<Cell>>,
    pub is_mini: bool, // flag to determine if this is a mini flowfield. A mini FF is a single unit FF that is created when a unit is inside of the destination radius
    pub size: IVec2,
    pub steering_map: HashMap<Entity, Vec3>,
    pub units: Vec<Entity>,
}

impl FlowField {
    pub fn new(
        cell_diameter: f32,
        grid_size: IVec2,
        units: Vec<Entity>,
        unit_size: f32,
        is_mini: bool,
    ) -> Self {
        let steering_map: HashMap<Entity, Vec3> = units
            .iter()
            .map(|&unit| (unit, Vec3::ZERO)) // or use Vec3::default()
            .collect();

        FlowField {
            cell_radius: cell_diameter / 2.0,
            cell_diameter,
            destination_cell: Cell::default(),
            // destination_radius: (units.len() as f32 * unit_size).sqrt() * 4.0, // TODO: remove (or put into dbg only logic)
            destination_radius: (units.len() as f32 * unit_size).sqrt() * 20.0, // TODO: remove (or put into dbg only logic)
            grid: Vec::default(),
            is_mini,
            size: grid_size,
            steering_map,
            units,
        }
    }

    pub fn create_integration_field(&mut self, grid: Vec<Vec<Cell>>, destination_idx: IVec2) {
        // println!("Start Integration Field Create");

        self.grid = grid;

        // Initialize the destination cell in the grid
        let dest_cell = &mut self.grid[destination_idx.y as usize][destination_idx.x as usize];
        dest_cell.cost = 0;
        dest_cell.best_cost = 0;
        self.destination_cell = dest_cell.clone();

        let mut cells_to_check: VecDeque<IVec2> = VecDeque::new();
        cells_to_check.push_back(destination_idx);

        while let Some(cur_idx) = cells_to_check.pop_front() {
            let cur_x = cur_idx.x as usize;
            let cur_y = cur_idx.y as usize;

            let cur_cell_best_cost = self.grid[cur_y][cur_x].best_cost;

            // Iterate over cardinal directions
            for direction in GridDirection::cardinal_directions() {
                let delta = direction.vector();
                let neighbor_idx = cur_idx + delta;

                if neighbor_idx.x >= 0
                    && neighbor_idx.x < self.size.x
                    && neighbor_idx.y >= 0
                    && neighbor_idx.y < self.size.y
                {
                    let neighbor_x = neighbor_idx.x as usize;
                    let neighbor_y = neighbor_idx.y as usize;

                    let neighbor_cell = &mut self.grid[neighbor_y][neighbor_x];

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

    pub fn get_cell_from_world_position(&self, world_pos: Vec3) -> Cell {
        let cell = utils::get_cell_from_world_position_helper(
            world_pos,
            self.size,
            self.cell_diameter,
            &self.grid,
        );

        return cell;
    }

    pub fn remove_unit(&mut self, unit: Entity, cmds: &mut Commands) {
        self.units.retain(|&u| u != unit);
        self.steering_map.retain(|&u, _| u != unit);
        // cmds.entity(unit).remove::<Destination>();
    }
}

fn update_flowfields(
    mut cmds: Commands,
    mut q_flowfields: Query<(Entity, &mut FlowField)>,
    q_transform: Query<(&Transform, &UnitSize)>,
    q_destination_radius: Query<(Entity, &DestinationRadius)>, // TODO: Remove
    // mut q_destination: Query<&mut Destination>,
    grid: Res<Grid>,
    mut gizmos: Gizmos,
) {
    for (flowfield_entity, mut flowfield) in q_flowfields.iter_mut() {
        let destination_pos = flowfield.destination_cell.world_pos;
        let mut units_to_remove = Vec::new();

        // Identify units that need to be removed
        for &unit_entity in flowfield.units.iter() {
            if let Ok((unit_transform, unit_size)) = q_transform.get(unit_entity) {
                let unit_pos = unit_transform.translation;

                // Use squared distance for efficiency
                let distance_squared = (destination_pos - unit_pos).length_squared();
                let radius_squared = flowfield.destination_radius.squared(); // TODO: Remove

                if distance_squared < radius_squared && !flowfield.is_mini {
                    units_to_remove.push(unit_entity);

                    let (min, max) = get_min_max(
                        flowfield.destination_radius,
                        flowfield.destination_cell.world_pos,
                        &grid,
                    );

                    // convert original grid idx to the mini grid idx
                    let new_idx = IVec2::new(
                        flowfield.destination_cell.idx.x - min.x,
                        flowfield.destination_cell.idx.y - min.y,
                    );

                    let mini_grid = build_mini_grid(min.x, min.y, max.x, max.y, &grid);
                    let mini_grid_size =
                        IVec2::new(mini_grid[0].len() as i32, mini_grid.len() as i32);

                    let mut mini_ff = FlowField::new(
                        flowfield.cell_diameter,
                        mini_grid_size,
                        vec![unit_entity],
                        unit_size.0.x,
                        true,
                    );

                    mini_ff.destination_cell = flowfield.destination_cell.clone();
                    mini_ff.create_integration_field(mini_grid, new_idx);
                    mini_ff.create_flowfield();

                    // for x in flowfield.steering_map.iter() {
                    //     println!("Steering Map: {:?}", x);
                    // }

                    // for x in mini_ff.steering_map.iter() {
                    //     println!("Steering Map: {:?}", x);
                    // }

                    // for y in mini_ff.grid.iter() {
                    //     for x in y.iter() {
                    //         println!("Best Direction: {:?}", x.best_direction.vector());
                    //     }
                    // }

                    // TODO: Remove: Debugging purposes
                    cmds.trigger(SetActiveFlowfieldEv(Some(mini_ff.clone())));
                    cmds.spawn(mini_ff);

                    // TODO: Remove
                    let mut isometry = Isometry3d::from_translation(
                        flowfield.destination_cell.world_pos
                            - Vec3::new(flowfield.cell_radius, 0.0, flowfield.cell_radius),
                    );
                    isometry.rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
                    let cell_count = UVec2::new(mini_grid_size.x as u32, mini_grid_size.y as u32);
                    let spacing = Vec2::splat(flowfield.cell_diameter);
                    let color = Color::srgb(1.0, 0.65, 0.0);
                    gizmos.grid(isometry, cell_count, spacing, color);
                }

                let cell_diamaeter_squared = flowfield.cell_diameter.squared(); //TODO: May need adjustment
                if distance_squared < cell_diamaeter_squared && flowfield.is_mini {
                    units_to_remove.push(unit_entity);
                }
            }
        }

        // Remove units from the flowfield only once all units are in the destination radius
        // TODO: potential bug: What if a unit is destroyed before it reaches the destination radius?
        for unit in units_to_remove {
            // TODO: Remove
            for (ent, d) in q_destination_radius.iter() {
                if d.0 == flowfield_entity.index() {
                    cmds.entity(ent).despawn_recursive();
                }
            }

            flowfield.remove_unit(unit, &mut cmds);
        }

        if flowfield.units.len() == 0 {
            cmds.entity(flowfield_entity).despawn_recursive();
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

fn build_mini_grid(min_x: i32, min_y: i32, max_x: i32, max_y: i32, grid: &Grid) -> Vec<Vec<Cell>> {
    // create a new grid
    let mut mini_grid = Vec::new();
    for y in min_y..max_y {
        let mut row = Vec::new();
        for x in min_x..max_x {
            let cell = grid.grid[y as usize][x as usize].clone();
            row.push(cell);
        }
        mini_grid.push(row);
    }

    // return the new grid
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
    mut q_flowfields: Query<(Entity, &mut FlowField)>,
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
    for (flowfield_entity, mut flowfield) in q_flowfields.iter_mut() {
        // 1) Filter out any units from `flowfield.units` that are in `units`
        //    i.e. the ones that are about to be added to the new flowfield.
        flowfield.units.retain(|ent| !units.contains(ent));
        flowfield.steering_map.retain(|ent, _| !units.contains(ent));

        // 2) If after removal, the flowfield is now empty, *then* despawn it.
        if flowfield.units.is_empty() {
            cmds.entity(flowfield_entity).despawn_recursive();

            // Also remove any "destination radius" entity that references this flowfield
            // TODO: Remove
            for (ent, d) in q_destination_radius.iter() {
                if d.0 == flowfield_entity.index() {
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

    // Create a new flowfield
    let mut flowfield = FlowField::new(
        grid.cell_diameter,
        grid.size,
        units.clone(),
        unit_positions[0].1.x,
        false,
    );
    flowfield.create_integration_field(grid.grid.clone(), destination_cell.idx);
    flowfield.create_flowfield();

    // Spawn the new flowfield
    // cmds.spawn(flowfield.clone()); // TODO: Uncomment
    let ff = cmds.spawn(flowfield.clone()).id(); // TODO: remove

    // TODO: Remove
    let mesh = Mesh3d(meshes.add(Cylinder::new(flowfield.destination_radius, 2.0)));
    let material = MeshMaterial3d(materials.add(Color::srgba(1.0, 1.0, 0.33, 0.85)));
    cmds.spawn((
        DestinationRadius(ff.index()),
        mesh,
        material,
        Transform::from_translation(flowfield.destination_cell.world_pos),
    ));

    cmds.trigger(SetActiveFlowfieldEv(Some(flowfield)));
}
