use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct RotatingModel;

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct ColorSquare {
    pub color_index: usize,
}

impl ColorSquare {
    pub fn new(color_index: usize) -> Self {
        Self { color_index }
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct SelectionBorder {
    pub color_index: usize,
}

impl SelectionBorder {
    pub fn new(color_index: usize) -> Self {
        Self { color_index }
    }
}

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct RecoloredFace {
    pub color_index: usize,
    pub timestamp: f64,
}

impl RecoloredFace {
    pub fn new(color_index: usize, timestamp: f64) -> Self {
        Self {
            color_index,
            timestamp,
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct TouchState {
    pub last_touch_pos: Option<Vec2>,
    pub selected_color: Option<usize>,
    pub rotation_sensitivity: f32,
    pub rotation_threshold: f32,
    pub is_rotating: bool,
    pub rotation_cooldown_timer: f32,
    pub pending_selection_pos: Option<Vec2>,
    pub pending_selection_timer: f32,
}

impl Default for TouchState {
    fn default() -> Self {
        Self {
            last_touch_pos: None,
            selected_color: Some(0),
            rotation_sensitivity: 0.005,
            rotation_threshold: 2.0,
            is_rotating: false,
            rotation_cooldown_timer: 0.0,
            pending_selection_pos: None,
            pending_selection_timer: 0.0,
        }
    }
}

impl TouchState {
    pub fn set_selected_color(&mut self, color_index: usize) -> Option<usize> {
        let previous = self.selected_color;
        self.selected_color = Some(color_index);
        previous
    }

    pub fn clear_selected_color(&mut self) -> Option<usize> {
        self.selected_color.take()
    }

    pub fn should_rotate(&self, movement_delta: f32) -> bool {
        movement_delta > self.rotation_threshold
    }

    pub fn start_rotation(&mut self) {
        self.is_rotating = true;
        self.rotation_cooldown_timer = 0.2; // 200ms cooldown after rotation
        // Cancel any pending selection when rotation starts
        self.pending_selection_pos = None;
        self.pending_selection_timer = 0.0;
    }

    pub fn update_rotation_state(&mut self, delta_time: f32) {
        if self.rotation_cooldown_timer > 0.0 {
            self.rotation_cooldown_timer -= delta_time;
            if self.rotation_cooldown_timer <= 0.0 {
                self.is_rotating = false;
            }
        }

        // Update pending selection timer
        if self.pending_selection_timer > 0.0 {
            self.pending_selection_timer -= delta_time;
        }
    }

    pub fn start_pending_selection(&mut self, position: Vec2) {
        self.pending_selection_pos = Some(position);
        self.pending_selection_timer = 0.1; // 100ms delay before selection
    }

    pub fn should_trigger_pending_selection(&self) -> bool {
        self.pending_selection_pos.is_some()
            && self.pending_selection_timer <= 0.0
            && !self.is_rotating
    }

    pub fn consume_pending_selection(&mut self) -> Option<Vec2> {
        if self.should_trigger_pending_selection() {
            let pos = self.pending_selection_pos.take();
            self.pending_selection_timer = 0.0;
            pos
        } else {
            None
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct ColorManager {
    pub selected_color: Option<usize>,
    pub usage_counts: [u32; 6], // Count for each of the 6 colors - max 9 each
    pub max_faces_per_color: u32,
}

impl Default for ColorManager {
    fn default() -> Self {
        Self {
            selected_color: Some(0), // Default to white
            usage_counts: [0; 6],
            max_faces_per_color: 9,
        }
    }
}

impl ColorManager {
    /// Select a color (allows selecting any color, even at limit, for decoloring)
    pub fn try_select_color(&mut self, color_index: usize) -> Result<(), String> {
        if color_index >= 6 {
            return Err(format!("Invalid color index: {}", color_index));
        }

        // Allow selecting any color, even if at limit (for decoloring faces)
        self.selected_color = Some(color_index);
        Ok(())
    }

    /// Apply color to a face, handling old color decrement and new color increment
    pub fn apply_color_to_face(
        &mut self,
        color_index: usize,
        previous_color: Option<usize>,
    ) -> Result<bool, String> {
        if color_index >= 6 {
            return Err(format!("Invalid color index: {}", color_index));
        }

        if self.is_at_limit(color_index) {
            return Err(format!(
                "Cannot apply color {} - limit reached ({}/{})",
                color_index, self.usage_counts[color_index], self.max_faces_per_color
            ));
        }

        // Decrement previous color if it exists
        if let Some(prev_color) = previous_color {
            self.decrement_color(prev_color);
        }

        // Increment new color
        let reached_limit = self.increment_color(color_index);
        Ok(reached_limit)
    }

    fn increment_color(&mut self, color_index: usize) -> bool {
        if color_index < 6 {
            self.usage_counts[color_index] += 1;
            self.usage_counts[color_index] >= self.max_faces_per_color
        } else {
            false
        }
    }

    /// Decrement color count (made public for decoloring functionality)
    pub fn decrement_color(&mut self, color_index: usize) {
        if color_index < 6 && self.usage_counts[color_index] > 0 {
            self.usage_counts[color_index] -= 1;
        }
    }

    pub fn get_count(&self, color_index: usize) -> u32 {
        if color_index < 6 {
            self.usage_counts[color_index]
        } else {
            0
        }
    }

    pub fn is_at_limit(&self, color_index: usize) -> bool {
        self.get_count(color_index) >= self.max_faces_per_color
    }

    pub fn can_use_color(&self, color_index: usize) -> bool {
        !self.is_at_limit(color_index)
    }

    pub fn get_selected_color(&self) -> Option<usize> {
        self.selected_color
    }

    pub fn get_usage_info(&self, color_index: usize) -> String {
        format!(
            "{}/{}",
            self.get_count(color_index),
            self.max_faces_per_color
        )
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub enum Orientation {
    Up,
    Down,
    Front,
    Back,
    Right,
    Left,
}

impl Orientation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Orientation::Up => "Top",
            Orientation::Down => "Bottom",
            Orientation::Front => "Front",
            Orientation::Back => "Back",
            Orientation::Right => "Right",
            Orientation::Left => "Left",
        }
    }

    pub fn facelet_offset(&self) -> usize {
        match self {
            Orientation::Up => 0,     // U1-U9: positions 0-8
            Orientation::Right => 9,  // R1-R9: positions 9-17
            Orientation::Front => 18, // F1-F9: positions 18-26
            Orientation::Down => 27,  // D1-D9: positions 27-35
            Orientation::Left => 36,  // L1-L9: positions 36-44
            Orientation::Back => 45,  // B1-B9: positions 45-53
        }
    }

    pub fn to_cube_face(&self) -> crate::cube_moves::CubeFace {
        match self {
            Orientation::Up => crate::cube_moves::CubeFace::Up,
            Orientation::Right => crate::cube_moves::CubeFace::Right,
            Orientation::Front => crate::cube_moves::CubeFace::Front,
            Orientation::Down => crate::cube_moves::CubeFace::Down,
            Orientation::Left => crate::cube_moves::CubeFace::Left,
            Orientation::Back => crate::cube_moves::CubeFace::Back,
        }
    }

    pub fn from_cube_face(cube_face: crate::cube_moves::CubeFace) -> Self {
        match cube_face {
            crate::cube_moves::CubeFace::Up => Orientation::Up,
            crate::cube_moves::CubeFace::Down => Orientation::Down,
            crate::cube_moves::CubeFace::Front => Orientation::Front,
            crate::cube_moves::CubeFace::Back => Orientation::Back,
            crate::cube_moves::CubeFace::Right => Orientation::Right,
            crate::cube_moves::CubeFace::Left => Orientation::Left,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct Face {
    pub parent_cube: Entity,
}
