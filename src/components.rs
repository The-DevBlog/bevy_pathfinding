use std::collections::HashSet;

use bevy::prelude::*;

/// A marker component for the map base. Insert this into your base map entity.
#[derive(Component)]
pub struct MapBase;

/// A marker component for the primary camera. Insert this into your camera entity.
#[derive(Component)]
pub struct GameCamera;

/// Destination marker. This is dynamically added to every boid entity when the flowfield is initialized.
#[derive(Component)]
pub struct Destination;

/// Obstacle marker. Insert this into any entity that you want to be considered an obstacle by the flowfield(s).
/// # Parameters
/// - `Vec2`: The size of the obstacles mesh. Only the x and z values are used.
#[derive(Component, Default)]
pub struct Obstacle(pub Vec2);

/// Boid component with settings. Insert this into your entities that you want to control with the flowfields. Use the Boid::new() to use custom settings.
#[derive(Component, Debug)]
pub struct Boid {
    pub steering: Vec3,
    pub prev_neighbors: HashSet<Entity>, // store last frame's neighbors
    pub velocity: Vec3,
    pub prev_steer: Vec3, // start at rest
    pub info: BoidsInfo,
}

impl Default for Boid {
    fn default() -> Self {
        Self {
            steering: Vec3::ZERO,
            prev_neighbors: HashSet::new(),
            velocity: Vec3::ZERO,
            prev_steer: Vec3::ZERO, // start at rest
            info: BoidsInfo::default(),
        }
    }
}

impl Boid {
    /// Creates a new boid with the given parameters.
    ///
    /// # Example
    /// ```
    /// Boid::new(50.0, 1.0, 1.0, 5.0);
    /// ```
    ///
    /// # Parameters
    /// - `separation`: The separation weight. This determines how much the boid will try to avoid other boids.
    /// - `alignment`: The alignment weight. This determines how much the boid will try to align its velocity with other boids.
    /// - `cohesion`: The cohesion weight. This determines how much the boid will try to move towards the center of its neighbors.
    /// - `radius`: The radius of the boid. This determines how far the boid can see its neighbors before it applies the steering forces.
    pub fn new(separation: f32, alignment: f32, cohesion: f32, radius: f32) -> Self {
        let neighbor_radius = radius;
        Self {
            steering: Vec3::ZERO,
            prev_neighbors: HashSet::new(),
            velocity: Vec3::ZERO,
            prev_steer: Vec3::ZERO,
            info: BoidsInfo {
                separation,
                alignment,
                cohesion,
                neighbor_radius: neighbor_radius,
                neighbor_exit_radius: neighbor_radius * 1.05,
                max_speed: 3.0,
                max_force: 0.5,
            },
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
    /// How fast the boid is allowed to go (world-units per second)
    pub max_speed: f32,
    /// How strong each steering force pulse can be
    pub max_force: f32,
}

impl Default for BoidsInfo {
    fn default() -> Self {
        let neighbor_radius = 5.0;
        Self {
            separation: 50.0,                             // strongest urge to avoid collisions
            alignment: 0.0,                               // medium urge to line up
            cohesion: 0.0,                                // medium urge to stay together
            neighbor_radius: neighbor_radius,             // in world‐units (tweak to taste)
            neighbor_exit_radius: neighbor_radius * 1.05, // new: slightly larger
            max_speed: 3.0,                               // 3 world-units per second
            max_force: 0.5,                               // max steering change per second
        }
    }
}
