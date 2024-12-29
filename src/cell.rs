use bevy::prelude::*;
use std::u16;

use crate::grid_direction::GridDirection;

#[derive(Clone, Default, Copy, Debug, PartialEq, Reflect)]
pub struct Cell {
    pub best_cost: u16,
    pub best_direction: GridDirection,
    pub cost: u8,
    pub idx: IVec2,
    pub world_pos: Vec3,
}

impl Cell {
    pub fn new(world_position: Vec3, grid_idx: IVec2) -> Self {
        Cell {
            best_cost: u16::MAX,
            best_direction: GridDirection::None,
            cost: 1,
            idx: grid_idx,
            world_pos: world_position,
        }
    }

    pub fn increase_cost(&mut self, amount: u8) {
        if self.cost == u8::MAX {
            return;
        }

        if let Some(new_cost) = self.cost.checked_add(amount) {
            self.cost = new_cost;
        } else {
            self.cost = u8::MAX;
        }
    }
}
