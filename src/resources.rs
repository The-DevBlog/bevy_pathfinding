use std::collections::HashMap;
use bevy::prelude::*;

use crate::{flowfield::FlowField, grid::Grid};

pub struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveDebugFlowfield>()
            .init_resource::<CostMap>()
            .add_systems(
                Update,
                init_costfield_map.run_if(resource_exists::<Grid>.and(run_once)),
            );
    }
}

#[derive(Resource, Default)]
pub struct ActiveDebugFlowfield(pub Option<FlowField>);

#[derive(Resource, Default)]
pub struct CostMap(pub HashMap<i32, u8>);

fn init_costfield_map(grid: Res<Grid>, mut costmap: ResMut<CostMap>) {
    for cell_row in grid.grid.iter() {
        for cell in cell_row.iter() {
            costmap.0.insert(cell.idx_to_id(grid.grid.len()), cell.cost);
        }
    }
}
