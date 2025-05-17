use bevy::prelude::*;

use crate::debug::DebugPlugin;
use crate::events::*;
use crate::resources::*;

pub mod boids;
mod cell;
pub mod components;
pub mod debug;
pub mod events;
pub mod flowfield;
pub mod grid;
pub mod grid_direction;
pub mod resources;
pub mod utils;

use boids::BoidsPlugin;
use flowfield::FlowfieldPlugin;
use grid::GridPlugin;
use resources::ResourcesPlugin;

pub struct BevyPathfindingPlugin;

impl Plugin for BevyPathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BoidsPlugin,
            FlowfieldPlugin,
            ResourcesPlugin,
            GridPlugin,
            #[cfg(feature = "debug")]
            DebugPlugin,
        ));
    }
}
