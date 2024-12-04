use super::draw::{DrawMode, RtsPfDebug};
use bevy::prelude::*;

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
struct ToggleModeEv(pub OptionsSet);

#[derive(Component)]
struct OptionBox(pub OptionsSet);

#[derive(Component)]
struct DropdownOptions(pub OptionsSet);

#[derive(Component)]
struct ActiveOption {
    set: OptionsSet,
    txt: String,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum OptionsSet {
    One,
    Two,
}

impl OptionsSet {
    fn to_num(&self) -> i32 {
        match self {
            OptionsSet::One => 1,
            OptionsSet::Two => 2,
        }
    }
}

#[derive(Component)]
struct DropdownBtn(pub OptionsSet);

fn handle_draw_mode_interaction(
    mut cmds: Commands,
    mut q_option: Query<
        (&Interaction, &ActiveOption, &mut BackgroundColor),
        (Changed<Interaction>,),
    >,
    mut dbg: ResMut<RtsPfDebug>,
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

fn update_active_dropdown_option(
    _trigger: Trigger<UpdateDropdownOptionEv>,
    dbg: Res<RtsPfDebug>,
    mut q_txt: Query<(&mut Text, &OptionBox)>,
) {
    for (mut txt, options) in q_txt.iter_mut() {
        let num = match options.0 {
            OptionsSet::One => 1,
            OptionsSet::Two => 2,
        };

        txt.sections[0].value = dbg.mode_string(num);
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

fn toggle_dropdown_visibility(
    trigger: Trigger<ToggleModeEv>,
    mut q_dropdown: Query<(&mut Style, &DropdownOptions)>,
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

fn draw_ui_box(mut cmds: Commands, dbg: Res<RtsPfDebug>) {
    let root_ctr = (
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            border_color: CLR_BORDER.into(),
            border_radius: BorderRadius::all(Val::Px(10.0)),
            background_color: CLR_BACKGROUND_1.into(),
            ..default()
        },
        Name::new("Debug Container"),
    );

    let title_ctr = (
        NodeBundle {
            border_color: CLR_BORDER.into(),
            style: Style {
                border: UiRect::bottom(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            ..default()
        },
        Name::new("Title Bar"),
    );
    let title = TextBundle {
        text: Text::from_section(
            "Pathfinding Debug",
            TextStyle {
                font_size: FONT_SIZE + 2.0,
                color: CLR_TITLE,
                ..default()
            },
        ),
        style: Style {
            margin: UiRect::all(Val::Auto),
            ..default()
        },
        ..default()
    };

    let dropdown_btn = |set: OptionsSet, border: UiRect| -> (ButtonBundle, DropdownBtn, Name) {
        let radius = match set {
            OptionsSet::One => BorderRadius::ZERO,
            OptionsSet::Two => BorderRadius::bottom(Val::Px(10.0)),
        };

        (
            ButtonBundle {
                background_color: CLR_BACKGROUND_2.into(),
                border_color: CLR_BORDER.into(),
                border_radius: radius,
                style: Style {
                    border,
                    ..default()
                },
                ..default()
            },
            DropdownBtn(set),
            Name::new("Dropdown Button"),
        )
    };

    let active_option_ctr = || -> (NodeBundle, Name) {
        (
            NodeBundle {
                border_color: BorderColor(CLR_BORDER.into()),
                style: Style {
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                ..default()
            },
            Name::new("Draw Mode Container"),
        )
    };

    let draw_mode_txt = |txt: String| -> (TextBundle, Name) {
        (
            TextBundle {
                text: Text::from_section(
                    txt,
                    TextStyle {
                        color: CLR_TXT.into(),
                        font_size: FONT_SIZE,
                        ..default()
                    },
                ),
                ..default()
            },
            Name::new("Draw Mode Text"),
        )
    };

    let active_option = |txt: String, set: OptionsSet| -> (TextBundle, OptionBox) {
        (
            TextBundle {
                text: Text::from_section(
                    txt,
                    TextStyle {
                        color: CLR_TXT.into(),
                        font_size: FONT_SIZE,
                        ..default()
                    },
                ),
                ..default()
            },
            OptionBox(set),
        )
    };

    let options_container = |radius: Option<BorderRadius>| -> (NodeBundle, Name) {
        let radius = match radius {
            Some(r) => r,
            None => BorderRadius::ZERO,
        };

        (
            NodeBundle {
                background_color: CLR_BACKGROUND_2.into(),
                border_radius: radius,
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

    let btn_option = |set: OptionsSet,
                      txt: String,
                      radius: Option<BorderRadius>|
     -> (ButtonBundle, ActiveOption) {
        let radius = match radius {
            Some(r) => r,
            None => BorderRadius::ZERO,
        };

        (
            ButtonBundle {
                border_radius: radius,
                style: Style {
                    padding: UiRect::new(Val::Px(5.0), Val::Px(5.0), Val::Px(2.5), Val::Px(2.5)),
                    ..default()
                },
                ..default()
            },
            ActiveOption { set, txt },
        )
    };

    let option_txt = |txt: String| -> TextBundle {
        TextBundle {
            text: Text::from_section(
                txt.clone(),
                TextStyle {
                    color: CLR_TXT.into(),
                    font_size: FONT_SIZE,
                    ..default()
                },
            ),
            ..default()
        }
    };

    // Root Container
    cmds.spawn(root_ctr).with_children(|ctr| {
        // Title Bar
        ctr.spawn(title_ctr).with_children(|title_bar| {
            title_bar.spawn(title);
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
        ctr.spawn((options_container(None), DropdownOptions(OptionsSet::One)))
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
        ctr.spawn((
            options_container(Some(BorderRadius::bottom(Val::Px(10.0)))),
            DropdownOptions(OptionsSet::Two),
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
