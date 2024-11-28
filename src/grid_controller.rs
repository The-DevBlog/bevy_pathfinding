use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier3d::plugin::RapierContext;

use crate::{debug::DrawDebugEv, flowfield::*, utils, GameCamera, InitializeFlowFieldEv, MapBase};

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
    pub cur_flowfield: FlowField,
}

impl GridController {
    pub fn initialize_flowfield(&mut self) {
        self.cur_flowfield = FlowField::new(self.cell_radius, self.grid_size);
        self.cur_flowfield.create_grid();
    }

    pub fn cell_diameter(&self) -> f32 {
        self.cell_radius * 2.0
    }
}

fn initialize_flowfield(
    _trigger: Trigger<InitializeFlowFieldEv>,
    mut cmds: Commands,
    mut q_grid_controller: Query<&mut GridController>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_cam: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    rapier_ctx: Res<RapierContext>,
    q_map_base: Query<&GlobalTransform, With<MapBase>>,
) {
    let Some(mouse_pos) = q_windows.single().cursor_position() else {
        return;
    };

    let cam = match q_cam.get_single() {
        Ok(value) => value,
        Err(_) => return,
    };

    let map_base = match q_map_base.get_single() {
        Ok(value) => value,
        Err(_) => return,
    };

    for mut grid_controller in q_grid_controller.iter_mut() {
        grid_controller.initialize_flowfield();
        grid_controller.cur_flowfield.create_costfield(&rapier_ctx);

        let world_mouse_pos = utils::get_world_pos(map_base, cam.1, cam.0, mouse_pos);
        let destination_cell = grid_controller
            .cur_flowfield
            .get_cell_from_world_position(world_mouse_pos);

        grid_controller
            .cur_flowfield
            .create_integration_field(destination_cell);

        grid_controller.cur_flowfield.create_flowfield();
    }

    cmds.trigger(DrawDebugEv);
}
