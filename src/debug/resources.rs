use bevy::prelude::*;

pub struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugOptions>()
            .init_resource::<Digits>()
            .register_type::<DebugOptions>();
    }
}

#[derive(Resource, Default)]
pub struct Digits(pub [Handle<Image>; 10]);

#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct DebugOptions {
    pub draw_grid: bool,
    pub draw_mode_1: DrawMode,
    pub draw_mode_2: DrawMode,
}

impl Default for DebugOptions {
    fn default() -> Self {
        DebugOptions {
            draw_grid: true,
            draw_mode_1: DrawMode::FlowField,
            draw_mode_2: DrawMode::Index,
        }
    }
}

impl DebugOptions {
    pub fn draw_mode_to_string(mode: DrawMode) -> String {
        match mode {
            DrawMode::None => String::from("None"),
            DrawMode::CostField => String::from("CostField"),
            DrawMode::FlowField => String::from("FlowField"),
            DrawMode::IntegrationField => String::from("IntegrationField"),
            DrawMode::Index => String::from("Index"),
        }
    }

    pub fn mode_string(&self, mode: i32) -> String {
        if mode == 1 {
            return Self::draw_mode_to_string(self.draw_mode_1);
        }

        return Self::draw_mode_to_string(self.draw_mode_2);
    }

    pub fn mode1_string(&self) -> String {
        Self::draw_mode_to_string(self.draw_mode_1)
    }

    pub fn mode2_string(&self) -> String {
        Self::draw_mode_to_string(self.draw_mode_2)
    }
}

#[derive(Reflect, PartialEq, Clone, Copy)]
pub enum DrawMode {
    None,
    CostField,
    FlowField,
    IntegrationField,
    Index,
}

impl DrawMode {
    pub fn cast(mode: String) -> Self {
        match mode.as_str() {
            "None" => DrawMode::None,
            "CostField" => DrawMode::CostField,
            "FlowField" => DrawMode::FlowField,
            "IntegrationField" => DrawMode::IntegrationField,
            "Index" => DrawMode::Index,
            _ => DrawMode::None,
        }
    }
}
