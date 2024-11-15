use crate::*;

pub struct BevyRtsPathFindingDebugPlugin;

impl Plugin for BevyRtsPathFindingDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_flowfield, draw_grid));
    }
}

fn draw_flowfield(
    flowfield_q: Query<&FlowField>,
    selected_q: Query<Entity, With<Selected>>,
    grid: Res<Grid>,
    mut gizmos: Gizmos,
) {
    if selected_q.is_empty() {
        return;
    }

    let arrow_length = grid.cell_size * 0.75 / 2.0;

    let mut selected_entity_ids = Vec::new();
    for selected_entity in selected_q.iter() {
        selected_entity_ids.push(selected_entity);
    }

    for flowfield in flowfield_q.iter() {
        if !selected_entity_ids
            .iter()
            .any(|item| flowfield.entities.contains(item))
        {
            continue;
        }

        for x in 0..grid.rows {
            for z in 0..grid.columns {
                let cell = &flowfield.cells[x][z];
                if cell.occupied || cell.flow_vector == Vec3::ZERO {
                    // Draw an 'X' for each occupied cell
                    let top_left =
                        cell.world_position + Vec3::new(-arrow_length, 0.0, -arrow_length);
                    let top_right =
                        cell.world_position + Vec3::new(arrow_length, 0.0, -arrow_length);
                    let bottom_left =
                        cell.world_position + Vec3::new(-arrow_length, 0.0, arrow_length);
                    let bottom_right =
                        cell.world_position + Vec3::new(arrow_length, 0.0, arrow_length);

                    gizmos.line(top_left, bottom_right, RED);
                    gizmos.line(top_right, bottom_left, RED);
                    continue;
                }

                let flow_direction = cell.flow_vector.normalize();

                let start = cell.world_position - flow_direction * arrow_length;
                let end = cell.world_position + flow_direction * arrow_length;

                gizmos.arrow(start, end, COLOR_ARROWS);
            }
        }
    }
}

fn draw_grid(mut gizmos: Gizmos, grid: Res<Grid>) {
    gizmos.grid(
        Vec3::ZERO,
        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        UVec2::new(grid.rows as u32, grid.columns as u32),
        Vec2::new(grid.cell_size, grid.cell_size),
        COLOR_GRID,
    );
}
