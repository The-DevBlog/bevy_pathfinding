use bevy::prelude::*;
use bevy_rapier3d::{plugin::RapierContext, prelude::*};

use crate::cell::*;

#[derive(Component, Clone, Default)]
pub struct FlowField {
    pub cell_radius: f32,
    pub cell_diameter: f32,
    pub grid: Vec<Vec<Cell>>,
    pub grid_size: IVec2,
}

impl FlowField {
    pub fn new(cell_radius: f32, grid_size: IVec2) -> Self {
        FlowField {
            cell_radius,
            cell_diameter: cell_radius * 2.,
            grid: Vec::default(),
            grid_size,
        }
    }

    pub fn create_grid(&mut self) {
        self.grid = (0..self.grid_size.x)
            .map(|x| {
                (0..self.grid_size.y)
                    .map(|y| {
                        let x_pos = self.cell_diameter * x as f32 + self.cell_radius;
                        let y_pos = self.cell_diameter * y as f32 + self.cell_radius;
                        let world_pos = Vec3::new(x_pos, 0.0, y_pos);
                        Cell::new(world_pos, IVec2::new(x, y))
                    })
                    .collect()
            })
            .collect();
    }

    pub fn create_costfield(&mut self, rapier_ctx: &RapierContext) {
        // let cell_half_extens = Vec3::ONE * self.cell_radius;

        for cell_row in self.grid.iter_mut() {
            let mut has_increased_cost = false;

            for cell in cell_row.iter_mut() {
                let hit = rapier_ctx.intersection_with_shape(
                    cell.world_position,
                    Quat::IDENTITY,
                    &Collider::cuboid(self.cell_radius, self.cell_radius, self.cell_radius),
                    QueryFilter::default().exclude_sensors(),
                );

                if let Some(_) = hit {
                    if !has_increased_cost {
                        cell.increase_cost(255);
                        has_increased_cost = true;
                    }
                }
            }
        }
    }
}

// // Phase 3
// fn detect_colliders(
//     _trigger: Trigger<DetectCollidersEv>,
//     mut cmds: Commands,
//     mut flowfield_q: Query<&mut FlowField>,
//     rapier_context: Res<RapierContext>,
//     grid: Res<Grid>,
//     selected_q: Query<Entity, With<Selected>>,
// ) {
//     // println!("PHASE 3: Detect Colliders");
//     let selected_entities: HashSet<Entity> = selected_q.iter().collect();

//     for mut flowfield in flowfield_q.iter_mut() {
//         for x in 0..grid.rows {
//             for z in 0..grid.columns {
//                 let cell = &mut flowfield.cells[x][z];
//                 cell.occupied = false; // Reset obstacle status

//                 let cell_size = grid.cell_size / 2.0;
//                 let cell_shape = Collider::cuboid(cell_size, cell_size, cell_size);
//                 let mut cell_occupied = false;

//                 // Capture selected_entities by reference for use in the closure
//                 let selected_entities = &selected_entities;

//                 rapier_context.intersections_with_shape(
//                     cell.world_position,
//                     Quat::IDENTITY, // No rotation
//                     &cell_shape,
//                     QueryFilter::default().exclude_sensors(),
//                     |collider_entity| {
//                         if !selected_entities.contains(&collider_entity) {
//                             // Collider is overlapping the cell and is not a selected unit
//                             cell_occupied = true;
//                             false
//                         } else {
//                             // Collider is a selected unit, ignore it
//                             true
//                         }
//                     },
//                 );

//                 cell.occupied = cell_occupied;
//             }
//         }
//     }

//     // println!("PHASE 3: Detect Colliders Done");
//     cmds.trigger(CalculateFlowFieldEv);
// }
