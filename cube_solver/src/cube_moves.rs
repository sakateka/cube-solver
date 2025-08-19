use bevy::prelude::*;

/// Component to mark entities that should be rotated during a cube move
#[derive(Component)]
pub struct CubeMoveTarget {
    pub face: CubeFace,
    pub layer: i32, // -1, 0, or 1 for the layer position
}

impl CubeMoveTarget {
    /// Determine the primary face for a cube position
    pub fn determine_face_from_position(position: &Vec3) -> CubeFace {
        // Find the face with the largest absolute coordinate
        let abs_x = position.x.abs();
        let abs_y = position.y.abs();
        let abs_z = position.z.abs();

        if abs_x >= abs_y && abs_x >= abs_z {
            if position.x > 0.0 {
                CubeFace::Right
            } else {
                CubeFace::Left
            }
        } else if abs_y >= abs_z {
            if position.y > 0.0 {
                CubeFace::Up
            } else {
                CubeFace::Down
            }
        } else if position.z > 0.0 {
            CubeFace::Front
        } else {
            CubeFace::Back
        }
    }
}

/// Enum representing the six faces of the cube
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeFace {
    Front, // Z = 1
    Back,  // Z = -1
    Right, // X = 1
    Left,  // X = -1
    Up,    // Y = 1
    Down,  // Y = -1
}

/// Enum representing move types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveType {
    Clockwise,        // No suffix
    CounterClockwise, // ' (prime)
    Double,           // 2
}

/// Parse move notation and return face and move type
pub fn parse_move_notation(notation: &str) -> Option<(CubeFace, MoveType)> {
    if notation.is_empty() {
        return None;
    }

    let face = match notation.chars().next()? {
        'F' => CubeFace::Front,
        'B' => CubeFace::Back,
        'R' => CubeFace::Right,
        'L' => CubeFace::Left,
        'U' => CubeFace::Up,
        'D' => CubeFace::Down,
        _ => return None,
    };

    let move_type = if notation.len() > 1 {
        match &notation[1..] {
            "'" => MoveType::CounterClockwise,
            "2" => MoveType::Double,
            _ => return None,
        }
    } else {
        MoveType::Clockwise
    };

    Some((face, move_type))
}

/// Event for triggering cube moves
#[derive(Event, Default)]
pub struct CubeMoveEvent {
    pub notation: String,
}
