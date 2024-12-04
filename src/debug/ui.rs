use super::draw::{DrawMode, RtsPfDebug};
use bevy::prelude::*;

const CLR_TXT_ACTIVE: Color = Color::WHITE;
const CLR_TXT_DORMANT: Color = Color::srgb(0.8, 0.8, 0.8);
const CLR_BTN_HOVER: Color = Color::srgba(0.27, 0.27, 0.27, 1.0);
const CLR_BORDER: Color = Color::srgba(0.53, 0.53, 0.53, 1.0);
const CLR_BACKGROUND_1: Color = Color::srgba(0.18, 0.18, 0.18, 1.0);
const CLR_BACKGROUND_2: Color = Color::srgba(0.11, 0.11, 0.11, 1.0);
const FONT_SIZE: f32 = 16.0;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, draw_ui_box)
            .add_systems(
                Update,
                (handle_dropdown_click, handle_draw_mode_interaction),
            )
            .observe(toggle_dropdown_visibility)
            .observe(update_active_dropdown_option);
    }
}

#[derive(Event)]
struct UpdateDropdownOptionEv;

#[derive(Event)]
struct ToggleModeEv(pub i32);

#[derive(Component)]
struct ActiveOption1(pub String);

#[derive(Component)]
struct ActiveOption2(pub String);

#[derive(Component)]
struct OptionBox(pub i32);

#[derive(Component)]
struct DropdownOptions(pub i32);

#[derive(Component)]
struct DropdownBtn(pub i32);

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
    mut q_txt: Query<(&mut Text, &OptionBox)>,
) {
    for (mut txt, options) in q_txt.iter_mut() {
        txt.sections[0].value = dbg.mode_string(options.0);
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
            Interaction::None => background.0 = CLR_BACKGROUND_1.into(),
        }
    }
}

fn toggle_dropdown_visibility(
    trigger: Trigger<ToggleModeEv>,
    mut q_dropdown: Query<(&mut Style, &DropdownOptions)>,
    // mut q_dropdown: Query<&mut Style, With<Options1>>,
    // mut q_dropdown_2: Query<&mut Style, (With<Options2>, Without<Options1>)>,
) {
    let option = trigger.event().0;

    for (mut dropdown, dropdown_options) in q_dropdown.iter_mut() {
        if option == 0 && dropdown_options.0 == 0 {
            if dropdown.display == Display::Flex {
                dropdown.display = Display::None;
            } else if dropdown.display == Display::None {
                dropdown.display = Display::Flex
            }
        } else if option == 1 && dropdown_options.0 == 1 {
            if dropdown.display == Display::Flex {
                dropdown.display = Display::None;
            } else if dropdown.display == Display::None {
                dropdown.display = Display::Flex
            }
        }
    }
}

fn draw_ui_box(mut cmds: Commands, dbg: Res<RtsPfDebug>) {
    let root_container = (
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(0.5)),
                ..default()
            },
            border_color: CLR_BORDER.into(),
            border_radius: BorderRadius::all(Val::Px(10.0)),
            background_color: CLR_BACKGROUND_1.into(),
            ..default()
        },
        Name::new("Debug Container"),
    );

    let draw_mode_txt = |txt: String| -> (TextBundle, Name) {
        (
            TextBundle {
                text: Text::from_section(
                    txt,
                    TextStyle {
                        color: CLR_TXT_DORMANT.into(),
                        font_size: FONT_SIZE,
                        ..default()
                    },
                ),
                ..default()
            },
            Name::new("Draw Mode Text"),
        )
    };

    let active_option_container = |border: UiRect| -> (NodeBundle, Name) {
        (
            NodeBundle {
                border_color: BorderColor(CLR_BORDER.into()),
                style: Style {
                    padding: UiRect::all(Val::Px(5.0)),
                    border: border,
                    ..default()
                },
                ..default()
            },
            Name::new("Draw Mode Container"),
        )
    };

    let active_option_container_1 = active_option_container(UiRect::bottom(Val::Px(0.25)));
    let active_option_container_2 = active_option_container(UiRect::top(Val::Px(0.25)));

    let active_option = |txt: String, option: i32| -> (TextBundle, OptionBox) {
        (
            TextBundle {
                text: Text::from_section(
                    txt,
                    TextStyle {
                        color: CLR_TXT_DORMANT.into(),
                        font_size: FONT_SIZE,
                        ..default()
                    },
                ),
                ..default()
            },
            OptionBox(option),
        )
    };

    let options_container = || -> (NodeBundle, Name) {
        (
            NodeBundle {
                background_color: CLR_BACKGROUND_2.into(),
                style: Style {
                    flex_direction: FlexDirection::Column,
                    display: Display::None,
                    ..default()
                },
                ..default()
            },
            Name::new("Options Container"),
        )
    };

    let option_txt = |txt: String| -> TextBundle {
        TextBundle {
            text: Text::from_section(
                txt.clone(),
                TextStyle {
                    color: CLR_TXT_DORMANT.into(),
                    font_size: FONT_SIZE,
                    ..default()
                },
            ),
            style: Style {
                padding: UiRect::all(Val::Px(2.5)),
                ..default()
            },
            ..default()
        }
    };

    let btn_option_1 = |txt: String| -> (ButtonBundle, ActiveOption1) {
        (ButtonBundle::default(), ActiveOption1(txt))
    };

    let btn_option_2 = |txt: String| -> (ButtonBundle, ActiveOption2) {
        (ButtonBundle::default(), ActiveOption2(txt))
    };

    let dropdown_btn = |idx: i32| -> (ButtonBundle, DropdownBtn, Name) {
        let radius = match idx {
            0 => BorderRadius::top(Val::Px(10.0)),
            1 => BorderRadius::bottom(Val::Px(10.0)),
            _ => BorderRadius::all(Val::Px(10.0)), // should not be hit
        };

        (
            ButtonBundle {
                border_radius: radius,
                ..default()
            },
            DropdownBtn(idx),
            Name::new("Dropdown Button"),
        )
    };

    // Root Container
    cmds.spawn(root_container).with_children(|container| {
        // Draw Mode 1 Container
        container
            .spawn(dropdown_btn(0))
            .with_children(|dropdown_btn| {
                dropdown_btn
                    .spawn(active_option_container_1)
                    .with_children(|draw_mode_1| {
                        // Active Dropdown Option
                        draw_mode_1.spawn(draw_mode_txt("Draw Mode 1 : ".to_string()));
                        draw_mode_1.spawn(active_option(dbg.mode1_string(), 0));
                    });
            });

        // Dropdown Options Container
        container
            .spawn((options_container(), DropdownOptions(0)))
            // Dropdown Options
            .with_children(|options| {
                options
                    .spawn(btn_option_1("None".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("> None".to_string()));
                    });
                options
                    .spawn(btn_option_1("IntegrationField".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("> IntegrationField".to_string()));
                    });
                options
                    .spawn(btn_option_1("FlowField".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("> FlowField".to_string()));
                    });
                options
                    .spawn(btn_option_1("CostField".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("> CostField".to_string()));
                    });
                options
                    .spawn(btn_option_1("Index".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("> Index".to_string()));
                    });
            });

        // Draw Mode 2 Container
        container
            .spawn(dropdown_btn(1))
            .with_children(|dropdown_btn| {
                dropdown_btn
                    .spawn(active_option_container_2)
                    .with_children(|draw_mode_2| {
                        // Dropdown Active Option
                        draw_mode_2.spawn(draw_mode_txt("Draw Mode 2 : ".to_string()));
                        draw_mode_2.spawn(active_option(dbg.mode2_string(), 1));
                    });
            });

        // Dropdown Options Container
        container
            .spawn((options_container(), DropdownOptions(1)))
            // Dropdown Options
            .with_children(|options| {
                options
                    .spawn(btn_option_2("None".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("> None".to_string()));
                    });
                options
                    .spawn(btn_option_2("IntegrationField".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("> IntegrationField".to_string()));
                    });
                options
                    .spawn(btn_option_2("FlowField".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("> FlowField".to_string()));
                    });
                options
                    .spawn(btn_option_2("CostField".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("> CostField".to_string()));
                    });
                options
                    .spawn(btn_option_2("Index".to_string()))
                    .with_children(|btn| {
                        btn.spawn(option_txt("> Index".to_string()));
                    });
            });
    });
}
