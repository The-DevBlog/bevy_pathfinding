use bevy::prelude::*;

use crate::*;

#[derive(Component)]
pub struct MapBase;

#[derive(Component)]
pub struct Selected(pub bool);

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component)]
pub struct Unit;

#[derive(Component, Default)]
pub struct Destination {
    pub endpoint: Option<Vec3>,
    pub waypoints: Vec<Cell>,
}
