use bevy::prelude::*;
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::components::*;
use crate::events::*;
use crate::{cell::*, grid::Grid, grid_direction::GridDirection, utils};

pub struct FlowfieldPlugin;

impl Plugin for FlowfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (flowfield_group_stop_system,))
            .add_observer(update_fields)
            .add_observer(initialize_flowfield);
    }
}

// TODO: Remove. This is just for visualizing the destination radius
#[derive(Component)]
pub struct DestinationRadius(pub u32);

#[derive(Component, Clone, Default, PartialEq)]
pub struct FlowField {
    pub arrived: bool,
    pub destination_grid_size: IVec2,
    pub destination_cell: Cell,
    pub destination_radius: f32,
    pub grid: Vec<Vec<Cell>>,
    pub offset: Vec3,
    pub size: IVec2,
    pub steering_map: HashMap<Entity, Vec3>,
    pub units: Vec<Entity>,
}

impl FlowField {
    pub fn new(size: IVec2, units: Vec<Entity>, unit_count: f32, offset: Vec3) -> Self {
        let steering_map: HashMap<Entity, Vec3> =
            units.iter().map(|&unit| (unit, Vec3::ZERO)).collect();

        FlowField {
            destination_radius: (units.len() as f32 * unit_count).sqrt() * 5.0,
            offset,
            size,
            steering_map,
            units: units.clone(),
            ..default()
        }
    }

    pub fn create_flowfield(&mut self) {
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

    /// Gets the Cell at the given world position.
    pub fn get_cell_from_world_position(&self, position: Vec3, grid: &Grid) -> Cell {
        let pos = position;
        let cell_diameter = grid.cell_diameter;
        let size = self.size;

        // Calculate the offset for the grid's top-left corner
        let adjusted_x = pos.x - (-size.x as f32 * cell_diameter / 2.0);
        let adjusted_y = pos.z - (-size.y as f32 * cell_diameter / 2.0);

        // Calculate percentages within the grid
        let percent_x = adjusted_x / (size.x as f32 * cell_diameter);
        let percent_y = adjusted_y / (size.y as f32 * cell_diameter);

        let offset = Some(Vec2::new(percent_x, percent_y));

        utils::get_cell_from_world_position_helper(pos, size, cell_diameter, &self.grid, offset)
    }

    /// Smoothly sample the best_direction at an arbitrary world-space point
    /// by bilinearly interpolating between the four enclosing cells.
    pub fn sample_direction(&self, world_pos: Vec3, grid: &Grid) -> Vec2 {
        // 1) Map world -> [0..1] uv over the grid
        let (u, v) = self.world_to_uv(world_pos, grid);

        // 2) Scale uv to your discrete grid indices in float-space
        let cols = self.size.x as f32;
        let rows = self.size.y as f32;
        let fx = u * (cols - 1.0);
        let fy = v * (rows - 1.0);

        // 3) Corners
        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1).min(self.size.x as usize - 1);
        let y1 = (y0 + 1).min(self.size.y as usize - 1);

        let sx = fx - x0 as f32;
        let sy = fy - y0 as f32;

        // 4) Pull the four best_direction vectors (Vec2)
        let d00 = self.grid[y0][x0].best_direction.vector().as_vec2();
        let d10 = self.grid[y0][x1].best_direction.vector().as_vec2();
        let d01 = self.grid[y1][x0].best_direction.vector().as_vec2();
        let d11 = self.grid[y1][x1].best_direction.vector().as_vec2();

        // 5) Bilinear lerp
        let lerp = |a: Vec2, b: Vec2, t: f32| a * (1.0 - t) + b * t;
        let d0 = lerp(d00, d10, sx);
        let d1 = lerp(d01, d11, sx);
        let smooth = lerp(d0, d1, sy).normalize_or_zero();

        smooth.normalize_or_zero()
    }

    /// Convert a world-space position into UV [0..1] over the grid.
    fn world_to_uv(&self, world_pos: Vec3, grid: &Grid) -> (f32, f32) {
        // Offset so (0,0) is top-left of your grid
        let local = world_pos - self.offset;
        let cell_d = grid.cell_diameter;
        let cols = self.size.x as f32;
        let rows = self.size.y as f32;

        let u = (local.x + (cols * cell_d * 0.5)) / (cols * cell_d);
        let v = (local.z + (rows * cell_d * 0.5)) / (rows * cell_d);

        (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
    }

    fn create_integration_field(&mut self, grid: Vec<Vec<Cell>>, destination_idx: IVec2) {
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
}

fn flowfield_group_stop_system(
    mut cmds: Commands,
    mut query: Query<(Entity, &mut FlowField)>,
    tf_query: Query<&Transform>,
) {
    for (ff_ent, mut ff) in query.iter_mut() {
        // skip empty or already‐stopped fields
        if ff.arrived || ff.units.is_empty() {
            continue;
        }

        // compute centroid of the group
        let (sum, count) = ff
            .units
            .iter()
            .filter_map(|&u| tf_query.get(u).ok().map(|tf| tf.translation))
            .fold((Vec3::ZERO, 0), |(sum, count), pos| (sum + pos, count + 1));
        if count == 0 {
            continue;
        }
        let centroid = sum / count as f32;

        // compare centroid to the world‐space goal
        let goal = ff.destination_cell.world_pos;
        if (centroid - goal).length() < ff.destination_radius {
            // → only _now_ do we stop the entire group
            ff.arrived = true;
            ff.steering_map.clear();

            for &unit_ent in &ff.units {
                cmds.entity(unit_ent).remove::<Destination>();
            }

            ff.units.clear();

            cmds.entity(ff_ent).despawn();
        }
    }
}

fn initialize_flowfield(
    trigger: Trigger<InitializeFlowFieldEv>,
    mut cmds: Commands,
    grid: ResMut<Grid>,
    mut q_ff: Query<(Entity, &mut FlowField)>,
    mut _meshes: ResMut<Assets<Mesh>>, // TODO: Remove
    mut _materials: ResMut<Assets<StandardMaterial>>, // TODO: Remove
    q_destination_radius: Query<(Entity, &DestinationRadius)>, // TODO: Remove
) {
    let destination_pos = trigger.event().destination_pos;
    let units = trigger.event().entities.clone();
    if units.is_empty() {
        return;
    }

    // insert Destination component to all units
    for unit in units.iter() {
        cmds.entity(*unit).insert(Destination);
    }

    // Remove existing flowfields that contain any of the units
    for (ff_ent, mut ff) in q_ff.iter_mut() {
        // 1) Filter out any units from `flowfield.units` that are in `units`
        //    i.e. the ones that are about to be added to the new flowfield.
        ff.units.retain(|ent| !units.contains(ent));

        ff.steering_map.retain(|ent, _| !units.contains(ent));

        // 2) If after removal, the flowfield is now empty, *then* despawn it.
        if ff.units.is_empty() {
            cmds.entity(ff_ent).despawn();

            // Also remove any "destination radius" entity that references this flowfield
            // TODO: Remove
            for (ent, d) in q_destination_radius.iter() {
                if d.0 == ff_ent.index() {
                    cmds.entity(ent).despawn();
                }
            }
        }
    }

    // let world_mouse_pos = utils::get_world_pos(map_base, cam.1, cam.0, cursor_pos);
    let destination_cell = grid.get_cell_from_world_position(destination_pos);

    let mut ff = FlowField::new(grid.size, units.clone(), units.len() as f32, Vec3::ZERO);

    ff.create_integration_field(grid.grid.clone(), destination_cell.idx);
    ff.create_flowfield();

    // Spawn the new flowfield
    // cmds.spawn(flowfield.clone()); // TODO: Uncomment
    let _ff_ent = cmds
        .spawn((
            ff.clone(),
            Name::new("ParentFlowField"),
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    // TODO: Remove (debugging purposes)
    // {
    //     let mesh = Mesh3d(_meshes.add(Cylinder::new(ff.destination_radius, 2.0)));
    //     let material = MeshMaterial3d(_materials.add(Color::srgba(1.0, 1.0, 0.33, 0.85)));
    //     cmds.entity(_ff_ent).with_children(|parent| {
    //         parent.spawn((
    //             DestinationRadius(ff_ent.index()),
    //             mesh,
    //             material,
    //             Transform::from_translation(ff.destination_cell.world_pos),
    //         ));
    //     });
    // }

    cmds.trigger(SetActiveFlowfieldEv(Some(ff)));
}

// TODO: Causes huge performance dip
// Updates integration fields and flowfields whenever a cost field is updated
fn update_fields(
    _trigger: Trigger<UpdateCostEv>,
    mut cmds: Commands,
    mut q_ff: Query<&mut FlowField>,
    grid: Res<Grid>,
) {
    // if there is not FF, then we still want to draw the cost field
    // debug feature only
    if q_ff.is_empty() {
        cmds.trigger(DrawCostFieldEv);
        return;
    }

    let mut active_ff = None;
    for mut ff in q_ff.iter_mut() {
        let dest_idx = ff.destination_cell.idx;
        ff.create_integration_field(grid.grid.clone(), dest_idx);
        ff.create_flowfield();

        active_ff = Some(ff.clone());
    }

    // TODO: This does not work perfectly. It will set the last flowfield as the active one.
    // debug feature only
    cmds.trigger(SetActiveFlowfieldEv(active_ff));
}
