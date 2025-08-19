use crate::ui::rotations_panel::MoveQueue;
use bevy::color::palettes::css;
use bevy::prelude::*;

#[derive(Component)]
pub struct SolveButton;

#[derive(Component)]
pub struct SolveButtonContainer;

/// Creates a solve button container at the bottom of the screen
pub fn create_solve_button(
    mut commands: Commands,
    solver: Res<crate::solver_integration::CubeSolverResource>,
) {
    info!("Creating solve button container");

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(30.0),
                left: Val::Px(50.0),
                right: Val::Px(50.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },
            SolveButtonContainer,
            Name::new("Solve Button Container"),
        ))
        .with_children(|parent| {
            // Create Prev button (left of Solve)
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(80.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(css::DARK_GRAY.into()),
                    BorderColor(css::GRAY.into()),
                    crate::ui::navigation::NavigationPrevButton,
                    Name::new("Navigation Prev Button"),
                ))
                .with_children(|button_parent| {
                    button_parent.spawn((
                        Text::new("Prev"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(css::DIM_GRAY.into()),
                    ));
                });

            // Create the main solve button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(120.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(if solver.is_solvable() {
                        css::LIGHT_GREEN.into()
                    } else {
                        css::DARK_GRAY.into()
                    }),
                    BorderColor(css::WHITE.into()),
                    SolveButton,
                    Name::new("Solve Button"),
                ))
                .with_children(|button_parent| {
                    button_parent.spawn((
                        Text::new("Solve"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(if solver.is_solvable() {
                            css::WHITE.into()
                        } else {
                            css::DIM_GRAY.into()
                        }),
                    ));
                });

            // Create Next button (right of Solve)
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(80.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(css::DARK_GRAY.into()),
                    BorderColor(css::GRAY.into()),
                    crate::ui::navigation::NavigationNextButton,
                    Name::new("Navigation Next Button"),
                ))
                .with_children(|button_parent| {
                    button_parent.spawn((
                        Text::new("Next"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(css::DIM_GRAY.into()),
                    ));
                });
        });

    info!(
        "Solve button container created (solvable: {})",
        solver.is_solvable()
    );
}

/// System to update solve button state based on solver validation and move queue
pub fn update_solve_button(
    solver: Res<crate::solver_integration::CubeSolverResource>,
    move_queue: Res<MoveQueue>,
    mut button_query: Query<(&mut BackgroundColor, &mut BorderColor), With<SolveButton>>,
    mut text_query: Query<(&mut Text, &mut TextColor), (With<Text>, Without<SolveButton>)>,
    solve_button_query: Query<Entity, With<SolveButton>>,
    children_query: Query<&Children>,
) {
    // Determine if solve button should be active
    // It should be active if cube is valid AND either:
    // 1. No solution has been found yet, OR
    // 2. Solution was found but rotation panel is empty (user cleared it)
    let should_be_active =
        solver.is_valid() && (!solver.is_solving() || move_queue.pending.is_empty());

    let (bg_color, text_color) = if should_be_active {
        if solver.is_solvable() {
            (css::LIGHT_GREEN.into(), css::WHITE.into())
        } else {
            (css::ORANGE.into(), css::WHITE.into())
        }
    } else {
        (css::DARK_GRAY.into(), css::DIM_GRAY.into())
    };

    // Update button appearance
    if let Ok((mut bg_color_component, mut border_color)) = button_query.get_single_mut() {
        *bg_color_component = BackgroundColor(bg_color);
        *border_color = BorderColor(css::WHITE.into());
    }

    // Update text color
    if let Ok(solve_button_entity) = solve_button_query.get_single()
        && let Ok(children) = children_query.get(solve_button_entity)
    {
        for &child in children.iter() {
            if let Ok((_, mut text_color_component)) = text_query.get_mut(child) {
                *text_color_component = TextColor(text_color);
            }
        }
    }
}

/// System to handle solve button clicks
pub fn handle_solve_button_clicks(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<SolveButton>)>,
    mut solver: ResMut<crate::solver_integration::CubeSolverResource>,
    mut move_queue: ResMut<MoveQueue>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // Only allow solve if button is active
            let should_be_active =
                solver.is_valid() && (!solver.is_solving() || move_queue.pending.is_empty());

            if should_be_active {
                // Start solution execution - perform full solve
                log::info!("Solve button pressed - performing full solve!");

                if solver.perform_full_solve() {
                    log::info!("Solution found: {} moves", solver.solve_moves().len());
                    log::info!("Solution moves: {:?}", solver.solve_moves());

                    // Insert solution moves into the rotation panel
                    move_queue.pending = solver.solve_moves().clone();
                    move_queue.current = None;
                    move_queue.highlight_index = Some(0); // Start at the first move

                    // Start solution execution mode
                    solver.set_solving(true);

                    log::info!(
                        "Solution execution started with {} moves",
                        solver.solve_moves().len()
                    );
                } else {
                    log::info!("Solve failed: {}", solver.get_validation_message());
                }
            } else {
                log::info!("Solve button pressed but not active");
                log::info!("Issue: {}", solver.get_validation_message());
            }
        }
    }
}

/// System to handle move completion by clearing current move when animation finishes
pub fn handle_solution_move_completion(
    mut move_queue: ResMut<MoveQueue>,
    mut rotation_completed_events: EventReader<
        crate::ui::rotations_panel::LayerRotationCompletedEvent,
    >,
) {
    for _event in rotation_completed_events.read() {
        if move_queue.current.is_some() {
            move_queue.current = None;
            info!("Solution move completed, cleared current move");
        }
    }
}
