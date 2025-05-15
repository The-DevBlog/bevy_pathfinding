use bevy::prelude::*;

#[derive(Component)]
#[require(Node)]
pub struct ActiveOptionCtr;

#[derive(Component, Clone, Copy)]
pub struct VisibleNode;

#[derive(Component)]
#[require(Button)]
pub struct HideDbgBtn;

#[derive(Component)]
pub struct DebugUI;

#[derive(Component, PartialEq)]
#[require(Button)]
pub enum DrawGridBtn {
    Grid,
    SpatialGrid,
}

#[derive(Component, PartialEq)]
pub enum DrawGridTxt {
    Grid,
    SpatialGrid,
}

#[derive(Component)]
#[require(Button)]
pub struct DropdownBtn(pub OptionsSet);

#[derive(Component)]
#[require(Node)]
pub struct DropdownOptions(pub OptionsSet);

#[derive(Component)]
#[require(Text)]
pub struct DrawModeTxt;

#[derive(Component)]
#[require(Text)]
pub struct OptionTxt;

#[derive(Component)]
#[require(Text)]
pub struct OptionBox(pub OptionsSet);

#[derive(Component)]
#[require(Node)]
pub struct RootCtr;

#[derive(Component)]
#[require(Button)]
pub struct SetActiveOption {
    pub set: OptionsSet,
    pub txt: String,
}

#[derive(Component)]
#[require(Text)]
pub struct Title;

#[derive(Component)]
#[require(Button, Node)]
pub struct TitleBar;

#[derive(Event)]
pub struct ToggleModeEv(pub OptionsSet);

#[derive(Event)]
pub struct UpdateDropdownOptionEv;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OptionsSet {
    One,
    Two,
}

impl OptionsSet {
    pub fn to_num(&self) -> i32 {
        match self {
            OptionsSet::One => 1,
            OptionsSet::Two => 2,
        }
    }
}

#[derive(Component, Copy, Clone)]
pub struct CostMarker;

#[derive(Component, Copy, Clone)]
pub struct BestCostMarker;

#[derive(Component, Copy, Clone)]
pub struct IndexMarker;

#[derive(Component, Clone, Copy)]
pub struct FlowFieldMarker;
