use super::rotations_panel::MoveQueue;
use crate::layer_rotation::parse_extended_move_notation;
use bevy::color::palettes::css;
use bevy::prelude::*;

#[derive(Component)]
pub struct MoveTestPanel;

#[derive(Component)]
pub struct OpenMoveSelectionButton;

#[derive(Component)]
pub struct RstButton;

#[derive(Component)]
pub struct FixButton;

#[derive(Component)]
pub struct ClrButton;

#[derive(Component)]
pub struct MoveSelectionPanel;

#[derive(Component)]
pub struct MoveSelectionOverlay;

#[derive(Component)]
pub struct MoveSelectionButton {
    pub move_notation: String,
}

#[derive(Component)]
pub struct BackspaceButton;

#[derive(Resource, Default)]
pub struct MoveSelectionState {
    pub is_open: bool,
}

/// Creates a move testing UI panel above the solve button
pub fn create_move_test_panel(mut commands: Commands) {
    info!("Creating move test panel");

    // Create a parent node for the entire move test UI
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                ..default()
            },
            Name::new("Move Test UI Container"),
        ))
        .with_children(|container_parent| {
            // Overlay to block touch events when move selection panel is open
            container_parent.spawn((
                Button, // Add Button component to ensure it's detected as an active UI element
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(Color::from(css::BLACK).with_alpha(0.01)),
                Name::new("Move Selection Overlay"),
                MoveSelectionOverlay,
                Visibility::Hidden, // Hidden by default
            ));

            // Move selection panel (hidden by default)
            container_parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(220.0), // Higher up to color buttons
                        left: Val::Px(50.0),
                        right: Val::Px(50.0),
                        height: Val::Px(460.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        overflow: Overflow::clip(), // Enable clipping for scrolling
                        ..default()
                    },
                    BackgroundColor(Color::from(css::SLATE_GRAY).with_alpha(0.95)),
                    MoveSelectionPanel,
                    Name::new("Move Selection Panel"),
                    Visibility::Hidden, // Hidden by default
                ))
                .with_children(|panel_parent| {
                    // Title
                    panel_parent.spawn((
                        Text::new("Select"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(css::WHITE.into()),
                        Node {
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        },
                    ));

                    // Scrollable grid of face move buttons
                    panel_parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(380.0), // Taller to show more rows
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                justify_content: JustifyContent::Center,
                                overflow: Overflow::scroll_y(), // Enable vertical scrolling
                                ..default()
                            },
                            Name::new("Move Buttons Grid"),
                        ))
                        .with_children(|grid_parent| {
                            // Create buttons for all move variants
                            let moves = [
                                // Face moves (including 2-turns)
                                "F", "F'", "F2", "B", "B'", "B2", "R", "R'", "R2", "L", "L'", "L2",
                                "U", "U'", "U2", "D", "D'", "D2", // Middle layer moves
                                "M", "M'", "M2", "E", "E'", "E2", "S", "S'", "S2",
                            ];
                            for move_name in &moves {
                                grid_parent
                                    .spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(50.0),
                                            height: Val::Px(40.0),
                                            margin: UiRect::all(Val::Px(5.0)),
                                            border: UiRect::all(Val::Px(1.0)),
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        },
                                        BackgroundColor(css::DIM_GRAY.into()),
                                        BorderColor(css::WHITE.into()),
                                        Name::new(format!("{} Selection Button", move_name)),
                                        MoveSelectionButton {
                                            move_notation: move_name.to_string(),
                                        },
                                    ))
                                    .with_children(|button_parent| {
                                        button_parent.spawn((
                                            Text::new(move_name.to_string()),
                                            TextFont {
                                                font_size: 16.0,
                                                ..default()
                                            },
                                            TextColor(css::WHITE.into()),
                                        ));
                                    });
                            }

                            // Clear rotations button (last in the grid)
                            grid_parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(50.0),
                                        height: Val::Px(40.0),
                                        margin: UiRect::all(Val::Px(5.0)),
                                        border: UiRect::all(Val::Px(1.0)),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    BackgroundColor(css::RED.into()),
                                    BorderColor(css::WHITE.into()),
                                    BackspaceButton,
                                    Name::new("Backspace Button"),
                                ))
                                .with_children(|button_parent| {
                                    button_parent.spawn((
                                        Text::new("<="),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(css::WHITE.into()),
                                    ));
                                });
                        });
                });

            // Move test panel
            container_parent
                .spawn((
                    Node {
                        position_type: PositionType::Relative,
                        bottom: Val::Px(150.0),
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        height: Val::Px(60.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.0),
                        ..default()
                    },
                    BackgroundColor(Color::from(css::SLATE_GRAY).with_alpha(0.9)),
                    MoveTestPanel,
                    Name::new("Move Test Panel"),
                ))
                .with_children(|parent| {
                    // Rst button (renamed from Reset, on the left)
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(50.0),
                                height: Val::Px(40.0),
                                border: UiRect::all(Val::Px(2.0)),
                                margin: UiRect::left(Val::Px(10.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::from(css::PURPLE).with_alpha(0.8)),
                            BorderColor(css::WHITE.into()),
                            RstButton,
                            Name::new("Reset Button"),
                        ))
                        .with_children(|button_parent| {
                            button_parent.spawn((
                                Text::new("®"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(css::WHITE.into()),
                            ));
                        });

                    // Fix button (middle)
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(50.0),
                                height: Val::Px(40.0),
                                border: UiRect::all(Val::Px(2.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::from(css::ORANGE).with_alpha(0.8)),
                            BorderColor(css::WHITE.into()),
                            FixButton,
                            Name::new("Fix Button"),
                        ))
                        .with_children(|button_parent| {
                            button_parent.spawn((
                                Text::new("¥"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(css::WHITE.into()),
                            ));
                        });

                    // Clr button (new, clears colors)
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(50.0),
                                height: Val::Px(40.0),
                                border: UiRect::all(Val::Px(2.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::from(css::RED).with_alpha(0.8)),
                            BorderColor(css::WHITE.into()),
                            ClrButton,
                            Name::new("Clear Button"),
                        ))
                        .with_children(|button_parent| {
                            button_parent.spawn((
                                Text::new("©"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(css::WHITE.into()),
                            ));
                        });

                    // Button to open move selection panel
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(40.0),
                                border: UiRect::all(Val::Px(2.0)),
                                margin: UiRect::right(Val::Px(10.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::from(css::BLACK).with_alpha(0.8)),
                            BorderColor(css::WHITE.into()),
                            OpenMoveSelectionButton,
                            Name::new("Move Selection Button"),
                        ))
                        .with_children(|button_parent| {
                            button_parent.spawn((
                                Text::new("Ω"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(css::WHITE.into()),
                                Node {
                                    margin: UiRect::all(Val::Px(5.0)),
                                    ..default()
                                },
                            ));
                        });
                });
        });

    info!("Move test panel created");
}

/// Plugin for the move test UI functionality.
///
/// This plugin sets up all the systems, resources, and events needed
/// for the move test UI with proper separation of concerns.
pub struct MoveTestPlugin;

impl Plugin for MoveTestPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register resources
            .init_resource::<MoveSelectionState>()
            // MoveQueue is part of production plugin now
            // Add systems with proper scheduling
            .add_systems(Startup, create_move_test_panel)
            .add_systems(
                Update,
                (
                    handle_select_button,
                    handle_rst_button,
                    handle_fix_button,
                    handle_clr_button,
                    handle_move_selection,
                    handle_backspace_button,
                    update_move_selection_state,
                    handle_move_completion,
                    close_move_selection_on_button_press,
                ),
            );
    }
}

/// System to handle Rst button clicks (resets only position)
pub fn handle_rst_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<RstButton>)>,
    mut cube_transform_query: Query<&mut Transform, With<crate::components::RotatingModel>>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            info!("Rst button clicked - resetting cube position only");

            // Reset cube transform to initial state
            if let Ok(mut cube_transform) = cube_transform_query.get_single_mut() {
                cube_transform.translation = Vec3::ZERO;
                cube_transform.rotation = Quat::IDENTITY;
                cube_transform.scale = Vec3::splat(1.0);
            }

            info!("Cube position reset to initial state");
        }
    }
}

/// System to handle Clr button clicks (clears all colors)
pub fn handle_clr_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<ClrButton>)>,
    mut commands: Commands,
    mut color_manager: ResMut<crate::components::ColorManager>,
    mut solver: ResMut<crate::solver_integration::CubeSolverResource>,
    mut move_queue: ResMut<MoveQueue>,
    colored_faces_query: Query<(Entity, &crate::components::RecoloredFace)>,
    placeholder_material: Res<crate::colors::PlaceholderMaterial>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            info!("Clr button clicked - clearing all face colors and solver state");

            // Clear all existing colors from the color manager
            color_manager.usage_counts = [0; 6];

            // Remove all existing RecoloredFace components and reset materials to placeholder
            for (entity, _recolored_face) in colored_faces_query.iter() {
                commands
                    .entity(entity)
                    .remove::<crate::components::RecoloredFace>()
                    .insert(MeshMaterial3d(placeholder_material.0.clone()));
            }

            // Reset solver state
            solver.clear_solution();

            // Clear move queue
            move_queue.pending.clear();
            move_queue.current = None;
            move_queue.highlight_index = None;

            info!("All face colors and solver state cleared");
        }
    }
}

/// Helper function to calculate facelet index for reset (simplified version)
fn calculate_facelet_index_for_reset(
    face_entity: Entity,
    face_query: &Query<(Entity, &crate::components::Face)>,
    small_cube_transforms: &Query<&GlobalTransform, With<crate::cube_moves::CubeMoveTarget>>,
    main_cube_transforms: &Query<&GlobalTransform, With<crate::components::RotatingModel>>,
    face_transforms: &Query<&GlobalTransform, With<crate::components::Face>>,
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
                    let face_relative_to_main =
                        main_cube_transform.affine().inverse() * face_transform.affine();
                    let (_, _, face_main_pos) =
                        face_relative_to_main.to_scale_rotation_translation();

                    // Determine face orientation from face's position in main cube coordinates
                    let face_orientation =
                        determine_face_orientation_from_main_position(&face_main_pos);

                    // Calculate the small cube's position relative to the main cube in LOCAL SPACE
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

                    return Some(facelet_index);
                }
            }
        }
    }

    None
}

/// Helper function to determine face orientation from main position
fn determine_face_orientation_from_main_position(
    position: &Vec3,
) -> crate::components::Orientation {
    // Find the axis with the largest absolute value
    let abs_x = position.x.abs();
    let abs_y = position.y.abs();
    let abs_z = position.z.abs();

    if abs_x >= abs_y && abs_x >= abs_z {
        if position.x > 0.0 {
            crate::components::Orientation::Right
        } else {
            crate::components::Orientation::Left
        }
    } else if abs_y >= abs_z {
        if position.y > 0.0 {
            crate::components::Orientation::Up
        } else {
            crate::components::Orientation::Down
        }
    } else if position.z > 0.0 {
        crate::components::Orientation::Front
    } else {
        crate::components::Orientation::Back
    }
}

/// Helper function to convert world position to local indices
fn world_to_local_indices(position: &Vec3) -> Vec3 {
    const GRID_STEP: f32 = 2.0 / 3.0;

    let x = (position.x / GRID_STEP).round() as i32;
    let y = (position.y / GRID_STEP).round() as i32;
    let z = (position.z / GRID_STEP).round() as i32;

    let x = x.clamp(-1, 1);
    let y = y.clamp(-1, 1);
    let z = z.clamp(-1, 1);

    Vec3::new(x as f32, y as f32, z as f32)
}

/// Helper function to calculate position in face from indices
fn calculate_position_in_face_from_indices(
    local_indices: &Vec3,
    face_orientation: crate::components::Orientation,
) -> usize {
    // Convert local indices to grid coordinates based on face orientation
    let (grid_x, grid_y) = match face_orientation {
        crate::components::Orientation::Up => (local_indices.x + 1.0, local_indices.z + 1.0),
        crate::components::Orientation::Down => (local_indices.x + 1.0, -(local_indices.z) + 1.0),
        crate::components::Orientation::Front => (local_indices.x + 1.0, local_indices.y + 1.0),
        crate::components::Orientation::Back => (-(local_indices.x) + 1.0, local_indices.y + 1.0),
        crate::components::Orientation::Right => (local_indices.z + 1.0, local_indices.y + 1.0),
        crate::components::Orientation::Left => (-(local_indices.z) + 1.0, local_indices.y + 1.0),
    };

    let grid_x = grid_x as usize;
    let grid_y = grid_y as usize;

    // Convert to 0-8 position within face

    grid_y * 3 + grid_x
}

/// Helper function to create face material
fn create_face_material(
    base_color: Color,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color,
        metallic: 0.0,
        perceptual_roughness: 0.3,
        ..default()
    })
}

/// System to handle fix button clicks (copy of original Reset functionality)
pub fn handle_fix_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<FixButton>)>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut color_manager: ResMut<crate::components::ColorManager>,
    cube_colors: Res<crate::colors::CubeColors>,
    colored_faces_query: Query<(Entity, &crate::components::RecoloredFace)>,
    all_faces_query: Query<(Entity, &crate::components::Face)>,
    small_cube_transforms: Query<&GlobalTransform, With<crate::cube_moves::CubeMoveTarget>>,
    main_cube_transforms: Query<&GlobalTransform, With<crate::components::RotatingModel>>,
    face_transforms: Query<&GlobalTransform, With<crate::components::Face>>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            info!("Fix button clicked - resetting cube to solved state");

            // Clear all existing colors from the color manager
            color_manager.usage_counts = [0; 6];

            // Remove all existing RecoloredFace components
            for (entity, _recolored_face) in colored_faces_query.iter() {
                commands
                    .entity(entity)
                    .remove::<crate::components::RecoloredFace>();
            }

            // Create solved facelet string: UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
            let solved_facelets = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";

            // Map solved facelets to cube faces
            for (entity, _face) in all_faces_query.iter() {
                // Calculate facelet index based on parent small cube position
                if let Some(facelet_index) = calculate_facelet_index_for_reset(
                    entity,
                    &all_faces_query,
                    &small_cube_transforms,
                    &main_cube_transforms,
                    &face_transforms,
                ) && facelet_index < solved_facelets.len()
                {
                    let facelet_char = solved_facelets.chars().nth(facelet_index).unwrap_or(' ');
                    if facelet_char != ' ' {
                        // Convert facelet character to color index
                        let color_index = match facelet_char {
                            'U' => 0, // White
                            'D' => 1, // Yellow
                            'R' => 2, // Red
                            'L' => 3, // Orange
                            'B' => 4, // Blue
                            'F' => 5, // Green
                            _ => continue,
                        };

                        // Create material with the solved color
                        let color = cube_colors.get(color_index);
                        let material = create_face_material(color, &mut materials);

                        // Apply color to face
                        commands
                            .entity(entity)
                            .insert(MeshMaterial3d(material))
                            .insert(crate::components::RecoloredFace::new(
                                color_index,
                                bevy::utils::Instant::now().elapsed().as_secs_f64(),
                            ));

                        // Update color manager
                        color_manager.usage_counts[color_index] += 1;
                    }
                }
            }

            info!("Cube reset to solved state - all faces colored");
        }
    }
}

/// System to handle move selection button interactions
pub fn handle_select_button(
    mut interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<OpenMoveSelectionButton>),
    >,
    mut move_selection_panel_query: Query<
        &mut Visibility,
        (With<MoveSelectionPanel>, Without<MoveSelectionOverlay>),
    >,
    mut move_selection_overlay_query: Query<
        &mut Visibility,
        (With<MoveSelectionOverlay>, Without<MoveSelectionPanel>),
    >,
    mut move_selection_state: ResMut<MoveSelectionState>,
    mut move_queue: ResMut<MoveQueue>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            if move_selection_state.is_open {
                // Panel is open, close it
                if let Ok(mut panel_visibility) = move_selection_panel_query.get_single_mut() {
                    *panel_visibility = Visibility::Hidden;
                }
                if let Ok(mut overlay_visibility) = move_selection_overlay_query.get_single_mut() {
                    *overlay_visibility = Visibility::Hidden;
                }
                move_selection_state.is_open = false;
                info!("Move selection button clicked - closing move selection panel");
            } else {
                // Panel is closed, open it and clear existing rotations
                move_queue.pending.clear();
                move_queue.current = None;
                move_queue.highlight_index = None;

                if let Ok(mut panel_visibility) = move_selection_panel_query.get_single_mut() {
                    *panel_visibility = Visibility::Visible;
                }
                if let Ok(mut overlay_visibility) = move_selection_overlay_query.get_single_mut() {
                    *overlay_visibility = Visibility::Visible;
                }
                move_selection_state.is_open = true;
                info!(
                    "Move selection button clicked - opening move selection panel and clearing rotations"
                );
            }
        }
    }
}

/// System to handle move selection button clicks
pub fn handle_move_selection(
    mut interaction_query: Query<
        (&Interaction, &MoveSelectionButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut move_queue: ResMut<MoveQueue>,
) {
    for (interaction, move_button) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // Add the selected move directly to the rotation pane
            if parse_extended_move_notation(&move_button.move_notation).is_some() {
                move_queue.pending.push(move_button.move_notation.clone());
                info!("Added move to rotation pane: {}", move_button.move_notation);
            } else {
                warn!("Invalid move notation: {}", move_button.move_notation);
            }
            // Panel stays open so user can select more moves
        }
    }
}

/// System to handle backspace button clicks
pub fn handle_backspace_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<BackspaceButton>)>,
    mut move_queue: ResMut<MoveQueue>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // Remove the last added rotation (backspace functionality)
            if let Some(removed_move) = move_queue.pending.pop() {
                info!("Removed last rotation: {}", removed_move);

                // Update highlight index if it was pointing to the removed move
                if let Some(highlight_index) = move_queue.highlight_index
                    && highlight_index > move_queue.pending.len()
                {
                    move_queue.highlight_index = Some(move_queue.pending.len());
                }
            } else {
                info!("No rotations to remove");
            }
            // Panel stays open so user can select more moves
        }
    }
}

/// System to handle move completion by clearing current move when animation finishes
pub fn handle_move_completion(
    mut move_queue: ResMut<MoveQueue>,
    mut rotation_completed_events: EventReader<
        crate::ui::rotations_panel::LayerRotationCompletedEvent,
    >,
) {
    for _event in rotation_completed_events.read() {
        if move_queue.current.is_some() {
            move_queue.current = None;
            info!("Move completed, cleared current move");
        }
    }
}

/// System to update the MoveSelectionState resource when the panel visibility changes
pub fn update_move_selection_state(
    move_selection_panel_query: Query<&Visibility, With<MoveSelectionPanel>>,
    mut move_selection_state: ResMut<MoveSelectionState>,
) {
    if let Ok(visibility) = move_selection_panel_query.get_single() {
        move_selection_state.is_open = *visibility == Visibility::Visible;
    }
}

/// System to close move selection panel when any move test panel button is pressed
pub fn close_move_selection_on_button_press(
    mut interaction_query: Query<
        &Interaction,
        (
            Changed<Interaction>,
            Or<(With<RstButton>, With<FixButton>, With<ClrButton>)>,
        ),
    >,
    mut move_selection_panel_query: Query<
        &mut Visibility,
        (With<MoveSelectionPanel>, Without<MoveSelectionOverlay>),
    >,
    mut move_selection_overlay_query: Query<
        &mut Visibility,
        (With<MoveSelectionOverlay>, Without<MoveSelectionPanel>),
    >,
    mut move_selection_state: ResMut<MoveSelectionState>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            if let Ok(mut panel_visibility) = move_selection_panel_query.get_single_mut() {
                *panel_visibility = Visibility::Hidden;
            }
            if let Ok(mut overlay_visibility) = move_selection_overlay_query.get_single_mut() {
                *overlay_visibility = Visibility::Hidden;
            }
            move_selection_state.is_open = false;
        }
    }
}
