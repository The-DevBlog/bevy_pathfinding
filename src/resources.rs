use bevy::prelude::*;

use crate::flowfield::FlowField;

pub struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveDebugFlowfield>();
    }
}

#[derive(Resource, Default)]
pub struct ActiveDebugFlowfield(pub Option<FlowField>);
