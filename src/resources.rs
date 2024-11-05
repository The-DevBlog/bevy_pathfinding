use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct SetGridOccupantsOnce(pub bool);

#[derive(Resource)]
pub struct DelayedRunTimer(pub Timer);

impl Default for DelayedRunTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.5, TimerMode::Once)) // 0.5 seconds delay
    }
}

#[derive(Resource, Default)]
pub struct TargetCell {
    pub row: Option<u32>,
    pub column: Option<u32>,
}
