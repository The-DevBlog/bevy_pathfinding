use bevy::color::palettes;
use bevy_mod_billboard::{Billboard, BillboardDepth, BillboardTextBundle};
use bevy_rapier3d::na::Translation;
use grid_controller::GridController;

use crate::*;

pub struct BevyRtsPathFindingDebugPlugin;

impl Plugin for BevyRtsPathFindingDebugPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Update, (draw_flowfield, draw_grid));
        app.add_systems(Update, draw_grid).observe(draw_costfield);
    }
}

#[derive(Event)]
pub struct DrawCostFieldEv;

fn draw_grid(grid_controller: Query<&GridController>, mut gizmos: Gizmos) {
    let grid = grid_controller.get_single().unwrap();

    gizmos.grid(
        Vec3::ZERO,
        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        UVec2::new(grid.grid_size.x as u32, grid.grid_size.y as u32),
        Vec2::new(grid.cell_radius * 2., grid.cell_radius * 2.),
        COLOR_GRID,
    );
}

fn draw_costfield(
    _trigger: Trigger<DrawCostFieldEv>,
    mut cmds: Commands,
    grid_controller: Query<&GridController>,
) {
    let grid = grid_controller.get_single().unwrap();

    // Calculate the offset based on the map size
    let offset_x = -grid.map_size.x / 2.;
    let offset_z = -grid.map_size.y / 2.;

    for cell_row in grid.current_flowfield.grid.iter() {
        for cell in cell_row.iter() {
            let adjusted_position = Vec3::new(
                cell.world_position.x + offset_x,
                cell.world_position.y,
                cell.world_position.z + offset_z,
            );

            let cost = BillboardTextBundle {
                billboard_depth: BillboardDepth(false),
                text: Text::from_section(
                    cell.cost.to_string(),
                    TextStyle {
                        color: COLOR_COST.into(),
                        font_size: 100.0,
                        ..default()
                    },
                ),
                transform: Transform {
                    translation: adjusted_position,
                    scale: Vec3::splat(0.035),
                    ..default()
                },
                ..default()
            };

            cmds.spawn(cost);
        }
    }
}

// fn draw_flowfield(
//     flowfield_q: Query<&FlowField>,
//     selected_q: Query<Entity, With<Selected>>,
//     grid: Res<Grid>,
//     mut gizmos: Gizmos,
// ) {
//     if selected_q.is_empty() {
//         return;
//     }

//     let arrow_length = grid.cell_size * 0.75 / 2.0;

//     let mut selected_entity_ids = Vec::new();
//     for selected_entity in selected_q.iter() {
//         selected_entity_ids.push(selected_entity);
//     }

//     for flowfield in flowfield_q.iter() {
//         if !selected_entity_ids
//             .iter()
//             .any(|item| flowfield.entities.contains(item))
//         {
//             continue;
//         }

//         for x in 0..grid.rows {
//             for z in 0..grid.columns {
//                 let cell = &flowfield.cells[x][z];
//                 if cell.occupied || cell.flow_vector == Vec3::ZERO {
//                     // Draw an 'X' for each occupied cell
//                     let top_left =
//                         cell.world_position + Vec3::new(-arrow_length, 0.0, -arrow_length);
//                     let top_right =
//                         cell.world_position + Vec3::new(arrow_length, 0.0, -arrow_length);
//                     let bottom_left =
//                         cell.world_position + Vec3::new(-arrow_length, 0.0, arrow_length);
//                     let bottom_right =
//                         cell.world_position + Vec3::new(arrow_length, 0.0, arrow_length);

//                     gizmos.line(top_left, bottom_right, RED);
//                     gizmos.line(top_right, bottom_left, RED);
//                     continue;
//                 }

//                 let flow_direction = cell.flow_vector.normalize();

//                 let start = cell.world_position - flow_direction * arrow_length;
//                 let end = cell.world_position + flow_direction * arrow_length;

//                 gizmos.arrow(start, end, COLOR_ARROWS);
//             }
//         }
//     }
// }

// fn draw_grid(mut gizmos: Gizmos, grid: Res<Grid>) {
//     gizmos.grid(
//         Vec3::ZERO,
//         Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
//         UVec2::new(grid.rows as u32, grid.columns as u32),
//         Vec2::new(grid.cell_size, grid.cell_size),
//         COLOR_GRID,
//     );
// }
