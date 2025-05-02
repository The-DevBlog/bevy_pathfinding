use std::collections::HashSet;

use bevy::prelude::*;

use crate::{components::*, flowfield::FlowField, grid::Grid};

pub struct BoidsPlugin;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, calculate_boid_steering);
    }
}

fn calculate_boid_steering(
    time: Res<Time>,
    mut q_boids: Query<(Entity, &mut Transform, &mut Boid)>,
    mut q_ff: Query<&mut FlowField>,
    grid: Res<Grid>,
) {
    let dt = time.delta_secs();

    // snapshot positions+velocities
    let boid_snapshot: Vec<(Entity, Vec3, Vec3)> = q_boids
        .iter()
        .map(|(e, tf, b)| (e, tf.translation, b.velocity))
        .collect();

    // 0.5) build set of all units in all flow-fields
    let mut ff_units = HashSet::new();
    for mut ff in q_ff.iter_mut() {
        ff_units.extend(ff.units.iter().copied());
    }

    // 1) GLOBAL SEPARATION — skip any that are in a flowfield
    for (ent, mut tf, mut boid) in q_boids.iter_mut() {
        // ← skip double-dippers
        if ff_units.contains(&ent) {
            continue;
        }

        let mut sep_force = Vec3::ZERO;
        let r2 = boid.neighbor_radius * boid.neighbor_radius;
        for (other_ent, pos, _) in &boid_snapshot {
            if *other_ent == ent {
                continue;
            }
            let delta = tf.translation - *pos;
            if delta.length_squared() < r2 {
                sep_force += delta.normalize() / delta.length();
            }
        }

        if sep_force != Vec3::ZERO {
            let desired = sep_force.normalize() * boid.max_speed;
            let steer = (desired - boid.velocity).clamp_length_max(boid.max_force);
            boid.velocity += steer * dt;
            boid.velocity = boid.velocity.clamp_length_max(boid.max_speed);
            tf.translation += boid.velocity * dt;
        }
    }

    // ——— 2) FLOW-FIELD + ALI/COH (optional) ———
    for mut ff in q_ff.iter_mut() {
        // 1) buffer of what we need to insert
        let mut pending: Vec<(Entity, Vec3)> = Vec::new();

        // for &unit in &ff.flowfield_props.units {
        for &unit in &ff.units {
            if let Ok((_ent, mut tf, mut boid)) = q_boids.get_mut(unit) {
                // rebuild neighbors with hysteresis and compute ali/coh
                let enter_r2 = boid.neighbor_radius.powi(2);
                let exit_r2 = boid.neighbor_exit_radius.powi(2);
                let mut current_neighbors = HashSet::new();

                let neighbor_data: Vec<(Vec3, Vec3)> = boid_snapshot
                    .iter()
                    .filter_map(|(other_ent, pos, vel)| {
                        let dist2 = tf.translation.distance_squared(*pos);
                        let was_neighbor = boid.prev_neighbors.contains(other_ent);
                        if dist2 < enter_r2 || (was_neighbor && dist2 < exit_r2) {
                            current_neighbors.insert(*other_ent);
                            Some((*pos, *vel))
                        } else {
                            None
                        }
                    })
                    .collect();

                let (sep, ali, coh) = compute_boids(&neighbor_data, tf.translation, &boid);

                let dir2d = ff.sample_direction(tf.translation, &grid);
                let flow_force = Vec3::new(dir2d.x, 0.0, dir2d.y);

                let raw = sep + ali + coh + flow_force;
                let desired = raw.clamp_length_max(boid.max_speed);
                let steer = (desired - boid.velocity).clamp_length_max(boid.max_force);

                boid.velocity += steer * dt;
                boid.velocity = boid.velocity.clamp_length_max(boid.max_speed);
                tf.translation += boid.velocity * dt;

                // buffer insertion and update neighbors
                pending.push((unit, steer));
                boid.prev_neighbors = current_neighbors;
            }
        }

        // 3) now that the immutable borrow of `units` is done, do one mutable borrow
        for (unit, steer) in pending {
            ff.steering_map.insert(unit, steer);
        }
    }
}

// neighbors: slice of (position, velocity)
fn compute_boids(neighbors: &[(Vec3, Vec3)], current_pos: Vec3, boid: &Boid) -> (Vec3, Vec3, Vec3) {
    let mut separation = Vec3::ZERO;
    let mut alignment = Vec3::ZERO;
    let mut cohesion = Vec3::ZERO;
    let count = neighbors.len() as f32;
    if count > 0.0 {
        // 1) Separation (same as before)
        for (n_pos, _) in neighbors {
            let offset = current_pos - *n_pos;
            let dist = offset.length();
            if dist > 0.0 {
                separation += offset.normalize() / dist;
            }
        }
        separation = (separation / count) * boid.separation_weight;

        // 2) Alignment: average neighbor velocity
        for (_, n_vel) in neighbors {
            alignment += *n_vel;
        }
        // normalize & weight
        alignment = (alignment / count).normalize_or_zero() * boid.alignment_weight;

        // 3) Cohesion: same as before
        let center = neighbors.iter().map(|(n_pos, _)| *n_pos).sum::<Vec3>() / count;
        let to_center = center - current_pos;
        cohesion = to_center.normalize_or_zero() * boid.cohesion_weight;
    }
    (separation, alignment, cohesion)
}
