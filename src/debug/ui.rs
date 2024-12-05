use super::draw::{DrawMode, RtsPfDebug};
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
        app.add_systems(Startup, draw_ui_box)
            .add_systems(
                Update,
                (
                    handle_dropdown_click,
                    handle_drawmode_option_interaction,
                    handle_drag,
                ),
            )
            .add_observer(toggle_dropdown_visibility)
            .add_observer(update_active_dropdown_option);
    }
}

#[derive(Component)]
struct DebugUI;

#[derive(Component)]
#[require(Text)]
struct DrawModeTxt;

#[derive(Component)]
#[require(Text)]
struct Title;

#[derive(Component)]
#[require(Text)]
struct OptionTxt;

#[derive(Component)]
#[require(Button, Node)]
struct TitleBar;

#[derive(Component)]
#[require(Node)]
struct ActiveOptionCtr;

#[derive(Event)]
struct UpdateDropdownOptionEv;

#[derive(Event)]
struct ToggleModeEv(pub OptionsSet);

#[derive(Component)]
#[require(Text)]
struct OptionBox(pub OptionsSet);

#[derive(Component)]
#[require(Node)]
struct DropdownOptions(pub OptionsSet);

#[derive(Component)]
#[require(Node)]
struct RootCtr;

#[derive(Component)]
#[require(Button)]
struct SetActiveOption {
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
#[require(Button)]
struct DropdownBtn(pub OptionsSet);

fn handle_drawmode_option_interaction(
    mut cmds: Commands,
    mut q_option: Query<
        (&Interaction, &SetActiveOption, &mut BackgroundColor),
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

    for interaction in q_title_bar.iter() {
        match interaction {
            Interaction::Pressed => {
                ui_style.left = Val::Px(cursor_pos.x);
                ui_style.top = Val::Px(cursor_pos.y)
            }
            _ => (),
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

#[derive(Bundle)]
struct DropDownBtnBundle {
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

fn draw_ui_box(mut cmds: Commands, dbg: Res<RtsPfDebug>) {
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

    let title = (
        Title,
        Text::new("Pathfinding Debug".to_string()),
        Node {
            margin: UiRect::all(Val::Auto),
            ..default()
        },
        TextFont::from_font_size(FONT_SIZE + 2.0),
        TextColor::from(CLR_TITLE),
    );

    let dropdown_btn = |set: OptionsSet, border: UiRect| -> DropDownBtnBundle {
        let radius = match set {
            OptionsSet::One => BorderRadius::ZERO,
            OptionsSet::Two => BorderRadius::bottom(Val::Px(10.0)),
        };
        DropDownBtnBundle {
            comp: DropdownBtn(set),
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
        ctr.spawn((options_container(None, OptionsSet::One)))
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
