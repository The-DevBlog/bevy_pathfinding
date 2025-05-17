use bevy::{image::*, prelude::*, render::render_resource::*};
use image::ImageFormat;

use crate::components::{Boid, BoidsInfo};

const DBG_ICON: &[u8] = include_bytes!("../../assets/imgs/dbg_icon.png");

pub struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DbgOptions>()
            .init_resource::<DbgIcon>()
            .init_resource::<BoidsUpdater>()
            .register_type::<DbgOptions>()
            .register_type::<BoidsUpdater>()
            .add_systems(Startup, load_dbg_icon)
            .add_systems(Update, update_boids);
    }
}

#[derive(Resource, Default)]
pub struct DbgIcon(pub Handle<Image>);

#[derive(Reflect, Resource, Clone, Copy)]
#[reflect(Resource)]
pub struct DbgOptions {
    pub boids_info: BoidsInfo,
    pub draw_grid: bool,
    pub draw_spatial_grid: bool,
    pub draw_spatial_hashing_grid: bool,
    pub draw_radius: bool,
    pub draw_mode_1: DrawMode,
    pub draw_mode_2: DrawMode,
    pub hide: bool,
    pub hover: bool,
    pub print_statements: bool,
}

impl Default for DbgOptions {
    fn default() -> Self {
        DbgOptions {
            boids_info: BoidsInfo::default(),
            draw_grid: true,
            draw_spatial_grid: false,
            draw_spatial_hashing_grid: false,
            draw_radius: false,
            draw_mode_1: DrawMode::FlowField,
            draw_mode_2: DrawMode::None,
            hide: false,
            hover: false,
            print_statements: false,
        }
    }
}

impl DbgOptions {
    pub fn draw_mode_to_string(mode: DrawMode) -> String {
        match mode {
            DrawMode::None => String::from("None"),
            DrawMode::CostField => String::from("CostField"),
            DrawMode::FlowField => String::from("FlowField"),
            DrawMode::IntegrationField => String::from("IntegrationField"),
            DrawMode::Index => String::from("Index"),
        }
    }

    pub fn print(&self, msg: &str) {
        if self.print_statements {
            println!("{}", msg);
        }
    }

    pub fn mode_string(&self, mode: i32) -> String {
        if mode == 1 {
            return Self::draw_mode_to_string(self.draw_mode_1);
        }

        return Self::draw_mode_to_string(self.draw_mode_2);
    }

    pub fn mode1_string(&self) -> String {
        Self::draw_mode_to_string(self.draw_mode_1)
    }

    pub fn mode2_string(&self) -> String {
        Self::draw_mode_to_string(self.draw_mode_2)
    }
}

#[derive(Reflect, PartialEq, Clone, Copy)]
pub enum DrawMode {
    None,
    CostField,
    FlowField,
    IntegrationField,
    Index,
}

impl DrawMode {
    pub fn cast(mode: String) -> Self {
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

/// DO NOT USE. This component is updated whenever the boids info in the debug UI menu changes.
#[derive(Resource, Reflect)]
pub struct BoidsUpdater {
    pub separation_weight: f32,    // push apart
    pub alignment_weight: f32,     // match heading
    pub cohesion_weight: f32,      // pull toward center
    pub neighbor_radius: f32,      // how far you “see” neighbors
    pub neighbor_exit_radius: f32, // new: slightly larger
}

impl Default for BoidsUpdater {
    fn default() -> Self {
        let neighbor_radius = 5.0;
        Self {
            separation_weight: 50.0,          // strongest urge to avoid collisions
            alignment_weight: 0.0,            // medium urge to line up
            cohesion_weight: 0.0,             // medium urge to stay together
            neighbor_radius: neighbor_radius, // in world‐units (tweak to taste)
            neighbor_exit_radius: neighbor_radius * 1.05, // new: slightly larger
        }
    }
}

fn update_boids(mut q_boids: Query<&mut Boid>, boid_updater: Res<BoidsUpdater>) {
    for mut boid in q_boids.iter_mut() {
        boid.info.separation = boid_updater.separation_weight;
        boid.info.alignment = boid_updater.alignment_weight;
        boid.info.cohesion = boid_updater.cohesion_weight;
        boid.info.neighbor_radius = boid_updater.neighbor_radius;
        boid.info.neighbor_exit_radius = boid_updater.neighbor_exit_radius;
    }
}

pub fn load_dbg_icon(mut images: ResMut<Assets<Image>>, mut dbg_icon: ResMut<DbgIcon>) {
    // Decode the image
    let image = image::load_from_memory_with_format(DBG_ICON, ImageFormat::Png)
        .expect("Failed to load digit image");
    let rgba_image = image.to_rgba8();
    let (width, height) = rgba_image.dimensions();

    let image = Image {
        data: Some(rgba_image.to_vec()),
        texture_descriptor: TextureDescriptor {
            label: None,
            size: Extent3d {
                width,
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

    // Add the image to Bevy's asset storage
    let handle = images.add(image);
    dbg_icon.0 = handle;
}
