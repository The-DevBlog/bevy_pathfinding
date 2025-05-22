// This example uses a simple point-and-click movement system to move units around the map.
// This is a stress test using 5000 units.

use bevy::{
    color::palettes::tailwind::*, math::bounding::Aabb2d, prelude::*, window::PrimaryWindow,
};
use bevy_pathfinding::{
    components::*,
    debug::resources::{BoidUpdater, DbgOptions},
    events::InitializeFlowFieldEv,
    grid::Grid,
    rvo::RVOAgent,
    utils, BevyPathfindingPlugin,
};
use bevy_rts_camera::{Ground, RtsCamera, RtsCameraControls, RtsCameraPlugin};

const CELL_SIZE: f32 = 10.0; // size of each cell in the grid
const BUCKETS: f32 = 150.0; // size of each bucket (spatial partitioning) in the grid
const MAP_GRID: IVec2 = IVec2::new(300, 300); // number of cell rows and columns

// size of the map is determined by the grid size and cell size
const MAP_WIDTH: f32 = MAP_GRID.x as f32 * CELL_SIZE;
const MAP_DEPTH: f32 = MAP_GRID.y as f32 * CELL_SIZE;

const UNIT_COUNT: usize = 10000;

fn main() {
    let mut app = App::new();

    app.insert_resource(Grid::new(BUCKETS, MAP_GRID, CELL_SIZE)) // ADD THIS!
        .insert_resource(BoidUpdater::new(10.0, 0.0, 0.0, 5.0))
        .add_plugins((
            DefaultPlugins,
            BevyPathfindingPlugin, // ADD THIS!
            RtsCameraPlugin,
        ))
        .add_systems(Startup, (camera, setup, spawn_units))
        // .add_systems(Update, (set_unit_destination, move_unit))
        .run();
}

#[derive(Component)]
struct Speed(f32);

fn camera(mut cmds: Commands) {
    cmds.spawn((
        Camera3d::default(),
        GameCamera, // ADD THIS!
        Transform::from_translation(Vec3::new(0.0, 2000.0, 1500.0)).looking_at(Vec3::ZERO, Vec3::Y),
        RtsCamera {
            bounds: Aabb2d::new(Vec2::ZERO, Vec2::new(MAP_WIDTH / 2.0, MAP_DEPTH / 2.0)),
            min_angle: 60.0f32.to_radians(),
            // height_max: 300.0,
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

// spawn ground and light
fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ground = (
        Mesh3d(meshes.add(Plane3d::default().mesh().size(MAP_WIDTH, MAP_DEPTH))),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(GREEN_600))),
        MapBase, // ADD THIS!
        Ground,
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
    let mut unit = |pos: Vec3| {
        (
            Mesh3d(meshes.add(Cuboid::new(5.0, 5.0, 5.0))),
            MeshMaterial3d(materials.add(StandardMaterial::from_color(BLUE_500))),
            Transform::from_translation(pos),
            Speed(150.0),
            RVOAgent {
                radius: 5.0,
                max_speed: 25.0,
            },
            // Boid::new(115.0, 0.0, 0.0, 7.5),
            Name::new("Unit"),
        )
    };

    let side = (UNIT_COUNT as f32).sqrt().ceil() as u32;

    // spacing between units
    let spacing = 10.0;

    // offset to center the whole formation on (0,0)
    let half = (side as f32 - 1.0) * spacing * 0.5;

    for idx in 0..UNIT_COUNT {
        let col = (idx as u32) % side;
        let row = (idx as u32) / side;

        let x = col as f32 * spacing - half;
        let z = row as f32 * spacing - half;

        cmds.spawn(unit(Vec3::new(x, 2.5, z)));
    }
}

// uses the mouse position to set the destination of all units
fn set_unit_destination(
    mut cmds: Commands,
    input: Res<ButtonInput<MouseButton>>,
    mut q_units: Query<Entity, With<RVOAgent>>,
    q_map: Query<&GlobalTransform, With<MapBase>>,
    q_cam: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    dbg_options: Option<ResMut<DbgOptions>>,
) {
    // if hovering over the debug UI, then do not set the destination
    if let Some(dbg) = dbg_options {
        if dbg.hover {
            return;
        }
    };

    if !input.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(map_tf) = q_map.single() else {
        return;
    };

    let Ok((cam, cam_transform)) = q_cam.single() else {
        return;
    };

    let Ok(window) = q_window.single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // ADD THIS!
    {
        // collect all the units you wish to assign to a flowfield
        let mut units = Vec::new();
        for unit_entity in q_units.iter_mut() {
            units.push(unit_entity);
        }

        // get the destination position in world space using the 'get_world_pos' function
        let destination_pos = utils::get_world_pos(map_tf, cam_transform, cam, cursor_pos);

        // create a flowfield and assign the units and destination position it it
        cmds.trigger(InitializeFlowFieldEv {
            entities: units,
            destination_pos,
        });
    }
}

// // ADD THIS!
// // moves all units (boids) that have a destination, towards it
// // if you are using a physics engine, you would want to swap out the 'Transform' here
// fn move_unit(
//     mut q_units: Query<(&mut Transform, &mut Boid, &Speed), With<Destination>>,
//     time: Res<Time>,
// ) {
//     let delta_secs = time.delta_secs();

//     for (mut tf, boid, speed) in q_units.iter_mut() {
//         tf.translation += boid.steering.normalize_or_zero() * delta_secs * speed.0;
//     }
// }
