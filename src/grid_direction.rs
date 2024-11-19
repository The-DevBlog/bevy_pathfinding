use bevy::prelude::*;

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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
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
    /// Get the vector associated with the direction
    pub fn vector(self) -> IVec2 {
        match self {
            GridDirection::None => IVec2::new(0, 0),
            GridDirection::North => IVec2::new(0, 1),
            GridDirection::South => IVec2::new(0, -1),
            GridDirection::East => IVec2::new(1, 0),
            GridDirection::West => IVec2::new(-1, 0),
            GridDirection::NorthEast => IVec2::new(1, 1),
            GridDirection::NorthWest => IVec2::new(-1, 1),
            GridDirection::SouthEast => IVec2::new(1, -1),
            GridDirection::SouthWest => IVec2::new(-1, -1),
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
}
