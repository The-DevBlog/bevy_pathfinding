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
            .add_systems(Startup, (setup, load_texture_atlas, test))
            .add_systems(Update, (draw_grid, detect_debug_change))
            .add_systems(Update, set_draw_mode_onclick)
            .add_systems(Update, (t_1, t_2))
            .observe(draw_flowfield)
            .observe(draw_integration_field)
            .observe(draw_costfield)
            .observe(draw_index)
            .observe(toggle_dropdown_visibility_1)
            .observe(toggle_dropdown_visibility_2)
            .observe(update_active_dropdown_option);
    }
}

fn set_draw_mode_onclick(
    q_option_1: Query<(&Interaction, &ActiveOption1), (Changed<Interaction>, With<ActiveOption1>)>,
    q_option_2: Query<(&Interaction, &ActiveOption2), (Changed<Interaction>, With<ActiveOption2>)>,
    mut dbg: ResMut<RtsPfDebug>,
) {
    if let Ok((interaction, option)) = q_option_1.get_single() {
        match interaction {
            Interaction::Pressed => {
                dbg.draw_mode_1 = DrawMode::cast(option.0.clone());
            }
            _ => (),
        }
    };

    if let Ok((interaction, option)) = q_option_2.get_single() {
        match interaction {
            Interaction::Pressed => {
                dbg.draw_mode_2 = DrawMode::cast(option.0.clone());
            }
            _ => (),
        }
    };
}

fn update_active_dropdown_option(
    _trigger: Trigger<UpdateDropdownOptionEv>,
    dbg: Res<RtsPfDebug>,
    mut q_txt_1: Query<&mut Text, (With<ActiveOption1>, Without<ActiveOption2>)>,
    mut q_txt_2: Query<&mut Text, With<ActiveOption2>>,
) {
    println!("Getting Active Option");
    if let Ok(mut txt) = q_txt_1.get_single_mut() {
        println!("Setting Active Option");
        txt.sections[0].value = dbg.mode1_string();
    }

    if let Ok(mut txt) = q_txt_2.get_single_mut() {
        txt.sections[0].value = dbg.mode2_string();
    }
}

fn t_1(mut cmds: Commands, q_btn: Query<&Interaction, With<ExpandBtn1>>) {
    if let Ok(interaction) = q_btn.get_single() {
        match interaction {
            Interaction::Pressed => cmds.trigger(ToggleMode1Ev),
            _ => (),
        }
    }
}

fn t_2(mut cmds: Commands, q_btn: Query<&Interaction, With<ExpandBtn2>>) {
    if let Ok(interaction) = q_btn.get_single() {
        match interaction {
            Interaction::Pressed => cmds.trigger(ToggleMode2Ev),
            _ => (),
        }
    }
}

fn toggle_dropdown_visibility_1(
    _trigger: Trigger<ToggleMode1Ev>,
    mut cmds: Commands,
    mut q_dropdown: Query<&mut Style, With<Dropdown1>>,
) {
    if let Ok(mut dropdown) = q_dropdown.get_single_mut() {
        if dropdown.display == Display::Flex {
            dropdown.display = Display::None;
        } else if dropdown.display == Display::None {
            dropdown.display = Display::Flex
        }
    }

    cmds.trigger(UpdateDropdownOptionEv);
}

fn toggle_dropdown_visibility_2(
    _trigger: Trigger<ToggleMode2Ev>,
    mut cmds: Commands,
    mut q_dropdown: Query<&mut Style, With<Dropdown2>>,
) {
    if let Ok(mut dropdown) = q_dropdown.get_single_mut() {
        if dropdown.display == Display::Flex {
            dropdown.display = Display::None;
        } else if dropdown.display == Display::None {
            dropdown.display = Display::Flex
        }
    }

    cmds.trigger(UpdateDropdownOptionEv);
}

#[derive(Event)]
struct UpdateDropdownOptionEv;

#[derive(Event)]
struct ToggleMode1Ev;

#[derive(Event)]
struct ToggleMode2Ev;

#[derive(Component)]
struct ActiveOption1(pub String);

#[derive(Component)]
struct ActiveOption2(pub String);

#[derive(Component)]
struct Dropdown1;

#[derive(Component)]
struct Dropdown2;

#[derive(Component)]
struct ExpandBtn1;

#[derive(Component)]
struct ExpandBtn2;

fn test(mut cmds: Commands, mut dbg: ResMut<RtsPfDebug>) {
    let root_container = (
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                ..default()
            },
            border_radius: BorderRadius::all(Val::Px(5.0)),
            background_color: Srgba::BLACK.into(),
            ..default()
        },
        Name::new("Debug Container"),
    );

    let draw_container = || -> NodeBundle {
        NodeBundle {
            style: Style {
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            ..default()
        }
    };

    let draw_mode_txt = |txt: String| -> TextBundle {
        TextBundle {
            text: Text::from_section(txt, TextStyle::default()),
            ..default()
        }
    };

    let options_container = || -> NodeBundle {
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                display: Display::None,
                ..default()
            },
            ..default()
        }
    };

    let option_txt = |txt: String| -> TextBundle {
        TextBundle {
            text: Text::from_section(txt.clone(), TextStyle::default()),
            ..default()
        }
    };

    let btn_1 = |txt: String| -> (ButtonBundle, ActiveOption1) {
        (ButtonBundle::default(), ActiveOption1(txt))
    };

    let btn_2 = |txt: String| -> (ButtonBundle, ActiveOption2) {
        (ButtonBundle::default(), ActiveOption2(txt))
    };

    let active_option = |txt: String| -> TextBundle {
        TextBundle {
            text: Text::from_section(txt, TextStyle::default()),
            ..default()
        }
    };

    let change_option_btn_1 = (ButtonBundle::default(), ExpandBtn1);
    let change_option_btn_2 = (ButtonBundle::default(), ExpandBtn2);

    let change_option_txt = || -> TextBundle {
        TextBundle {
            text: Text::from_section(String::from("V"), TextStyle::default()),
            ..default()
        }
    };

    let dropdown_1 = NodeBundle::default();
    let dropdown_2 = NodeBundle::default();

    let active_option_1 = (
        active_option(dbg.mode1_string()),
        ActiveOption1(dbg.mode1_string()),
    );
    let active_option_2 = (
        active_option(dbg.mode2_string()),
        ActiveOption2(dbg.mode2_string()),
    );

    // Root Container
    cmds.spawn(root_container).with_children(|container| {
        // Draw Mode 1 Container
        container
            .spawn(draw_container())
            .with_children(|draw_mode_1| {
                draw_mode_1.spawn(draw_mode_txt("Draw Mode 1".to_string()));

                // Dropdown Container
                draw_mode_1.spawn(dropdown_1).with_children(|dropdown| {
                    // Dropdown Active Option
                    dropdown.spawn(active_option_1);
                    dropdown.spawn(change_option_btn_1).with_children(|btn| {
                        btn.spawn(change_option_txt());
                    });

                    // Dropdown Options Container
                    dropdown
                        .spawn((options_container(), Dropdown1))
                        // Dropdown Options
                        .with_children(|options| {
                            options
                                .spawn(btn_1("None".to_string()))
                                .with_children(|btn| {
                                    btn.spawn(option_txt("None".to_string()));
                                });
                            options
                                .spawn(btn_1("IntegrationField".to_string()))
                                .with_children(|btn| {
                                    btn.spawn(option_txt("IntegrationField".to_string()));
                                });
                            options
                                .spawn(btn_1("FlowField".to_string()))
                                .with_children(|btn| {
                                    btn.spawn(option_txt("FlowField".to_string()));
                                });
                            options
                                .spawn(btn_1("CostField".to_string()))
                                .with_children(|btn| {
                                    btn.spawn(option_txt("CostField".to_string()));
                                });
                            options
                                .spawn(btn_1("Index".to_string()))
                                .with_children(|btn| {
                                    btn.spawn(option_txt("Index".to_string()));
                                });
                        });
                });
            });

        // Draw Mode 2 Container
        container
            .spawn(draw_container())
            .with_children(|draw_mode_2| {
                draw_mode_2.spawn(draw_mode_txt("Draw Mode 2".to_string()));

                // Dropdown Container
                draw_mode_2.spawn(dropdown_2).with_children(|dropdown| {
                    // Dropdown Active Option
                    dropdown.spawn(active_option_2);
                    dropdown.spawn(change_option_btn_2).with_children(|btn| {
                        btn.spawn(change_option_txt());
                    });

                    // Dropdown Options Container
                    dropdown
                        .spawn((options_container(), Dropdown2))
                        // Dropdown Options
                        .with_children(|options| {
                            options
                                .spawn(btn_2("None".to_string()))
                                .with_children(|btn| {
                                    btn.spawn(option_txt("None".to_string()));
                                });
                            options
                                .spawn(btn_2("IntegrationField".to_string()))
                                .with_children(|btn| {
                                    btn.spawn(option_txt("IntegrationField".to_string()));
                                });
                            options
                                .spawn(btn_2("FlowField".to_string()))
                                .with_children(|btn| {
                                    btn.spawn(option_txt("FlowField".to_string()));
                                });
                            options
                                .spawn(btn_1("CostField".to_string()))
                                .with_children(|btn| {
                                    btn.spawn(option_txt("CostField".to_string()));
                                });
                            options
                                .spawn(btn_2("Index".to_string()))
                                .with_children(|btn| {
                                    btn.spawn(option_txt("Index".to_string()));
                                });
                        });
                });
            });
    });
}

#[derive(Resource, Default)]
struct Digits([Handle<Image>; 10]);

#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct RtsPfDebug {
    draw_grid: bool,
    draw_mode_1: DrawMode,
    draw_mode_2: DrawMode,
}

impl Default for RtsPfDebug {
    fn default() -> Self {
        RtsPfDebug {
            draw_grid: true,
            draw_mode_1: DrawMode::FlowField,
            draw_mode_2: DrawMode::Index,
        }
    }
}

impl RtsPfDebug {
    fn draw_mode_to_string(mode: DrawMode) -> String {
        match mode {
            DrawMode::None => String::from("None"),
            DrawMode::CostField => String::from("CostField"),
            DrawMode::FlowField => String::from("FlowField"),
            DrawMode::IntegrationField => String::from("IntegrationField"),
            DrawMode::Index => String::from("Index"),
        }
    }

    fn mode1_string(&self) -> String {
        Self::draw_mode_to_string(self.draw_mode_1)
    }

    fn mode2_string(&self) -> String {
        Self::draw_mode_to_string(self.draw_mode_2)
    }
}

#[derive(Reflect, PartialEq, Clone, Copy)]
enum DrawMode {
    None,
    CostField,
    FlowField,
    IntegrationField,
    Index,
}

impl DrawMode {
    fn cast(mode: String) -> Self {
        match mode.as_str() {
            "None" => DrawMode::None,
            "CostField" => DrawMode::CostField,
            "FlowField" => DrawMode::FlowField,
            "IntegrationField" => DrawMode::IntegrationField,
            "Index" => DrawMode::Index,
            _ => DrawMode::None,
        }
    }
}

#[derive(Event)]
pub struct DrawDebugEv;

#[derive(Component, Copy, Clone)]
struct Cost;

#[derive(Component, Copy, Clone)]
struct BestCost;

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

    let grid = q_grid_controller.get_single().unwrap();

    let mut marker_scale = 0.7;
    if (dbg.draw_mode_1 == DrawMode::None || dbg.draw_mode_2 == DrawMode::None)
        || (dbg.draw_mode_1 == DrawMode::FlowField && dbg.draw_mode_2 == DrawMode::FlowField)
    {
        marker_scale = 1.0;
    }

    let offset = match calculate_offset(&grid, dbg, DrawMode::FlowField) {
        Some(offset) => offset,
        None => return,
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

    // Create shared meshes and materials
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

            // Use the shared mesh and material
            let marker = (
                PbrBundle {
                    mesh,
                    material: material.clone(),
                    transform: Transform {
                        translation: cell.world_position + offset,
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
                    material: material.clone(),
                    transform: Transform {
                        translation: Vec3::ZERO,
                        rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                        ..default()
                    },
                    ..default()
                },
                Name::new("Arrowhead"),
            );

            let mut draw = cmds.spawn(marker);

            if !is_destination_cell {
                draw.with_children(|parent| {
                    parent.spawn(arrow_head);
                });
            }
        }
        // println!();
    }
}

fn draw_integration_field(
    _trigger: Trigger<DrawDebugEv>,
    dbg: Res<RtsPfDebug>,
    digits: Res<Digits>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    q_grid: Query<&GridController>,
    q_cost: Query<Entity, With<BestCost>>,
    mut cmds: Commands,
) {
    // Remove current cost field before rendering new one
    for cost_entity in q_cost.iter() {
        cmds.entity(cost_entity).despawn_recursive();
    }

    let grid = q_grid.get_single().unwrap();

    let offset = match calculate_offset(&grid, dbg, DrawMode::IntegrationField) {
        Some(offset) => offset,
        None => return,
    };

    let str = |cell: &Cell| format!("{}", cell.best_cost);
    draw::<BestCost>(
        meshes, materials, &grid, digits, BestCost, cmds, str, offset,
    );
}

fn draw_costfield(
    _trigger: Trigger<DrawDebugEv>,
    dbg: Res<RtsPfDebug>,
    digits: Res<Digits>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    q_grid: Query<&GridController>,
    q_cost: Query<Entity, With<Cost>>,
    mut cmds: Commands,
) {
    // Remove current cost field before rendering new one
    for cost_entity in q_cost.iter() {
        cmds.entity(cost_entity).despawn_recursive();
    }

    let grid = q_grid.get_single().unwrap();

    let offset = match calculate_offset(&grid, dbg, DrawMode::CostField) {
        Some(offset) => offset,
        None => return,
    };

    let str = |cell: &Cell| format!("{}", cell.cost);
    draw::<Cost>(meshes, materials, &grid, digits, Cost, cmds, str, offset);
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

    let offset = match calculate_offset(&grid, dbg, DrawMode::Index) {
        Some(offset) => offset,
        None => return,
    };

    let str = |cell: &Cell| format!("{}{}", cell.grid_idx.y, cell.grid_idx.x);
    draw(meshes, materials, &grid, digits, Index, cmds, str, offset);
}

fn calculate_offset(
    grid: &GridController,
    dbg: Res<RtsPfDebug>,
    draw_mode: DrawMode,
) -> Option<Vec3> {
    let mode = if dbg.draw_mode_1 == draw_mode {
        Some(1)
    } else if dbg.draw_mode_2 == draw_mode {
        Some(2)
    } else {
        None
    };

    if mode.is_none() {
        return None; // nothing to draw
    }

    // Base offset when only one mode is active
    let mut offset = Vec3::new(0.0, 0.01, 0.0);
    if (dbg.draw_mode_1 == DrawMode::None || dbg.draw_mode_2 == DrawMode::None)
        || (dbg.draw_mode_1 == draw_mode && dbg.draw_mode_2 == draw_mode)
    {
        offset.z = 0.0;
    } else {
        match mode {
            Some(1) => offset.z = -grid.cell_diameter() * 0.25,
            Some(2) => offset.z = grid.cell_diameter() * 0.25,
            _ => (),
        };
    }

    return Some(offset);
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
