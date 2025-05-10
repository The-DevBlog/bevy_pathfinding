use crate::BoidsInfoUpdater;

use super::components::*;
use super::resources;
use super::resources::*;

use bevy::color::palettes::css::WHITE;
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
                    handle_dropdown_interaction,
                    handle_boids_dropdown_interaction,
                    handle_hide_dbg_interaction,
                    handle_drawmode_option_interaction,
                    handle_draw_grid_interaction,
                    handle_slider_arrow_interaction,
                    handle_drag,
                ),
            )
            .add_observer(hide_options)
            .add_observer(toggle_dbg_visibility)
            .add_observer(toggle_dropdown_visibility)
            .add_observer(update_active_dropdown_option)
            .add_observer(toggle_boids_dropdown_visibility);
    }
}

#[derive(Event)]
struct ToggleDbgVisibilityEv(bool);

#[derive(Event)]
struct ToggleBoidsDropdown;

#[derive(Event)]
struct HideOptionsEv;

#[derive(Component)]
struct BoidsDropwdownOptions;

#[derive(Bundle)]
struct DropDownBtnBundle {
    visible_node: VisibleNode,
    comp: DropdownBtn,
    btn: Button,
    background_clr: BackgroundColor,
    border_clr: BorderColor,
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
    visible_node: VisibleNode,
    background_clr: BackgroundColor,
    border_radius: BorderRadius,
    node: Node,
    name: Name,
}

#[derive(Component)]
struct BoidsDropdownOptionsCtr;

#[derive(Component)]
struct BoidsInfoCtr;

#[derive(Component)]
struct BoidsSliderValue;

#[derive(Component)]
struct BoidsInfo(BoidsInfoOptions);

#[derive(Clone, Copy, PartialEq)]
enum BoidsInfoOptions {
    Separation,
    Alignment,
    Cohesion,
}

#[derive(Component)]
struct BoidsSliderBtn(BoidsSliderBtnOptions);

enum BoidsSliderBtnOptions {
    Left,
    Right,
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
    mut dbg: ResMut<DbgOptions>,
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
    mut dbg: ResMut<DbgOptions>,
) {
    for (interaction, mut background) in q_draw_grid.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                dbg.draw_grid = !dbg.draw_grid;

                if let Ok(mut txt) = q_txt.single_mut() {
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
    mut dbg: ResMut<DbgOptions>,
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
    mut q_border: Query<&mut BorderRadius, With<BoidsInfoCtr>>,
    mut cmds: Commands,
) {
    let visible = trigger.event().0;

    let Ok(mut border) = q_border.single_mut() else {
        return;
    };

    for mut node in q_node.iter_mut() {
        if visible {
            *border = BorderRadius::bottom(Val::Px(10.0));
            node.display = Display::Flex;
            cmds.trigger(HideOptionsEv);
        } else {
            node.display = Display::None;
        }
    }
}

fn hide_options(
    _trigger: Trigger<HideOptionsEv>,
    mut q_node: Query<&mut Node, Or<(With<DropdownOptions>, With<BoidsDropdownOptionsCtr>)>>,
) {
    for mut node in q_node.iter_mut() {
        node.display = Display::None;
    }
}

fn update_active_dropdown_option(
    _trigger: Trigger<UpdateDropdownOptionEv>,
    dbg: Res<DbgOptions>,
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

fn handle_dropdown_interaction(
    mut cmds: Commands,
    mut q_btn: Query<(&Interaction, &DropdownBtn, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, dropdown, mut background) in q_btn.iter_mut() {
        match interaction {
            Interaction::Pressed => cmds.trigger(ToggleModeEv(dropdown.0)),
            Interaction::Hovered => background.0 = CLR_BTN_HOVER.into(),
            Interaction::None => background.0 = CLR_BACKGROUND_2.into(),
        }
    }
}

fn handle_boids_dropdown_interaction(
    mut cmds: Commands,
    mut q_btn: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<BoidsInfoCtr>),
    >,
) {
    for (interaction, mut background) in q_btn.iter_mut() {
        match interaction {
            Interaction::Pressed => cmds.trigger(ToggleBoidsDropdown),
            Interaction::Hovered => background.0 = CLR_BTN_HOVER.into(),
            Interaction::None => background.0 = CLR_BACKGROUND_2.into(),
        }
    }
}

fn toggle_boids_dropdown_visibility(
    _trigger: Trigger<ToggleBoidsDropdown>,
    mut q_node: Query<&mut Node, With<BoidsDropdownOptionsCtr>>,
    mut q_border: Query<&mut BorderRadius, (With<BoidsInfoCtr>, Without<BoidsDropdownOptionsCtr>)>,
) {
    let Ok(mut border) = q_border.single_mut() else {
        return;
    };

    for mut dropdown in q_node.iter_mut() {
        if dropdown.display == Display::Flex {
            dropdown.display = Display::None;
            *border = BorderRadius::bottom(Val::Px(10.0));
        } else if dropdown.display == Display::None {
            *border = BorderRadius::bottom(Val::Px(0.0));
            dropdown.display = Display::Flex
        }
    }
}

fn handle_drag(
    mut local_offset: Local<Option<Vec2>>,
    q_title_bar: Query<&Interaction, With<TitleBar>>,
    mut q_ui: Query<&mut Node, With<DebugUI>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(mut ui_style) = q_ui.single_mut() else {
        return;
    };

    let Ok(window) = window_q.single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
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
                dropdown.display = Display::Flex;
            }
        } else if option == OptionsSet::Two && dropdown_options.0.to_num() == 2 {
            if dropdown.display == Display::Flex {
                dropdown.display = Display::None;
            } else if dropdown.display == Display::None {
                dropdown.display = Display::Flex;
            }
        }
    }
}

fn draw_ui_box(
    mut cmds: Commands,
    dbg: Res<DbgOptions>,
    dbg_icon: Res<DbgIcon>,
    mut q_boids_info: Query<&BoidsInfoUpdater>,
) {
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

    let dropdown_btn = |set: OptionsSet| DropDownBtnBundle {
        comp: DropdownBtn(set),
        visible_node: VisibleNode,
        btn: Button::default(),
        background_clr: BackgroundColor::from(CLR_BACKGROUND_2),
        border_clr: BorderColor::from(CLR_BORDER),
        node: Node {
            border: UiRect::top(Val::Px(1.0)),
            ..default()
        },
        name: Name::new("Dropdown Button"),
    };

    let active_option_ctr = || ActiveOptionCtrBundle {
        comp: ActiveOptionCtr,
        border_clr: BorderColor::from(CLR_BORDER),
        node: Node {
            padding: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        name: Name::new("Draw Mode Container"),
    };

    let draw_mode_txt = |txt: String| DrawModeTxtCtr {
        comp: DrawModeTxt,
        txt: Text::new(txt),
        txt_font: TextFont::from_font_size(FONT_SIZE),
        txt_clr: TextColor::from(CLR_TXT),
        name: Name::new("Draw Mode Text"),
    };

    let active_option = |txt: String, set: OptionsSet| OptionBoxCtr {
        comp: OptionBox(set),
        txt: Text::new(txt),
        txt_font: TextFont::from_font_size(FONT_SIZE),
        txt_clr: TextColor::from(CLR_TXT),
    };

    let options_container =
        |border_radius: BorderRadius, padding: Option<f32>| DropdownOptionsCtr {
            visible_node: VisibleNode,
            background_clr: BackgroundColor::from(CLR_BACKGROUND_2),
            border_radius,
            node: Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::bottom(match padding {
                    Some(p) => Val::Px(p),
                    None => Val::Px(0.0),
                }),
                display: Display::None,
                ..default()
            },
            name: Name::new("Options Container"),
        };

    let btn_option = |set: OptionsSet, txt: String, radius: Option<BorderRadius>| {
        let radius = match radius {
            Some(r) => r,
            None => BorderRadius::ZERO,
        };

        (
            SetActiveOption { set, txt },
            radius,
            Node {
                padding: UiRect::new(Val::Px(5.0), Val::Px(5.0), Val::Px(2.5), Val::Px(2.5)),
                margin: UiRect::left(Val::Px(5.0)),
                ..default()
            },
        )
    };

    let option_txt = |txt: String| OptionTxtCtr {
        comp: OptionTxt,
        txt: Text::new(txt),
        txt_font: TextFont::from_font_size(FONT_SIZE),
        txt_clr: TextColor::from(CLR_TXT),
    };

    let boids_dropdown_txt_ctr = (
        VisibleNode,
        BoidsInfoCtr,
        Button::default(),
        BackgroundColor::from(CLR_BACKGROUND_2),
        BorderRadius::bottom(Val::Px(10.0)),
        BorderColor::from(CLR_BORDER),
        Node {
            padding: UiRect::all(Val::Px(5.0)),
            border: UiRect::top(Val::Px(1.0)),
            ..default()
        },
        Name::new("Boids Info Dropdown Button"),
    );

    let boids_info_dropwdown_txt = (
        TextFont::from_font_size(FONT_SIZE),
        TextColor::from(CLR_TXT),
        Text::new("Boids Info"),
        Name::new("Boids Info Dropdown Text"),
    );

    let boids_option_btn = |txt: String, radius: Option<BorderRadius>| {
        let radius = match radius {
            Some(r) => r,
            None => BorderRadius::ZERO,
        };

        (
            Button::default(),
            radius,
            Node {
                padding: UiRect::new(Val::Px(5.0), Val::Px(5.0), Val::Px(2.5), Val::Px(2.5)),
                margin: UiRect::new(Val::Px(5.0), Val::Px(15.0), Val::Auto, Val::Auto),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            Name::new(format!("Boids Option Btn {}", txt)),
        )
    };

    let Ok(boids_info) = q_boids_info.single_mut() else {
        return;
    };

    let labels = &[
        (
            "Separation",
            boids_info.separation_weight,
            BoidsInfoOptions::Separation,
            None,
        ),
        (
            "Alignment",
            boids_info.alignment_weight,
            BoidsInfoOptions::Alignment,
            None,
        ),
        (
            "Cohesion",
            boids_info.cohesion_weight,
            BoidsInfoOptions::Cohesion,
            Some(BorderRadius::top(Val::Px(10.0))),
        ),
    ];

    let boids_slider_ctr = || {
        (
            Node {
                width: Val::Percent(40.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            Name::new("Boids Option Slider"),
        )
    };

    let boids_option_slider_arrow_btn = |info: BoidsInfoOptions, arrow: BoidsSliderBtnOptions| {
        (
            BackgroundColor::from(CLR_BACKGROUND_1),
            BorderColor::from(CLR_BORDER),
            BorderRadius::all(Val::Px(10.0)),
            BoidsInfo(info),
            BoidsSliderBtn(arrow),
            Node {
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::horizontal(Val::Px(5.0)),
                ..default()
            },
            Button::default(),
            Name::new(format!("Boids Option Slider Btn")),
        )
    };

    let boids_option_slider_arrow_txt = |txt: String| {
        (
            Text::new(txt.clone()),
            TextColor::from(CLR_TXT),
            TextFont::from_font_size(FONT_SIZE),
            Name::new(format!("Boids Option Slider txt")),
        )
    };

    let boids_option_slider_value = |val: String, info: BoidsInfoOptions| {
        (
            BoidsSliderValue,
            BoidsInfo(info),
            Node {
                margin: UiRect::horizontal(Val::Px(2.5)),
                top: Val::Px(2.0),
                ..default()
            },
            Text::new(val),
            TextColor::from(CLR_TXT),
            TextFont::from_font_size(FONT_SIZE),
            Name::new("Boids Option Slider Value"),
        )
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
        ctr.spawn(dropdown_btn(OptionsSet::One))
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
        ctr.spawn((
            options_container(BorderRadius::all(Val::ZERO), None),
            DropdownOptions(OptionsSet::One),
        ))
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
        ctr.spawn(dropdown_btn(OptionsSet::Two))
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
            options_container(BorderRadius::all(Val::ZERO), None),
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
                .spawn(btn_option(OptionsSet::Two, "Index".to_string(), None))
                .with_children(|btn| {
                    btn.spawn(option_txt("> Index".to_string()));
                });
        });

        // Boids Info Dropdown Button
        ctr.spawn(boids_dropdown_txt_ctr).with_children(|dropdown| {
            dropdown.spawn(boids_info_dropwdown_txt);
        });

        // Boids Info Dropdown Options Ctr
        ctr.spawn((
            BoidsDropdownOptionsCtr,
            options_container(BorderRadius::bottom(Val::Px(10.0)), Some(5.0)),
        ))
        .with_children(|options| {
            // Boids Info Dropdown Options
            for (label, value, info, radius) in labels {
                options
                    .spawn(boids_option_btn(label.to_string(), *radius))
                    .with_children(|btn| {
                        // Options Txt
                        btn.spawn(option_txt(label.to_string()));

                        // Slider
                        btn.spawn(boids_slider_ctr()).with_children(|slider| {
                            slider
                                .spawn(boids_option_slider_arrow_btn(
                                    *info,
                                    BoidsSliderBtnOptions::Left,
                                ))
                                .with_child(boids_option_slider_arrow_txt("<".to_string()));
                            slider.spawn(boids_option_slider_value(value.to_string(), *info));
                            slider
                                .spawn(boids_option_slider_arrow_btn(
                                    *info,
                                    BoidsSliderBtnOptions::Right,
                                ))
                                .with_child(boids_option_slider_arrow_txt(">".to_string()));
                        });
                    });
            }
        });
    });
}

fn handle_slider_arrow_interaction(
    mut boids_updater: Query<&mut BoidsInfoUpdater>,
    mut q_slider: Query<
        (
            &Interaction,
            &BoidsSliderBtn,
            &BoidsInfo,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut q_txt: Query<(&mut Text, &BoidsInfo), (With<BoidsSliderValue>, Without<BoidsSliderBtn>)>,
) {
    let Ok(mut updater) = boids_updater.single_mut() else {
        return;
    };

    for (interaction, slider, boids_info, mut background_clr, mut border_clr) in q_slider.iter_mut()
    {
        match interaction {
            Interaction::Pressed => {
                if let Some((mut txt, _)) = q_txt
                    .iter_mut()
                    .find(|(_txt, info2)| info2.0 == boids_info.0)
                {
                    // grab the old value…
                    let mut val = match boids_info.0 {
                        BoidsInfoOptions::Separation => updater.separation_weight,
                        BoidsInfoOptions::Alignment => updater.alignment_weight,
                        BoidsInfoOptions::Cohesion => updater.cohesion_weight,
                    };

                    // bump it
                    match slider.0 {
                        BoidsSliderBtnOptions::Left => val -= 1.0,
                        BoidsSliderBtnOptions::Right => val += 1.0,
                    }

                    // update BoidsUpdater
                    match boids_info.0 {
                        BoidsInfoOptions::Separation => updater.separation_weight = val,
                        BoidsInfoOptions::Alignment => updater.alignment_weight = val,
                        BoidsInfoOptions::Cohesion => updater.cohesion_weight = val,
                    }

                    txt.0 = val.to_string();
                }

                background_clr.0 = CLR_BTN_HOVER.into();
                border_clr.0 = WHITE.into();
            }
            Interaction::Hovered => {
                background_clr.0 = CLR_BTN_HOVER.into();
                border_clr.0 = WHITE.into();
            }
            Interaction::None => {
                background_clr.0 = CLR_BACKGROUND_1.into();
                border_clr.0 = CLR_BORDER.into();
            }
        }
    }
}
