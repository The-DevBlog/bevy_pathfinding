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
pub struct FlowFieldEntity(pub Entity);
