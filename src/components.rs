use std::collections::HashSet;

use bevy::prelude::*;

pub struct ComponentsPlugin;

impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BoidsInfoUpdater>()
            .init_resource::<BoidsInfoUpdater>()
            .add_systems(Update, change_boids);
    }
}

/// A marker component for the map base. Insert this into your base map entity.
#[derive(Component)]
pub struct MapBase;

/// A marker component for the primary camera. Insert this into your camera entity.
#[derive(Component)]
pub struct GameCamera;

/// Destination marker. This is dynamically added to every boid entity when the flowfield is initialized.
#[derive(Component)]
pub struct Destination;

/// The mesh size of the component, only considering the x and y axis.
#[derive(Component, Default)]
pub struct RtsObj(pub Vec2);

/// Boid component with settings. Insert this into your entities that you want to control with the flowfields. Use the Boid::new() to control boid behavior and settings.
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
        }
    }
}

/// DO NOT USE. This component is updated whenever the boids info in the debug UI menu changes.
#[derive(Resource, Reflect)]
pub struct BoidsInfoUpdater {
    pub separation_weight: f32,    // push apart
    pub alignment_weight: f32,     // match heading
    pub cohesion_weight: f32,      // pull toward center
    pub neighbor_radius: f32,      // how far you “see” neighbors
    pub neighbor_exit_radius: f32, // new: slightly larger
}

impl Default for BoidsInfoUpdater {
    fn default() -> Self {
        let neighbor_radius = 5.0;
        Self {
            separation_weight: 50.0,          // strongest urge to avoid collisions
            alignment_weight: 0.0,            // medium urge to line up
            cohesion_weight: 0.0,             // medium urge to stay together
            neighbor_radius: neighbor_radius, // in world‐units (tweak to taste)
            neighbor_exit_radius: neighbor_radius * 1.05, // new: slightly larger
        }
    }
}

fn change_boids(mut q_boids: Query<&mut Boid>, boid_updater: Res<BoidsInfoUpdater>) {
    for mut boid in q_boids.iter_mut() {
        boid.info.separation = boid_updater.separation_weight;
        boid.info.alignment = boid_updater.alignment_weight;
        boid.info.cohesion = boid_updater.cohesion_weight;
        boid.info.neighbor_radius = boid_updater.neighbor_radius;
        boid.info.neighbor_exit_radius = boid_updater.neighbor_exit_radius;
    }
}
