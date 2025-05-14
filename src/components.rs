use std::collections::HashSet;

use bevy::prelude::*;

pub struct ComponentsPlugin;

impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BoidsInfoUpdater>()
            .add_systems(PreStartup, spawn_boids_updater)
            .add_systems(Update, change_boids);
    }
}

#[derive(Component)]
pub struct MapBase;

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct Destination;

#[derive(Component, Default)]
pub struct RtsObj(pub Vec2);

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

// used for debugging purposes
#[derive(Component, Reflect)]
pub struct BoidsInfoUpdater {
    pub separation_weight: f32,    // push apart
    pub alignment_weight: f32,     // match heading
    pub cohesion_weight: f32,      // pull toward center
    pub neighbor_radius: f32,      // how far you “see” neighbors
    pub neighbor_exit_radius: f32, // new: slightly larger
}

impl Default for BoidsInfoUpdater {
    fn default() -> Self {
        let neighbor_radius = 45.0;
        Self {
            separation_weight: 50.0,          // strongest urge to avoid collisions
            alignment_weight: 0.0,            // medium urge to line up
            cohesion_weight: 0.0,             // medium urge to stay together
            neighbor_radius: neighbor_radius, // in world‐units (tweak to taste)
            neighbor_exit_radius: neighbor_radius * 1.05, // new: slightly larger
        }
    }
}

fn spawn_boids_updater(mut cmds: Commands) {
    cmds.spawn((BoidsInfoUpdater::default(), Name::new("Boids Info")));
}

fn change_boids(mut q_boids: Query<&mut Boid>, q_boid_values: Query<&BoidsInfoUpdater>) {
    let Ok(new_boids_info) = q_boid_values.single() else {
        return;
    };

    for mut boid in q_boids.iter_mut() {
        boid.info.separation = new_boids_info.separation_weight;
        boid.info.alignment = new_boids_info.alignment_weight;
        boid.info.cohesion = new_boids_info.cohesion_weight;
        boid.info.neighbor_radius = new_boids_info.neighbor_radius;
        boid.info.neighbor_exit_radius = new_boids_info.neighbor_exit_radius;
    }
}
