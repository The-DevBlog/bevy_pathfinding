fn draw_flowfield(
    _trigger: Trigger<DrawDebugEv>,
    dbg: Res<RtsPfDebug>,
    q_grid: Query<&GridController>,
    q_flowfield_arrow: Query<Entity, With<FlowFieldArrow>>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Remove current arrows before rendering new ones
    for arrow_entity in q_flowfield_arrow.iter() {
        cmds.entity(arrow_entity).despawn_recursive();
    }

    let Ok(grid) = q_grid.get_single() else {
        return;
    };

    // Determine scale based on draw modes
    let mut marker_scale = if (dbg.draw_mode_1 == DrawMode::None
        || dbg.draw_mode_2 == DrawMode::None)
        || (dbg.draw_mode_1 == DrawMode::FlowField && dbg.draw_mode_2 == DrawMode::FlowField)
    {
        1.0
    } else {
        0.7
    };

    let Some(offset) = calculate_offset(&grid, dbg, DrawMode::FlowField) else {
        return;
    };

    let cell_size = grid.cell_diameter();
    let arrow_length = cell_size * 0.6 * marker_scale;
    let arrow_width = cell_size * 0.1 * marker_scale;
    let arrow_clr = Color::WHITE;

    // Create arrow meshes
    let arrow_mesh = meshes.add(Plane3d::default().mesh().size(arrow_length, arrow_width));

    // Arrow head calculations
    let half_arrow = arrow_length / 2.0;
    let d1 = half_arrow - cell_size * 0.09;
    let d2 = arrow_width + cell_size * 0.0125;
    let tip = Vec2::new(half_arrow + cell_size * 0.05, 0.0);
    let side1 = Vec2::new(d1, d2);
    let side2 = Vec2::new(d1, -(arrow_width + cell_size * 0.0125));

    let arrow_head_mesh = meshes.add(Triangle2d::new(tip, side1, side2));
    let material = materials.add(StandardMaterial {
        base_color: arrow_clr,
        ..default()
    });

    // Helper closures for spawning markers
    let spawn_destination_marker = |cell_pos: Vec3| {
        let circle_mesh = meshes.add(Circle::new(grid.cell_radius / 3.0 * marker_scale));
        cmds.spawn((
            Mesh3d(circle_mesh),
            MeshMaterial3d(material.clone()),
            Transform {
                translation: cell_pos + offset,
                rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                ..default()
            },
            FlowFieldArrow,
            Name::new("Flowfield Destination Marker"),
        ));
    };

    let spawn_arrow_marker = |cell_pos: Vec3, angle: f32| {
        let mut arrow_cmd = cmds.spawn((
            Mesh3d(arrow_mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform {
                translation: cell_pos + offset,
                rotation: Quat::from_rotation_y(angle),
                ..default()
            },
            FlowFieldArrow,
            Name::new("Flowfield Arrow"),
        ));

        arrow_cmd.with_children(|parent| {
            parent.spawn((
                Mesh3d(arrow_head_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform {
                    rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                    ..default()
                },
                Name::new("Arrowhead"),
            ));
        });
    };

    let spawn_cross_marker = |cell_pos: Vec3| {
        // Two overlapping markers rotated by FRAC_PI_4 to form a cross
        let cross_mesh = meshes.add(Plane3d::default().mesh().size(arrow_length, arrow_width));
        let mut spawn_cross = |rotation_angle: f32| {
            cmds.spawn((
                Mesh3d(cross_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform {
                    translation: cell_pos + offset,
                    rotation: Quat::from_rotation_y(rotation_angle),
                    ..default()
                },
                FlowFieldArrow,
                Name::new("Flowfield Marker"),
            ));
        };
        spawn_cross(3.0 * FRAC_PI_4);
        spawn_cross(FRAC_PI_4);
    };

    // Iterate over cells to spawn appropriate markers
    for cell_row in grid.cur_flowfield.grid.iter() {
        for cell in cell_row.iter() {
            let is_destination = grid.cur_flowfield.destination_cell.grid_idx == cell.grid_idx;
            if is_destination {
                spawn_destination_marker(cell.world_position);
                continue;
            }

            if cell.cost < u8::MAX {
                let angle = cell.best_direction.to_angle();
                spawn_arrow_marker(cell.world_position, angle);
            } else {
                spawn_cross_marker(cell.world_position);
            }
        }
    }
}
