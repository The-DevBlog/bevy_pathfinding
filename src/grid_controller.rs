use bevy::prelude::*;
use bevy_rapier3d::plugin::RapierContext;

use crate::{debug::DrawCostFieldEv, flowfield::*, InitializeFlowFieldEv};

pub struct GridControllerPlugin;

impl Plugin for GridControllerPlugin {
    fn build(&self, app: &mut App) {
        app.observe(initialize_flowfield);
    }
}

#[derive(Component)]
pub struct GridController {
    pub map_size: Vec2,
    pub grid_size: IVec2,
    pub cell_radius: f32,
    pub current_flowfield: FlowField,
}

impl GridController {
    pub fn initialize_flowfield(&mut self) {
        self.current_flowfield = FlowField::new(self.cell_radius, self.grid_size);
        self.current_flowfield.create_grid();
    }
}

fn initialize_flowfield(
    _trigger: Trigger<InitializeFlowFieldEv>,
    mut cmds: Commands,
    mut grid_controller_q: Query<&mut GridController>,
    rapier_ctx: Res<RapierContext>,
) {
    for mut grid_controller in grid_controller_q.iter_mut() {
        grid_controller.initialize_flowfield();
        grid_controller
            .current_flowfield
            .create_costfield(&rapier_ctx);
    }

    cmds.trigger(DrawCostFieldEv);
}
