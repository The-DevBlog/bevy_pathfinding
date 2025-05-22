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
            .add_systems(Update, rvo_system)
            // then have some movement/apply system that reads Velocity
            .add_systems(Update, apply_velocity_system)
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

fn rvo_system(
    grid: Res<Grid>,
    time: Res<Time>,
    q_ff: Query<&FlowField>,
    mut q_agents: Query<(Entity, &Transform, &RVOAgent)>,
    mut q_vel: Query<&mut RVOVelocity>,
) {
    let dt = time.delta_secs().max(0.0);
    if dt == 0.0 {
        return;
    }

    // 1) Build ORCA agents & prefs
    let mut agents = Vec::new();
    let mut entities = Vec::new();
    let mut prefs = Vec::new();
    for (ent, tf, rvo) in q_agents.iter_mut() {
        agents.push(Agent {
            position: tf.translation.into(),
            velocity: Vec3::ZERO,
            radius: rvo.radius,
            avoidance_responsibility: 1.0,
        });
        entities.push(ent);

        let dir2 = q_ff
            .iter()
            .find_map(|ff| Some(ff.sample_direction(tf.translation, &grid)))
            .unwrap_or_default();
        prefs.push(Vec3::new(dir2.x, 0.0, dir2.y) * rvo.max_speed);
    }

    // 2) Partition into buckets
    let buckets_count = grid.buckets as usize;
    let world_w = grid.size.x as f32 * grid.cell_diameter;
    let world_d = grid.size.y as f32 * grid.cell_diameter;
    let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); buckets_count * buckets_count];
    let mut agent_bucket = Vec::with_capacity(agents.len());

    for (i, ag) in agents.iter().enumerate() {
        // map X from [-world_w/2..+world_w/2] to [0..buckets_count)
        let bx = (((ag.position.x + world_w * 0.5) / world_w) * buckets_count as f32)
            .clamp(0.0, buckets_count as f32 - 1.0)
            .floor() as usize;
        // same for Z
        let bz = (((ag.position.z + world_d * 0.5) / world_d) * buckets_count as f32)
            .clamp(0.0, buckets_count as f32 - 1.0)
            .floor() as usize;

        buckets[bx + bz * buckets_count].push(i);
        agent_bucket.push((bx, bz));
    }

    // 3) ORCA solve, only probing local buckets
    let opts = AvoidanceOptions { time_horizon: 8.0 };
    let mut new_vels = Vec::with_capacity(agents.len());
    for i in 0..agents.len() {
        let (bx, bz) = agent_bucket[i];
        let mut neighbours = Vec::new();

        // probe 3x3 surrounding buckets
        for dx in -1..=1 {
            for dz in -1..=1 {
                let nbx = bx as isize + dx;
                let nbz = bz as isize + dz;
                if (0..buckets_count as isize).contains(&nbx)
                    && (0..buckets_count as isize).contains(&nbz)
                {
                    let idx = nbx as usize + nbz as usize * buckets_count;
                    for &j in &buckets[idx] {
                        if j != i {
                            neighbours.push(Cow::Borrowed(&agents[j]));
                        }
                    }
                }
            }
        }

        let v = agents[i].compute_avoiding_velocity(
            &neighbours,
            prefs[i],
            prefs[i].length(),
            dt,
            &opts,
        );
        new_vels.push(v);
    }

    // 4) Write back
    for (ent, v) in entities.into_iter().zip(new_vels) {
        if let Ok(mut vel) = q_vel.get_mut(ent) {
            **vel = v;
        }
    }
}

fn apply_velocity_system(time: Res<Time>, mut q: Query<(&mut Transform, &RVOVelocity)>) {
    let dt = time.delta_secs();
    for (mut tf, vel) in &mut q {
        tf.translation.x += vel.x * dt;
        tf.translation.z += vel.z * dt;
    }
}
