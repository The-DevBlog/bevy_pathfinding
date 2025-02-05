use bevy::prelude::*;

use crate::{components::*, flowfield::FlowField};

pub struct BoidsPlugin;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Update, calculate_boid_steering);
    }
}

pub fn calculate_boid_steering(
    q_boids: Query<(Entity, &Transform, &Boid), With<Destination>>,
    mut q_flowfields: Query<&mut FlowField>,
) {
    let mut boids_data = Vec::new();
    for (ent, position, boid) in q_boids.iter() {
        boids_data.push((ent, position, boid));
    }

    for mut flowfield in q_flowfields.iter_mut() {
        // Filter down which boids are in this flowfield
        let relevant_boids: Vec<_> = boids_data
            .iter()
            .filter(|(ent, _, _)| flowfield.units.contains(ent))
            .collect();

        // For each boid, build neighbor list and compute boid vectors
        for (ent, pos, boid) in &relevant_boids {
            let my_pos = pos;

            // Gather neighbor positions
            let mut neighbor_positions = Vec::new();
            for (other_ent, other_pos, _boid) in &relevant_boids {
                if *other_ent == *ent {
                    continue;
                }
                let dist = my_pos.translation.distance(other_pos.translation);
                if dist < boid.neighbor_radius {
                    neighbor_positions.push(other_pos.translation);
                }
            }

            // Classical boids: separation, alignment, cohesion
            let mut separation = Vec3::ZERO;
            let mut alignment = Vec3::ZERO; // if you store velocity, do alignment properly
            let mut cohesion = Vec3::ZERO;

            if !neighbor_positions.is_empty() {
                // Separation
                for n_pos in &neighbor_positions {
                    let offset = my_pos.translation - *n_pos;
                    let dist = offset.length();
                    if dist > 0.0 {
                        separation += offset.normalize() / dist;
                    }
                }
                separation /= neighbor_positions.len() as f32;
                separation *= boid.separation_weight;

                // Cohesion
                let center =
                    neighbor_positions.iter().sum::<Vec3>() / neighbor_positions.len() as f32;
                let to_center = center - my_pos.translation;
                cohesion = to_center.normalize_or_zero() * boid.cohesion_weight;

                // Alignment – you’d need neighbor velocities to do it right
                alignment *= boid.alignment_weight;
            }

            // Flowfield direction
            let cell = flowfield.get_cell_from_world_position(my_pos.translation);
            let ff_dir_2d = cell.best_direction.vector();

            // Convert to 3D
            let ff_dir_3d = Vec3::new(ff_dir_2d.x as f32, 0.0, ff_dir_2d.y as f32);
            let flow_weight = 1.0; // if you want to tweak how strong flowfield is
            let flowfield_force = ff_dir_3d * flow_weight;

            // Sum up final steering
            let mut steering = separation + cohesion + alignment + flowfield_force;

            // Optionally clamp
            if steering.length() > boid.max_speed {
                steering = steering.normalize() * boid.max_speed;
            }

            // Store in the map so we can apply it later
            flowfield.steering_map.insert(*ent, steering);
        }
    }
}
