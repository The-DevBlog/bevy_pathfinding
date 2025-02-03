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
        app.add_systems(Update, update_flowfields)
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
    pub size: IVec2,
    pub steering_map: HashMap<Entity, Vec3>,
    pub units: Vec<Entity>,
}

impl FlowField {
    pub fn new(cell_radius: f32, grid_size: IVec2, units: Vec<Entity>, unit_size: f32) -> Self {
        FlowField {
            cell_radius,
            cell_diameter: cell_radius * 2.0,
            destination_cell: Cell::default(),
            destination_radius: (units.len() as f32 * unit_size).sqrt() * 4.0, // TODO: remove (or put into dbg only logic)
            // destination_radius: (units.len() as f32 * unit_size).sqrt() * 20.0, // TODO: remove (or put into dbg only logic)
            grid: Vec::default(),
            size: grid_size,
            steering_map: HashMap::new(),
            units,
        }
    }

    pub fn create_integration_field(&mut self, grid: ResMut<Grid>, destination_cell: Cell) {
        // println!("Start Integration Field Create");

        self.grid = grid.grid.clone();

        // Initialize the destination cell in the grid
        let dest_idx = destination_cell.idx;
        let dest_cell = &mut self.grid[dest_idx.y as usize][dest_idx.x as usize];
        dest_cell.cost = 0;
        dest_cell.best_cost = 0;
        self.destination_cell = dest_cell.clone();

        let mut cells_to_check: VecDeque<IVec2> = VecDeque::new();
        cells_to_check.push_back(dest_idx);

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
        cmds.entity(unit).remove::<Destination>();
    }
}

fn update_flowfields(
    mut cmds: Commands,
    mut q_flowfields: Query<(Entity, &mut FlowField)>,
    q_transform: Query<&Transform>,
    q_destination_radius: Query<(Entity, &DestinationRadius)>, // TODO: Remove
    mut q_destination: Query<&mut Destination>,
) {
    for (flowfield_entity, mut flowfield) in q_flowfields.iter_mut() {
        let destination_pos = flowfield.destination_cell.world_pos;
        let mut units_to_remove = Vec::new();

        // Identify units that need to be removed
        for &unit_entity in flowfield.units.iter() {
            if let Ok(transform) = q_transform.get(unit_entity) {
                let unit_pos = transform.translation;

                // Use squared distance for efficiency
                let distance_squared = (destination_pos - unit_pos).length_squared();
                let cell_diamaeter_squared = flowfield.cell_diameter.squared(); //TODO: May need adjustment

                let radius_squared = flowfield.destination_radius.squared(); // TODO: Remove

                if distance_squared < cell_diamaeter_squared {
                    units_to_remove.push(unit_entity);
                }
            }
        }

        // Remove units from the flowfield only once all units are in the destination radius
        // TODO: potential bug: What if a unit is destroyed before it reaches the destination radius?
        for unit in units_to_remove {
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
            // (if that logic still applies)
            for (ent, d) in q_destination_radius.iter() {
                if d.0 == flowfield_entity.index() {
                    cmds.entity(ent).despawn_recursive();
                }
            }
        }
    }
    // for (flowfield_entity, mut flowfield) in q_flowfields.iter_mut() {
    //     // TODO: Remove
    //     for (ent, d) in q_destination_radius.iter() {
    //         if d.0 == flowfield_entity.index() {
    //             cmds.entity(ent).despawn_recursive();
    //         }
    //     }

    //     if flowfield.units.iter().any(|unit| units.contains(unit)) {
    //         cmds.entity(flowfield_entity).despawn_recursive();
    //     }
    // }

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
        grid.cell_radius,
        grid.size,
        units.clone(),
        unit_positions[0].1.x,
    );
    flowfield.create_integration_field(grid, destination_cell);
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

fn initialize_mini_flowfield(
    trigger: Trigger<InitializeFlowFieldEv>,
    mut cmds: Commands,
    grid: ResMut<Grid>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_cam: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    q_map_base: Query<&GlobalTransform, With<MapBase>>,
    q_unit_info: Query<(&Transform, &UnitSize)>,
    q_flowfields: Query<(Entity, &FlowField)>,
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
    for (flowfield_entity, flowfield) in q_flowfields.iter() {
        // TODO: Remove
        for (ent, d) in q_destination_radius.iter() {
            if d.0 == flowfield_entity.index() {
                cmds.entity(ent).despawn_recursive();
            }
        }

        if flowfield.units.iter().any(|unit| units.contains(unit)) {
            cmds.entity(flowfield_entity).despawn_recursive();
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
        grid.cell_radius,
        grid.size,
        units.clone(),
        unit_positions[0].1.x,
    );
    flowfield.create_integration_field(grid, destination_cell);
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
