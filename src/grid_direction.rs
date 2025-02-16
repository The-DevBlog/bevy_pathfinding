use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

/// A static array of all directions for lookup
const DIRECTIONS: [GridDirection; 9] = [
    GridDirection::None,
    GridDirection::North,
    GridDirection::NorthEast,
    GridDirection::East,
    GridDirection::SouthEast,
    GridDirection::South,
    GridDirection::SouthWest,
    GridDirection::West,
    GridDirection::NorthWest,
];

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Reflect)]
pub enum GridDirection {
    #[default]
    None,
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl GridDirection {
    pub fn vector(self) -> IVec2 {
        match self {
            GridDirection::None => IVec2::new(0, 0),
            GridDirection::West => IVec2::new(-1, 0),
            GridDirection::East => IVec2::new(1, 0),
            GridDirection::South => IVec2::new(0, 1),
            GridDirection::North => IVec2::new(0, -1),
            GridDirection::SouthWest => IVec2::new(-1, 1),
            GridDirection::NorthWest => IVec2::new(-1, -1),
            GridDirection::SouthEast => IVec2::new(1, 1),
            GridDirection::NorthEast => IVec2::new(1, -1),
        }
    }

    pub fn print_short(&self) {
        match self {
            GridDirection::None => print!("X , "),
            GridDirection::North => print!("N , "),
            GridDirection::South => print!("S , "),
            GridDirection::East => print!("E , "),
            GridDirection::West => print!("W , "),
            GridDirection::NorthEast => print!("NE, "),
            GridDirection::NorthWest => print!("NW, "),
            GridDirection::SouthEast => print!("SE, "),
            GridDirection::SouthWest => print!("SW, "),
        }
    }

    /// Get the direction from a given vector
    pub fn from_vector2(vector: IVec2) -> Option<GridDirection> {
        DIRECTIONS.iter().find(|&&d| d.vector() == vector).copied()
    }

    /// Cardinal directions (N, S, E, W)
    pub fn cardinal_directions() -> Vec<GridDirection> {
        vec![
            GridDirection::North,
            GridDirection::East,
            GridDirection::South,
            GridDirection::West,
        ]
    }

    /// Cardinal and intercardinal directions
    pub fn cardinal_and_intercardinal_directions() -> Vec<GridDirection> {
        vec![
            GridDirection::North,
            GridDirection::NorthEast,
            GridDirection::East,
            GridDirection::SouthEast,
            GridDirection::South,
            GridDirection::SouthWest,
            GridDirection::West,
            GridDirection::NorthWest,
        ]
    }

    /// All directions (including None)
    pub fn all_directions() -> Vec<GridDirection> {
        vec![
            GridDirection::None,
            GridDirection::North,
            GridDirection::NorthEast,
            GridDirection::East,
            GridDirection::SouthEast,
            GridDirection::South,
            GridDirection::SouthWest,
            GridDirection::West,
            GridDirection::NorthWest,
        ]
    }

    pub fn to_angle(&self) -> f32 {
        match self {
            GridDirection::None => 0.0,
            GridDirection::North => FRAC_PI_2,
            GridDirection::NorthEast => FRAC_PI_4,
            GridDirection::East => 0.0,
            GridDirection::SouthEast => -FRAC_PI_4,
            GridDirection::South => -FRAC_PI_2,
            GridDirection::SouthWest => -3.0 * FRAC_PI_4,
            GridDirection::West => PI,
            GridDirection::NorthWest => 3.0 * FRAC_PI_4,
        }
    }
}
