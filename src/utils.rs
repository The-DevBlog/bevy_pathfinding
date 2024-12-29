use crate::cell::Cell;

use bevy::prelude::*;
use std::cmp::min;

pub fn get_world_pos(
    map_base_trans: &GlobalTransform,
    cam_transform: &GlobalTransform,
    cam: &Camera,
    cursor_pos: Vec2,
) -> Vec3 {
    let plane_origin = map_base_trans.translation();
    let plane = InfinitePlane3d::new(map_base_trans.up());
    let ray = cam.viewport_to_world(cam_transform, cursor_pos).unwrap();
    let distance = ray.intersect_plane(plane_origin, plane).unwrap();
    return ray.get_point(distance);
}

pub fn to_viewport_coords(
    cam: &Camera,
    cam_transform: &GlobalTransform,
    world_position: Vec3,
) -> Vec2 {
    let viewport_position = cam.world_to_viewport(cam_transform, world_position);
    return viewport_position.unwrap();
}

pub fn get_cell_from_world_position_helper(
    world_pos: Vec3,
    grid_size: IVec2,
    cell_diameter: f32,
    grid: &Vec<Vec<Cell>>,
) -> Cell {
    // Adjust world position relative to the grid's top-left corner
    let adjusted_x = world_pos.x - (-grid_size.x as f32 * cell_diameter / 2.0);
    let adjusted_y = world_pos.z - (-grid_size.y as f32 * cell_diameter / 2.0);

    // Calculate percentages within the grid
    let mut percent_x = adjusted_x / (grid_size.x as f32 * cell_diameter);
    let mut percent_y = adjusted_y / (grid_size.y as f32 * cell_diameter);

    // Clamp percentages to ensure they're within [0.0, 1.0]
    percent_x = percent_x.clamp(0.0, 1.0);
    percent_y = percent_y.clamp(0.0, 1.0);

    // Calculate grid indices
    let x = ((grid_size.x as f32) * percent_x).floor() as usize;
    let y = ((grid_size.y as f32) * percent_y).floor() as usize;

    let x = min(x, grid_size.x as usize - 1);
    let y = min(y, grid_size.y as usize - 1);

    grid[y][x].clone() // Swap x and y
}
