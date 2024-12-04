use super::draw::{DrawMode, RtsPfDebug};
use bevy::prelude::*;

const CLR_TXT_ACTIVE: Color = Color::WHITE;
const CLR_TXT_DORMANT: Color = Color::srgb(0.8, 0.8, 0.8);
const CLR_BTN_HOVER: Color = Color::srgba(0.27, 0.27, 0.27, 1.0);
const CLR_BORDER: Color = Color::srgba(0.53, 0.53, 0.53, 1.0);
const CLR_BACKGROUND_1: Color = Color::srgba(0.18, 0.18, 0.18, 1.0);
const CLR_BACKGROUND_2: Color = Color::srgba(0.11, 0.11, 0.11, 1.0);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, draw_ui_box)
            .add_systems(Update, handle_draw_mode_interaction)
            .add_systems(Update, handle_dropdown_click)
            .observe(toggle_dropdown_visibility_1)
            .observe(toggle_dropdown_visibility_2)
            .observe(update_active_dropdown_option);
    }
}

#[derive(Event)]
struct UpdateDropdownOptionEv;

#[derive(Event)]
struct ToggleMode1Ev;

#[derive(Event)]
struct ToggleMode2Ev;

#[derive(Component)]
struct ActiveOption1(pub String);

#[derive(Component)]
struct ActiveOption2(pub String);

#[derive(Component)]
struct OptionBox1;

#[derive(Component)]
struct OptionBox2;

#[derive(Component)]
struct Options1;

#[derive(Component)]
struct Options2;

#[derive(Component)]
struct DropdownBtn1;

#[derive(Component)]
struct DropdownBtn2;

fn handle_draw_mode_interaction(
    mut cmds: Commands,
    mut q_option_1: Query<
        (&Interaction, &ActiveOption1, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<ActiveOption1>,
            Without<ActiveOption2>,
        ),
    >,
    mut q_option_2: Query<
        (&Interaction, &ActiveOption2, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<ActiveOption2>,
            Without<ActiveOption1>,
        ),
    >,
    mut dbg: ResMut<RtsPfDebug>,
) {
    for (interaction, option, mut background) in q_option_1.iter_mut() {
        background.0 = CLR_BACKGROUND_2.into();
        match interaction {
            Interaction::Pressed => {
                dbg.draw_mode_1 = DrawMode::cast(option.0.clone());
                cmds.trigger(UpdateDropdownOptionEv);
            }
            Interaction::Hovered => background.0 = CLR_BTN_HOVER.into(),
            Interaction::None => background.0 = CLR_BACKGROUND_2.into(),
        }
    }

    for (interaction, option, mut background) in q_option_2.iter_mut() {
        background.0 = CLR_BACKGROUND_2.into();
        match interaction {
            Interaction::Pressed => {
                dbg.draw_mode_2 = DrawMode::cast(option.0.clone());
                cmds.trigger(UpdateDropdownOptionEv);
            }
            Interaction::Hovered => background.0 = CLR_BTN_HOVER.into(),
            Interaction::None => background.0 = CLR_BACKGROUND_2.into(),
        }
    }
}

fn update_active_dropdown_option(
    _trigger: Trigger<UpdateDropdownOptionEv>,
    dbg: Res<RtsPfDebug>,
    mut q_txt_1: Query<&mut Text, (With<OptionBox1>, Without<OptionBox2>)>,
    mut q_txt_2: Query<&mut Text, (With<OptionBox2>, Without<OptionBox1>)>,
) {
    if let Ok(mut txt) = q_txt_1.get_single_mut() {
        txt.sections[0].value = dbg.mode1_string();
    }

    if let Ok(mut txt) = q_txt_2.get_single_mut() {
        txt.sections[0].value = dbg.mode2_string();
    }
}

fn handle_dropdown_click(
    mut cmds: Commands,
    q_btn_1: Query<&Interaction, (Changed<Interaction>, With<DropdownBtn1>)>,
    q_btn_2: Query<&Interaction, (Changed<Interaction>, With<DropdownBtn2>)>,
) {
    if let Ok(interaction) = q_btn_1.get_single() {
        match interaction {
            Interaction::Pressed => cmds.trigger(ToggleMode1Ev),
            _ => (),
        }
    }

    if let Ok(interaction) = q_btn_2.get_single() {
        match interaction {
            Interaction::Pressed => cmds.trigger(ToggleMode2Ev),
            _ => (),
        }
    }
}

fn toggle_dropdown_visibility_1(
    _trigger: Trigger<ToggleMode1Ev>,
    mut q_dropdown: Query<&mut Style, With<Options1>>,
) {
    if let Ok(mut dropdown) = q_dropdown.get_single_mut() {
        if dropdown.display == Display::Flex {
            dropdown.display = Display::None;
        } else if dropdown.display == Display::None {
            dropdown.display = Display::Flex
        }
    }
}

fn toggle_dropdown_visibility_2(
    _trigger: Trigger<ToggleMode2Ev>,
    mut q_dropdown: Query<&mut Style, With<Options2>>,
) {
    if let Ok(mut dropdown) = q_dropdown.get_single_mut() {
        if dropdown.display == Display::Flex {
            dropdown.display = Display::None;
        } else if dropdown.display == Display::None {
            dropdown.display = Display::Flex
        }
    }
}

fn draw_ui_box(mut cmds: Commands, dbg: Res<RtsPfDebug>) {
    let root_container = (
        NodeBundle {
            style: Style {
                padding: UiRect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(0.25)),
                ..default()
            },
            border_color: CLR_BORDER.into(),
            border_radius: BorderRadius::all(Val::Px(5.0)),
            background_color: CLR_BACKGROUND_1.into(),
            ..default()
        },
        Name::new("Debug Container"),
    );

    let draw_container = || -> NodeBundle {
        NodeBundle {
            style: Style {
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            ..default()
        }
    };

    let dropdown_btn = || ButtonBundle {
        style: Style {
            padding: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        ..default()
    };

    let draw_mode_txt = |txt: String| -> TextBundle {
        TextBundle {
            text: Text::from_section(
                txt,
                TextStyle {
                    color: CLR_TXT_DORMANT.into(),
                    font_size: 16.0,
                    ..default()
                },
            ),
            ..default()
        }
    };

    let options_container = || -> NodeBundle {
        NodeBundle {
            background_color: CLR_BACKGROUND_2.into(),
            border_color: CLR_BACKGROUND_2.into(),
            border_radius: BorderRadius::all(Val::Px(5.0)),
            style: Style {
                padding: UiRect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::Column,
                display: Display::None,
                ..default()
            },
            ..default()
        }
    };

    let option_txt = |txt: String| -> TextBundle {
        TextBundle {
            text: Text::from_section(
                txt.clone(),
                TextStyle {
                    color: CLR_TXT_DORMANT.into(),
                    font_size: 16.0,
                    ..default()
                },
            ),
            ..default()
        }
    };

    let btn_option_1 = |txt: String| -> (ButtonBundle, ActiveOption1) {
        (ButtonBundle::default(), ActiveOption1(txt))
    };

    let btn_option_2 = |txt: String| -> (ButtonBundle, ActiveOption2) {
        (ButtonBundle::default(), ActiveOption2(txt))
    };

    let active_option = |txt: String| -> TextBundle {
        TextBundle {
            text: Text::from_section(
                txt,
                TextStyle {
                    color: CLR_TXT_DORMANT.into(),
                    font_size: 16.0,
                    ..default()
                },
            ),
            ..default()
        }
    };

    let change_option_txt = || -> TextBundle {
        TextBundle {
            text: Text::from_section(
                String::from(" +"),
                TextStyle {
                    color: CLR_TXT_DORMANT.into(),
                    font_size: 18.0,
                    ..default()
                },
            ),
            ..default()
        }
    };

    let dropdown_1 = NodeBundle::default();
    let dropdown_2 = NodeBundle::default();

    let active_option_1 = (active_option(dbg.mode1_string()), OptionBox1);
    let active_option_2 = (active_option(dbg.mode2_string()), OptionBox2);

    // Root Container
    cmds.spawn(root_container).with_children(|container| {
        // Draw Mode 1 Container
        container
            .spawn((DropdownBtn1, dropdown_btn()))
            .with_children(|dropdown| {
                dropdown
                    .spawn(draw_container())
                    .with_children(|draw_container_btn| {
                        draw_container_btn.spawn(change_option_txt());
                    })
                    .with_children(|draw_mode_1| {
                        draw_mode_1.spawn(draw_mode_txt("Draw Mode 1: ".to_string()));

                        // Dropdown Container
                        draw_mode_1.spawn(dropdown_1).with_children(|dropdown| {
                            // Dropdown Active Option
                            dropdown.spawn(active_option_1);
                        });
                    });
            });

        // Dropdown Options Container
        container
            .spawn((options_container(), Options1))
            // Dropdown Options
            .with_children(|options| {
                options
                    .spawn(btn_option_1("None".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("None".to_string()));
                    });
                options
                    .spawn(btn_option_1("IntegrationField".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("IntegrationField".to_string()));
                    });
                options
                    .spawn(btn_option_1("FlowField".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("FlowField".to_string()));
                    });
                options
                    .spawn(btn_option_1("CostField".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("CostField".to_string()));
                    });
                options
                    .spawn(btn_option_1("Index".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("Index".to_string()));
                    });
            });

        // Draw Mode 2 Container
        container
            .spawn((DropdownBtn2, dropdown_btn()))
            .with_children(|dropdown| {
                dropdown
                    .spawn(draw_container())
                    .with_children(|draw_container_btn| {
                        draw_container_btn.spawn(change_option_txt());
                    })
                    .with_children(|draw_mode_2| {
                        draw_mode_2.spawn(draw_mode_txt("Draw Mode 2: ".to_string()));

                        // Dropdown Container
                        draw_mode_2.spawn(dropdown_2).with_children(|dropdown| {
                            // Dropdown Active Option
                            dropdown.spawn(active_option_2);
                        });
                    });
            });

        // Dropdown Options Container
        container
            .spawn((options_container(), Options2))
            // Dropdown Options
            .with_children(|options| {
                options
                    .spawn(btn_option_2("None".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("None".to_string()));
                    });
                options
                    .spawn(btn_option_2("IntegrationField".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("IntegrationField".to_string()));
                    });
                options
                    .spawn(btn_option_2("FlowField".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("FlowField".to_string()));
                    });
                options
                    .spawn(btn_option_2("CostField".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("CostField".to_string()));
                    });
                options
                    .spawn(btn_option_2("Index".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("Index".to_string()));
                    });
            });
    });
}
