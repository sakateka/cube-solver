use bevy::prelude::*;

/// Component to identify a layer group entity that contains 9 cubes
#[derive(Component, Debug, Clone)]
pub struct CubeLayer {
    pub face: LayerFace,
    pub layer_index: i32, // -1, 0, 1 for the three layers along each axis
}

/// Represents the nine possible layer orientations in a Rubik's cube
/// Each axis (X, Y, Z) has three layers: outer negative (-1), middle (0), outer positive (1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerFace {
    // X-axis layers
    Right,   // X = 1 layer (outer right)
    MiddleX, // X = 0 layer (middle vertical slice)
    Left,    // X = -1 layer (outer left)

    // Y-axis layers
    Up,      // Y = 1 layer (outer top)
    MiddleY, // Y = 0 layer (middle horizontal slice)
    Down,    // Y = -1 layer (outer bottom)

    // Z-axis layers
    Front,   // Z = 1 layer (outer front)
    MiddleZ, // Z = 0 layer (middle depth slice)
    Back,    // Z = -1 layer (outer back)
}

impl LayerFace {
    /// Get the rotation axis for this layer face
    pub fn rotation_axis(&self) -> Vec3 {
        match self {
            LayerFace::Front | LayerFace::MiddleZ | LayerFace::Back => Vec3::Z,
            LayerFace::Right | LayerFace::MiddleX | LayerFace::Left => Vec3::X,
            LayerFace::Up | LayerFace::MiddleY | LayerFace::Down => Vec3::Y,
        }
    }

    /// Get the rotation direction multiplier (1.0 for counter-clockwise, -1.0 for clockwise)
    /// Middle layers use the same direction as their positive counterparts
    pub fn rotation_direction(&self) -> f32 {
        match self {
            LayerFace::Back | LayerFace::Down | LayerFace::Left => 1.0,
            LayerFace::Front
            | LayerFace::Right
            | LayerFace::MiddleZ
            | LayerFace::MiddleX
            | LayerFace::Up
            | LayerFace::MiddleY => -1.0,
        }
    }

    /// Get the layer index (-1, 0, 1) for this face
    pub fn layer_index(&self) -> i32 {
        match self {
            LayerFace::Right | LayerFace::Up | LayerFace::Front => 1,
            LayerFace::MiddleX | LayerFace::MiddleY | LayerFace::MiddleZ => 0,
            LayerFace::Left | LayerFace::Down | LayerFace::Back => -1,
        }
    }

    /// Convert from the old CubeFace enum (only handles outer layers)
    pub fn from_cube_face(face: crate::cube_moves::CubeFace) -> Self {
        match face {
            crate::cube_moves::CubeFace::Front => LayerFace::Front,
            crate::cube_moves::CubeFace::Back => LayerFace::Back,
            crate::cube_moves::CubeFace::Right => LayerFace::Right,
            crate::cube_moves::CubeFace::Left => LayerFace::Left,
            crate::cube_moves::CubeFace::Up => LayerFace::Up,
            crate::cube_moves::CubeFace::Down => LayerFace::Down,
        }
    }
}

/// Component to mark individual cubes within a layer
#[derive(Component, Debug, Clone)]
pub struct LayersCube {
    pub layer_face: LayerFace,
    pub position_in_layer: Vec2, // Position within the 3x3 grid of the layer (-1, 0, 1)
}

/// Component for layer rotation animations
#[derive(Component, Debug)]
pub struct LayerRotationAnimation {
    pub target_rotation: f32,         // Target rotation in radians
    pub current_rotation: f32,        // Current rotation progress // FIXME: unused
    pub duration: f32,                // Animation duration in seconds
    pub elapsed: f32,                 // Elapsed time
    pub axis: Vec3,                   // Rotation axis
    pub initial_transform: Transform, // Store initial transform for proper rotation
    pub move_type: LayerMoveType,     // Type of move (CW, CCW, Double)
}

impl LayerRotationAnimation {
    pub fn new(
        target_angle: f32,
        duration: f32,
        axis: Vec3,
        initial_transform: Transform,
        move_type: LayerMoveType,
    ) -> Self {
        Self {
            target_rotation: target_angle,
            current_rotation: 0.0,
            duration,
            elapsed: 0.0,
            axis,
            initial_transform,
            move_type,
        }
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Get current rotation progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    /// Get current rotation angle
    pub fn current_angle(&self) -> f32 {
        self.target_rotation * self.progress()
    }
}

/// Enum representing move types for layers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerMoveType {
    Clockwise,        // 90° clockwise
    CounterClockwise, // 90° counter-clockwise
    Double,           // 180° rotation
}

impl LayerMoveType {
    /// Get the rotation angle in radians for this move type
    pub fn rotation_angle(&self) -> f32 {
        match self {
            LayerMoveType::Clockwise => std::f32::consts::FRAC_PI_2,
            LayerMoveType::CounterClockwise => -std::f32::consts::FRAC_PI_2,
            LayerMoveType::Double => std::f32::consts::PI,
        }
    }

    /// Convert from the old MoveType enum
    pub fn from_move_type(move_type: crate::cube_moves::MoveType) -> Self {
        match move_type {
            crate::cube_moves::MoveType::Clockwise => LayerMoveType::Clockwise,
            crate::cube_moves::MoveType::CounterClockwise => LayerMoveType::CounterClockwise,
            crate::cube_moves::MoveType::Double => LayerMoveType::Double,
        }
    }
}

/// Helper function to get position within a layer's 3x3 grid
pub fn get_position_in_layer(position: Vec3, layer_face: LayerFace) -> Vec2 {
    match layer_face {
        LayerFace::Front | LayerFace::MiddleZ | LayerFace::Back => {
            Vec2::new(position.x, position.y)
        }
        LayerFace::Right | LayerFace::MiddleX | LayerFace::Left => {
            Vec2::new(position.z, position.y)
        }
        LayerFace::Up | LayerFace::MiddleY | LayerFace::Down => Vec2::new(position.x, position.z),
    }
}

/// Helper function to check if a cube belongs to a specific layer
pub fn cube_belongs_to_layer(cube_position: Vec3, layer_face: LayerFace) -> bool {
    let tolerance = 0.1; // Small tolerance for floating point comparison

    match layer_face {
        LayerFace::Right => cube_position.x > 0.5 - tolerance,
        LayerFace::MiddleX => cube_position.x.abs() < 0.5 + tolerance,
        LayerFace::Left => cube_position.x < -0.5 + tolerance,

        LayerFace::Up => cube_position.y > 0.5 - tolerance,
        LayerFace::MiddleY => cube_position.y.abs() < 0.5 + tolerance,
        LayerFace::Down => cube_position.y < -0.5 + tolerance,

        LayerFace::Front => cube_position.z > 0.5 - tolerance,
        LayerFace::MiddleZ => cube_position.z.abs() < 0.5 + tolerance,
        LayerFace::Back => cube_position.z < -0.5 + tolerance,
    }
}

/// Get all cubes that belong to a specific layer
pub fn get_layer_cubes(
    cube_query: &Query<(Entity, &Transform), With<crate::cube_moves::CubeMoveTarget>>,
    layer_face: LayerFace,
) -> Vec<(Entity, Transform)> {
    cube_query
        .iter()
        .filter(|(_, transform)| cube_belongs_to_layer(transform.translation, layer_face))
        .map(|(entity, transform)| (entity, *transform))
        .collect()
}
