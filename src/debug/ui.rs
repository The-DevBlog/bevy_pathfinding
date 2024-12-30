use super::components::*;
use super::resources;
use super::resources::*;

use bevy::{prelude::*, window::PrimaryWindow};

const CLR_TXT: Color = Color::srgb(0.8, 0.8, 0.8);
const CLR_TITLE: Color = Color::srgb(0.6, 0.6, 0.6);
const CLR_BTN_HOVER: Color = Color::srgba(0.27, 0.27, 0.27, 1.0);
const CLR_BORDER: Color = Color::srgba(0.53, 0.53, 0.53, 1.0);
const CLR_BACKGROUND_1: Color = Color::srgba(0.18, 0.18, 0.18, 1.0);
const CLR_BACKGROUND_2: Color = Color::srgba(0.11, 0.11, 0.11, 1.0);
const FONT_SIZE: f32 = 14.0;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, draw_ui_box.after(resources::load_dbg_icon))
            .add_systems(
                Update,
                (
                    handle_dropdown_click,
                    handle_hide_dbg_interaction,
                    handle_drawmode_option_interaction,
                    handle_draw_grid_interaction,
                    handle_drag,
                ),
            )
            .add_observer(hide_options)
            .add_observer(toggle_dbg_visibility)
            .add_observer(toggle_dropdown_visibility)
            .add_observer(update_active_dropdown_option);
    }
}

#[derive(Event)]
struct ToggleDbgVisibilityEv(bool);

#[derive(Event)]
struct HideOptionsEv;

#[derive(Bundle)]
struct DropDownBtnBundle {
    visible_node: VisibleNode,
    comp: DropdownBtn,
    btn: Button,
    background_clr: BackgroundColor,
    border_clr: BorderColor,
    border_radius: BorderRadius,
    node: Node,
    name: Name,
}

#[derive(Bundle)]
struct ActiveOptionCtrBundle {
    comp: ActiveOptionCtr,
    border_clr: BorderColor,
    node: Node,
    name: Name,
}

#[derive(Bundle)]
struct DrawModeTxtCtr {
    comp: DrawModeTxt,
    txt: Text,
    txt_font: TextFont,
    txt_clr: TextColor,
    name: Name,
}

#[derive(Bundle)]
struct OptionBoxCtr {
    comp: OptionBox,
    txt: Text,
    txt_font: TextFont,
    txt_clr: TextColor,
}

#[derive(Bundle)]
struct DropdownOptionsCtr {
    comp: DropdownOptions,
    visible_node: VisibleNode,
    background_clr: BackgroundColor,
    border_radius: BorderRadius,
    node: Node,
    name: Name,
}

#[derive(Bundle)]
struct OptionTxtCtr {
    comp: OptionTxt,
    txt: Text,
    txt_font: TextFont,
    txt_clr: TextColor,
}

fn handle_drawmode_option_interaction(
    mut cmds: Commands,
    mut q_option: Query<
        (&Interaction, &SetActiveOption, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut dbg: ResMut<DebugOptions>,
) {
    for (interaction, option, mut background) in q_option.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                match option.set {
                    OptionsSet::One => dbg.draw_mode_1 = DrawMode::cast(option.txt.clone()),
                    OptionsSet::Two => dbg.draw_mode_2 = DrawMode::cast(option.txt.clone()),
                }

                cmds.trigger(UpdateDropdownOptionEv);
            }
            Interaction::Hovered => background.0 = CLR_BTN_HOVER.into(),
            Interaction::None => background.0 = CLR_BACKGROUND_2.into(),
        }
    }
}

fn handle_draw_grid_interaction(
    mut q_draw_grid: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<DrawGridBtn>),
    >,
    mut q_txt: Query<&mut Text, With<DrawGridTxt>>,
    mut dbg: ResMut<DebugOptions>,
) {
    for (interaction, mut background) in q_draw_grid.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                dbg.draw_grid = !dbg.draw_grid;

                if let Ok(mut txt) = q_txt.get_single_mut() {
                    txt.0 = format!("Grid: {}", dbg.draw_grid);
                }
            }
            Interaction::Hovered => background.0 = CLR_BTN_HOVER.into(),
            Interaction::None => background.0 = CLR_BACKGROUND_2.into(),
        }
    }
}

fn handle_hide_dbg_interaction(
    mut q_hide_dbg: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<HideDbgBtn>),
    >,
    mut dbg: ResMut<DebugOptions>,
    mut cmds: Commands,
) {
    for (interaction, mut background) in q_hide_dbg.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                cmds.trigger(ToggleDbgVisibilityEv(dbg.hide));
                dbg.hide = !dbg.hide;
            }
            Interaction::Hovered => background.0 = CLR_BTN_HOVER.into(),
            Interaction::None => background.0 = CLR_BACKGROUND_1.into(),
        }
    }
}

fn toggle_dbg_visibility(
    trigger: Trigger<ToggleDbgVisibilityEv>,
    mut q_node: Query<&mut Node, With<VisibleNode>>,
    mut q_title_bar: Query<&mut BorderRadius, With<TitleBar>>,
    mut cmds: Commands,
) {
    let visible = trigger.event().0;
    let Ok(mut title_bar) = q_title_bar.get_single_mut() else {
        return;
    };

    for mut node in q_node.iter_mut() {
        if visible {
            node.display = Display::Flex;
            *title_bar = BorderRadius::all(Val::Px(0.0));
            cmds.trigger(HideOptionsEv);
        } else {
            node.display = Display::None;
            *title_bar = BorderRadius::bottom(Val::Px(10.0));
        }
    }
}

fn hide_options(
    _trigger: Trigger<HideOptionsEv>,
    mut q_node: Query<&mut Node, With<DropdownOptions>>,
) {
    for mut node in q_node.iter_mut() {
        node.display = Display::None;
    }
}

fn update_active_dropdown_option(
    _trigger: Trigger<UpdateDropdownOptionEv>,
    dbg: Res<DebugOptions>,
    mut q_txt: Query<(&mut Text, &OptionBox)>,
) {
    for (mut txt, options) in q_txt.iter_mut() {
        let num = match options.0 {
            OptionsSet::One => 1,
            OptionsSet::Two => 2,
        };

        txt.0 = dbg.mode_string(num);
    }
}

fn handle_dropdown_click(
    mut cmds: Commands,
    mut q_btn: Query<
        (&Interaction, &DropdownBtn, &mut BackgroundColor),
        (Changed<Interaction>, With<DropdownBtn>),
    >,
) {
    for (interaction, dropdown, mut background) in q_btn.iter_mut() {
        match interaction {
            Interaction::Pressed => cmds.trigger(ToggleModeEv(dropdown.0)),
            Interaction::Hovered => background.0 = CLR_BTN_HOVER.into(),
            Interaction::None => background.0 = CLR_BACKGROUND_2.into(),
        }
    }
}

fn handle_drag(
    mut local_offset: Local<Option<Vec2>>,
    q_title_bar: Query<&Interaction, With<TitleBar>>,
    mut q_ui: Query<&mut Node, With<DebugUI>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(mut ui_style) = q_ui.get_single_mut() else {
        return;
    };

    let Some(cursor_pos) = window_q.single().cursor_position() else {
        return;
    };

    // Get the current UI position
    let current_left = match ui_style.left {
        Val::Px(val) => val,
        _ => 0.0,
    };
    let current_top = match ui_style.top {
        Val::Px(val) => val,
        _ => 0.0,
    };

    for interaction in &q_title_bar {
        match interaction {
            Interaction::Pressed => {
                // store offset on initial press
                if local_offset.is_none() {
                    *local_offset = Some(Vec2::new(
                        cursor_pos.x - current_left,
                        cursor_pos.y - current_top,
                    ));
                }

                // Use the stored offset to position the UI precisely
                if let Some(offset) = *local_offset {
                    ui_style.left = Val::Px(cursor_pos.x - offset.x);
                    ui_style.top = Val::Px(cursor_pos.y - offset.y);
                }
            }
            _ => *local_offset = None,
        }
    }
}

fn toggle_dropdown_visibility(
    trigger: Trigger<ToggleModeEv>,
    mut q_dropdown: Query<(&mut Node, &DropdownOptions)>,
) {
    let option = trigger.event().0;

    for (mut dropdown, dropdown_options) in q_dropdown.iter_mut() {
        if option == OptionsSet::One && dropdown_options.0.to_num() == 1 {
            if dropdown.display == Display::Flex {
                dropdown.display = Display::None;
            } else if dropdown.display == Display::None {
                dropdown.display = Display::Flex
            }
        } else if option == OptionsSet::Two && dropdown_options.0.to_num() == 2 {
            if dropdown.display == Display::Flex {
                dropdown.display = Display::None;
            } else if dropdown.display == Display::None {
                dropdown.display = Display::Flex
            }
        }
    }
}

fn draw_ui_box(mut cmds: Commands, dbg: Res<DebugOptions>, dbg_icon: Res<DbgIcon>) {
    let root_ctr = (
        RootCtr,
        Node {
            flex_direction: FlexDirection::Column,
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BorderColor::from(CLR_BORDER),
        BorderRadius::all(Val::Px(10.0)),
        BackgroundColor::from(CLR_BACKGROUND_1),
        DebugUI,
        Name::new("Debug Container"),
    );

    let title_bar = (
        TitleBar,
        BorderColor::from(CLR_BORDER),
        Node {
            border: UiRect::bottom(Val::Px(1.0)),
            padding: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        Name::new("Title Bar"),
    );

    let hide_dbg_btn = (
        HideDbgBtn,
        ImageNode::new(dbg_icon.0.clone()),
        BorderRadius::all(Val::Px(10.0)),
        Node {
            width: Val::Px(25.0),
            height: Val::Px(25.0),
            ..default()
        },
        Name::new("Debug Icon"),
    );

    let title_bar_txt = (
        Title,
        VisibleNode,
        Text::new("Pathfinding Debug"),
        Node {
            margin: UiRect::new(Val::Px(5.0), Val::Auto, Val::Auto, Val::Auto),
            ..default()
        },
        TextFont::from_font_size(FONT_SIZE + 2.0),
        TextColor::from(CLR_TITLE),
    );

    let draw_grid_btn = (
        DrawGridBtn,
        VisibleNode,
        BackgroundColor::from(CLR_BACKGROUND_2),
        BorderColor::from(CLR_BORDER),
        Node {
            padding: UiRect::all(Val::Px(5.0)),
            border: UiRect::bottom(Val::Px(1.0)),
            ..default()
        },
        Name::new("Draw Grid Button"),
    );

    let draw_grid_txt = (
        DrawGridTxt,
        Text::new(format!("Grid: {}", dbg.draw_grid)),
        TextFont::from_font_size(FONT_SIZE),
        TextColor::from(CLR_TXT),
        Name::new("Draw Grid Txt"),
    );

    let dropdown_btn = |set: OptionsSet, border: UiRect| -> DropDownBtnBundle {
        let radius = match set {
            OptionsSet::One => BorderRadius::ZERO,
            OptionsSet::Two => BorderRadius::bottom(Val::Px(10.0)),
        };
        DropDownBtnBundle {
            comp: DropdownBtn(set),
            visible_node: VisibleNode,
            btn: Button::default(),
            background_clr: BackgroundColor::from(CLR_BACKGROUND_2),
            border_clr: BorderColor::from(CLR_BORDER),
            border_radius: radius,
            node: Node {
                border,
                ..default()
            },
            name: Name::new("Dropdown Button"),
        }
    };

    let active_option_ctr = || -> ActiveOptionCtrBundle {
        ActiveOptionCtrBundle {
            comp: ActiveOptionCtr,
            border_clr: BorderColor::from(CLR_BORDER),
            node: Node {
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            name: Name::new("Draw Mode Container"),
        }
    };

    let draw_mode_txt = |txt: String| -> DrawModeTxtCtr {
        DrawModeTxtCtr {
            comp: DrawModeTxt,
            txt: Text::new(txt),
            txt_font: TextFont::from_font_size(FONT_SIZE),
            txt_clr: TextColor::from(CLR_TXT),
            name: Name::new("Draw Mode Text"),
        }
    };

    let active_option = |txt: String, set: OptionsSet| -> OptionBoxCtr {
        OptionBoxCtr {
            comp: OptionBox(set),
            txt: Text::new(txt),
            txt_font: TextFont::from_font_size(FONT_SIZE),
            txt_clr: TextColor::from(CLR_TXT),
        }
    };

    let options_container =
        |radius: Option<BorderRadius>, options_set: OptionsSet| -> DropdownOptionsCtr {
            let radius = match radius {
                Some(r) => r,
                None => BorderRadius::ZERO,
            };
            DropdownOptionsCtr {
                comp: DropdownOptions(options_set),
                visible_node: VisibleNode,
                background_clr: BackgroundColor::from(CLR_BACKGROUND_2),
                border_radius: radius,
                node: Node {
                    flex_direction: FlexDirection::Column,
                    display: Display::None,
                    ..default()
                },
                name: Name::new("Options Container"),
            }
        };

    let btn_option = |set: OptionsSet,
                      txt: String,
                      radius: Option<BorderRadius>|
     -> (SetActiveOption, BorderRadius, Node) {
        let radius = match radius {
            Some(r) => r,
            None => BorderRadius::ZERO,
        };

        (
            SetActiveOption { set, txt },
            radius,
            Node {
                padding: UiRect::new(Val::Px(5.0), Val::Px(5.0), Val::Px(2.5), Val::Px(2.5)),
                ..default()
            },
        )
    };

    let option_txt = |txt: String| -> OptionTxtCtr {
        OptionTxtCtr {
            comp: OptionTxt,
            txt: Text::new(txt),
            txt_font: TextFont::from_font_size(FONT_SIZE),
            txt_clr: TextColor::from(CLR_TXT),
        }
    };

    // Root Container
    cmds.spawn(root_ctr).with_children(|ctr| {
        // Title Bar
        ctr.spawn(title_bar).with_children(|title_bar| {
            title_bar.spawn(hide_dbg_btn);
            title_bar.spawn(title_bar_txt);
        });

        // Draw Grid
        ctr.spawn(draw_grid_btn).with_children(|ctr| {
            ctr.spawn(draw_grid_txt);
        });

        // Draw Mode 1 Container
        ctr.spawn(dropdown_btn(OptionsSet::One, UiRect::bottom(Val::Px(0.5))))
            .with_children(|dropdown| {
                dropdown
                    .spawn(active_option_ctr())
                    .with_children(|draw_mode| {
                        // Active Dropdown Option
                        draw_mode.spawn(draw_mode_txt("Draw Mode 1 : ".to_string()));
                        draw_mode.spawn(active_option(dbg.mode1_string(), OptionsSet::One));
                    });
            });

        // Dropdown Options Container
        ctr.spawn(options_container(None, OptionsSet::One))
            // Dropdown Options
            .with_children(|options| {
                options
                    .spawn(btn_option(OptionsSet::One, "None".to_string(), None))
                    .with_children(|btn| {
                        btn.spawn(option_txt("> None".to_string()));
                    });
                options
                    .spawn(btn_option(
                        OptionsSet::One,
                        "IntegrationField".to_string(),
                        None,
                    ))
                    .with_children(|btn| {
                        btn.spawn(option_txt("> IntegrationField".to_string()));
                    });
                options
                    .spawn(btn_option(OptionsSet::One, "FlowField".to_string(), None))
                    .with_children(|btn| {
                        btn.spawn(option_txt("> FlowField".to_string()));
                    });
                options
                    .spawn(btn_option(OptionsSet::One, "CostField".to_string(), None))
                    .with_children(|btn| {
                        btn.spawn(option_txt("> CostField".to_string()));
                    });
                options
                    .spawn(btn_option(OptionsSet::One, "Index".to_string(), None))
                    .with_children(|btn| {
                        btn.spawn(option_txt("> Index".to_string()));
                    });
            });

        // Draw Mode 2 Container
        ctr.spawn(dropdown_btn(OptionsSet::Two, UiRect::top(Val::Px(0.5))))
            .with_children(|dropdown| {
                dropdown
                    .spawn(active_option_ctr())
                    .with_children(|draw_mode| {
                        // Dropdown Active Option
                        draw_mode.spawn(draw_mode_txt("Draw Mode 2 : ".to_string()));
                        draw_mode.spawn(active_option(dbg.mode2_string(), OptionsSet::Two));
                    });
            });

        // Dropdown Options Container
        ctr.spawn(options_container(
            Some(BorderRadius::bottom(Val::Px(10.0))),
            OptionsSet::Two,
        ))
        // Dropdown Options
        .with_children(|options| {
            options
                .spawn(btn_option(OptionsSet::Two, "None".to_string(), None))
                .with_children(|btn| {
                    btn.spawn(option_txt("> None".to_string()));
                });
            options
                .spawn(btn_option(
                    OptionsSet::Two,
                    "IntegrationField".to_string(),
                    None,
                ))
                .with_children(|btn| {
                    btn.spawn(option_txt("> IntegrationField".to_string()));
                });
            options
                .spawn(btn_option(OptionsSet::Two, "FlowField".to_string(), None))
                .with_children(|btn| {
                    btn.spawn(option_txt("> FlowField".to_string()));
                });
            options
                .spawn(btn_option(OptionsSet::Two, "CostField".to_string(), None))
                .with_children(|btn| {
                    btn.spawn(option_txt("> CostField".to_string()));
                });
            options
                .spawn(btn_option(
                    OptionsSet::Two,
                    "Index".to_string(),
                    Some(BorderRadius::bottom(Val::Px(10.0))),
                ))
                .with_children(|btn| {
                    btn.spawn(option_txt("> Index".to_string()));
                });
        });
    });
}
