use bevy::prelude::*;

use crate::{cell::Cell, flowfield::FlowField};

#[derive(Event)]
pub struct InitializeFlowFieldEv(pub Vec<Entity>);

#[derive(Event)]
pub struct InitializeDestinationFlowFieldsEv(pub Entity); // Entity is the parent FF

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
