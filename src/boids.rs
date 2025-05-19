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

                gizmos.circle(iso, boid.info.neighbor_radius, RED);
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
                let enter_r2 = boid.info.neighbor_radius.powi(2) * 2.0;
                let exit_r2 = boid.info.neighbor_exit_radius.powi(2);
                let mut current_neighbors = HashSet::new();
                let mut neighbor_data: Vec<(Vec3, Vec3)> = Vec::new();

                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if let Some(bucket) = buckets.get(&(bx + dx, by + dy)) {
                            for &(_e, pos, vel) in bucket {
                                let dist2 = tf.translation.distance_squared(pos);
                                let was_neighbor = boid.prev_neighbors.contains(&_e);
                                if dist2 < enter_r2 || (was_neighbor && dist2 < exit_r2) {
                                    current_neighbors.insert(_e);
                                    neighbor_data.push((pos, vel));
                                }
                            }
                        }
                    }
                }

                // compute classic boid forces
                let (sep, ali, coh) = compute_boids(&neighbor_data, tf.translation, &boid);

                // sample your flow‐field
                let dir2d = ff.sample_direction(tf.translation, &grid);
                let flow_force = Vec3::new(dir2d.x, 0.0, dir2d.y);

                // smooth and integrate
                let sep_w = boid.info.separation; // e.g. 1.5
                let ali_w = boid.info.alignment; // e.g. 1.0
                let coh_w = boid.info.cohesion; // e.g. 1.0
                let flow_w = 0.5; // < 1.0 so flow can’t overwhelm separation

                let raw = sep * sep_w + ali * ali_w + coh * coh_w + flow_force * flow_w;

                // let raw = sep + ali + coh + flow_force;
                let smooth = boid.prev_steer.lerp(raw, 0.1);
                boid.prev_steer = smooth;
                boid.steering = smooth;
                boid.velocity += smooth * dt;

                pending.push((unit, smooth));
                boid.prev_neighbors = current_neighbors;
            }
        }

        // 5) write back into the FlowField if you still need it
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
        separation = (separation / count) * boid.info.separation;

        // 2) Alignment: average neighbor velocity
        for (_, n_vel) in neighbors {
            alignment += *n_vel;
        }
        // normalize & weight
        alignment = (alignment / count).normalize_or_zero() * boid.info.alignment;

        // 3) Cohesion: same as before
        let center = neighbors.iter().map(|(n_pos, _)| *n_pos).sum::<Vec3>() / count;
        let to_center = center - current_pos;
        cohesion = to_center.normalize_or_zero() * boid.info.cohesion;
    }
    (separation, alignment, cohesion)
}
