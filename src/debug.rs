use crate::*;
use bevy::render::{
    render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    texture::{ImageSampler, ImageSamplerDescriptor},
};
use grid_controller::GridController;
use image::ImageFormat;
use std::f32::consts::FRAC_PI_2;

const DIGIT_ATLAS: &[u8] = include_bytes!("../assets/digits/digit_atlas.png");

pub struct BevyRtsPathFindingDebugPlugin;

impl Plugin for BevyRtsPathFindingDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RtsPfDebug>()
            .init_resource::<Digits>()
            .register_type::<RtsPfDebug>()
            .add_systems(Startup, load_texture_atlas)
            .add_systems(Update, draw_grid)
            .add_systems(Update, detect_debug_change)
            .observe(draw_costfield)
            .observe(draw_flowfield);
    }
}

#[derive(Resource, Default)]
struct Digits([Handle<Image>; 10]);

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

fn load_texture_atlas(mut images: ResMut<Assets<Image>>, mut digits: ResMut<Digits>) {
    let digit_bytes = DIGIT_ATLAS;

    // Decode the image
    let image = image::load_from_memory_with_format(digit_bytes, ImageFormat::Png)
        .expect("Failed to load digit image");
    let rgba_image = image.to_rgba8();
    let (width, height) = rgba_image.dimensions();
    let digit_width = width / 10;

    // Extract each digit as a separate texture
    for idx in 0..10 {
        let x_start = idx * digit_width;
        let cropped_digit_data =
            image::imageops::crop_imm(&rgba_image, x_start, 0, digit_width, height)
                .to_image()
                .into_raw();

        let cropped_digit = Image {
            data: cropped_digit_data,
            texture_descriptor: TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: digit_width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            sampler: ImageSampler::Descriptor(ImageSamplerDescriptor::default()),
            texture_view_descriptor: None,
            asset_usage: Default::default(),
        };

        digits.0[idx as usize] = images.add(cropped_digit);
    }
}

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

    let mut arrow_scale = 1.0;
    if debug.draw_costfield || debug.draw_flowfield {
        arrow_scale = 0.7;
    }

    let arrow_length = 6.0 * arrow_scale;
    let arrow_width = 1.0 * arrow_scale;
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
            // translation.x -= 0.5;
            // translation.y += offset_y;
            // translation.x -= offset_x;

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
    digits: Res<Digits>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_grid_controller: Query<&GridController>,
    q_cost: Query<Entity, With<CostField>>,
    mut cmds: Commands,
) {
    // Remove current cost field before rendering new one
    for cost_entity in q_cost.iter() {
        cmds.entity(cost_entity).despawn_recursive();
    }

    if !debug.draw_costfield {
        return;
    }

    let grid = q_grid_controller.get_single().unwrap();

    let mut digit_spacing = 2.75;
    let mut base_offset = Vec3::new(0.0, 0.01, 0.0);
    let mut scale = Vec3::splat(0.3);

    if debug.draw_flowfield || debug.draw_integration_field {
        digit_spacing = 1.5;
        base_offset.z = grid.cell_radius - 2.0;
        scale = Vec3::splat(0.2);
    }

    let mesh = meshes.add(Rectangle::new(grid.cell_diameter(), grid.cell_diameter()));

    for cell_row in grid.cur_flowfield.grid.iter() {
        for cell in cell_row.iter() {
            // Convert the cost value to its individual digits
            let cost_digits: Vec<u32> = cell
                .cost
                .to_string()
                .chars()
                .map(|c| c.to_digit(10).unwrap())
                .collect();

            let x_offset = -(cost_digits.len() as f32 - 1.0) * digit_spacing / 2.0;

            for (i, &digit) in cost_digits.iter().enumerate() {
                // Calculate the offset for each digit
                let mut offset = base_offset;
                offset.x += x_offset + i as f32 * digit_spacing;

                let material = materials.add(StandardMaterial {
                    base_color_texture: Some(digits.0[digit as usize].clone()),
                    base_color: Color::WHITE,
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                });

                // Spawn each digit as a separate PBR entity
                let dig = (
                    PbrBundle {
                        mesh: mesh.clone(),
                        material,
                        transform: Transform {
                            translation: cell.world_position + offset,
                            rotation: Quat::from_rotation_x(-FRAC_PI_2),
                            scale,
                            ..default()
                        },
                        ..default()
                    },
                    CostField,
                );

                cmds.spawn(dig);
            }
        }
    }
}

fn detect_debug_change(mut cmds: Commands, debug: Res<RtsPfDebug>) {
    if debug.is_changed() {
        cmds.trigger(DrawDebugEv);
    }
}
