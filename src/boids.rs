use std::{
    collections::{HashMap, HashSet},
    f32::consts::PI,
};

use bevy::{
    color::palettes::css::{RED, YELLOW},
    prelude::*,
};

use crate::{components::*, debug::resources::DbgOptions, flowfield::FlowField, grid::Grid};

pub struct BoidsPlugin;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, calculate_boid_steering);
    }
}

// New code. Bucketing for performance (big gains), but jittery
pub fn calculate_boid_steering(
    time: Res<Time>,
    mut q_boids: Query<(Entity, &Transform, &mut Boid)>,
    mut q_ff: Query<&mut FlowField>,
    grid: Res<Grid>,
    mut gizmos: Gizmos,
    dbg_options: Option<Res<DbgOptions>>,
) {
    let dt = time.delta_secs();

    // → WORLD dimensions (in world‐units), given cell_diameter and grid.size
    let world_width = grid.size.x as f32 * grid.cell_diameter;
    let world_depth = grid.size.y as f32 * grid.cell_diameter;

    // → Size of each bucket in world‐space
    let bucket_size_x = world_width / grid.buckets;
    let bucket_size_y = world_depth / grid.buckets;

    // → Find the “center” origin same as your bucket math
    let cols = grid.grid.len();
    let rows = grid.grid[0].len();
    let origin = grid.grid[cols / 2][rows / 2].world_pos;

    // 1) Snapshot all positions & velocities
    let snapshot: Vec<(Entity, Vec3, Vec3)> = q_boids
        .iter()
        .map(|(e, tf, b)| (e, tf.translation, b.velocity))
        .collect();

    // 2) Draw the grid(s) (optional)
    if let Some(dbg) = dbg_options {
        if dbg.draw_spatial_grid {
            gizmos.grid(
                Isometry3d::from_rotation(Quat::from_rotation_x(PI / 2.0)),
                UVec2::new(grid.buckets as u32, grid.buckets as u32),
                Vec2::new(bucket_size_x, bucket_size_y),
                YELLOW,
            );
        }

        if dbg.draw_radius {
            for (_, tf, boid) in q_boids.iter() {
                let pos: Vec3 = tf.translation;
                let rot = Quat::from_rotation_x(std::f32::consts::PI / 2.0);
                let iso = Isometry3d::new(pos, rot);

                gizmos.circle(iso, boid.info.neighbor_radius / 2.0, RED);
            }
        }
    }

    // 3) Build bucket map: (bx,by) → list of boids in that cell
    let mut buckets: HashMap<(i32, i32), Vec<(Entity, Vec3, Vec3)>> =
        HashMap::with_capacity(snapshot.len());

    for &(ent, pos, vel) in &snapshot {
        let bx = ((pos.x - origin.x) / bucket_size_x).floor() as i32;
        let by = ((pos.z - origin.y) / bucket_size_y).floor() as i32;
        buckets.entry((bx, by)).or_default().push((ent, pos, vel));
    }

    // 4) For each FlowField, compute steering only against 3×3 neighbor buckets
    for mut ff in q_ff.iter_mut() {
        let mut pending: Vec<(Entity, Vec3)> = Vec::with_capacity(ff.units.len());

        for &unit in &ff.units {
            if let Ok((_, tf, mut boid)) = q_boids.get_mut(unit) {
                // determine which bucket this boid is in
                let bx = ((tf.translation.x - origin.x) / bucket_size_x).floor() as i32;
                let by = ((tf.translation.z - origin.y) / bucket_size_y).floor() as i32;

                // gather neighbors with hysteresis
                let enter_r2 = boid.info.neighbor_radius.powi(2);
                let exit_r2 = boid.info.neighbor_exit_radius.powi(2);

                let mut current_neighbors = HashSet::new();

                let mut neighbor_data: Vec<(Vec3, Vec3)> = Vec::new();
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if let Some(bucket) = buckets.get(&(bx + dx, by + dy)) {
                            for &(other, pos, vel) in bucket {
                                if other == unit {
                                    continue; // ← don’t treat yourself as a neighbor!
                                }
                                let dist2 = tf.translation.distance_squared(pos);
                                let was_neighbor = boid.prev_neighbors.contains(&other);
                                if dist2 < enter_r2 || (was_neighbor && dist2 < exit_r2) {
                                    current_neighbors.insert(other);
                                    neighbor_data.push((pos, vel));
                                }
                            }
                        }
                    }
                }

                let (sep, ali, coh) = compute_boids(&neighbor_data, tf.translation, &boid);
                let dir2d = ff.sample_direction(tf.translation, &grid);
                let flow_force = Vec3::new(dir2d.x, 0.0, dir2d.y);

                // 1) build raw, then lerp into the old steering
                let raw = sep + ali + coh + flow_force;
                // let smooth = boid.prev_steer.lerp(raw, boid.info.steer_smoothing);
                let smooth = boid.prev_steer.lerp(raw, 0.1);

                // 2) now clamp that blended steering to max_force
                let accel = if smooth.length_squared() > boid.info.max_force * boid.info.max_force {
                    smooth.normalize_or_zero() * boid.info.max_force
                } else {
                    smooth
                };

                // 3) integrate & clamp speed
                boid.velocity += accel * dt;
                if boid.velocity.length_squared() > boid.info.max_speed * boid.info.max_speed {
                    boid.velocity = boid.velocity.normalize_or_zero() * boid.info.max_speed;
                }

                // 4) write out for next frame
                boid.prev_steer = accel;
                boid.steering = accel;

                // write back to flowfield
                pending.push((unit, accel));
            }
        }

        // 5) write back into the FlowField if you still need it
        for (unit, steer) in pending {
            ff.steering_map.insert(unit, steer);
        }
    }
}

fn compute_boids(neighbors: &[(Vec3, Vec3)], current_pos: Vec3, boid: &Boid) -> (Vec3, Vec3, Vec3) {
    let mut sep_sum = Vec3::ZERO;
    let mut ali_sum = Vec3::ZERO;
    let count = neighbors.len() as f32;

    if count > 0.0 {
        // -- Separation: distance-weighted—
        for &(n_pos, _) in neighbors {
            let offset = current_pos - n_pos;
            let dist = offset.length().max(0.01);
            // only repel inside neighbor_radius:
            if dist < boid.info.neighbor_radius {
                let strength = (boid.info.neighbor_radius - dist) / dist;
                sep_sum += offset.normalize() * strength;
            }
        }
        let desired_sep = sep_sum.normalize_or_zero() * boid.info.max_speed;
        let sep_steer = (desired_sep - boid.velocity).clamp_length_max(boid.info.max_force);
        // **weight once here**:
        let separation = sep_steer * boid.info.separation;

        // -- Alignment: average neighbor velocity
        for &(_, n_vel) in neighbors {
            ali_sum += n_vel;
        }
        let alignment = (ali_sum / count).normalize_or_zero() * boid.info.alignment;

        // -- Cohesion: steer toward average position
        let center = neighbors.iter().map(|(p, _)| *p).sum::<Vec3>() / count;
        let to_center = center - current_pos;
        let cohesion = to_center.normalize_or_zero() * boid.info.cohesion;

        return (separation, alignment, cohesion);
    }

    (Vec3::ZERO, Vec3::ZERO, Vec3::ZERO)
}
