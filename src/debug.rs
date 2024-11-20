use crate::*;
use bevy_mod_billboard::*;
use grid_controller::GridController;

const FONT: &[u8] = include_bytes!("../assets/fonts/FiraSans-Bold.ttf");

pub struct BevyRtsPathFindingDebugPlugin;

impl Plugin for BevyRtsPathFindingDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RtsPfDebug>()
            .register_type::<RtsPfDebug>()
            .add_systems(Update, draw_grid)
            .observe(draw_costfield)
            .observe(draw_flowfield);
    }
}

#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct RtsPfDebug {
    pub draw_grid: bool,
    pub draw_flowfield: bool,
    pub draw_costfield: bool,
    pub draw_integration_field: bool,
}

impl Default for RtsPfDebug {
    fn default() -> Self {
        RtsPfDebug {
            draw_grid: true,
            draw_flowfield: true,
            draw_costfield: true,
            draw_integration_field: false,
        }
    }
}

#[derive(Event)]
pub struct DrawDebugEv;

#[derive(Component)]
struct CostField;

#[derive(Component)]
struct FlowFieldArrow;

fn draw_grid(grid_controller: Query<&GridController>, mut gizmos: Gizmos, debug: Res<RtsPfDebug>) {
    if !debug.draw_grid {
        return;
    }

    let grid = grid_controller.get_single().unwrap();

    gizmos.grid(
        Vec3::ZERO,
        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        UVec2::new(grid.grid_size.x as u32, grid.grid_size.y as u32),
        Vec2::new(grid.cell_radius * 2., grid.cell_radius * 2.),
        COLOR_GRID,
    );
}

fn draw_integration_field(_trigger: Trigger<DrawDebugEv>, debug: Res<RtsPfDebug>) {
    if !debug.draw_integration_field {
        return;
    }
}

fn draw_flowfield(
    _trigger: Trigger<DrawDebugEv>,
    debug: Res<RtsPfDebug>,
    q_grid_controller: Query<&GridController>,
    q_flowfield_arrow: Query<Entity, With<FlowFieldArrow>>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Remove current arrows before rendering new ones
    for arrow_entity in q_flowfield_arrow.iter() {
        cmds.entity(arrow_entity).despawn_recursive();
    }

    if !debug.draw_flowfield {
        return;
    }

    let grid_controller = q_grid_controller.get_single().unwrap();

    let arrow_length = 6.0;
    let arrow_width = 1.0;
    let arrow_clr = Color::WHITE;

    // Create shared meshes and materials
    let arrow_shaft_mesh = meshes.add(Plane3d::default().mesh().size(arrow_length, arrow_width));
    let arrow_material = materials.add(StandardMaterial {
        base_color: arrow_clr,
        ..default()
    });

    // Create the arrowhead mesh
    let half_arrow_size = arrow_length / 2.0;
    let a = Vec2::new(half_arrow_size + 1.0, 0.0); // Tip of the arrowhead
    let b = Vec2::new(half_arrow_size - 1.5, arrow_width + 0.25);
    let c = Vec2::new(half_arrow_size - 1.5, -arrow_width - 0.25);
    let arrow_head_mesh = meshes.add(Triangle2d::new(a, b, c));

    for cell_row in grid_controller.cur_flowfield.grid.iter() {
        for cell in cell_row.iter() {
            let rotation = Quat::from_rotation_y(cell.best_direction.to_angle());
            let mut translation = cell.world_position;
            translation.y += 0.01;
            translation.x -= 0.5;

            // Use the shared mesh and material
            let arrow_shaft = (
                PbrBundle {
                    mesh: arrow_shaft_mesh.clone(),
                    material: arrow_material.clone(),
                    transform: Transform {
                        translation,
                        rotation,
                        ..default()
                    },
                    ..default()
                },
                FlowFieldArrow,
                Name::new("Flowfield Arrow"),
            );

            // Use the shared arrowhead mesh and material
            let arrow_head = (
                PbrBundle {
                    mesh: arrow_head_mesh.clone(),
                    material: arrow_material.clone(),
                    transform: Transform {
                        translation: Vec3::ZERO,
                        rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                        ..default()
                    },
                    ..default()
                },
                Name::new("Arrowhead"),
            );

            cmds.spawn(arrow_shaft).with_children(|parent| {
                parent.spawn(arrow_head);
            });
        }
    }
}

fn draw_costfield(
    _trigger: Trigger<DrawDebugEv>,
    debug: Res<RtsPfDebug>,
    q_grid_controller: Query<&GridController>,
    q_cost: Query<Entity, With<CostField>>,
    mut cmds: Commands,
) {
    // remove current cost field before rendering new one
    for cost_entity in q_cost.iter() {
        cmds.entity(cost_entity).despawn_recursive();
    }

    if !debug.draw_costfield {
        return;
    }

    let grid_controller = q_grid_controller.get_single().unwrap();

    for cell_row in grid_controller.cur_flowfield.grid.iter() {
        for cell in cell_row.iter() {
            let cost = (
                BillboardTextBundle {
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
                        translation: cell.world_position,
                        scale: Vec3::splat(0.03),
                        ..default()
                    },
                    ..default()
                },
                CostField,
            );

            cmds.spawn(cost);
        }
    }
}
