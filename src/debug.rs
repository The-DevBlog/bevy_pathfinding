use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_mod_billboard::*;
use grid_controller::GridController;

use crate::*;

pub struct BevyRtsPathFindingDebugPlugin;

impl Plugin for BevyRtsPathFindingDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RtsPfDebug>()
            .register_type::<RtsPfDebug>()
            .add_systems(Update, draw_grid)
            .observe(draw_costfield);
    }
}

#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
pub struct RtsPfDebug {
    pub draw_costfield: bool,
    pub draw_grid: bool,
    pub draw_integration_field: bool,
}

#[derive(Event)]
pub struct DrawCostFieldEv;

#[derive(Component)]
struct CostField;

#[derive(Component)]
struct CostTxt;

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

fn draw_costfield(
    _trigger: Trigger<DrawCostFieldEv>,
    mut cmds: Commands,
    q_grid_controller: Query<&GridController>,
    q_cost: Query<Entity, With<CostTxt>>,
    debug: Res<RtsPfDebug>,
) {
    // remove current cost field before rendering new one
    for cost_entity in q_cost.iter() {
        cmds.entity(cost_entity).despawn_recursive();
    }

    if !debug.draw_costfield {
        return;
    }

    let grid_controller = q_grid_controller.get_single().unwrap();
    let parent = cmds.spawn(CostField).insert(Name::new("CostField")).id(); // Get the entity ID for parenting

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
                CostTxt,
            );

            cmds.spawn(cost).set_parent(parent); // TODO: Not working as expected
        }
    }
}
