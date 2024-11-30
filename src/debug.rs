use crate::*;
use bevy::render::{
    render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    texture::{ImageSampler, ImageSamplerDescriptor},
};
use cell::Cell;
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
            .add_systems(Startup, (setup, load_texture_atlas))
            .add_systems(Update, (draw_grid, detect_debug_change))
            .observe(draw_costfield)
            .observe(draw_flowfield)
            .observe(draw_index);
    }
}

#[derive(Resource, Default)]
struct Digits([Handle<Image>; 10]);

#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct RtsPfDebug {
    draw_grid: bool,
    draw_mode_1: DrawMode1,
    draw_mode_2: DrawMode2,
}

#[derive(Reflect, PartialEq)]
enum DrawMode1 {
    None,
    CostField,
    FlowField,
    IntegrationField,
    Index,
}

#[derive(Reflect, PartialEq)]
enum DrawMode2 {
    None,
    CostField,
    FlowField,
    IntegrationField,
    Index,
}

impl Default for RtsPfDebug {
    fn default() -> Self {
        RtsPfDebug {
            draw_grid: true,
            draw_mode_1: DrawMode1::FlowField,
            draw_mode_2: DrawMode2::None,
        }
    }
}

#[derive(Event)]
pub struct DrawDebugEv;

#[derive(Component, Copy, Clone)]
struct CostField;

#[derive(Component, Copy, Clone)]
struct Index;

#[derive(Component)]
struct FlowFieldArrow;

fn setup(mut cmds: Commands) {
    cmds.trigger(DrawDebugEv);
}

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
    // if !debug.draw_integration_field {
    //     return;
    // }
}

fn draw_flowfield(
    _trigger: Trigger<DrawDebugEv>,
    dbg: Res<RtsPfDebug>,
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

    if dbg.draw_mode_1 != DrawMode1::FlowField {
        return;
    }

    let grid = q_grid_controller.get_single().unwrap();

    let mut arrow_scale = 1.0;
    if dbg.draw_mode_1 != DrawMode1::None {
        arrow_scale = 0.7;
    }

    let arrow_length = grid.cell_diameter() * 0.6 * arrow_scale;
    let arrow_width = grid.cell_diameter() * 0.1 * arrow_scale;
    let arrow_clr = Color::WHITE;

    // Create shared meshes and materials
    let arrow_shaft_mesh = meshes.add(Plane3d::default().mesh().size(arrow_length, arrow_width));
    let arrow_material = materials.add(StandardMaterial {
        base_color: arrow_clr,
        ..default()
    });

    // Create the arrowhead mesh
    let half_arrow_size = arrow_length / 2.0;
    let d1 = half_arrow_size - grid.cell_diameter() * 0.09;
    let d2 = arrow_width + grid.cell_diameter() * 0.0125;
    let a = Vec2::new(half_arrow_size + grid.cell_diameter() * 0.05, 0.0); // Tip of the arrowhead
    let b = Vec2::new(d1, d2);
    let c = Vec2::new(d1, -arrow_width - grid.cell_diameter() * 0.0125);
    let arrow_head_mesh = meshes.add(Triangle2d::new(a, b, c));

    for cell_row in grid.cur_flowfield.grid.iter() {
        for cell in cell_row.iter() {
            let rotation = Quat::from_rotation_y(cell.best_direction.to_angle());
            let mut translation = cell.world_position;
            translation.y += 0.01;

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
    dbg: Res<RtsPfDebug>,
    digits: Res<Digits>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    q_grid: Query<&GridController>,
    q_cost: Query<Entity, With<CostField>>,
    mut cmds: Commands,
) {
    // Remove current cost field before rendering new one
    for cost_entity in q_cost.iter() {
        cmds.entity(cost_entity).despawn_recursive();
    }

    let grid = q_grid.get_single().unwrap();

    // Check if Index is in either draw_mode_1 or draw_mode_2
    let mode = if dbg.draw_mode_1 == DrawMode1::CostField {
        Some(1)
    } else if dbg.draw_mode_2 == DrawMode2::CostField {
        Some(2)
    } else {
        None
    };

    if mode.is_none() {
        return; // Index mode is not active; nothing to draw
    }
    // Determine if both modes are active
    let both_modes_active =
        dbg.draw_mode_1 != DrawMode1::None && dbg.draw_mode_2 != DrawMode2::None;

    // Base offset when only one mode is active
    let base_offset_single = Vec3::new(0.0, 0.01, 0.0);

    // Offsets for each mode when both are active
    // let base_offset_mode_1 = if both_modes_active {
    let base_offset_mode_1 = if dbg.draw_mode_1 != DrawMode1::None {
        // Move upwards
        Vec3::new(0.0, 0.01, -8.0)
        // Vec3::new(0.0, 0.01, 8.0)
    } else {
        base_offset_single
    };

    // let base_offset_mode_2 = if both_modes_active {
    let base_offset_mode_2 = if dbg.draw_mode_2 != DrawMode2::None {
        // Move downwards
        Vec3::new(0.0, 0.01, 8.0)
        // Vec3::new(0.0, 0.01, -8.0)
    } else {
        base_offset_single
    };

    // Select base_offset based on which mode Index is in
    let base_offset = match mode {
        Some(1) => base_offset_mode_1,
        Some(2) => base_offset_mode_2,
        _ => base_offset_single, // Should not occur
    };

    let str = |cell: &Cell| format!("{}", cell.cost);
    draw::<CostField>(
        meshes,
        materials,
        &grid,
        digits,
        CostField,
        cmds,
        str,
        base_offset,
    );
}

fn draw_index(
    _trigger: Trigger<DrawDebugEv>,
    dbg: Res<RtsPfDebug>,
    digits: Res<Digits>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    q_grid_controller: Query<&GridController>,
    q_idx: Query<Entity, With<Index>>,
    mut cmds: Commands,
) {
    // Remove current index entities before rendering new ones
    for idx_entity in q_idx.iter() {
        cmds.entity(idx_entity).despawn_recursive();
    }

    let grid = q_grid_controller.get_single().unwrap();

    // Check if Index is in either draw_mode_1 or draw_mode_2
    let mode = if dbg.draw_mode_1 == DrawMode1::Index {
        Some(1)
    } else if dbg.draw_mode_2 == DrawMode2::Index {
        Some(2)
    } else {
        None
    };

    if mode.is_none() {
        return; // Index mode is not active; nothing to draw
    }

    // Determine if both modes are active
    let both_modes_active =
        dbg.draw_mode_1 != DrawMode1::None && dbg.draw_mode_2 != DrawMode2::None;

    // Base offset when only one mode is active
    let base_offset_single = Vec3::new(0.0, 0.01, 0.0);

    // Offsets for each mode when both are active
    // let base_offset_mode_1 = if both_modes_active {
    let base_offset_mode_1 = if dbg.draw_mode_1 != DrawMode1::None {
        // Move upwards
        Vec3::new(0.0, 0.01, -8.0)
        // Vec3::new(0.0, 0.01, 8.0)
    } else {
        base_offset_single
    };

    // let base_offset_mode_2 = if both_modes_active {
    let base_offset_mode_2 = if dbg.draw_mode_2 != DrawMode2::None {
        // Move downwards
        Vec3::new(0.0, 0.01, 8.0)
        // Vec3::new(0.0, 0.01, -8.0)
    } else {
        base_offset_single
    };

    // Select base_offset based on which mode Index is in
    let base_offset = match mode {
        Some(1) => base_offset_mode_1,
        Some(2) => base_offset_mode_2,
        _ => base_offset_single, // Should not occur
    };

    let str = |cell: &Cell| format!("{}{}", cell.grid_idx.y, cell.grid_idx.x);
    draw(
        meshes,
        materials,
        &grid,
        digits,
        Index,
        cmds,
        str,
        base_offset,
    );
}

fn draw<T: Component + Copy>(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    grid: &GridController,
    digits: Res<Digits>,
    comp: T,
    mut cmds: Commands,
    get_str: impl Fn(&Cell) -> String,
    base_offset: Vec3,
) {
    let base_digit_spacing = grid.cell_diameter() * 0.275;
    let base_scale = 0.25;

    let mesh = meshes.add(Rectangle::new(grid.cell_diameter(), grid.cell_diameter()));

    for cell_row in grid.cur_flowfield.grid.iter() {
        for cell in cell_row.iter() {
            // Generate the string using the closure
            let value_str = get_str(cell);

            // Convert the string into individual digits
            let digits_vec: Vec<u32> = value_str.chars().filter_map(|c| c.to_digit(10)).collect();
            let digit_count = digits_vec.len() as f32;

            // Set initial scale and digit spacing
            let mut scale = Vec3::splat(base_scale);
            let mut digit_spacing = base_digit_spacing;

            let digit_width = grid.cell_diameter() * scale.x;
            let total_digit_width = digit_count * digit_width;
            let total_spacing_width = (digit_count - 1.0) * digit_spacing;
            let total_width = total_digit_width + total_spacing_width;

            // If total width exceeds cell diameter, adjust scale and spacing
            if total_width > grid.cell_diameter() {
                let scale_factor = grid.cell_diameter() / total_width;
                scale *= scale_factor;
                digit_spacing *= scale_factor;
            }

            let x_offset = -(digits_vec.len() as f32 - 1.0) * digit_spacing / 2.0;

            for (i, &digit) in digits_vec.iter().enumerate() {
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
                    comp,
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
