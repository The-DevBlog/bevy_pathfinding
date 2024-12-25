use bevy::prelude::*;

use crate::{cell::Cell, flowfield::FlowField};

#[derive(Event)]
pub struct InitializeFlowFieldEv(pub Vec<Entity>);

#[derive(Event)]
pub struct SetActiveFlowfieldEv(pub Option<FlowField>);

#[derive(Event)]
pub struct UpdateCellEv {
    pub cell: Cell,
    pub entity: Entity,
}

impl UpdateCellEv {
    pub fn new(cell: Cell, entity: Entity) -> Self {
        Self { cell, entity }
    }
}
