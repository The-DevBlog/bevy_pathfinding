use bevy::prelude::*;

use crate::{
    cell::Cell,
    components::*,
    flowfield::{FlowField, FlowFieldProps},
};

pub struct BoidsPlugin;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, calculate_boid_steering);
    }
}

fn calculate_boid_steering(
    q_boids: Query<(Entity, &Transform, &Boid), With<Destination>>,
    mut q_ff: Query<&mut FlowField>,
) {
    let mut boids_data = Vec::new();
    for (ent, position, boid) in q_boids.iter() {
        boids_data.push((ent, position, boid));
    }

    for mut ff in q_ff.iter_mut() {
        // PARENT FLOWFIELD
        // Filter down which boids are in this flowfield
        let relevant_boids: Vec<_> = boids_data
            .iter()
            .filter(|(ent, _, _)| ff.flowfield_props.units.contains(ent))
            .collect();

        // For each boid, build neighbor list and compute boid vectors
        for (ent, pos, boid) in &relevant_boids {
            let neighbor_pos = gather_neighbors_positions(&relevant_boids, pos, ent, boid);
            let (separation, alignment, cohesion) = compute_boids(neighbor_pos, pos, boid);

            // Flowfield direction
            let cell = ff.get_cell_from_world_position(pos.translation);
            let computed_boids = separation + cohesion + alignment;
            let steering = compute_steering(cell, computed_boids, boid);

            // Store in the map so we can apply it later
            ff.flowfield_props.steering_map.insert(*ent, steering);
        }

        // DESTINATION FLOWFIELD
        // Filter down which boids are in this flowfield
        if !ff.destination_flowfield.initialized {
            continue;
        }

        let relevant_boids: Vec<_> = boids_data
            .iter()
            .filter(|(ent, _, _)| ff.destination_flowfield.flowfield_props.units.contains(ent))
            .collect();

        // For each boid, build neighbor list and compute boid vectors
        for (ent, pos, boid) in &relevant_boids {
            let dest_ff = &mut ff.destination_flowfield;
            let neighbor_pos = gather_neighbors_positions(&relevant_boids, pos, ent, boid);
            let (separation, alignment, cohesion) = compute_boids(neighbor_pos, pos, boid);

            // Flowfield direction
            let cell = dest_ff.get_cell_from_world_position(pos.translation);

            let computed_boids = separation + cohesion + alignment;
            let steering = compute_steering(cell, computed_boids, boid);

            // Store in the map so we can apply it later
            dest_ff.flowfield_props.steering_map.insert(*ent, steering);
        }
    }
}

fn compute_steering(cell: Cell, computed_boids: Vec3, boid: &Boid) -> Vec3 {
    let ff_dir_2d = cell.best_direction.vector();

    // Convert to 3D
    let ff_dir_3d = Vec3::new(ff_dir_2d.x as f32, 0.0, ff_dir_2d.y as f32);
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

fn compute_boids(neighbor_pos: Vec<Vec3>, pos: &Transform, boid: &Boid) -> (Vec3, Vec3, Vec3) {
    // Classical boids: separation, alignment, cohesion
    let mut separation = Vec3::ZERO;
    let mut alignment = Vec3::ZERO;
    let mut cohesion = Vec3::ZERO;

    if !neighbor_pos.is_empty() {
        // Separation
        for n_pos in &neighbor_pos {
            let offset = pos.translation - *n_pos;
            let dist = offset.length();
            if dist > 0.0 {
                separation += offset.normalize() / dist;
            }
        }
        separation /= neighbor_pos.len() as f32;
        separation *= boid.separation_weight;

        // Cohesion
        let center = neighbor_pos.iter().sum::<Vec3>() / neighbor_pos.len() as f32;
        let to_center = center - pos.translation;
        cohesion = to_center.normalize_or_zero() * boid.cohesion_weight;

        // Alignment – you’d need neighbor velocities to do it right
        alignment *= boid.alignment_weight;
    }

    (separation, alignment, cohesion)
}

fn gather_neighbors_positions(
    relevant_boids: &[&(Entity, &Transform, &Boid)],
    pos: &Transform,
    ent: &Entity,
    boid: &Boid,
) -> Vec<Vec3> {
    // Gather neighbor positions
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
