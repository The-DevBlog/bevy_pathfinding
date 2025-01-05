use bevy::{color::palettes::css::GRAY, prelude::*};

use draw::DrawPlugin;
use resources::ResourcesPlugin;
use shader::ShaderPlugin;
use ui::UiPlugin;

mod components;
pub mod draw;
mod events;
mod resources;
mod shader;
mod ui;

const COLOR_GRID: Srgba = GRAY;

pub struct BevyRtsPathFindingDebugPlugin;

impl Plugin for BevyRtsPathFindingDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DrawPlugin, UiPlugin, ResourcesPlugin, ShaderPlugin));
    }
}
