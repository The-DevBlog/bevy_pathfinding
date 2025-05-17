use bevy::prelude::*;

use crate::flowfield::FlowField;

#[derive(Event)]
pub struct InitializeFlowFieldEv {
    pub entities: Vec<Entity>,
    pub destination_pos: Vec3,
}

#[derive(Event)]
pub struct SetActiveFlowfieldEv(pub Option<FlowField>);

#[derive(Event)]
pub struct DrawCostFieldEv;

#[derive(Event)]
pub struct UpdateCostEv;

#[derive(Event)]
pub struct DrawAllEv;

#[derive(Event)]
pub struct DrawGridEv;

#[derive(Event)]
pub struct DrawIntegrationFieldEv;

#[derive(Event)]
pub struct DrawFlowFieldEv;
