use bevy::prelude::*;
use components::Boid;

use crate::debug::DebugPlugin;
use crate::events::*;
use crate::resources::*;

mod boids;
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

pub struct BevyRtsPathFindingPlugin;

impl Plugin for BevyRtsPathFindingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BoidsInfoEgui>();

        app.add_plugins((
            BoidsPlugin,
            FlowfieldPlugin,
            ResourcesPlugin,
            GridPlugin,
            #[cfg(feature = "debug")]
            DebugPlugin,
        ));

        app.add_systems(Update, change_boids);
    }
}

#[derive(Component, Reflect)]
pub struct BoidsInfoEgui {
    pub velocity: Vec3,            // start at rest
    pub separation_weight: f32,    // push apart
    pub alignment_weight: f32,     // match heading
    pub cohesion_weight: f32,      // pull toward center
    pub neighbor_radius: f32,      // how far you “see” neighbors
    pub neighbor_exit_radius: f32, // new: slightly larger
}

impl Default for BoidsInfoEgui {
    fn default() -> Self {
        let neighbor_radius = 45.0;
        Self {
            velocity: Vec3::ZERO,
            separation_weight: 50.0, // strongest urge to avoid collisions
            alignment_weight: 0.0,   // medium urge to line up
            cohesion_weight: 0.0,    // medium urge to stay together
            neighbor_radius: neighbor_radius, // in world‐units (tweak to taste)
            neighbor_exit_radius: neighbor_radius * 1.05, // new: slightly larger
        }
    }
}

fn change_boids(
    mut cmds: Commands,
    mut q_boids: Query<&mut Boid>,
    q_boid_values: Query<&BoidsInfoEgui>,
) {
    let Ok(new_boids_info) = q_boid_values.single() else {
        cmds.spawn((BoidsInfoEgui::default(), Name::new("Boids Info")));
        return;
    };

    for mut boid in q_boids.iter_mut() {
        boid.info.separation = new_boids_info.separation_weight;
        boid.info.alignment = new_boids_info.alignment_weight;
        boid.info.cohesion = new_boids_info.cohesion_weight;
        boid.info.neighbor_radius = new_boids_info.neighbor_radius;
        boid.info.neighbor_exit_radius = new_boids_info.neighbor_exit_radius;
        boid.velocity = new_boids_info.velocity;
    }
}
