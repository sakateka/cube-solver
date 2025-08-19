use crate::components::{Face, Orientation, RecoloredFace};
use crate::ui::rotations_panel::LayerRotationCompletedEvent;
use bevy::prelude::*;
use min2phase::solve;
use std::collections::HashMap;
use std::fmt;

// Default center face orientations in solved state: U, R, F, D, L, B
const DEFAULT_CENTER_FACES: [char; 6] = ['U', 'R', 'F', 'D', 'L', 'B'];

// Center facelet indices in the facelet string (position 4 of each face)
const CENTER_FACELET_INDICES: [usize; 6] = [4, 13, 22, 31, 40, 49];

/// Face colors for the cube
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaceColor {
    White,  // U
    Red,    // R
    Green,  // F
    Yellow, // D
    Orange, // L
    Blue,   // B
}

impl FaceColor {
    /// Convert to min2phase facelet character
    pub fn to_facelet_char(self) -> char {
        match self {
            FaceColor::White => 'U',
            FaceColor::Red => 'R',
            FaceColor::Green => 'F',
            FaceColor::Yellow => 'D',
            FaceColor::Orange => 'L',
            FaceColor::Blue => 'B',
        }
    }

    /// Convert from color index (0-5)
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => FaceColor::White,
            1 => FaceColor::Yellow,
            2 => FaceColor::Red,
            3 => FaceColor::Orange,
            4 => FaceColor::Blue,
            5 => FaceColor::Green,
            _ => unreachable!(),
        }
    }
}

/// min2phase error codes and their descriptions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Min2PhaseError {
    // Facelet parsing errors (from_facelet function)
    InvalidFaceletLength,
    InvalidFaceletCharacter,
    IncorrectColorCount,

    // Cube verification errors (verify function)
    MissingEdges,     // -2: Not all 12 edges exist exactly once
    EdgeFlipError,    // -3: One edge has to be flipped
    MissingCorners,   // -4: Not all corners exist exactly once
    CornerTwistError, // -5: One corner has to be twisted
    ParityError,      // -6: Two corners or two edges have to be exchanged

    // Solving errors (solve function)
    NoSolutionExists,   // Error 7: No solution exists for the given maxDepth
    ProbeLimitExceeded, // Error 8: Probe limit exceeded, no solution within given probMax
}

impl Min2PhaseError {
    /// Convert min2phase error code to human-readable description
    pub fn from_error_code(error_code: &str) -> Option<Self> {
        match error_code {
            "Error 1" => Some(Min2PhaseError::IncorrectColorCount),
            "Error 2" => Some(Min2PhaseError::MissingEdges),
            "Error 3" => Some(Min2PhaseError::EdgeFlipError),
            "Error 4" => Some(Min2PhaseError::MissingCorners),
            "Error 5" => Some(Min2PhaseError::CornerTwistError),
            "Error 6" => Some(Min2PhaseError::ParityError),
            "Error 7" => Some(Min2PhaseError::NoSolutionExists),
            "Error 8" => Some(Min2PhaseError::ProbeLimitExceeded),
            _ => None,
        }
    }

    /// Convert verification error code to human-readable description
    pub fn from_verify_code(verify_code: i32) -> Option<Self> {
        match verify_code {
            -1 => Some(Min2PhaseError::InvalidFaceletLength),
            -2 => Some(Min2PhaseError::MissingEdges),
            -3 => Some(Min2PhaseError::EdgeFlipError),
            -4 => Some(Min2PhaseError::MissingCorners),
            -5 => Some(Min2PhaseError::CornerTwistError),
            -6 => Some(Min2PhaseError::ParityError),
            _ => None,
        }
    }

    /// Get human-readable description of the error
    pub fn description(&self) -> &'static str {
        match self {
            Min2PhaseError::InvalidFaceletLength => {
                "Invalid facelet string: incorrect length or format"
            }
            Min2PhaseError::InvalidFaceletCharacter => {
                "Invalid facelet string: contains invalid characters"
            }
            Min2PhaseError::IncorrectColorCount => {
                "Invalid cube: there is not exactly one facelet of each color"
            }
            Min2PhaseError::MissingEdges => "Invalid cube: not all 12 edges exist exactly once",
            Min2PhaseError::EdgeFlipError => "Invalid cube: one edge has to be flipped",
            Min2PhaseError::MissingCorners => "Invalid cube: not all 8 corners exist exactly once",
            Min2PhaseError::CornerTwistError => "Invalid cube: one corner has to be twisted",
            Min2PhaseError::ParityError => {
                "Invalid cube: two corners or two edges have to be exchanged"
            }
            Min2PhaseError::NoSolutionExists => {
                "Cube is valid but no solution exists within the given move limit"
            }
            Min2PhaseError::ProbeLimitExceeded => {
                "Cube is valid but no solution found within the probe limit"
            }
        }
    }

    /// Get detailed explanation with suggestions
    pub fn detailed_explanation(&self) -> &'static str {
        match self {
            Min2PhaseError::InvalidFaceletLength => {
                "The cube facelet string has an incorrect length or format. \
                A valid cube must have exactly 54 facelets in the format: \
                U1U2...U9R1R2...R9F1..F9D1..D9L1..L9B1..B9"
            }
            Min2PhaseError::InvalidFaceletCharacter => {
                "The cube facelet string contains invalid characters. \
                Only the characters U, R, F, D, L, B are allowed."
            }
            Min2PhaseError::IncorrectColorCount => {
                "The cube has an incorrect number of facelets for each color. \
                Each color (U, R, F, D, L, B) must appear exactly 9 times."
            }
            Min2PhaseError::MissingEdges => {
                "The cube is missing some edges or has duplicate edges. \
                A valid cube must have exactly 12 edges, each appearing once."
            }
            Min2PhaseError::EdgeFlipError => {
                "The cube has an edge that is flipped incorrectly. \
                This means one edge piece is oriented the wrong way."
            }
            Min2PhaseError::MissingCorners => {
                "The cube is missing some corners or has duplicate corners. \
                A valid cube must have exactly 8 corners, each appearing once."
            }
            Min2PhaseError::CornerTwistError => {
                "The cube has a corner that is twisted incorrectly. \
                This means one corner piece is rotated the wrong way."
            }
            Min2PhaseError::ParityError => {
                "The cube has a parity error. This means two pieces need to be swapped. \
                This is impossible to solve with standard moves."
            }
            Min2PhaseError::NoSolutionExists => {
                "The cube is valid but cannot be solved within the current move limit. \
                Try increasing the maximum number of moves allowed."
            }
            Min2PhaseError::ProbeLimitExceeded => {
                "The cube is valid but the solver couldn't find a solution within the time limit. \
                This usually means the cube requires many moves to solve."
            }
        }
    }

    /// Get suggestions for fixing the error
    pub fn suggestions(&self) -> &'static str {
        match self {
            Min2PhaseError::InvalidFaceletLength | Min2PhaseError::InvalidFaceletCharacter => {
                "Check that all 54 cube faces are properly colored and mapped."
            }
            Min2PhaseError::IncorrectColorCount => {
                "Make sure each color appears exactly 9 times on the cube."
            }
            Min2PhaseError::MissingEdges | Min2PhaseError::MissingCorners => {
                "Check that all cube pieces are in their correct positions."
            }
            Min2PhaseError::EdgeFlipError | Min2PhaseError::CornerTwistError => {
                "Check that all pieces are oriented correctly. You may need to physically twist or flip pieces."
            }
            Min2PhaseError::ParityError => {
                "This cube cannot be solved with standard moves. You may need to disassemble and reassemble it."
            }
            Min2PhaseError::NoSolutionExists | Min2PhaseError::ProbeLimitExceeded => {
                "Try increasing the solver's move limit or probe limit. This cube may require many moves to solve."
            }
        }
    }
}

impl fmt::Display for Min2PhaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Cube validation result
#[derive(Debug, Clone)]
pub enum CubeValidation {
    NotValidated,
    Valid,
    Invalid(String),
    SolvingFailed(String),
}

impl CubeValidation {
    pub fn is_valid(&self) -> bool {
        matches!(self, CubeValidation::Valid)
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            CubeValidation::NotValidated => None,
            CubeValidation::Valid => None,
            CubeValidation::Invalid(msg) | CubeValidation::SolvingFailed(msg) => Some(msg),
        }
    }
}

impl Default for CubeValidation {
    fn default() -> Self {
        Self::NotValidated
    }
}

/// Represents a Rubik's cube state in facelet format
#[derive(Debug, Clone)]
pub struct CubeState {
    facelets: String,
    validation: CubeValidation,
    solution: Option<String>,
}

impl CubeState {
    const FACE_SIZE: usize = 9;
    const TOTAL_FACELETS: usize = 54;
    const VALID_CHARS: [char; 6] = ['U', 'R', 'F', 'D', 'L', 'B'];

    pub fn new() -> Self {
        Self {
            facelets: String::new(),
            validation: CubeValidation::NotValidated,
            solution: None,
        }
    }

    pub fn from_facelets(facelets: String) -> Self {
        let mut state = Self::new();
        state.facelets = facelets;
        state.validate();
        state
    }

    pub fn facelets(&self) -> &str {
        &self.facelets
    }

    pub fn validation(&self) -> &CubeValidation {
        &self.validation
    }

    pub fn solution(&self) -> Option<&str> {
        self.solution.as_deref()
    }

    pub fn solution_moves(&self) -> Vec<String> {
        self.solution
            .as_ref()
            .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }

    /// Perform lightweight validation only (no solving)
    fn perform_lightweight_validation_only(&mut self) {
        // Check length
        if self.facelets.len() != Self::TOTAL_FACELETS {
            self.validation = CubeValidation::Invalid(format!(
                "Invalid facelet length: {} (expected {})",
                self.facelets.len(),
                Self::TOTAL_FACELETS
            ));
            return;
        }

        // Check for incomplete state (spaces)
        if self.facelets.contains(' ') {
            let unmapped_count = self.facelets.chars().filter(|&c| c == ' ').count();
            self.validation = CubeValidation::Invalid(format!(
                "Incomplete cube: {} faces are not colored",
                unmapped_count
            ));
            return;
        }

        // Validate characters
        if let Err(error) = self.validate_characters() {
            self.validation = CubeValidation::Invalid(error);
            return;
        }

        // Validate color counts
        if let Err(error) = self.validate_color_counts() {
            self.validation = CubeValidation::Invalid(error);
            return;
        }

        // Validate cube structure
        if let Err(error) = self.validate_cube_structure() {
            self.validation = CubeValidation::Invalid(error);
            return;
        }

        // Lightweight validation passed - mark as valid but not solved
        self.validation = CubeValidation::Valid;

        // Clear any previous solution since we're only doing lightweight validation
        self.solution = None;
    }

    /// Perform full validation including solving attempt
    fn validate(&mut self) {
        // Reset validation state and clear any previous solution
        self.validation = CubeValidation::NotValidated;
        self.solution = None;

        // First do lightweight validation
        self.perform_lightweight_validation_only();

        // If lightweight validation passed, attempt to solve
        if matches!(self.validation, CubeValidation::Valid) {
            log::debug!("Lightweight validation passed, attempting to solve...");
            self.attempt_solve();
        } else {
            log::debug!("Lightweight validation failed, skipping solve attempt");
        }
    }

    /// Perform lightweight validation only (no solving)
    fn validate_lightweight_only(&mut self) {
        // Reset validation state and clear any previous solution
        self.validation = CubeValidation::NotValidated;
        self.solution = None;

        // Do lightweight validation only
        self.perform_lightweight_validation_only();
    }

    fn validate_characters(&self) -> Result<(), String> {
        for (i, c) in self.facelets.chars().enumerate() {
            if !Self::VALID_CHARS.contains(&c) {
                return Err(format!("Invalid character '{}' at position {}", c, i));
            }
        }
        Ok(())
    }

    fn validate_color_counts(&self) -> Result<(), String> {
        let mut counts = HashMap::new();

        for c in self.facelets.chars() {
            *counts.entry(c).or_insert(0) += 1;
        }

        for &color in &Self::VALID_CHARS {
            let count = counts.get(&color).copied().unwrap_or(0);
            if count != Self::FACE_SIZE {
                return Err(format!(
                    "Invalid color count: {} appears {} times (expected {})",
                    color,
                    count,
                    Self::FACE_SIZE
                ));
            }
        }
        Ok(())
    }

    fn validate_cube_structure(&self) -> Result<(), String> {
        // Check center pieces
        let centers = [
            (4, 'U'),  // U5
            (13, 'R'), // R5
            (22, 'F'), // F5
            (31, 'D'), // D5
            (40, 'L'), // L5
            (49, 'B'), // B5
        ];

        for (index, expected_color) in centers {
            let actual_color = self.facelets.chars().nth(index).unwrap();
            if actual_color != expected_color {
                return Err(format!(
                    "Center piece at position {} should be {}, but is {}",
                    index, expected_color, actual_color
                ));
            }
        }

        /*
        // Check corner pieces
        let corners = [
            (8, 9, 20, "URF"),
            (6, 18, 38, "UFL"),
            (0, 36, 46, "ULB"),
            (2, 45, 10, "UBR"),
            (30, 15, 26, "DRF"),
            (28, 24, 44, "DFL"),
            (34, 42, 52, "DLB"),
            (32, 51, 16, "DBR"),
        ];

        for (f1, f2, f3, name) in corners {
            let colors = [
                self.facelets.chars().nth(f1).unwrap(),
                self.facelets.chars().nth(f2).unwrap(),
                self.facelets.chars().nth(f3).unwrap(),
            ];

            // Check for duplicate colors
            if colors[0] == colors[1] || colors[1] == colors[2] || colors[0] == colors[2] {
                return Err(format!(
                    "Corner {} has duplicate colors: {}{}{}",
                    name, colors[0], colors[1], colors[2]
                ));
            }

            // Validate corner colors
            let valid_colors = match name {
                "URF" => ['U', 'R', 'F'],
                "UFL" => ['U', 'F', 'L'],
                "ULB" => ['U', 'L', 'B'],
                "UBR" => ['U', 'B', 'R'],
                "DRF" => ['D', 'R', 'F'],
                "DFL" => ['D', 'F', 'L'],
                "DLB" => ['D', 'L', 'B'],
                "DBR" => ['D', 'B', 'R'],
                _ => unreachable!(),
            };

            for color in colors {
                if !valid_colors.contains(&color) {
                    return Err(format!("Corner {} has invalid color: {}", name, color));
                }
            }
        }

        // Check edge pieces
        let edges = [
            (5, 11, "UR"),
            (7, 19, "UF"),
            (3, 37, "UL"),
            (1, 47, "UB"),
            (33, 17, "DR"),
            (29, 25, "DF"),
            (31, 43, "DL"),
            (35, 53, "DB"),
            (23, 12, "FR"),
            (21, 41, "FL"),
            (50, 39, "BL"),
            (48, 14, "BR"),
        ];

        for (f1, f2, name) in edges {
            let colors = [
                self.facelets.chars().nth(f1).unwrap(),
                self.facelets.chars().nth(f2).unwrap(),
            ];

            if colors[0] == colors[1] {
                return Err(format!(
                    "Edge {} has duplicate colors: {}{}",
                    name, colors[0], colors[1]
                ));
            }

            let valid_colors = match name {
                "UR" => ['U', 'R'],
                "UF" => ['U', 'F'],
                "UL" => ['U', 'L'],
                "UB" => ['U', 'B'],
                "DR" => ['D', 'R'],
                "DF" => ['D', 'F'],
                "DL" => ['D', 'L'],
                "DB" => ['D', 'B'],
                "FR" => ['F', 'R'],
                "FL" => ['F', 'L'],
                "BL" => ['B', 'L'],
                "BR" => ['B', 'R'],
                _ => unreachable!(),
            };

            for color in colors {
                if !valid_colors.contains(&color) {
                    return Err(format!("Edge {} has invalid color: {}", name, color));
                }
            }
        }
         */

        Ok(())
    }

    fn attempt_solve(&mut self) {
        // Try to solve with min2phase
        let solution = solve(&self.facelets, 21);

        if solution.starts_with("Error") {
            // Parse the error code and provide human-readable description
            if let Some(error) = Min2PhaseError::from_error_code(&solution) {
                let detailed_message = format!(
                    "{} (Error: {})\n\nExplanation: {}\n\nSuggestion: {}",
                    error.description(),
                    solution,
                    error.detailed_explanation(),
                    error.suggestions()
                );
                self.validation = CubeValidation::SolvingFailed(detailed_message);
            } else {
                // Fallback for unknown error codes
                self.validation =
                    CubeValidation::SolvingFailed(format!("Unknown min2phase error: {}", solution));
            }
        } else {
            self.validation = CubeValidation::Valid;
            self.solution = Some(solution);
        }
    }
}

impl Default for CubeState {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper for cube solver that implements Resource
#[derive(Resource, Debug, Clone, Default)]
pub struct CubeSolverResource {
    current_state: Option<CubeState>,
    is_solving: bool,
}

impl CubeSolverResource {
    pub fn update_from_entities(
        &mut self,
        all_faces_query: &Query<(Entity, &Face)>,
        colored_faces_query: &Query<(Entity, &RecoloredFace)>,
        small_cube_transforms: &Query<&GlobalTransform, With<crate::cube_moves::CubeMoveTarget>>,
        main_cube_transforms: &Query<&GlobalTransform, With<crate::components::RotatingModel>>,
        face_transforms: &Query<&GlobalTransform, With<Face>>,
    ) {
        let facelets = self.map_entities_to_facelets(
            all_faces_query,
            colored_faces_query,
            small_cube_transforms,
            main_cube_transforms,
            face_transforms,
        );

        if facelets.len() == CubeState::TOTAL_FACELETS {
            let new_state = CubeState::from_facelets(facelets);
            self.current_state = Some(new_state);
        } else {
            self.current_state = None;
        }

        // Reset solving state when cube state changes
        self.set_solving(false);
    }

    pub fn perform_lightweight_validation(&mut self) {
        // Perform lightweight validation without calling the heavy solve process
        // This is called on every recolor and rotation event for immediate feedback

        if let Some(state) = &mut self.current_state {
            // Use the lightweight-only validation method
            state.validate_lightweight_only();
        }
    }

    pub fn perform_full_solve(&mut self) -> bool {
        if let Some(state) = &mut self.current_state {
            log::info!("Performing full solve - redoing all validation from scratch");

            // Perform full validation (including solving attempt) from scratch
            state.validate();

            match state.validation() {
                CubeValidation::Valid => {
                    // Check if we have a solution
                    if state.solution().is_some() {
                        log::info!(
                            "Full solve successful - solution found with {} moves",
                            state.solution_moves().len()
                        );
                        self.set_solving(true);
                        true
                    } else {
                        log::warn!("Cube is valid but no solution was found");
                        false
                    }
                }
                CubeValidation::Invalid(msg) => {
                    log::warn!("Cannot solve invalid cube: {}", msg);
                    false
                }
                CubeValidation::SolvingFailed(msg) => {
                    log::warn!("Solving failed: {}", msg);
                    false
                }
                CubeValidation::NotValidated => {
                    log::warn!("Cube not yet validated");
                    false
                }
            }
        } else {
            log::warn!("No cube state available");
            false
        }
    }

    pub fn get_validation_message(&self) -> String {
        match &self.current_state {
            Some(state) => match state.validation() {
                CubeValidation::NotValidated => "Cube not yet validated".to_string(),
                CubeValidation::Valid => {
                    if let Some(solution) = state.solution() {
                        let move_count = solution.split_whitespace().count();
                        format!("Valid cube, solvable in {} moves", move_count)
                    } else {
                        "Valid cube (press Solve to find solution)".to_string()
                    }
                }
                CubeValidation::Invalid(msg) => format!("Invalid: {}", msg),
                CubeValidation::SolvingFailed(msg) => format!("Solving failed: {}", msg),
            },
            None => "No cube state available".to_string(),
        }
    }

    pub fn is_solvable(&self) -> bool {
        // Check if cube is valid AND has a solution
        if let Some(state) = &self.current_state {
            matches!(state.validation(), CubeValidation::Valid) && state.solution().is_some()
        } else {
            false
        }
    }

    pub fn is_valid(&self) -> bool {
        // Check if cube passes lightweight validation
        matches!(
            self.current_state.as_ref().map(|s| s.validation()),
            Some(CubeValidation::Valid)
        )
    }

    pub fn is_solving(&self) -> bool {
        self.is_solving
    }

    pub fn set_solving(&mut self, solving: bool) {
        self.is_solving = solving;
    }

    pub fn clear_solution(&mut self) {
        self.current_state = None;
        self.is_solving = false;
    }

    pub fn solve_moves(&self) -> Vec<String> {
        self.current_state
            .as_ref()
            .map(|s| s.solution_moves())
            .unwrap_or_default()
    }

    pub fn facelets(&self) -> Option<&str> {
        self.current_state.as_ref().map(|s| s.facelets())
    }

    /// Solve a Rubik's cube represented in facelet
    /// Facelet for the rubik's cube:
    /// ```text
    ///          +--------+
    ///          |U1 U2 U3|
    ///          |U4 U5 U6|
    ///          |U7 U8 U9|
    /// +--------+--------+--------+--------+
    /// |L1 L2 L3|F1 F2 F3|R1 R2 R3|B1 B2 B3|
    /// |L4 L5 L6|F4 F5 F6|R4 R5 R6|B4 B5 B6|
    /// |L7 L8 L9|F7 F8 F9|R7 R8 R9|B7 B8 B9|
    /// +--------+--------+--------+--------+
    ///          |D1 D2 D3|
    ///          |D4 D5 D6|
    ///          |D7 D8 D9|
    ///          +--------+
    /// ```
    /// should be: U1U2...U9R1R2...R9F1..F9D1..D9L1..L9B1..B9
    /// Example, facelet of solved cube is UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
    /// Map cube face entities to min2phase facelet string
    fn map_entities_to_facelets(
        &self,
        all_faces_query: &Query<(Entity, &Face)>,
        colored_faces_query: &Query<(Entity, &RecoloredFace)>,
        small_cube_transforms: &Query<&GlobalTransform, With<crate::cube_moves::CubeMoveTarget>>,
        main_cube_transforms: &Query<&GlobalTransform, With<crate::components::RotatingModel>>,
        face_transforms: &Query<&GlobalTransform, With<Face>>,
    ) -> String {
        // Initialize facelets array with spaces
        let mut facelets = vec![' '; CubeState::TOTAL_FACELETS];

        // Create a map of entity IDs to their color indices for quick lookup
        let entity_colors: HashMap<Entity, usize> = colored_faces_query
            .iter()
            .map(|(entity, recolored_face)| (entity, recolored_face.color_index))
            .collect();

        // Map each entity to its position in the cube state
        for (entity, _face) in all_faces_query.iter() {
            // Calculate facelet letter based on entity color
            if let Some(&color_index) = entity_colors.get(&entity) {
                let face_color = FaceColor::from_index(color_index);
                let facelet_char = face_color.to_facelet_char();

                // Calculate facelet index based on parent small cube position
                if let Some(facelet_index) = calculate_facelet_index(
                    entity,
                    all_faces_query,
                    small_cube_transforms,
                    main_cube_transforms,
                    face_transforms,
                ) && facelet_index < facelets.len()
                {
                    facelets[facelet_index] = facelet_char;
                }
            }
        }

        let facelet_string = facelets.iter().collect::<String>();
        log::debug!(
            "Current facelet state: {}",
            facelet_string.replace(" ", ".")
        );

        // Remap facelets based on center face orientations
        let remapped_facelet_string = remap_facelets_by_centers(&facelet_string);
        log::debug!(
            "Remapped facelet state: {}",
            remapped_facelet_string.replace(" ", ".")
        );
        remapped_facelet_string
    }
}

/// Remap facelets based on center face orientations
/// Maps current center faces to default center faces and applies the mapping to all facelets
fn remap_facelets_by_centers(facelet_string: &str) -> String {
    if facelet_string.len() != 54 {
        log::warn!("Facelet string length is not 54, skipping remapping");
        return facelet_string.to_string();
    }

    // Extract current center facelets
    let mut current_centers = [' '; 6];
    for (i, &index) in CENTER_FACELET_INDICES.iter().enumerate() {
        if index < facelet_string.len() {
            current_centers[i] = facelet_string.chars().nth(index).unwrap_or(' ');
        }
    }

    // Create mapping from current centers to default centers
    let mut color_mapping = HashMap::new();
    for (i, &current_center) in current_centers.iter().enumerate() {
        if current_center != ' ' {
            color_mapping.insert(current_center, DEFAULT_CENTER_FACES[i]);
        }
    }

    // Apply mapping to all facelets
    let mut remapped_facelets = String::new();
    for c in facelet_string.chars() {
        if c == ' ' {
            remapped_facelets.push(c);
        } else {
            let mapped_char = color_mapping.get(&c).unwrap_or(&c);
            remapped_facelets.push(*mapped_char);
        }
    }

    log::debug!(
        "Facelet remapping: current centers {:?} -> default centers {:?}",
        current_centers.iter().collect::<String>(),
        DEFAULT_CENTER_FACES.iter().collect::<String>()
    );

    remapped_facelets
}

/// Calculate facelet index based on face's parent small cube position
fn calculate_facelet_index(
    face_entity: Entity,
    face_query: &Query<(Entity, &Face)>,
    small_cube_transforms: &Query<&GlobalTransform, With<crate::cube_moves::CubeMoveTarget>>,
    main_cube_transforms: &Query<&GlobalTransform, With<crate::components::RotatingModel>>,
    face_transforms: &Query<&GlobalTransform, With<Face>>,
) -> Option<usize> {
    // Get the parent small cube entity from the Face component
    if let Ok((_, face)) = face_query.get(face_entity) {
        let small_cube_entity = face.parent_cube;

        // Get the small cube's transform
        if let Ok(small_cube_transform) = small_cube_transforms.get(small_cube_entity) {
            // Get the main cube's transform
            if let Ok(main_cube_transform) = main_cube_transforms.get_single() {
                // Get the face's transform
                if let Ok(face_transform) = face_transforms.get(face_entity) {
                    // Calculate the face's position relative to the main cube
                    // This gives us the face's orientation in the main cube's coordinate system
                    let face_relative_to_main =
                        main_cube_transform.affine().inverse() * face_transform.affine();
                    let (_, _, face_main_pos) =
                        face_relative_to_main.to_scale_rotation_translation();

                    // Determine face orientation from face's position in main cube coordinates
                    let face_orientation =
                        determine_face_orientation_from_main_position(&face_main_pos);

                    // Calculate the small cube's position relative to the main cube in LOCAL SPACE
                    // This gives us the original grid position regardless of rotation
                    let relative_transform =
                        main_cube_transform.affine().inverse() * small_cube_transform.affine();
                    let (_, _, relative_position) =
                        relative_transform.to_scale_rotation_translation();

                    // Convert the relative position to local indices (-1, 0, 1)
                    let local_indices = world_to_local_indices(&relative_position);

                    // Calculate position within the face using local indices
                    let position_in_face =
                        calculate_position_in_face_from_indices(&local_indices, face_orientation);

                    // Calculate facelet index: group_offset + position_within_face
                    let group_offset = face_orientation.facelet_offset();
                    let facelet_index = group_offset + position_in_face;

                    log::debug!(
                        "Facelet mapping: face={:?} -> indices=({:.0},{:.0},{:.0}) -> {}{} -> index={}",
                        face_entity,
                        local_indices.x,
                        local_indices.y,
                        local_indices.z,
                        face_orientation.as_str(),
                        position_in_face + 1,
                        facelet_index
                    );

                    return Some(facelet_index);
                }
            }
        }
    }

    // If we can't calculate the index, return None (face will be treated as not colored)
    None
}

/// Convert world position to local indices (-1, 0, 1)
fn world_to_local_indices(position: &Vec3) -> Vec3 {
    // Grid step is 2.0/3.0 (from cube creation)
    const GRID_STEP: f32 = 2.0 / 3.0;

    // Convert to local indices by dividing by grid step and rounding
    let x = (position.x / GRID_STEP).round() as i32;
    let y = (position.y / GRID_STEP).round() as i32;
    let z = (position.z / GRID_STEP).round() as i32;

    // Clamp to valid range (-1, 0, 1)
    let x = x.clamp(-1, 1);
    let y = y.clamp(-1, 1);
    let z = z.clamp(-1, 1);

    Vec3::new(x as f32, y as f32, z as f32)
}

/// Determine face orientation from face's position in main cube coordinates
fn determine_face_orientation_from_main_position(face_main_pos: &Vec3) -> Orientation {
    // Determine which axis the face is on and in which direction
    // This matches the face spawning logic in cube.rs
    if face_main_pos.x.abs() > face_main_pos.y.abs()
        && face_main_pos.x.abs() > face_main_pos.z.abs()
    {
        if face_main_pos.x > 0.0 {
            Orientation::Right
        } else {
            Orientation::Left
        }
    } else if face_main_pos.y.abs() > face_main_pos.z.abs() {
        if face_main_pos.y > 0.0 {
            Orientation::Up
        } else {
            Orientation::Down
        }
    } else if face_main_pos.z > 0.0 {
        Orientation::Front
    } else {
        Orientation::Back
    }
}

/// Calculate position within a face using local indices
fn calculate_position_in_face_from_indices(indices: &Vec3, face_orientation: Orientation) -> usize {
    // For each face, we need to map the other two coordinates to a 3x3 grid
    // The grid layout is:
    // 0 1 2
    // 3 4 5
    // 6 7 8

    let (grid_x, grid_y) = match face_orientation {
        // Front face: use X and Y coordinates
        Orientation::Front => {
            let x = (indices.x + 1.0) as usize;
            let y = (-indices.y + 1.0) as usize; // Elegant Y inversion
            (x, y)
        }
        // Back face: use X and Y coordinates, but invert X
        Orientation::Back => {
            let x = (-indices.x + 1.0) as usize; // Invert X for Back face
            let y = (-indices.y + 1.0) as usize; // Elegant Y inversion
            (x, y)
        }
        // Left face: use Z and Y coordinates
        Orientation::Left => {
            let z = (indices.z + 1.0) as usize;
            let y = (-indices.y + 1.0) as usize; // Elegant Y inversion
            (z, y)
        }
        // Right face: use Z and Y coordinates, but invert Z
        Orientation::Right => {
            let z = (-indices.z + 1.0) as usize; // Invert Z for Right face
            let y = (-indices.y + 1.0) as usize; // Elegant Y inversion
            (z, y)
        }
        // Up face: use X and Z coordinates
        Orientation::Up => {
            let x = (indices.x + 1.0) as usize;
            let z = (indices.z + 1.0) as usize;
            (x, z)
        }
        // Down face: use X and Z coordinates, but invert Z
        Orientation::Down => {
            let x = (indices.x + 1.0) as usize;
            let z = (-indices.z + 1.0) as usize; // Invert Z to flip rows
            (x, z)
        }
    };

    // Convert to linear index (0-8)

    grid_y * 3 + grid_x
}

/// System to update solver state when cube faces change
pub fn update_solver_state(
    mut solver: ResMut<CubeSolverResource>,
    face_query: Query<(&RecoloredFace, &Face), Changed<RecoloredFace>>,
    all_faces_query: Query<(Entity, &Face)>,
    colored_faces_query: Query<(Entity, &RecoloredFace)>,
    small_cube_transforms: Query<&GlobalTransform, With<crate::cube_moves::CubeMoveTarget>>,
    main_cube_transforms: Query<&GlobalTransform, With<crate::components::RotatingModel>>,
    face_transforms: Query<&GlobalTransform, With<Face>>,
) {
    // Only update if there are changes
    if !face_query.is_empty() {
        solver.update_from_entities(
            &all_faces_query,
            &colored_faces_query,
            &small_cube_transforms,
            &main_cube_transforms,
            &face_transforms,
        );

        log::info!("Cube solver updated: {}", solver.get_validation_message());

        if solver.is_solvable() {
            log::info!("Generated {} solving moves", solver.solve_moves().len());
        }
    }
}

/// System to perform lightweight validation on recolor events
pub fn lightweight_validation_on_recolor(
    mut solver: ResMut<CubeSolverResource>,
    face_query: Query<(&RecoloredFace, &Face), Changed<RecoloredFace>>,
) {
    // Only perform validation if there are recolor events
    if !face_query.is_empty() {
        solver.perform_lightweight_validation();

        log::debug!(
            "Lightweight validation on recolor: valid={}, message={}",
            solver.is_solvable(),
            solver.get_validation_message()
        );
    }
}

/// System to perform lightweight validation on rotation completion events
pub fn lightweight_validation_on_rotation_complete(
    mut solver: ResMut<CubeSolverResource>,
    mut rotation_completed_events: EventReader<LayerRotationCompletedEvent>,
    all_faces_query: Query<(Entity, &Face)>,
    colored_faces_query: Query<(Entity, &RecoloredFace)>,
    small_cube_transforms: Query<&GlobalTransform, With<crate::cube_moves::CubeMoveTarget>>,
    main_cube_transforms: Query<&GlobalTransform, With<crate::components::RotatingModel>>,
    face_transforms: Query<&GlobalTransform, With<Face>>,
) {
    // Only perform validation if there are rotation completion events
    if !rotation_completed_events.is_empty() {
        // Clear the events to avoid processing them multiple times
        rotation_completed_events.clear();

        // Update solver state with current entity mappings (this calls map_entities_to_facelets)
        solver.update_from_entities(
            &all_faces_query,
            &colored_faces_query,
            &small_cube_transforms,
            &main_cube_transforms,
            &face_transforms,
        );

        log::info!(
            "Cube solver updated after rotation: {}",
            solver.get_validation_message()
        );

        if solver.is_solvable() {
            log::info!(
                "Generated {} solving moves after rotation",
                solver.solve_moves().len()
            );
        }
    }
}
