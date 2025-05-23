// rvo.rs

use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use dodgy_3d::{Agent, AvoidanceOptions, Vec3};
use std::borrow::Cow;

use crate::debug::resources::DbgOptions;
use crate::flowfield::FlowField;
use crate::grid::Grid;

/// Marker + parameters for each agent
#[derive(Component)]
pub struct RVOAgent {
    pub radius: f32,
    pub max_speed: f32,
}

/// Holds the “current velocity” computed by ORCA
#[derive(Component, Deref, DerefMut)]
pub struct RVOVelocity(pub Vec3);

pub struct RVOPlugin;

impl Plugin for RVOPlugin {
    fn build(&self, app: &mut App) {
        app
            // make sure your flowfields have already been computed
            // .add_systems(Update, rvo_system.after(crate::flowfield::FlowfieldPlugin))
            .add_systems(Update, rvo)
            // then have some movement/apply system that reads Velocity
            .add_systems(Update, apply_velocity)
            .add_systems(Update, draw_radius);
    }
}

fn draw_radius(q_agents: Query<(&RVOAgent, &Transform)>, mut gizmos: Gizmos, dbg: Res<DbgOptions>) {
    if dbg.draw_radius {
        for (agent, tf) in q_agents.iter() {
            let pos: Vec3 = tf.translation;
            let rot = Quat::from_rotation_x(std::f32::consts::PI / 2.0);
            let iso = Isometry3d::new(pos, rot);

            gizmos.circle(iso, agent.radius, RED);
        }
    }
}

fn rvo(
    grid: Res<Grid>,
    time: Res<Time>,
    q_ff: Query<&FlowField>,
    q_agents: Query<(Entity, &Transform, &RVOAgent)>,
    mut q_vel: Query<&mut RVOVelocity>,
) {
    let dt = time.delta_secs().max(0.0);
    if dt == 0.0 {
        return;
    }

    // 1) Collect agents, positions, prefs
    let mut agents = Vec::new();
    let mut entities = Vec::new();
    let mut prefs = Vec::new();
    let mut positions = Vec::new();

    for (ent, tf, rvo) in q_agents.iter() {
        let pos3 = tf.translation.into();
        agents.push(Agent {
            position: pos3,
            velocity: Vec3::ZERO,
            radius: rvo.radius,
            avoidance_responsibility: 1.0,
        });
        entities.push(ent);
        positions.push(pos3);

        let dir2 = q_ff
            .iter()
            .find_map(|ff| Some(ff.sample_direction(tf.translation, &grid)))
            .unwrap_or_default();
        prefs.push(Vec3::new(dir2.x, 0.0, dir2.y) * rvo.max_speed);
    }

    // === derive bucket structure from grid.buckets, grid.size & grid.cell_diameter ===
    let buckets_count = grid.buckets.max(1.0) as usize; // number of buckets per axis
    let nx = buckets_count;
    let nz = buckets_count;

    // total world span in X and Z:
    let total_x = grid.size.y as f32 * grid.cell_diameter;
    let total_z = grid.size.x as f32 * grid.cell_diameter;

    // size of each bucket cell:
    let bucket_w = total_x / buckets_count as f32;
    let bucket_d = total_z / buckets_count as f32;

    // prepare the empty bucket arrays
    let mut buckets = vec![Vec::new(); nx * nz];
    let mut bucket_idx = Vec::with_capacity(agents.len());

    // assign each agent to a (bx, bz)
    for (i, &pos) in positions.iter().enumerate() {
        // if your grid is centered, do: let x = pos.x + total_x/2.0; similarly for z.
        let bx = ((pos.x / bucket_w).floor().clamp(0.0, (nx - 1) as f32)) as usize;
        let bz = ((pos.z / bucket_d).floor().clamp(0.0, (nz - 1) as f32)) as usize;

        buckets[bz * nx + bx].push(i);
        bucket_idx.push((bx, bz));
    }

    // 2) ORCA solve using only the 3×3 neighbouring buckets
    let opts = AvoidanceOptions { time_horizon: 3.0 };
    let radius = 10.0;
    let radius2 = radius * radius;
    let mut new_vels = Vec::with_capacity(agents.len());

    for i in 0..agents.len() {
        let (bx, bz) = bucket_idx[i];
        let mut neigh = Vec::new();

        for dx in -1..=1 {
            for dz in -1..=1 {
                let nbx = bx.wrapping_add(dx as usize);
                let nbz = bz.wrapping_add(dz as usize);
                if nbx < nx && nbz < nz {
                    for &j in &buckets[nbz * nx + nbx] {
                        if j != i {
                            let delta2 = (agents[i].position - agents[j].position).length_squared();
                            if delta2 < radius2 {
                                neigh.push(std::borrow::Cow::Borrowed(&agents[j]));
                            }
                        }
                    }
                }
            }
        }

        let v = agents[i].compute_avoiding_velocity(&neigh, prefs[i], prefs[i].length(), dt, &opts);
        new_vels.push(v);
    }

    // 3) Write back into your RVOVelocity components
    for (ent, v) in entities.into_iter().zip(new_vels) {
        if let Ok(mut vel) = q_vel.get_mut(ent) {
            **vel = v;
        }
    }
}

// NO spatial partitioning, but works well
// fn rvo(
//     grid: Res<Grid>,
//     time: Res<Time>,
//     q_ff: Query<&FlowField>,
//     mut q_agents: Query<(Entity, &Transform, &RVOAgent)>,
//     mut q_vel: Query<&mut RVOVelocity>,
// ) {
//     let dt = time.delta_secs().max(0.0);
//     if dt == 0.0 {
//         return;
//     }

//     // 1) Build your ORCA agents, remember the entity order, and sample prefs
//     let mut agents = Vec::new();
//     let mut entities = Vec::new();
//     let mut prefs = Vec::new();

//     for (ent, tf, rvo) in q_agents.iter_mut() {
//         agents.push(Agent {
//             position: tf.translation.into(),
//             velocity: Vec3::ZERO,
//             radius: rvo.radius,
//             avoidance_responsibility: 1.0,
//         });
//         entities.push(ent);

//         let dir2 = q_ff
//             .iter()
//             .find_map(|ff| Some(ff.sample_direction(tf.translation, &grid)))
//             .unwrap_or_default();
//         prefs.push(Vec3::new(dir2.x, 0.0, dir2.y) * rvo.max_speed);
//     }

//     // 2) Solve ORCA with a longer look‐ahead and cull far neighbours
//     let opts = AvoidanceOptions { time_horizon: 3.0 };
//     // let neighbour_radius2 = 15.0 * 15.0;
//     let neighbour_radius2 = 10.0 * 10.0;
//     let mut new_vels = Vec::with_capacity(agents.len());

//     for i in 0..agents.len() {
//         let neighbours = agents
//             .iter()
//             .enumerate()
//             .filter_map(|(j, other)| {
//                 let delta2 = (agents[i].position - other.position).length_squared();
//                 if j != i && delta2 < neighbour_radius2 {
//                     Some(Cow::Borrowed(other))
//                 } else {
//                     None
//                 }
//             })
//             .collect::<Vec<_>>();

//         let v = agents[i].compute_avoiding_velocity(
//             &neighbours,
//             prefs[i],
//             prefs[i].length(), // your max_speed
//             dt,
//             &opts,
//         );
//         new_vels.push(v);
//     }

//     // 3) Write back into RVOVelocity
//     for (ent, v) in entities.into_iter().zip(new_vels) {
//         if let Ok(mut vel) = q_vel.get_mut(ent) {
//             **vel = v;
//         }
//     }
// }

fn apply_velocity(time: Res<Time>, mut q: Query<(&mut Transform, &RVOVelocity)>) {
    let dt = time.delta_secs();
    for (mut tf, vel) in &mut q {
        tf.translation.x += vel.x * dt;
        tf.translation.z += vel.z * dt;
    }
}
