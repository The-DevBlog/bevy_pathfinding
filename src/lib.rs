use bevy::prelude::*;

use crate::debug::DebugPlugin;
use crate::events::*;
use crate::resources::*;

mod cell;
pub mod components;
pub mod debug;
pub mod events;
pub mod flowfield;
pub mod grid;
mod grid_direction;
pub mod resources;
pub mod utils;

use flowfield::FlowfieldPlugin;
use grid::GridPlugin;
use resources::ResourcesPlugin;

pub struct BevyRtsPathFindingPlugin;

impl Plugin for BevyRtsPathFindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FlowfieldPlugin,
            ResourcesPlugin,
            GridPlugin,
            #[cfg(feature = "debug")]
            DebugPlugin,
        ));
    }
}
