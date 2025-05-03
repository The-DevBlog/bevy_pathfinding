use std::collections::HashSet;

use bevy::prelude::*;

use crate::{components::*, flowfield::FlowField, grid::Grid};

pub struct BoidsPlugin;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, calculate_boid_steering);
    }
}

pub fn calculate_boid_steering(
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

    // build set of all units in all flow-fields
    let mut ff_units = HashSet::new();
    for ff in q_ff.iter_mut() {
        ff_units.extend(ff.units.iter().copied());
    }

    // FLOW-FIELD + SEP/ALI/COH
    for mut ff in q_ff.iter_mut() {
        // 1) buffer of what we need to insert
        let mut pending: Vec<(Entity, Vec3)> = Vec::new();

        for &unit in &ff.units {
            if let Ok((_ent, mut boid_tf, mut boid)) = q_boids.get_mut(unit) {
                // rebuild neighbors with hysteresis
                let enter_r2 = boid.neighbor_radius.powi(2);
                let exit_r2 = boid.neighbor_exit_radius.powi(2);
                let mut current_neighbors = HashSet::new();

                // collect neighbor positions + velocities
                let neighbor_data: Vec<(Vec3, Vec3)> = boid_snapshot
                    .iter()
                    .filter_map(|(other_ent, pos, vel)| {
                        let dist2 = boid_tf.translation.distance_squared(*pos);
                        let was_neighbor = boid.prev_neighbors.contains(other_ent);
                        if dist2 < enter_r2 || (was_neighbor && dist2 < exit_r2) {
                            current_neighbors.insert(*other_ent);
                            Some((*pos, *vel))
                        } else {
                            None
                        }
                    })
                    .collect();

                // compute boid forces (sep, ali, coh)
                let (sep, ali, coh) = compute_boids(&neighbor_data, boid_tf.translation, &boid);

                // sample flow-field
                let dir2d = ff.sample_direction(boid_tf.translation, &grid);
                let flow_force = Vec3::new(dir2d.x, 0.0, dir2d.y);

                // STEP 3: Low-pass filter the final steering
                // first compute raw steering
                // let raw = sep + ali + coh + flow_force;
                let raw = sep + flow_force;
                let desired = raw.clamp_length_max(boid.max_speed);
                let unclamped_steer = desired - boid.velocity;
                // apply clamp
                let steer = unclamped_steer.clamp_length_max(boid.max_force);

                // low-pass filter
                let alpha = 0.1; // adjust for smoothness
                let smooth_steer = boid.prev_steer.lerp(steer, alpha);
                boid.prev_steer = smooth_steer;

                // apply smoothed steering
                boid.velocity += smooth_steer * dt;
                boid.velocity = boid.velocity.clamp_length_max(boid.max_speed);
                boid_tf.translation += boid.velocity * dt;

                // buffer insertion and update neighbors
                pending.push((unit, smooth_steer));
                boid.prev_neighbors = current_neighbors;
            }
        }

        // now that the immutable borrow of `units` is done, do one mutable borrow
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
