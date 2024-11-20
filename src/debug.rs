use crate::*;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
};
use bevy_mod_billboard::*;
use grid_controller::GridController;
use serde::Deserialize;
use thiserror::Error;

pub struct BevyRtsPathFindingDebugPlugin;

impl Plugin for BevyRtsPathFindingDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArrowImgHandle>()
            .init_resource::<RtsPfDebug>()
            .register_type::<RtsPfDebug>()
            // .add_systems(Startup, load_assets)
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

#[derive(Resource, Default)]
struct ArrowImgHandle(pub Handle<Image>);

#[derive(Event)]
pub struct DrawDebugEv;

#[derive(Component)]
struct CostField;

#[derive(Component)]
struct FlowFieldArrow;

// #[derive(Asset, TypePath, Debug, Deserialize)]
// struct CustomAsset {
//     #[allow(dead_code)]
//     value: i32,
// }

// #[derive(Default)]
// struct CustomAssetLoader;

// /// Possible errors that can be produced by [`CustomAssetLoader`]
// #[non_exhaustive]
// #[derive(Debug, Error)]
// enum CustomAssetLoaderError {
//     /// An [IO](std::io) Error
//     #[error("Could not load asset: {0}")]
//     Io(#[from] std::io::Error),
//     /// A [RON](ron) Error
//     #[error("Could not parse RON: {0}")]
//     RonSpannedError(#[from] ron::error::SpannedError),
// }

// impl AssetLoader for CustomAssetLoader {
//     type Asset = CustomAsset;
//     type Settings = ();
//     type Error = CustomAssetLoaderError;
//     async fn load<'a>(
//         &'a self,
//         reader: &'a mut Reader<'_>,
//         _settings: &'a (),
//         _load_context: &'a mut LoadContext<'_>,
//     ) -> Result<Self::Asset, Self::Error> {
//         let mut bytes = Vec::new();
//         reader.read_to_end(&mut bytes).await?;
//         let custom_asset = ron::de::from_bytes::<CustomAsset>(&bytes)?;
//         Ok(custom_asset)
//     }

//     fn extensions(&self) -> &[&str] {
//         &["custom"]
//     }
// }
// fn load_assets(mut arrow_img_handle: ResMut<ArrowImgHandle>, assets: Res<AssetServer>) {
//     arrow_img_handle.0 = assets.load("arrow.png");
// }

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
    arrow_img_handle: Res<ArrowImgHandle>,
    q_grid_controller: Query<&GridController>,
    q_flowfield_arrow: Query<Entity, With<FlowFieldArrow>>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // remove current cost field before rendering new one
    for cost_entity in q_flowfield_arrow.iter() {
        cmds.entity(cost_entity).despawn_recursive();
    }

    if !debug.draw_flowfield {
        return;
    }

    let color = Color::WHITE;
    let arrow = (
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(8., 2.)),
            material: materials.add(color),
            transform: Transform {
                translation: Vec3::new(0.0, 0.2, 0.0),
                rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                ..default()
            },
            // transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        Name::new("Flowfield Arrow"),
    );

    cmds.spawn(arrow);

    // for cell_row in grid_controller.cur_flowfield.grid.iter() {
    //     for cell in cell_row.iter() {
    //         let arrow = (FlowField,);

    //         cmds.spawn(arrow);
    //     }
    // }

    // let arrow = (
    //     // Image
    //     FlowFieldArrow,
    //     Name::new("FlowField Arrow"),
    // );

    // cmds.spawn(arrow);
}

// OG flowfield
// fn draw_flowfield(
//     flowfield_q: Query<&FlowField>,
//     selected_q: Query<Entity, With<Selected>>,
//     grid: Res<Grid>,
//     mut gizmos: Gizmos,
// ) {
//     if selected_q.is_empty() {
//         return;
//     }

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
//                     let top_left = cell.position + Vec3::new(-ARROW_LENGTH, 0.0, -ARROW_LENGTH);
//                     let top_right = cell.position + Vec3::new(ARROW_LENGTH, 0.0, -ARROW_LENGTH);
//                     let bottom_left = cell.position + Vec3::new(-ARROW_LENGTH, 0.0, ARROW_LENGTH);
//                     let bottom_right = cell.position + Vec3::new(ARROW_LENGTH, 0.0, ARROW_LENGTH);

//                     gizmos.line(top_left, bottom_right, RED);
//                     gizmos.line(top_right, bottom_left, RED);
//                     continue;
//                 }

//                 let flow_direction = cell.flow_vector.normalize();

//                 let start = cell.position - flow_direction * ARROW_LENGTH;
//                 let end = cell.position + flow_direction * ARROW_LENGTH;

//                 gizmos.arrow(start, end, COLOR_ARROWS);
//             }
//         }
//     }
// }

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
