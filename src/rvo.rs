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

    // 1) Build your ORCA agents, remember the entity order, and sample prefs
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

    // 2) Solve ORCA with a longer look‐ahead and cull far neighbours
    let opts = AvoidanceOptions { time_horizon: 3.0 };
    let neighbour_radius2 = 15.0 * 15.0;
    let mut new_vels = Vec::with_capacity(agents.len());

    for i in 0..agents.len() {
        let neighbours = agents
            .iter()
            .enumerate()
            .filter_map(|(j, other)| {
                let delta2 = (agents[i].position - other.position).length_squared();
                if j != i && delta2 < neighbour_radius2 {
                    Some(Cow::Borrowed(other))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let v = agents[i].compute_avoiding_velocity(
            &neighbours,
            prefs[i],
            prefs[i].length(), // your max_speed
            dt,
            &opts,
        );
        new_vels.push(v);
    }

    // 3) Write back into RVOVelocity
    for (ent, v) in entities.into_iter().zip(new_vels) {
        if let Ok(mut vel) = q_vel.get_mut(ent) {
            **vel = v;
        }
    }
}

fn apply_velocity_system(time: Res<Time>, mut q: Query<(&mut Transform, &RVOVelocity)>) {
    let dt = time.delta_secs();
    for (mut tf, vel) in &mut q {
        tf.translation += **vel * dt;
    }
}
