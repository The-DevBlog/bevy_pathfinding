use bevy::prelude::*;

use crate::{cell::Cell, flowfield::AvgDirection};

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
    position: Vec3,
    grid_size: IVec2,
    cell_diameter: f32,
    grid: &Vec<Vec<Cell>>,
    offset: Option<Vec2>,
) -> Cell {
    let mut x;
    let mut y;
    if let Some(offset) = offset {
        x = ((grid_size.x as f32) * offset.x).floor() as usize;
        y = ((grid_size.y as f32) * offset.y).floor() as usize;
    } else {
        x = (position.x / cell_diameter).floor() as usize;
        y = (position.z / cell_diameter).floor() as usize;
    }

    x = x.min(grid[0].len() - 1);
    y = y.min(grid.len() - 1);

    grid[y][x].clone()
}

// pub fn build_destinations(x: usize, grid: Vec<Vec<i32>>) -> Vec<IVec2> {
pub fn build_destinations(x: usize, grid: IVec2, avg_direction: AvgDirection) -> Vec<IVec2> {
    let rows = grid.x as usize;
    let cols = grid.y as usize;
    let total = rows * cols;
    let ones = x.max(0) as usize;

    // Create a result grid filled with 0's.
    let mut result = vec![vec![0; cols]; rows];

    // Build a list of all coordinates in the grid.
    let mut all_cells: Vec<(usize, usize)> = Vec::with_capacity(total);
    for r in 0..rows {
        for c in 0..cols {
            all_cells.push((r, c));
        }
    }

    // The greedy strategy: first, choose a starting cell.
    // We'll pick the cell closest to the center of the grid.
    let center = (rows as isize / 2, cols as isize / 2);
    let start_index = all_cells
        .iter()
        .enumerate()
        .min_by_key(|(_, &(r, c))| {
            let dr = r as isize - center.0;
            let dc = c as isize - center.1;
            dr * dr + dc * dc
        })
        .map(|(i, _)| i)
        .unwrap();
    let first = all_cells.swap_remove(start_index);
    let mut chosen = vec![first];

    // For each remaining one to place, choose the cell that maximizes
    // the minimum squared distance to any already–chosen cell.
    for _ in 1..ones {
        let mut best_index = 0;
        let mut best_distance_sq: isize = -1; // will store the best (maximum) distance² found.
        for (i, &cell) in all_cells.iter().enumerate() {
            // Compute the squared Euclidean distance from this candidate cell to the closest chosen cell.
            let distance_sq = chosen
                .iter()
                .map(|&(r, c)| {
                    let dr = r as isize - cell.0 as isize;
                    let dc = c as isize - cell.1 as isize;
                    dr * dr + dc * dc
                })
                .min()
                .unwrap();
            if distance_sq > best_distance_sq {
                best_distance_sq = distance_sq;
                best_index = i;
            }
        }
        // Remove the best candidate from the list and add it to chosen.
        let chosen_cell = all_cells.swap_remove(best_index);
        chosen.push(chosen_cell);
    }

    // Convert chosen cells to IVec2.
    let mut destinations: Vec<IVec2> = chosen
        .into_iter()
        .map(|(r, c)| IVec2::new(r as i32, c as i32))
        .collect();

    // Sort the occupied cells according to the specified fill direction.
    match avg_direction {
        AvgDirection::Right => destinations.sort_by(|a, b| a.x.cmp(&b.x).then(a.y.cmp(&b.y))),
        AvgDirection::Left => destinations.sort_by(|a, b| b.x.cmp(&a.x).then(a.y.cmp(&b.y))),
        AvgDirection::Down => destinations.sort_by(|a, b| b.y.cmp(&a.y).then(a.x.cmp(&b.x))),
        AvgDirection::Up => destinations.sort_by(|a, b| a.y.cmp(&b.y).then(a.x.cmp(&b.x))),
    }

    // Now mark the chosen positions in the result grid in the sorted order.
    // The printed numbers will reflect the fill order.
    for (i, cell) in destinations.iter().enumerate() {
        let r = cell.x as usize;
        let c = cell.y as usize;
        result[r][c] = i + 1;
    }

    destinations
}
