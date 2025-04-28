use bevy::prelude::*;

use crate::{cell::Cell, components::*, flowfield::FlowField};

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
    // let boid_positions: Vec<(Entity, Vec3)> = q_boids
    //     .iter()
    //     .map(|(ent, tf, _)| (ent, tf.translation))
    //     .collect();

    // at top of calculate_boid_steering
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
            // let neighbor_pos: Vec<Vec3> = boid_positions
            //     .iter()
            //     .filter(|(other_ent, pos)| {
            //         *other_ent != ent && transform.translation.distance(*pos) < boid.neighbor_radius
            //     })
            //     .map(|(_, pos)| *pos)
            //     .collect();

            let neighbor_pos: Vec<(Vec3, Vec3)> = boid_positions
                .iter()
                .filter(|(other_ent, other_pos, _)| {
                    *other_ent != ent
                        && transform.translation.distance(*other_pos) < boid.neighbor_radius
                })
                .map(|(_, pos, vel)| (*pos, *vel))
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

// fn compute_boids(neighbor_pos: Vec<Vec3>, pos: &Transform, boid: &Boid) -> (Vec3, Vec3, Vec3) {
//     // Classical boids: separation, alignment, cohesion
//     let mut separation = Vec3::ZERO;
//     let mut alignment = Vec3::ZERO;
//     let mut cohesion = Vec3::ZERO;

//     if !neighbor_pos.is_empty() {
//         // Separation
//         for n_pos in &neighbor_pos {
//             let offset = pos.translation - *n_pos;
//             let dist = offset.length();
//             if dist > 0.0 {
//                 separation += offset.normalize() / dist;
//             }
//         }
//         separation /= neighbor_pos.len() as f32;

//         separation *= boid.separation_weight;

//         // Cohesion
//         let center = neighbor_pos.iter().sum::<Vec3>() / neighbor_pos.len() as f32;
//         let to_center = center - pos.translation;
//         cohesion = to_center.normalize_or_zero() * boid.cohesion_weight;

//         // Alignment – you’d need neighbor velocities to do it right
//         alignment *= boid.alignment_weight;
//     }

//     (separation, alignment, cohesion)
// }

fn compute_steering(ff_dir_2d: Vec2, computed_boids: Vec3, boid: &Boid) -> Vec3 {
    // Convert to 3D
    let ff_dir_3d = Vec3::new(ff_dir_2d.x, 0.0, ff_dir_2d.y);
    let flow_weight = 1.0; // if you want to tweak how strong flowfield is
    let flowfield_force = ff_dir_3d * flow_weight;

    // Sum up final steering
    let mut steering = computed_boids + flowfield_force;

    // Optionally clamp
    if steering.length() > boid.max_speed {
        steering = steering.normalize() * boid.max_speed;
    }

    steering
}

// fn compute_steering(cell: Cell, computed_boids: Vec3, boid: &Boid) -> Vec3 {
//     let ff_dir_2d = cell.best_direction.vector();

//     // Convert to 3D
//     let ff_dir_3d = Vec3::new(ff_dir_2d.x as f32, 0.0, ff_dir_2d.y as f32);
//     let flow_weight = 1.0; // if you want to tweak how strong flowfield is
//     let flowfield_force = ff_dir_3d * flow_weight;

//     // Sum up final steering
//     let mut steering = computed_boids + flowfield_force;

//     // Optionally clamp
//     if steering.length() > boid.max_speed {
//         steering = steering.normalize() * boid.max_speed;
//     }

//     steering
// }

fn gather_neighbors_positions(
    relevant_boids: &[&(Entity, &Transform, &Boid)],
    pos: &Transform,
    ent: &Entity,
    boid: &Boid,
) -> Vec<Vec3> {
    let mut neighbor_positions = Vec::new();
    for (other_ent, other_pos, _boid) in relevant_boids.iter() {
        if *other_ent == *ent {
            continue;
        }
        let dist = pos.translation.distance(other_pos.translation);
        if dist < boid.neighbor_radius {
            neighbor_positions.push(other_pos.translation);
        }
    }

    neighbor_positions
}
