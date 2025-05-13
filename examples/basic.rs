use bevy::{
    color::palettes::tailwind::{BLUE_500, GREEN_600},
    math::bounding::Aabb2d,
    prelude::*,
};
use bevy_rts_camera::{Ground, RtsCamera, RtsCameraControls, RtsCameraPlugin};
use bevy_rts_pathfinding::{
    components::{Boid, Destination, GameCamera, MapBase, RtsObj},
    events::InitializeFlowFieldEv,
    grid::Grid,
    BevyRtsPathFindingPlugin,
};

const CELL_SIZE: f32 = 10.0;
const MAP_GRID: IVec2 = IVec2::new(50, 50);
const MAP_WIDTH: f32 = MAP_GRID.x as f32 * CELL_SIZE;
const MAP_DEPTH: f32 = MAP_GRID.y as f32 * CELL_SIZE;

fn main() {
    let mut app = App::new();

    app.insert_resource(Grid::new(MAP_GRID, CELL_SIZE)) // THIS
        .add_plugins((
            DefaultPlugins,
            RtsCameraPlugin,
            BevyRtsPathFindingPlugin, // THIS
        ))
        .add_systems(Startup, (camera, setup, spawn_units))
        .add_systems(
            Update,
            (
                set_unit_destination,
                move_unit.run_if(any_with_component::<Destination>),
                count_dest,
            ),
        )
        .run();
}

fn count_dest(q: Query<&Destination>) {
    println!("dest count: {}", q.iter().count());
}

#[derive(Component)]
struct Unit;

#[derive(Component)]
struct Speed(f32);

fn camera(mut cmds: Commands) {
    cmds.spawn((
        Camera3d::default(),
        GameCamera, // THIS
        RtsCamera {
            bounds: Aabb2d::new(Vec2::ZERO, Vec2::new(MAP_WIDTH, MAP_DEPTH)),
            min_angle: 60.0f32.to_radians(),
            height_max: 1000.0,
            height_min: 30.0,
            ..default()
        },
        RtsCameraControls {
            edge_pan_width: 0.01,
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            key_up: KeyCode::KeyW,
            key_down: KeyCode::KeyS,
            pan_speed: 165.0,
            zoom_sensitivity: 0.2,
            ..default()
        },
    ));
}

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ground = (
        Mesh3d(meshes.add(Plane3d::default().mesh().size(MAP_WIDTH, MAP_DEPTH))),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(GREEN_600))),
        Ground,
        MapBase, // THIS
        Name::new("Map Base"),
    );

    let translation = Vec3::new(0.0, 0.0, 0.0);
    let rotation = Quat::from_euler(EulerRot::XYZ, -0.7, 0.2, 0.0);
    let light = (
        DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: true,
            shadow_depth_bias: 1.5,
            shadow_normal_bias: 1.0,
            ..default()
        },
        Transform {
            translation,
            rotation,
            ..default()
        },
        Name::new("Sun Light"),
    );

    cmds.spawn(ground);
    cmds.spawn(light);
}

fn spawn_units(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let unit = (
        Mesh3d(meshes.add(Cuboid::new(5.0, 5.0, 5.0))),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(BLUE_500))),
        Transform::from_xyz(0.0, 2.5, 0.0),
        Speed(25.0),
        RtsObj, // THIS
        Unit,
        Name::new("Unit"),
    );

    cmds.spawn(unit);
}

// THIS
fn set_unit_destination(
    mut cmds: Commands,
    input: Res<ButtonInput<MouseButton>>,
    mut q_units: Query<Entity, With<Unit>>,
) {
    if !input.just_pressed(MouseButton::Left) {
        return;
    }

    let mut units = Vec::new();
    for unit_entity in q_units.iter_mut() {
        units.push(unit_entity);
    }

    cmds.trigger(InitializeFlowFieldEv(units));
}

// THIS
fn move_unit(
    mut q_units: Query<(&mut Transform, &mut Boid, &Speed), With<Destination>>,
    time: Res<Time>,
) {
    let delta_secs = time.delta_secs();

    for (mut tf, boid, speed) in q_units.iter_mut() {
        tf.translation += boid.steering.normalize_or_zero() * delta_secs * speed.0;
    }
}
