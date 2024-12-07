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

    let mut marker_scale = 0.7;
    if (dbg.draw_mode_1 == DrawMode::None || dbg.draw_mode_2 == DrawMode::None)
        || (dbg.draw_mode_1 == DrawMode::FlowField && dbg.draw_mode_2 == DrawMode::FlowField)
    {
        marker_scale = 1.0;
    }

    let Some(offset) = calculate_offset(&grid, dbg, DrawMode::FlowField) else {
        return;
    };

    let arrow_length = grid.cell_diameter() * 0.6 * marker_scale;
    let arrow_width = grid.cell_diameter() * 0.1 * marker_scale;
    let arrow_clr = Color::WHITE;

    // Create the arrowhead mesh
    let half_arrow_size = arrow_length / 2.0;
    let d1 = half_arrow_size - grid.cell_diameter() * 0.09;
    let d2 = arrow_width + grid.cell_diameter() * 0.0125;
    let a = Vec2::new(half_arrow_size + grid.cell_diameter() * 0.05, 0.0); // Tip of the arrowhead
    let b = Vec2::new(d1, d2);
    let c = Vec2::new(d1, -arrow_width - grid.cell_diameter() * 0.0125);

    // Mesh for arrow
    let arrow_mesh = meshes.add(Plane3d::default().mesh().size(arrow_length, arrow_width));
    let arrow_head_mesh = meshes.add(Triangle2d::new(a, b, c));

    let material = materials.add(StandardMaterial {
        base_color: arrow_clr,
        ..default()
    });

    for cell_row in grid.cur_flowfield.grid.iter() {
        for cell in cell_row.iter() {
            // print!(
            //     "({},{}) {:?},  ",
            //     cell.grid_idx.y, cell.grid_idx.x, cell.best_direction
            // );

            let is_destination_cell = grid.cur_flowfield.destination_cell.grid_idx == cell.grid_idx;

            let rotation = match is_destination_cell {
                true => Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                false => Quat::from_rotation_y(cell.best_direction.to_angle()),
            };

            let mesh = match is_destination_cell {
                true => meshes.add(Circle::new(grid.cell_radius / 3.0 * marker_scale)),
                false => arrow_mesh.clone(),
            };

            let marker = (
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform {
                    translation: cell.world_position + offset,
                    rotation,
                    ..default()
                },
                FlowFieldArrow,
                Name::new("Flowfield Arrow"),
            );

            let arrow_head = (
                Mesh3d(arrow_head_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform {
                    translation: Vec3::ZERO,
                    rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                    ..default()
                },
                Name::new("Arrowhead"),
            );

            if cell.cost < u8::MAX {
                let mut draw = cmds.spawn(marker);

                if !is_destination_cell {
                    draw.with_children(|parent| {
                        parent.spawn(arrow_head);
                    });
                }
            } else {
                let cross = (
                    Transform::default(),
                    Mesh3d(mesh),
                    MeshMaterial3d(material.clone()),
                    FlowFieldArrow,
                    Name::new("Flowfield Marker"),
                );

                let mut cross_1 = cross.clone();
                cross_1.0 = Transform {
                    translation: cell.world_position + offset,
                    rotation: Quat::from_rotation_y(3.0 * FRAC_PI_4),
                    ..default()
                };

                let mut cross_2 = cross.clone();
                cross_2.0 = Transform {
                    translation: cell.world_position + offset,
                    rotation: Quat::from_rotation_y(FRAC_PI_4),
                    ..default()
                };

                cmds.spawn(cross_1);
                cmds.spawn(cross_2);
            }
        }
        // println!();
    }
}
