use std::collections::HashSet;

use bevy::prelude::*;

use crate::{components::*, flowfield::FlowField};

pub struct BoidsPlugin;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, calculate_boid_steering);
    }
}

fn calculate_boid_steering(
    time: Res<Time>,
    mut q_boids: Query<(Entity, &mut Transform, &mut Boid), With<Destination>>,
    mut q_ff: Query<&mut FlowField>,
) {
    let dt = time.delta_secs();

    // ——— 1) SNAPSHOT POSITIONS ———
    // Collect just (Entity, Vec3) so we don't store any &Transform or &Boid.
    let boid_positions: Vec<(Entity, Vec3, Vec3)> = q_boids
        .iter()
        .map(|(ent, tf, boid)| (ent, tf.translation, boid.velocity))
        .collect();

    // ——— 2) ACTUAL STEERING PASS ———
    for mut ff in q_ff.iter_mut() {
        for (ent, mut transform, mut boid) in q_boids.iter_mut() {
            // skip if this boid isn't in *this* flowfield
            if !ff.flowfield_props.units.contains(&ent) {
                continue;
            }

            // Build neighbor‐position list from our snapshot
            // Prepare new empty set for this frame
            let mut current_neighbors = HashSet::new();

            let enter_r2 = boid.neighbor_radius * boid.neighbor_radius;
            let exit_r2 = boid.neighbor_exit_radius * boid.neighbor_exit_radius;

            // Filter snapshots with hysteresis
            let neighbor_pos: Vec<(Vec3, Vec3)> = boid_positions
                .iter()
                .filter_map(|(other_ent, pos, vel)| {
                    let dist2 = transform.translation.distance_squared(*pos);
                    let was_neighbor = boid.prev_neighbors.contains(other_ent);

                    // either newly inside enter radius, or still inside exit radius
                    if dist2 < enter_r2 || (was_neighbor && dist2 < exit_r2) {
                        current_neighbors.insert(*other_ent);
                        Some((*pos, *vel))
                    } else {
                        None
                    }
                })
                .collect();

            // Compute classic boid forces
            // let (sep, ali, coh) = compute_boids(neighbor_pos, &*transform, &*boid);
            let (sep, ali, coh) = compute_boids(&neighbor_pos, transform.translation, &boid);

            // Sample your now‐smooth flowfield
            let dir2d = ff.sample_direction(transform.translation);
            let flow_force = Vec3::new(dir2d.x, 0.0, dir2d.y);

            // Combine into a desired‐velocity vector
            let raw = sep + ali + coh + flow_force;
            let desired = raw.clamp_length_max(boid.max_speed);

            // Turn that into an acceleration (steering)
            let steer = (desired - boid.velocity).clamp_length_max(boid.max_force);

            // Integrate velocity & position
            boid.velocity += steer * dt;
            boid.velocity = boid.velocity.clamp_length_max(boid.max_speed);
            transform.translation += boid.velocity * dt;

            // (Optional) store for debugging / visualization
            ff.flowfield_props.steering_map.insert(ent, steer);
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
