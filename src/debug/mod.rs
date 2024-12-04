use bevy::prelude::*;
use draw::DrawPlugin;
use ui::UiPlugin;

pub mod draw;
mod ui;

pub struct BevyRtsPathFindingDebugPlugin;

impl Plugin for BevyRtsPathFindingDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DrawPlugin, UiPlugin));
    }
}
