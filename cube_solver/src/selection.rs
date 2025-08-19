use crate::colors::CubeColors;
use crate::ray_caster::RayCaster;
use bevy::prelude::*;

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Selectable {
    pub id: Option<String>,
    pub priority: f32,
    pub enabled: bool,
}

impl Default for Selectable {
    fn default() -> Self {
        Self {
            id: None,
            priority: 0.0,
            enabled: true,
        }
    }
}

impl Selectable {
    pub fn new(priority: f32) -> Self {
        Self {
            priority,
            enabled: true,
            ..Default::default()
        }
    }

    pub fn with_id(priority: f32, id: impl Into<String>) -> Self {
        Self {
            id: Some(id.into()),
            priority,
            enabled: true,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Selected {
    pub selection_type: SelectionType,
    pub selected_at: f64,
}

impl Selected {
    pub fn new(selection_type: SelectionType, timestamp: f64) -> Self {
        Self {
            selection_type,
            selected_at: timestamp,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum SelectionType {
    ColorPanel,
    CubeFace,
}

#[derive(Event, Debug, Clone, PartialEq)]
pub enum SelectionEvent {
    EntitySelected {
        entity: Entity,
        selection_type: SelectionType,
        position: Vec3,
    },
    EntityDeselected {
        entity: Entity,
        selection_type: SelectionType,
    },
    ColorSelected {
        color_index: usize,
        entity: Entity,
    },
    ColorApplied {
        face_entity: Entity,
        color_index: usize,
    },
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct SelectionState {
    pub selected_color_entity: Option<Entity>,
    pub selected_cube_faces: Vec<Entity>,
    pub last_selection_time: f64,
    pub multi_select_enabled: bool,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            selected_color_entity: None,
            selected_cube_faces: Vec::new(),
            last_selection_time: 0.0,
            multi_select_enabled: false,
        }
    }
}

impl SelectionState {
    pub fn clear_all(&mut self) -> (Option<Entity>, Vec<Entity>) {
        let prev_color = self.selected_color_entity.take();
        let prev_faces = std::mem::take(&mut self.selected_cube_faces);
        (prev_color, prev_faces)
    }

    pub fn set_color_selection(&mut self, entity: Entity, timestamp: f64) -> Option<Entity> {
        self.last_selection_time = timestamp;
        self.selected_color_entity.replace(entity)
    }

    pub fn add_cube_face(&mut self, entity: Entity, timestamp: f64) {
        self.last_selection_time = timestamp;
        if !self.multi_select_enabled {
            self.selected_cube_faces.clear();
        }
        if !self.selected_cube_faces.contains(&entity) {
            self.selected_cube_faces.push(entity);
        }
    }

    pub fn remove_cube_face(&mut self, entity: Entity) -> bool {
        if let Some(pos) = self.selected_cube_faces.iter().position(|&e| e == entity) {
            self.selected_cube_faces.remove(pos);
            true
        } else {
            false
        }
    }
}

/// System to detect touch input and cast rays for selection.
///
/// This system handles the input detection phase of selection, converting
/// touch coordinates to world-space rays and performing intersection tests.
pub fn detect_touch_selection(
    touches: Res<Touches>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window>,
    selectable_query: Query<(Entity, &GlobalTransform, &Selectable)>,
    mut selection_events: EventWriter<SelectionEvent>,
    mut touch_state: ResMut<crate::components::TouchState>,
    // Query to check if any UI elements are being interacted with
    ui_interaction_query: Query<&Interaction, With<Button>>,
) {
    // Check if any UI element is currently being interacted with
    let ui_is_active = ui_interaction_query
        .iter()
        .any(|interaction| matches!(interaction, Interaction::Pressed | Interaction::Hovered));

    // Only process touch input for selection if no UI elements are active
    if !ui_is_active {
        // Process new touches by starting a pending selection
        for touch in touches.iter() {
            if touches.just_pressed(touch.id()) {
                let touch_pos = touch.position();
                // Don't immediately select - start a pending selection instead
                touch_state.start_pending_selection(touch_pos);
            }
        }

        // Process pending selections if they should trigger
        if let Some(pending_pos) = touch_state.consume_pending_selection() {
            // Get camera and window - early return on failure
            let Ok((_camera, camera_transform)) = camera_query.get_single() else {
                warn!("No camera found for ray casting");
                return;
            };
            let Ok(window) = window_query.get_single() else {
                warn!("No window found for ray casting");
                return;
            };

            // Create ray from screen coordinates
            let Some(ray) = RayCaster::screen_to_world_ray(pending_pos, camera_transform, window)
            else {
                warn!(
                    "Failed to create ray from screen coordinates at {:?}",
                    pending_pos
                );
                return;
            };

            debug!(
                "Created ray: origin={:?}, direction={:?}, screen_pos={:?}",
                ray.origin, ray.direction, pending_pos
            );

            // Cast ray and get sorted hits
            let hits = RayCaster::cast_ray(&ray, &selectable_query);

            // Process the best hit
            if let Some(hit) = hits.first() {
                selection_events.send(SelectionEvent::EntitySelected {
                    entity: hit.entity,
                    selection_type: SelectionType::ColorPanel, // Will be refined in handler
                    position: hit.point,
                });

                debug!(
                    "Ray hit entity {:?} at distance {:.2} (priority: {:.1})",
                    hit.entity, hit.distance, hit.priority
                );
            } else {
                debug!("Ray cast found no selectable objects");
            }
        }
    } else {
        // UI is active, clear any pending selections to prevent selection when UI is touched
        if touch_state.pending_selection_pos.is_some() {
            touch_state.pending_selection_pos = None;
            touch_state.pending_selection_timer = 0.0;
            debug!("Cleared pending selection due to UI interaction");
        }
    }
}

/// System to handle selection events and update selection state.
///
/// This system processes selection events and determines the appropriate
/// selection type based on the entity's components.
pub fn handle_selection_events(
    mut selection_events: EventReader<SelectionEvent>,
    mut selection_state: ResMut<SelectionState>,
    mut touch_state: ResMut<crate::components::TouchState>,
    color_query: Query<&crate::components::ColorSquare>,
    name_query: Query<&Name>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for event in selection_events.read() {
        match event {
            SelectionEvent::EntitySelected { entity, .. } => {
                let timestamp = time.elapsed_secs_f64();
                let entity_name = name_query
                    .get(*entity)
                    .map(|n| n.as_str())
                    .unwrap_or("Unknown");

                // Determine selection type based on entity components
                if let Ok(color_square) = color_query.get(*entity) {
                    // This is a color panel selection
                    info!(
                        "Selected color panel: {} (index {})",
                        entity_name, color_square.color_index
                    );
                    handle_color_panel_selection(
                        *entity,
                        color_square,
                        &mut selection_state,
                        &mut touch_state,
                        &mut commands,
                        timestamp,
                    );
                } else {
                    // This is a cube face selection
                    info!("Selected cube face: {}", entity_name);
                    handle_cube_face_selection(
                        *entity,
                        &mut selection_state,
                        &touch_state,
                        &mut commands,
                        timestamp,
                    );
                }
            }
            _ => {} // Handle other event types as needed
        }
    }
}

/// Handles color panel selection logic.
fn handle_color_panel_selection(
    entity: Entity,
    color_square: &crate::components::ColorSquare,
    selection_state: &mut SelectionState,
    touch_state: &mut crate::components::TouchState,
    commands: &mut Commands,
    timestamp: f64,
) {
    // Clear previous color panel selection
    if let Some(prev_entity) = selection_state.selected_color_entity {
        commands.entity(prev_entity).remove::<Selected>();
    }

    // Select the new color square
    commands
        .entity(entity)
        .insert(Selected::new(SelectionType::ColorPanel, timestamp));
    selection_state.set_color_selection(entity, timestamp);
    touch_state.set_selected_color(color_square.color_index);

    info!("Selected color panel: index {}", color_square.color_index);
}

/// Handles cube face selection logic.
fn handle_cube_face_selection(
    entity: Entity,
    selection_state: &mut SelectionState,
    touch_state: &crate::components::TouchState,
    commands: &mut Commands,
    timestamp: f64,
) {
    // Don't allow cube face selection if we're currently rotating
    if touch_state.is_rotating {
        debug!("Ignoring cube face selection during rotation");
        return;
    }

    // Only select cube faces if a color is selected
    if touch_state.selected_color.is_some() {
        // Clear previous cube face selections if multi-select is disabled
        if !selection_state.multi_select_enabled {
            for face_entity in &selection_state.selected_cube_faces {
                commands.entity(*face_entity).remove::<Selected>();
            }
        }

        // Select the cube face
        commands
            .entity(entity)
            .insert(Selected::new(SelectionType::CubeFace, timestamp));
        selection_state.add_cube_face(entity, timestamp);

        info!("Colored cube face: {:?}", entity);
    } else {
        warn!("Cannot color cube face: no color selected");
    }
}

/// System to update visual selection borders based on color selection.
///
/// This system manages the visibility of selection borders around color squares
/// to provide visual feedback for the currently selected color.
pub fn update_selection_borders(
    selection_state: Res<SelectionState>,
    touch_state: Res<crate::components::TouchState>,
    mut border_query: Query<(&mut Visibility, &crate::components::SelectionBorder)>,
) {
    // Only update if there's been a change
    if !selection_state.is_changed() && !touch_state.is_changed() {
        return;
    }

    let selected_color_index = touch_state.selected_color;

    // Update all borders based on current selection
    for (mut visibility, border) in border_query.iter_mut() {
        *visibility = if Some(border.color_index) == selected_color_index {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// System to apply colors to selected cube faces when a color is chosen.
///
/// This system listens for color application events and updates the
/// material and components of cube faces accordingly.
pub fn apply_color_to_selected_faces(
    mut commands: Commands,
    selected_cube_faces: Query<Entity, (With<Selected>, Without<crate::components::ColorSquare>)>,
    recolored_faces_query: Query<&crate::components::RecoloredFace>,
    cube_colors: Res<CubeColors>,
    placeholder_material: Res<crate::colors::PlaceholderMaterial>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut color_manager: ResMut<crate::components::ColorManager>,
    time: Res<Time>,
    mut color_events: EventWriter<SelectionEvent>,
) {
    let Some(selected_color_index) = color_manager.get_selected_color() else {
        return;
    };

    if selected_color_index >= cube_colors.len() {
        warn!(
            "Selected color index {} is out of bounds",
            selected_color_index
        );
        return;
    }

    let selected_color = cube_colors.get(selected_color_index);
    let timestamp = time.elapsed_secs_f64();

    for entity in selected_cube_faces.iter() {
        // Get previous color if any
        let previous_color = recolored_faces_query
            .get(entity)
            .map(|face| face.color_index)
            .ok();

        // Check if we're decoloring (same color as selected)
        if let Some(prev_color) = previous_color
            && prev_color == selected_color_index
        {
            // Decolor the face - return to placeholder color
            commands
                .entity(entity)
                .insert(MeshMaterial3d(placeholder_material.0.clone()))
                .remove::<crate::components::RecoloredFace>()
                .remove::<Selected>();

            // Decrement the color count
            color_manager.decrement_color(selected_color_index);

            // Emit color removal event
            color_events.send(SelectionEvent::ColorApplied {
                face_entity: entity,
                color_index: selected_color_index, // Still send the color index for consistency
            });

            info!(
                "Decolored cube face {:?} from color {}, count now: {}",
                entity,
                selected_color_index,
                color_manager.get_usage_info(selected_color_index)
            );
            continue;
        }

        // Try to apply the color using the centralized manager
        match color_manager.apply_color_to_face(selected_color_index, previous_color) {
            Ok(reached_limit) => {
                // Create new material with selected color
                let material = create_face_material(selected_color, &mut materials);

                // Update the entity with new material and components
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(material))
                    .insert(crate::components::RecoloredFace::new(
                        selected_color_index,
                        timestamp,
                    ))
                    .remove::<Selected>(); // Remove selection after applying color

                if reached_limit {
                    info!("Color {} has reached its limit!", selected_color_index);
                }

                // Emit color application event
                color_events.send(SelectionEvent::ColorApplied {
                    face_entity: entity,
                    color_index: selected_color_index,
                });

                info!(
                    "Applied color {} to cube face {:?}, count now: {}",
                    selected_color_index,
                    entity,
                    color_manager.get_usage_info(selected_color_index)
                );
            }
            Err(err) => {
                warn!("Failed to apply color {}: {}", selected_color_index, err);
                commands.entity(entity).remove::<Selected>();
            }
        }
    }
}

/// Creates a material for a cube face with the specified color.
///
/// This function creates a PBR material with appropriate properties
/// for Rubik's cube faces, including emissive lighting for better visibility.
fn create_face_material(
    base_color: Color,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Handle<StandardMaterial> {
    let linear_color = base_color.to_linear();
    let emissive_color = bevy::color::LinearRgba::new(
        linear_color.red * 0.3,
        linear_color.green * 0.3,
        linear_color.blue * 0.3,
        linear_color.alpha,
    );

    materials.add(StandardMaterial {
        base_color,
        emissive: emissive_color,
        metallic: 0.3,
        perceptual_roughness: 0.8,
        ..default()
    })
}

/// System to initialize default selection state on startup.
///
/// This system runs once to set up the initial color selection and
/// ensure the UI reflects the default state properly.
pub fn initialize_default_selection(
    mut touch_state: ResMut<crate::components::TouchState>,
    mut selection_state: ResMut<SelectionState>,
    mut has_initialized: Local<bool>,
) {
    // Only run once
    if *has_initialized {
        return;
    }

    // Ensure default color is selected
    if touch_state.selected_color.is_none() {
        touch_state.set_selected_color(0); // Default to white
    }

    // Initialize selection state
    selection_state.last_selection_time = 0.0;

    *has_initialized = true;

    if let Some(color_index) = touch_state.selected_color {
        info!("Initialized default color selection: {}", color_index);
    }
}

/// Enhanced plugin providing modular selection functionality.
///
/// This plugin sets up all the systems, resources, and events needed
/// for ray-casting based selection with proper separation of concerns.
pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register resources
            .init_resource::<SelectionState>()
            .init_resource::<CubeColors>()
            .init_resource::<crate::components::TouchState>()
            // Register events
            .add_event::<SelectionEvent>()
            // Register reflection types for debugging
            .register_type::<Selectable>()
            .register_type::<Selected>()
            .register_type::<SelectionType>()
            .register_type::<SelectionState>()
            .register_type::<CubeColors>()
            .register_type::<crate::components::TouchState>()
            .register_type::<crate::components::ColorSquare>()
            .register_type::<crate::components::SelectionBorder>()
            .register_type::<crate::components::RecoloredFace>()
            // Add systems with proper scheduling
            .add_systems(Startup, initialize_default_selection)
            .add_systems(
                Update,
                (
                    // Input and detection phase
                    detect_touch_selection,
                    // Event processing phase
                    handle_selection_events,
                    // Visual feedback phase
                    update_selection_borders,
                    // Action phase
                    apply_color_to_selected_faces,
                )
                    .chain() // Ensure proper execution order
                    .run_if(any_with_component::<Selectable>), // Only run if there are selectable entities
            );
    }
}
