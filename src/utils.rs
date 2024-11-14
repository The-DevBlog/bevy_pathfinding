use bevy::prelude::*;

use crate::*;

pub fn get_world_coords(
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

// retrieves the cell row/column given a set a world coordinates
pub fn get_cell(grid: &Grid, coords: &Vec3) -> (u32, u32) {
    // Adjust mouse coordinates to the grid's coordinate system
    let grid_origin_x = -grid.width / 2.0;
    let grid_origin_z = -grid.depth / 2.0;
    let adjusted_x = coords.x - grid_origin_x; // Shift origin to (0, 0)
    let adjusted_z = coords.z - grid_origin_z;

    // Calculate the column and row indices
    let cell_width = grid.width / grid.rows as f32;
    let cell_depth = grid.depth / grid.columns as f32;
    let row = (adjusted_x / cell_width).floor() as u32;
    let column = (adjusted_z / cell_depth).floor() as u32;

    (row, column)
}
