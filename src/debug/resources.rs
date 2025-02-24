use bevy::{image::*, prelude::*, render::render_resource::*};
use image::ImageFormat;

const DBG_ICON: &[u8] = include_bytes!("../../assets/imgs/dbg_icon.png");

pub struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugOptions>()
            .init_resource::<DbgIcon>()
            .register_type::<DebugOptions>()
            .add_systems(Startup, load_dbg_icon);
    }
}

#[derive(Resource, Default)]
pub struct DbgIcon(pub Handle<Image>);

#[derive(Reflect, Resource, Clone, Copy)]
#[reflect(Resource)]
pub struct DebugOptions {
    pub hide: bool,
    pub draw_grid: bool,
    pub print_statements: bool,
    pub draw_mode_1: DrawMode,
    pub draw_mode_2: DrawMode,
}

impl Default for DebugOptions {
    fn default() -> Self {
        DebugOptions {
            hide: false,
            draw_grid: true,
            print_statements: true,
            draw_mode_1: DrawMode::None,
            draw_mode_2: DrawMode::CostField,
        }
    }
}

impl DebugOptions {
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

pub fn load_dbg_icon(mut images: ResMut<Assets<Image>>, mut dbg_icon: ResMut<DbgIcon>) {
    // Decode the image
    let image = image::load_from_memory_with_format(DBG_ICON, ImageFormat::Png)
        .expect("Failed to load digit image");
    let rgba_image = image.to_rgba8();
    let (width, height) = rgba_image.dimensions();

    let image = Image {
        data: rgba_image.to_vec(),
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
