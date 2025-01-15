use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use super::components::*;
use super::events::*;
use super::resources::*;
use crate::*;
use cell::Cell;
use events::UpdateCostEv;
use grid::Grid;

const BASE_SCALE: f32 = 0.25;

pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // draw_grid.run_if(resource_exists::<Grid>),
                detect_debug_change.run_if(resource_exists::<Grid>),
                // update_cell_cost
                //     .after(grid::update_costs)
                //     .run_if(resource_exists::<Grid>),
            ),
        )
        .add_observer(draw_grid)
        // .add_systems(Update, draw_flowfield.run_if(resource_exists::<Grid>))
        .add_observer(set_active_dbg_flowfield)
        .add_observer(draw_costfield)
        .add_observer(draw_flowfield)
        .add_observer(draw_integration_field)
        .add_observer(draw_index);
    }
}

fn set_active_dbg_flowfield(
    trigger: Trigger<SetActiveFlowfieldEv>,
    mut cmds: Commands,
    mut active_dbg_flowfield: ResMut<ActiveDebugFlowfield>,
) {
    if let Some(new_flowfield) = &trigger.event().0 {
        if let Some(current_flowfield) = &active_dbg_flowfield.0 {
            // Skip if the grid is the same
            if current_flowfield.grid == new_flowfield.grid {
                return;
            }
        }
        // Set the new flowfield and trigger debug draw
        active_dbg_flowfield.0 = Some(new_flowfield.clone());
        cmds.trigger(DrawDebugEv);
    } else {
        // Deactivate if there’s no new flowfield
        if active_dbg_flowfield.0.is_some() {
            active_dbg_flowfield.0 = None;
            cmds.trigger(DrawDebugEv);
        }
    }
}

#[derive(Component)]
struct GridMarker;

fn draw_grid(
    _trigger: Trigger<DrawDebugEv>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    q_grid_lines: Query<Entity, With<GridMarker>>,
    grid: Res<Grid>,
    dbg: Res<DebugOptions>,
) {
    // Remove old grid lines before re-drawing
    for line_entity in &q_grid_lines {
        cmds.entity(line_entity).despawn_recursive();
    }

    if !dbg.draw_grid {
        return;
    }

    dbg.print("\ndraw_grid() start");

    let line_length = grid.size.x as f32 * grid.cell_diameter;
    let mut row_instances = Vec::new();
    let mut column_instances = Vec::new();

    let row_count = grid.grid.len() + 1;
    let col_count = grid.grid[0].len() + 1;

    let offset = Vec3::new(-line_length / 2.0, 0.0, -line_length / 2.0);

    // Horizontal lines (rows)
    for row in 0..row_count {
        let y = row as f32 * grid.cell_diameter;

        row_instances.push(debug::shader::InstanceData {
            position: Vec3::new(line_length / 2.0, 0.1, y) + offset,
            scale: 1.0,
            rotation: Quat::IDENTITY.into(),
            color: [1.0, 1.0, 1.0, 1.0],
            digit: -1.0,
        });
    }

    // Vertical lines (columns)
    for col in 0..col_count {
        let x = col as f32 * grid.cell_diameter;

        column_instances.push(debug::shader::InstanceData {
            position: Vec3::new(x, 0.1, line_length / 2.0) + offset,
            scale: 1.0,
            rotation: Quat::IDENTITY.into(),
            color: [1.0, 1.0, 1.0, 1.0],
            digit: -1.0,
        });
    }

    cmds.spawn((
        GridMarker,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(line_length, 0.2))),
        debug::shader::InstanceMaterialData(row_instances),
    ));

    cmds.spawn((
        GridMarker,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(0.2, line_length))),
        debug::shader::InstanceMaterialData(column_instances),
    ));

    dbg.print("draw_grid() end");
}

pub fn draw_flowfield(
    _trigger: Trigger<DrawDebugEv>,
    dbg: Res<DebugOptions>,
    grid: Res<Grid>,
    active_dbg_flowfield: Res<ActiveDebugFlowfield>,
    q_flowfield_arrow: Query<Entity, With<FlowFieldMarker>>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Remove current arrows before rendering new ones
    for arrow_entity in &q_flowfield_arrow {
        cmds.entity(arrow_entity).despawn_recursive();
    }

    let Some(active_dbg_flowfield) = &active_dbg_flowfield.0 else {
        return;
    };

    let mut marker_scale = 0.7;
    if (dbg.draw_mode_1 == DrawMode::None || dbg.draw_mode_2 == DrawMode::None)
        || (dbg.draw_mode_1 == DrawMode::FlowField && dbg.draw_mode_2 == DrawMode::FlowField)
    {
        marker_scale = 1.0;
    }

    let offset = calculate_offset(
        active_dbg_flowfield.cell_diameter,
        &dbg,
        DrawMode::FlowField,
    );
    let Some(offset) = offset else {
        return;
    };

    dbg.print("\ndraw_flowfield() start");

    // Arrow properties
    let arrow_length = grid.cell_diameter * 0.6 * marker_scale;
    let arrow_width = grid.cell_diameter * 0.1 * marker_scale;

    // Create the arrowhead mesh
    let half_arrow_size = arrow_length / 2.0;
    let d1 = half_arrow_size - grid.cell_diameter * 0.09;
    let d2 = arrow_width + grid.cell_diameter * 0.0125;
    let a = Vec2::new(half_arrow_size + grid.cell_diameter * 0.05, 0.0); // Tip of the arrowhead
    let b = Vec2::new(d1, d2);
    let c = Vec2::new(d1, -arrow_width - grid.cell_diameter * 0.0125);

    // Mesh for arrow
    let arrow_shaft_mesh = meshes.add(Plane3d::default().mesh().size(arrow_length, arrow_width));
    let arrow_head_mesh = meshes.add(Triangle2d::new(a, b, c));

    // Instance data for all arrows
    let mut arrow_shaft_instances = Vec::new();
    let mut arrow_head_instances = Vec::new();
    let mut destination_instances = Vec::new();
    let mut occupied_cell_instances = Vec::new();

    for cell_row in active_dbg_flowfield.grid.iter() {
        for cell in cell_row.iter() {
            let is_destination_cell = active_dbg_flowfield.destination_cell.idx == cell.idx;

            let rotation = match is_destination_cell {
                true => Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                false => Quat::from_rotation_y(cell.best_direction.to_angle()),
            };

            let color = if cell.cost < u8::MAX {
                [1.0, 1.0, 1.0, 1.0] // White for valid cells
            } else {
                [1.0, 0.0, 0.0, 1.0] // Red for blocked cells
            };

            if !is_destination_cell {
                if cell.cost == u8::MAX {
                    occupied_cell_instances.push(debug::shader::InstanceData {
                        position: cell.world_pos + offset,
                        scale: marker_scale,
                        rotation: Quat::from_rotation_y(3.0 * FRAC_PI_4).into(),
                        color,
                        digit: -1.0,
                    });

                    occupied_cell_instances.push(debug::shader::InstanceData {
                        position: cell.world_pos + offset,
                        scale: marker_scale,
                        rotation: Quat::from_rotation_y(FRAC_PI_4).into(),
                        color,
                        digit: -1.0,
                    });

                    continue;
                }

                arrow_shaft_instances.push(debug::shader::InstanceData {
                    position: cell.world_pos + offset,
                    scale: marker_scale,
                    rotation: rotation.into(),
                    color,
                    digit: -1.0,
                });

                // Then push this final rotation into your instance data
                arrow_head_instances.push(debug::shader::InstanceData {
                    position: cell.world_pos + offset,
                    scale: 1.0,
                    rotation: (rotation * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                        .into(),
                    color,
                    digit: -1.0,
                });
            }

            if is_destination_cell {
                destination_instances.push(debug::shader::InstanceData {
                    position: cell.world_pos + offset,
                    scale: grid.cell_radius * 0.15 * marker_scale,
                    rotation: rotation.into(),
                    color,
                    digit: -1.0,
                });
            }
        }
    }

    // spawn occupied cell marker (if there are any)
    if !occupied_cell_instances.is_empty() {
        cmds.spawn((
            FlowFieldMarker,
            Mesh3d(arrow_shaft_mesh.clone()),
            debug::shader::InstanceMaterialData(occupied_cell_instances),
        ));
    }

    // spawn arrow shaft marker
    cmds.spawn((
        FlowFieldMarker,
        Mesh3d(arrow_shaft_mesh),
        debug::shader::InstanceMaterialData(arrow_shaft_instances),
    ));

    // spawn arrow head marker
    cmds.spawn((
        FlowFieldMarker,
        Mesh3d(arrow_head_mesh),
        debug::shader::InstanceMaterialData(arrow_head_instances),
    ));

    // spawn destination cell marker
    cmds.spawn((
        FlowFieldMarker,
        Mesh3d(meshes.add(Circle::new(grid.cell_radius / 3.0 * marker_scale))),
        debug::shader::InstanceMaterialData(destination_instances),
    ));

    dbg.print("draw_flowfield() end");
}

fn draw_costfield(
    _trigger: Trigger<DrawDebugEv>,
    mut costmap: ResMut<CostMap>,
    dbg: Res<DebugOptions>,
    mut meshes: ResMut<Assets<Mesh>>,
    grid: Res<Grid>,
    mut cmds: Commands,
    q_cost: Query<Entity, With<Cost>>,
) {
    // Remove current cost field before rendering new one
    for cost_entity in &q_cost {
        cmds.entity(cost_entity).despawn_recursive();
    }

    let base_offset = calculate_offset(grid.cell_diameter, &dbg, DrawMode::CostField);
    let Some(base_offset) = base_offset else {
        return;
    };

    dbg.print("\ndraw_costfield() start");

    let base_digit_spacing = grid.cell_diameter * 0.275;

    let mut instances = Vec::new();

    for cell_row in &grid.grid {
        for cell in cell_row.iter() {
            let digits_vec: Vec<u32> = cell
                .cost
                .to_string()
                .chars()
                .filter_map(|c| c.to_digit(10))
                .collect();

            // Calculate spacing and scale based on digit count
            let (digit_spacing, scale_factor) = calculate_digit_spacing_and_scale(
                grid.cell_diameter,
                digits_vec.len(),
                base_digit_spacing,
                0.2, // Base scale, adjust as needed
            );

            // Adjust marker_scale based on draw mode
            let mut marker_scale = scale_factor;
            if (dbg.draw_mode_1 == DrawMode::None || dbg.draw_mode_2 == DrawMode::None)
                || (dbg.draw_mode_1 == DrawMode::FlowField
                    && dbg.draw_mode_2 == DrawMode::FlowField)
            {
                marker_scale = scale_factor * 1.25; // Adjust multiplier as needed
            }

            let x_offset = -(digits_vec.len() as f32 - 1.0) * digit_spacing / 2.0;

            for (i, &digit) in digits_vec.iter().enumerate() {
                let mut offset = base_offset;
                offset.x += x_offset + i as f32 * digit_spacing;

                instances.push(debug::shader::InstanceData {
                    position: cell.world_pos + offset,
                    scale: marker_scale,
                    rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2).into(),
                    color: [1.0, 1.0, 1.0, 1.0],
                    digit: digit as f32,
                });
            }
        }
    }

    cmds.spawn((
        Cost,
        Mesh3d(meshes.add(Rectangle::new(grid.cell_diameter, grid.cell_diameter))),
        debug::shader::InstanceMaterialData(instances),
    ));

    dbg.print("draw_costfield() end");
}

fn draw_integration_field(
    _trigger: Trigger<DrawDebugEv>,
    dbg: Res<DebugOptions>,
    active_dbg_flowfield: Res<ActiveDebugFlowfield>,
    mut meshes: ResMut<Assets<Mesh>>,
    q_cost: Query<Entity, With<BestCost>>,
    grid: Res<Grid>,
    mut cmds: Commands,
) {
    // Remove current cost field before rendering new one
    for cost_entity in &q_cost {
        cmds.entity(cost_entity).despawn_recursive();
    }

    let Some(flowfield) = &active_dbg_flowfield.0 else {
        return;
    };

    let base_offset = calculate_offset(flowfield.cell_diameter, &dbg, DrawMode::IntegrationField);
    let Some(base_offset) = base_offset else {
        return;
    };

    dbg.print("\ndraw_integration_field() start");

    let base_digit_spacing = grid.cell_diameter * 0.275;

    let mut instances = Vec::new();

    for cell_row in &flowfield.grid {
        for cell in cell_row.iter() {
            let digits_vec: Vec<u32> = cell
                .best_cost
                .to_string()
                .chars()
                .filter_map(|c| c.to_digit(10))
                .collect();

            // Calculate spacing and scale based on digit count
            let (digit_spacing, scale_factor) = calculate_digit_spacing_and_scale(
                grid.cell_diameter,
                digits_vec.len(),
                base_digit_spacing,
                0.2, // Base scale, adjust as needed
            );

            // Adjust marker_scale based on draw mode
            let mut marker_scale = scale_factor;
            if (dbg.draw_mode_1 == DrawMode::None || dbg.draw_mode_2 == DrawMode::None)
                || (dbg.draw_mode_1 == DrawMode::FlowField
                    && dbg.draw_mode_2 == DrawMode::FlowField)
            {
                marker_scale = scale_factor * 1.25; // Adjust multiplier as needed
            }

            let x_offset = -(digits_vec.len() as f32 - 1.0) * digit_spacing / 2.0;

            for (i, &digit) in digits_vec.iter().enumerate() {
                let mut offset = base_offset;
                offset.x += x_offset + i as f32 * digit_spacing;

                instances.push(debug::shader::InstanceData {
                    position: cell.world_pos + offset,
                    scale: marker_scale,
                    rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2).into(),
                    color: [1.0, 1.0, 1.0, 1.0],
                    digit: digit as f32,
                });
            }
        }
    }

    cmds.spawn((
        BestCost,
        Mesh3d(meshes.add(Rectangle::new(grid.cell_diameter, grid.cell_diameter))),
        debug::shader::InstanceMaterialData(instances),
    ));

    dbg.print("draw_integration_field() end");
}

fn draw_index(
    _trigger: Trigger<DrawDebugEv>,
    dbg: Res<DebugOptions>,
    mut meshes: ResMut<Assets<Mesh>>,
    grid: Res<Grid>,
    q_idx: Query<Entity, With<Index>>,
    mut cmds: Commands,
) {
    // Remove current index entities before rendering new ones
    for idx_entity in &q_idx {
        cmds.entity(idx_entity).despawn_recursive();
    }

    if dbg.draw_mode_1 != DrawMode::Index && dbg.draw_mode_2 != DrawMode::Index {
        return;
    }

    let base_offset = calculate_offset(grid.cell_diameter, &dbg, DrawMode::Index);
    let Some(base_offset) = base_offset else {
        return;
    };

    dbg.print("\ndraw_index() start");

    let base_digit_spacing = grid.cell_diameter * 0.275; // Consider moving to a constant
    let mut instances = Vec::new();

    for cell_row in &grid.grid {
        for cell in cell_row.iter() {
            let mut digits_vec: Vec<u32> = cell
                .idx
                .to_string()
                .chars()
                .filter_map(|c| c.to_digit(10))
                .collect();

            digits_vec.reverse();

            // Calculate spacing and scale based on digit count
            let (digit_spacing, scale_factor) = calculate_digit_spacing_and_scale(
                grid.cell_diameter,
                digits_vec.len(),
                base_digit_spacing,
                0.2, // Base scale, adjust as needed
            );

            // Adjust marker_scale based on draw mode
            let mut marker_scale = scale_factor;
            if (dbg.draw_mode_1 == DrawMode::None || dbg.draw_mode_2 == DrawMode::None)
                || (dbg.draw_mode_1 == DrawMode::FlowField
                    && dbg.draw_mode_2 == DrawMode::FlowField)
            {
                marker_scale = scale_factor * 1.25; // Adjust multiplier as needed
            }

            let x_offset = if digits_vec.len() > 1 {
                -(digits_vec.len() as f32 - 1.0) * digit_spacing / 2.0
            } else {
                0.0
            };

            for (i, &digit) in digits_vec.iter().enumerate() {
                let mut offset = base_offset;
                offset.x += x_offset + i as f32 * digit_spacing;

                instances.push(debug::shader::InstanceData {
                    position: cell.world_pos + offset,
                    scale: marker_scale,
                    rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2).into(),
                    color: [1.0, 1.0, 1.0, 1.0],
                    digit: digit as f32,
                });
            }
        }
    }

    cmds.spawn((
        Index,
        Mesh3d(meshes.add(Rectangle::new(grid.cell_diameter, grid.cell_diameter))),
        debug::shader::InstanceMaterialData(instances),
    ));

    dbg.print("draw_index() end");
}

fn calculate_offset(
    cell_diameter: f32,
    dbg: &Res<DebugOptions>,
    draw_mode: DrawMode,
) -> Option<Vec3> {
    let mode = if dbg.draw_mode_1 == draw_mode {
        Some(1)
    } else if dbg.draw_mode_2 == draw_mode {
        Some(2)
    } else {
        None
    };

    if mode.is_none() {
        return None; // nothing to draw
    }

    // Base offset when only one mode is active
    let mut offset = Vec3::new(0.0, 0.01, 0.0);
    if (dbg.draw_mode_1 == DrawMode::None || dbg.draw_mode_2 == DrawMode::None)
        || (dbg.draw_mode_1 == draw_mode && dbg.draw_mode_2 == draw_mode)
    {
        offset.z = 0.0;
    } else {
        match mode {
            Some(1) => offset.z = -cell_diameter * 0.25,
            Some(2) => offset.z = cell_diameter * 0.25,
            _ => (),
        };
    }

    return Some(offset);
}

fn calculate_digit_spacing_and_scale(
    cell_diameter: f32,
    digit_count: usize,
    base_digit_spacing: f32,
    base_scale: f32,
) -> (f32, f32) {
    let digit_width = cell_diameter * base_scale;
    let total_digit_width = digit_count as f32 * digit_width;
    let total_spacing_width = if digit_count > 1 {
        (digit_count as f32 - 1.0) * base_digit_spacing
    } else {
        0.0
    };
    let total_width = total_digit_width + total_spacing_width;

    if total_width > cell_diameter {
        let scale = cell_diameter / total_width;
        let adjusted_spacing = base_digit_spacing * scale;
        let adjusted_scale = base_scale * scale;
        (adjusted_spacing, adjusted_scale)
    } else {
        (base_digit_spacing, base_scale)
    }
}

fn detect_debug_change(mut cmds: Commands, debug: Res<DebugOptions>) {
    if debug.is_changed() {
        cmds.trigger(DrawDebugEv);
    }
}
