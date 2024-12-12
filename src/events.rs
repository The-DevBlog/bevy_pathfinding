use bevy::prelude::*;

// Phase 1
#[derive(Event)]
pub struct SetTargetCellEv;

// Phase 2
#[derive(Event)]
pub struct SetFlowFieldEv;

// Phase 3
#[derive(Event)]
pub struct DetectCollidersEv;

// Phase 4
#[derive(Event)]
pub struct CalculateFlowFieldEv;

// Phase 5
#[derive(Event)]
pub struct CalculateFlowVectorsEv;

#[derive(Event)]
pub struct InitializeFlowFieldEv;

#[derive(Event)]
pub struct SetActiveFlowfieldEv;
