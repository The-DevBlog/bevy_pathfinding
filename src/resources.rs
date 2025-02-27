use bevy::prelude::*;

use crate::flowfield::FlowField;

pub struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveDbgFlowfield>();
    }
}

#[derive(Resource, Default, Clone)]
pub struct ActiveDbgFlowfield(pub Option<FlowField>);
