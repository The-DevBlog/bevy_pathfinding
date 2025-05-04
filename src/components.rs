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
    pub velocity: Vec3,
    pub prev_steer: Vec3, // start at rest
    pub info: BoidsInfo,
}

impl Default for Boid {
    fn default() -> Self {
        // let max_speed = 4.0;
        Self {
            prev_neighbors: HashSet::new(),
            velocity: Vec3::ZERO,
            prev_steer: Vec3::ZERO, // start at rest
            info: BoidsInfo::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, Reflect)]
pub struct BoidsInfo {
    pub separation: f32,           // push apart
    pub alignment: f32,            // match heading
    pub cohesion: f32,             // pull toward center
    pub neighbor_radius: f32,      // how far you “see” neighbors
    pub neighbor_exit_radius: f32, // new: slightly larger
}

impl Default for BoidsInfo {
    fn default() -> Self {
        let neighbor_radius = 45.0;
        Self {
            separation: 50.0,                             // strongest urge to avoid collisions
            alignment: 0.0,                               // medium urge to line up
            cohesion: 0.0,                                // medium urge to stay together
            neighbor_radius: neighbor_radius,             // in world‐units (tweak to taste)
            neighbor_exit_radius: neighbor_radius * 1.05, // new: slightly larger
        }
    }
}
