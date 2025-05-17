use bevy::prelude::*;

use crate::flowfield::FlowField;

/// Event to initialize the flowfield. This event is used to set the destination position for the flowfield and the entities that will be affected by it.
///
/// # Example
///
/// ```
/// #[derive(Component)]
/// struct Unit;
///
/// // uses the mouse position to set the destination of all units
/// fn set_unit_destination(
///     mut cmds: Commands,
///     input: Res<ButtonInput<MouseButton>>,
///     mut q_units: Query<Entity, With<Unit>>,
///     q_map: Query<&GlobalTransform, With<MapBase>>,
///     q_cam: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
///     q_window: Query<&Window, With<PrimaryWindow>>,
///     dbg_options: ResMut<DbgOptions>,
/// ) {
///     if dbg_options.hover {
///         return;
///     }
///
///     if !input.just_pressed(MouseButton::Left) {
///         return;
///     }
///
///     let Ok(map_tf) = q_map.single() else {
///         return;
///     };
///
///     let Ok((cam, cam_transform)) = q_cam.single() else {
///         return;
///     };
///
///     let Ok(window) = q_window.single() else {
///         return;
///     };
///
///     let Some(cursor_pos) = window.cursor_position() else {
///         return;
///     };
///
///     // collect all the units you wish to assign to a flowfield
///     let mut units = Vec::new();
///     for unit_entity in q_units.iter_mut() {
///         units.push(unit_entity);
///     }
///
///     // get the destination position in world space using the 'get_world_pos' function
///     let destination_pos = utils::get_world_pos(map_tf, cam_transform, cam, cursor_pos);
///
///     // create a flowfield and assign the units and destination position it it
///     cmds.trigger(InitializeFlowFieldEv {
///         entities: units,
///         destination_pos,
///     });
/// }
/// ```
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
