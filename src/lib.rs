use crate::events::*;
use crate::resources::*;

use bevy::color::palettes::css::*;
use bevy::prelude::*;
use flowfield::FlowfieldPlugin;
use resources::ResourcesPlugin;

mod cell;
pub mod components;
pub mod debug;
pub mod events;
pub mod flowfield;
pub mod grid;
mod grid_direction;
pub mod resources;
pub mod utils;

pub struct BevyRtsPathFindingPlugin;

impl Plugin for BevyRtsPathFindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((FlowfieldPlugin, ResourcesPlugin));
    }
}
