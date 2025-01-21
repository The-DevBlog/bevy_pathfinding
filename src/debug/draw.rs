use std::collections::HashMap;

use super::components::*;
use super::events::*;
use super::resources::*;
use crate::*;
use debug::shader::InstanceMaterialData;
use grid::Grid;

const BASE_SCALE: f32 = 0.2;

pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (detect_debug_change.run_if(resource_exists::<Grid>),),
        )
        .add_observer(trigger_events)
        .add_observer(update_costfield)
        .add_observer(draw_grid)
        .add_observer(set_active_dbg_flowfield)
        .add_observer(draw_costfield)
        .add_observer(draw_flowfield)
        .add_observer(update_flowfield)
        .add_observer(draw_integration_field)
        .add_observer(draw_index);
    }
}

#[derive(Component)]
struct GridLine;

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
        cmds.trigger(DrawAllEv);
    } else {
        // Deactivate if there’s no new flowfield
        if active_dbg_flowfield.0.is_some() {
            active_dbg_flowfield.0 = None;
            cmds.trigger(DrawAllEv);
        }
    }
}

fn trigger_events(_trigger: Trigger<DrawAllEv>, mut cmds: Commands) {
    cmds.trigger(DrawGridEv);
    cmds.trigger(DrawCostFieldEv);
    cmds.trigger(DrawFlowFieldEv);
    cmds.trigger(DrawIntegrationFieldEv);
}

fn draw_grid(
    _trigger: Trigger<DrawGridEv>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    q_grid_lines: Query<Entity, With<GridLine>>,
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
    let mut row_instances = HashMap::new();
    let mut column_instances = HashMap::new();

    let row_count = grid.grid.len() + 1;
    let col_count = grid.grid[0].len() + 1;

    let offset = Vec3::new(-line_length / 2.0, 0.0, -line_length / 2.0);

    // Horizontal lines (rows)
    for row in 0..row_count {
        let y = row as f32 * grid.cell_diameter;

        let mut instance_data = Vec::new();
        instance_data.push(debug::shader::InstanceData {
            id: 0,
            position: Vec3::new(line_length / 2.0, 0.1, y) + offset,
            scale: 1.0,
            rotation: Quat::IDENTITY.into(),
            color: [1.0, 1.0, 1.0, 1.0],
            texture: -4,
        });

        row_instances.insert(-(row as i32), instance_data);
    }

    // Vertical lines (columns)
    for col in 0..col_count {
        let x = col as f32 * grid.cell_diameter;

        let mut instance_data = Vec::new();
        instance_data.push(debug::shader::InstanceData {
            id: 0,
            position: Vec3::new(x, 0.1, line_length / 2.0) + offset,
            scale: 1.0,
            rotation: Quat::IDENTITY.into(),
            color: [1.0, 1.0, 1.0, 1.0],
            texture: -4,
        });

        column_instances.insert(-(col as i32), instance_data);
    }

    cmds.spawn((
        GridLine,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(line_length, 0.2))),
        debug::shader::InstanceMaterialData(row_instances),
    ));

    cmds.spawn((
        GridLine,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(0.2, line_length))),
        debug::shader::InstanceMaterialData(column_instances),
    ));

    dbg.print("draw_grid() end");
}

pub fn draw_flowfield(
    _trigger: Trigger<DrawFlowFieldEv>,
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

    let mut marker_scale = 0.6;
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

    let mut instances = HashMap::new();
    let color = [1.0, 1.0, 1.0, 1.0];

    for cell_row in active_dbg_flowfield.grid.iter() {
        for cell in cell_row.iter() {
            let is_destination_cell = active_dbg_flowfield.destination_cell.idx == cell.idx;
            let id = cell.idx_to_id(grid.grid.len());

            let mut instance_data = Vec::new();

            let flatten = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
            let heading = Quat::from_rotation_z(cell.best_direction.to_angle());
            let rotation = flatten * heading;

            if !is_destination_cell {
                if cell.cost == u8::MAX {
                    instance_data.push(debug::shader::InstanceData {
                        position: cell.world_pos + offset,
                        scale: marker_scale,
                        rotation: flatten.into(),
                        color,
                        texture: -2,
                        id,
                    });
                } else {
                    instance_data.push(debug::shader::InstanceData {
                        position: cell.world_pos + offset,
                        scale: marker_scale,
                        rotation: rotation.into(),
                        color,
                        texture: -1,
                        id: id,
                    });
                }

                instances.insert(id, instance_data);
            } else {
                instance_data.push(debug::shader::InstanceData {
                    position: cell.world_pos + offset,
                    scale: marker_scale * 0.65,
                    rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2).into(),
                    color,
                    texture: -3,
                    id,
                });

                instances.insert(id, instance_data);
            }
        }
    }

    // spawn arrow marker
    cmds.spawn((
        FlowFieldMarker,
        Mesh3d(meshes.add(Rectangle::new(grid.cell_diameter, grid.cell_diameter))),
        debug::shader::InstanceMaterialData(instances),
    ));

    dbg.print("draw_flowfield() end");
}

fn draw_costfield(
    _trigger: Trigger<DrawCostFieldEv>,
    dbg: Res<DebugOptions>,
    mut meshes: ResMut<Assets<Mesh>>,
    grid: Res<Grid>,
    mut cmds: Commands,
    q_cost: Query<Entity, With<CostMarker>>,
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

    let mut instances = HashMap::new();

    for cell_row in &grid.grid {
        for cell in cell_row.iter() {
            let digits_vec: Vec<u32> = cell.cost_to_vec();

            // Calculate spacing and scale based on digit count
            let (digit_spacing, scale_factor) = calculate_digit_spacing_and_scale(
                grid.cell_diameter,
                digits_vec.len(),
                base_digit_spacing,
                BASE_SCALE,
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

            let id = cell.idx_to_id(grid.grid.len());

            let mut instance_data = Vec::new();
            for (i, &digit) in digits_vec.iter().enumerate() {
                let mut offset = base_offset;
                offset.x += x_offset + i as f32 * digit_spacing;

                instance_data.push(debug::shader::InstanceData {
                    position: cell.world_pos + offset,
                    scale: marker_scale,
                    rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2).into(),
                    color: [1.0, 1.0, 1.0, 1.0],
                    texture: digit as i32,
                    id,
                });
            }

            instances.insert(id, instance_data);
        }
    }

    cmds.spawn((
        CostMarker,
        Mesh3d(meshes.add(Rectangle::new(grid.cell_diameter, grid.cell_diameter))),
        debug::shader::InstanceMaterialData(instances),
    ));

    dbg.print("draw_costfield() end");
}

fn draw_integration_field(
    _trigger: Trigger<DrawIntegrationFieldEv>,
    dbg: Res<DebugOptions>,
    active_dbg_flowfield: Res<ActiveDebugFlowfield>,
    mut meshes: ResMut<Assets<Mesh>>,
    q_cost: Query<Entity, With<BestCostMarker>>,
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

    let mut instances = HashMap::new();

    for cell_row in &flowfield.grid {
        for cell in cell_row.iter() {
            let digits_vec: Vec<u32> = cell.best_cost_to_vec();

            // Calculate spacing and scale based on digit count
            let (digit_spacing, scale_factor) = calculate_digit_spacing_and_scale(
                grid.cell_diameter,
                digits_vec.len(),
                base_digit_spacing,
                BASE_SCALE,
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

            let id = cell.idx_to_id(grid.grid.len());

            let mut instance_data = Vec::new();
            for (i, &digit) in digits_vec.iter().enumerate() {
                let mut offset = base_offset;
                offset.x += x_offset + i as f32 * digit_spacing;

                instance_data.push(debug::shader::InstanceData {
                    position: cell.world_pos + offset,
                    scale: marker_scale,
                    rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2).into(),
                    color: [1.0, 1.0, 1.0, 1.0],
                    texture: digit as i32,
                    id,
                });
            }

            instances.insert(id, instance_data);
        }
    }

    cmds.spawn((
        BestCostMarker,
        Mesh3d(meshes.add(Rectangle::new(grid.cell_diameter, grid.cell_diameter))),
        debug::shader::InstanceMaterialData(instances),
    ));

    dbg.print("draw_integration_field() end");
}

fn draw_index(
    _trigger: Trigger<DrawAllEv>,
    dbg: Res<DebugOptions>,
    mut meshes: ResMut<Assets<Mesh>>,
    grid: Res<Grid>,
    q_idx: Query<Entity, With<IndexMarker>>,
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
    let mut instances = HashMap::new();

    for cell_row in &grid.grid {
        for cell in cell_row.iter() {
            let digits_vec: Vec<u32> = format!("{}{}", cell.idx.y, cell.idx.x)
                .chars()
                .filter_map(|c| c.to_digit(10))
                .collect();

            // Calculate spacing and scale based on digit count
            let (digit_spacing, scale_factor) = calculate_digit_spacing_and_scale(
                grid.cell_diameter,
                digits_vec.len(),
                base_digit_spacing,
                BASE_SCALE,
            );

            // Adjust marker_scale based on draw mode
            let mut marker_scale = scale_factor;
            if (dbg.draw_mode_1 == DrawMode::None || dbg.draw_mode_2 == DrawMode::None)
                || (dbg.draw_mode_1 == DrawMode::FlowField
                    && dbg.draw_mode_2 == DrawMode::FlowField)
            {
                marker_scale = scale_factor * 1.25;
            }

            let x_offset = if digits_vec.len() > 1 {
                -(digits_vec.len() as f32 - 1.0) * digit_spacing / 2.0
            } else {
                0.0
            };

            let mut instance_data = Vec::new();
            let id = cell.idx_to_id(grid.grid.len());

            for (i, &digit) in digits_vec.iter().enumerate() {
                let mut offset = base_offset;
                offset.x += x_offset + i as f32 * digit_spacing;

                instance_data.push(debug::shader::InstanceData {
                    position: cell.world_pos + offset,
                    scale: marker_scale,
                    rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2).into(),
                    color: [1.0, 1.0, 1.0, 1.0],
                    texture: digit as i32,
                    id,
                });
            }

            instances.insert(id, instance_data);
        }
    }

    cmds.spawn((
        IndexMarker,
        Mesh3d(meshes.add(Rectangle::new(grid.cell_diameter, grid.cell_diameter))),
        debug::shader::InstanceMaterialData(instances),
    ));

    dbg.print("draw_index() end");
}

fn update_flowfield(
    trigger: Trigger<UpdateCostEv>,
    grid: Res<Grid>,
    active_dbg_flowfield: Res<ActiveDebugFlowfield>,
    mut q_instance: Query<&mut InstanceMaterialData, With<FlowFieldMarker>>,
    dbg: Res<DebugOptions>,
) {
    let base_offset = calculate_offset(grid.cell_diameter, &dbg, DrawMode::FlowField);
    let Some(base_offset) = base_offset else {
        return;
    };

    let Some(active_flowfield) = &active_dbg_flowfield.0 else {
        return;
    };

    let cell = trigger.cell;
    let cell = active_flowfield.grid[cell.idx.y as usize][cell.idx.x as usize];

    let id = cell.idx_to_id(grid.grid.len());

    let Ok(mut instance) = q_instance.get_single_mut() else {
        return;
    };

    let Some(instance) = instance.0.get_mut(&id) else {
        return;
    };

    let mut marker_scale = 0.6;
    if (dbg.draw_mode_1 == DrawMode::None || dbg.draw_mode_2 == DrawMode::None)
        || (dbg.draw_mode_1 == DrawMode::FlowField && dbg.draw_mode_2 == DrawMode::FlowField)
    {
        marker_scale = 1.0;
    }

    let flatten = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    let heading = Quat::from_rotation_z(cell.best_direction.to_angle());
    let rotation = flatten * heading;

    let mut new_instances = Vec::new();

    // occupied cell
    if cell.cost == u8::MAX {
        // println!("Updating cell at max cost");
        new_instances.push(debug::shader::InstanceData {
            position: cell.world_pos + base_offset,
            scale: marker_scale,
            rotation: flatten.into(),
            color: [1.0, 1.0, 1.0, 1.0],
            texture: -2,
            id,
        });
    } else {
        // println!("Updating cell to min cost");
        new_instances.push(debug::shader::InstanceData {
            position: cell.world_pos + base_offset,
            scale: marker_scale,
            rotation: rotation.into(),
            color: [1.0, 1.0, 1.0, 1.0],
            texture: -1,
            id: id,
        });
    }

    *instance = new_instances;

    // let flatten = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    // let heading = Quat::from_rotation_z(cell.best_direction.to_angle());
    // let rotation = flatten * heading;

    // let mut arrow_instance_data = Vec::new();
    // arrow_instance_data.push(debug::shader::InstanceData {
    //     position: cell.world_pos + offset,
    //     scale: marker_scale,
    //     rotation: rotation.into(),
    //     color,
    //     texture: -1,
    //     id: id,
    // });
}

fn update_costfield(
    trigger: Trigger<UpdateCostEv>,
    grid: Res<Grid>,
    mut q_instance: Query<&mut InstanceMaterialData, With<CostMarker>>,
    dbg: Res<DebugOptions>,
) {
    let base_digit_spacing = grid.cell_diameter * 0.275; // TODO move to constant

    let base_offset = calculate_offset(grid.cell_diameter, &dbg, DrawMode::CostField);
    let Some(base_offset) = base_offset else {
        return;
    };

    let cell = trigger.cell;
    let id = cell.idx_to_id(grid.grid.len());

    let Ok(mut instance) = q_instance.get_single_mut() else {
        return;
    };
    let Some(instance) = instance.0.get_mut(&id) else {
        return;
    };

    let digits_vec: Vec<u32> = cell.cost_to_vec();

    let (digit_spacing, scale_factor) = calculate_digit_spacing_and_scale(
        grid.cell_diameter,
        digits_vec.len(),
        base_digit_spacing,
        BASE_SCALE,
    );

    // Adjust marker_scale based on draw mode
    let mut marker_scale = scale_factor;
    if (dbg.draw_mode_1 == DrawMode::None || dbg.draw_mode_2 == DrawMode::None)
        || (dbg.draw_mode_1 == DrawMode::FlowField && dbg.draw_mode_2 == DrawMode::FlowField)
    {
        marker_scale = scale_factor * 1.25;
    }

    let x_offset = -(digits_vec.len() as f32 - 1.0) * digit_spacing / 2.0;

    let mut new_instances = Vec::new();
    for (i, &digit) in digits_vec.iter().enumerate() {
        let mut offset = base_offset;
        offset.x += x_offset + i as f32 * digit_spacing;

        new_instances.push(debug::shader::InstanceData {
            position: cell.world_pos + offset,
            scale: marker_scale,
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2).into(),
            color: [1.0, 1.0, 1.0, 1.0],
            texture: digit as i32,
            id,
        });
    }

    *instance = new_instances;
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
        cmds.trigger(DrawAllEv);
    }
}
