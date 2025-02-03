use bevy::prelude::*;

use crate::{cell::Cell, flowfield::FlowField};

#[derive(Event)]
pub struct InitializeFlowFieldEv(pub Vec<Entity>);

#[derive(Event)]
pub struct InitializeMiniFlowFieldEv(pub Entity);

#[derive(Event)]
pub struct SetActiveFlowfieldEv(pub Option<FlowField>);

#[derive(Event)]
pub struct DrawCostFieldEv;

#[derive(Event)]
pub struct UpdateCostEv {
    pub cell: Cell,
}

#[derive(Event)]
pub struct DrawFlowFieldEv;

impl UpdateCostEv {
    pub fn new(cell: Cell) -> Self {
        Self { cell }
    }
}
