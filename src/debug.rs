use bevy::prelude::*;
use bevy_mod_billboard::*;
use grid_controller::GridController;

use crate::*;

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
            draw_flowfield: false,
            draw_costfield: false,
            draw_integration_field: false,
        }
    }
}

#[derive(Event)]
pub struct DrawDebugEv;

#[derive(Component)]
struct CostField;

#[derive(Component)]
struct FlowField;

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
    q_flowfield: Query<Entity, With<FlowField>>,
    q_grid_controller: Query<&GridController>,
    mut cmds: Commands,
) {
    // remove current cost field before rendering new one
    for cost_entity in q_flowfield.iter() {
        cmds.entity(cost_entity).despawn_recursive();
    }

    if !debug.draw_flowfield {
        return;
    }

    let grid_controller = q_grid_controller.get_single().unwrap();

    for cell_row in grid_controller.cur_flowfield.grid.iter() {
        for cell in cell_row.iter() {
            let arrow = (FlowField,);

            cmds.spawn(arrow);
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
