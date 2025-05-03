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
        app.register_type::<BoidsInfo>();

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
pub struct BoidsInfo {
    pub velocity: Vec3,            // start at rest
    pub max_force: f32,            // how quickly you can turn
    pub separation_weight: f32,    // push apart
    pub alignment_weight: f32,     // match heading
    pub cohesion_weight: f32,      // pull toward center
    pub max_speed: f32,            // top movement speed
    pub neighbor_radius: f32,      // how far you “see” neighbors
    pub neighbor_exit_radius: f32, // new: slightly larger
}

impl Default for BoidsInfo {
    fn default() -> Self {
        let max_speed = 30.0;
        Self {
            velocity: Vec3::ZERO,
            max_force: max_speed * 0.1, // ~0.4 units/sec² of turn acceleration
            separation_weight: 50.0,    // strongest urge to avoid collisions
            alignment_weight: 0.0,      // medium urge to line up
            cohesion_weight: 0.0,       // medium urge to stay together
            max_speed,                  // units per second
            neighbor_radius: 40.0,      // in world‐units (tweak to taste)
            neighbor_exit_radius: 40.0, // new: slightly larger
        }
    }
}

fn change_boids(
    mut cmds: Commands,
    mut q_boids: Query<&mut Boid>,
    q_boid_values: Query<&BoidsInfo>,
) {
    let Ok(new_boids_info) = q_boid_values.get_single() else {
        cmds.spawn((BoidsInfo::default(), Name::new("Boids Info")));
        return;
    };

    for mut boid in q_boids.iter_mut() {
        boid.separation_weight = new_boids_info.separation_weight;
        boid.alignment_weight = new_boids_info.alignment_weight;
        boid.cohesion_weight = new_boids_info.cohesion_weight;
        boid.max_speed = new_boids_info.max_speed;
        boid.neighbor_radius = new_boids_info.neighbor_radius;
        boid.neighbor_exit_radius = new_boids_info.neighbor_exit_radius;
        boid.max_force = new_boids_info.max_force;
        boid.velocity = new_boids_info.velocity;
    }
}
