use crate::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_flowfield, draw_grid));
    }
}

// TODO: This is VERY expensive. Make optional
fn draw_flowfield(
    flowfield_q: Query<&FlowField>,
    selected_q: Query<Entity, With<Selected>>,
    grid: Res<Grid>,
    mut gizmos: Gizmos,
) {
    if selected_q.is_empty() {
        return;
    }

    let mut selected_entity_ids = Vec::new();
    for selected_entity in selected_q.iter() {
        selected_entity_ids.push(selected_entity);
    }

    let arrow_len = CELL_SIZE * 0.75 / 2.0;
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
                    let top_left = cell.position + Vec3::new(-arrow_len, 0.0, -arrow_len);
                    let top_right = cell.position + Vec3::new(arrow_len, 0.0, -arrow_len);
                    let bottom_left = cell.position + Vec3::new(-arrow_len, 0.0, arrow_len);
                    let bottom_right = cell.position + Vec3::new(arrow_len, 0.0, arrow_len);

                    gizmos.line(top_left, bottom_right, RED);
                    gizmos.line(top_right, bottom_left, RED);

                    continue;
                }

                let flow_direction = cell.flow_vector.normalize();

                let start = cell.position - flow_direction * arrow_len;
                let end = cell.position + flow_direction * arrow_len;

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
        Vec2::new(CELL_SIZE, CELL_SIZE),
        COLOR_GRID,
    );
}
