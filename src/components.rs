use std::collections::HashSet;

use bevy::prelude::*;

#[derive(Component)]
pub struct MapBase;

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct Destination;

#[derive(Component)]
pub struct RtsObjSize(pub Vec3);

#[derive(Component, Default)]
#[require(Boid)]
pub struct RtsObj;

#[derive(Component, Debug)]
pub struct Boid {
    pub prev_neighbors: HashSet<Entity>, // store last frame's neighbors
    pub velocity: Vec3,                  // start at rest
    pub max_force: f32,                  // how quickly you can turn
    pub separation_weight: f32,          // push apart
    pub alignment_weight: f32,           // match heading
    pub cohesion_weight: f32,            // pull toward center
    pub max_speed: f32,                  // top movement speed
    pub neighbor_radius: f32,            // how far you “see” neighbors
    pub neighbor_exit_radius: f32,       // new: slightly larger
}

impl Default for Boid {
    fn default() -> Self {
        let max_speed = 4.0;
        Self {
            prev_neighbors: HashSet::new(),
            velocity: Vec3::ZERO,
            max_force: max_speed * 0.1, // ~0.4 units/sec² of turn acceleration
            separation_weight: 40.0,    // strongest urge to avoid collisions
            alignment_weight: 5.0,      // medium urge to line up
            cohesion_weight: 1.0,       // medium urge to stay together
            max_speed,                  // units per second
            neighbor_radius: 25.0,      // in world‐units (tweak to taste)
            neighbor_exit_radius: 30.0, // new: slightly larger
        }
    }
}
