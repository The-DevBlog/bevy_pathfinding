use bevy::prelude::*;

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

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DrawPlugin, UiPlugin, ResourcesPlugin, ShaderPlugin));
    }
}
