use bevy::prelude::*;

#[derive(Component)]
pub struct MapBase;

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct Destination;

#[derive(Component)]
pub struct RtsObjSize(pub Vec2);

#[derive(Component, Default)]
#[require(Boid)]
pub struct RtsObj;

// separation: pushes boids away from each other
// alignment: aligns boids with their neighbors
// cohesion: pulls boids towards the center of their neighbors
#[derive(Component, Debug)]
pub struct Boid {
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
    pub max_speed: f32,
    pub neighbor_radius: f32,
}

impl Default for Boid {
    fn default() -> Self {
        Self {
            separation_weight: 50.0,
            alignment_weight: 25.0,
            cohesion_weight: 0.75,
            max_speed: 4.0,
            neighbor_radius: 30.0,
        }
    }
}
