use bevy::prelude::*;

use crate::flowfield::FlowField;

#[derive(Event)]
pub struct InitializeFlowFieldEv(pub Vec<Entity>);

#[derive(Event)]
pub struct SetActiveFlowfieldEv(pub Option<FlowField>);
