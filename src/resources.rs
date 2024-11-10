use bevy::prelude::*;

use crate::*;

#[derive(Resource)]
pub struct InitializeGrid {
    pub done: bool,
    pub delay: Timer,
}

impl Default for InitializeGrid {
    fn default() -> Self {
        Self {
            done: false,
            delay: Timer::from_seconds(0.05, TimerMode::Once),
        }
    }
}

#[derive(Resource)]
pub struct TargetCell {
    pub row: usize,
    pub column: usize,
}

impl TargetCell {
    pub fn new(cells_width: usize, cells_depth: usize) -> Self {
        let target = TargetCell {
            row: cells_width - 1,
            column: cells_depth - 1,
        };

        target
    }
}
