use bevy::prelude::*;

#[derive(Component)]
pub struct MapBase;

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct Destination;

#[derive(Component)]
pub struct UnitSize(pub Vec2);

#[derive(Component)]
#[require(Boid)]
pub struct Unit;

#[derive(Component)]
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
            separation_weight: 40.0,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,
            max_speed: 4.0,
            neighbor_radius: 25.0,
        }
    }
}
