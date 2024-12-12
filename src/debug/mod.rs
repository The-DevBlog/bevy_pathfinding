use bevy::prelude::*;
use draw::DrawPlugin;
use resources::ResourcesPlugin;
use ui::UiPlugin;

mod components;
pub mod draw;
mod events;
mod resources;
mod ui;

pub struct BevyRtsPathFindingDebugPlugin;

impl Plugin for BevyRtsPathFindingDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DrawPlugin, UiPlugin, ResourcesPlugin));
    }
}
