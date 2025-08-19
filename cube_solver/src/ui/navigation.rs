use crate::cube_moves::CubeMoveEvent;
use crate::ui::rotations_panel::MoveQueue;
use bevy::color::palettes::css;
use bevy::prelude::*;

#[derive(Component)]
pub struct NavigationPrevButton;

#[derive(Component)]
pub struct NavigationNextButton;

/// Generate the inverse notation for a move
fn get_inverse_notation(notation: &str) -> String {
    if notation.is_empty() {
        return notation.to_string();
    }

    // For double moves (2), the inverse is the same
    if notation.ends_with('2') {
        return notation.to_string();
    }

    // For moves with prime ('), remove the prime
    if notation.ends_with('\'') {
        return notation[..notation.len() - 1].to_string();
    }

    // For moves without prime, add prime
    format!("{}'", notation)
}

/// System to handle navigation prev button clicks
pub fn handle_navigation_prev_button_clicks(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<NavigationPrevButton>)>,
    mut move_queue: ResMut<MoveQueue>,
    mut move_events: EventWriter<CubeMoveEvent>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed && !move_queue.pending.is_empty() {
            // Execute previous move (inverse)
            if move_queue.current.is_some() {
                info!("Cannot go back while a move is in progress");
                return;
            }

            // Move highlight to previous position and execute inverse
            if let Some(current_index) = move_queue.highlight_index {
                if current_index > 0 {
                    let new_index = current_index - 1;
                    move_queue.highlight_index = Some(new_index);

                    // Execute the inverse of the move at the new position
                    if new_index < move_queue.pending.len() {
                        let original_move = move_queue.pending[new_index].clone();
                        let inverse_move = get_inverse_notation(&original_move);
                        move_queue.current = Some(inverse_move.clone());
                        info!(
                            "Executing inverse of move at position {}: {} -> {}",
                            new_index, &original_move, &inverse_move
                        );
                        move_events.send(CubeMoveEvent {
                            notation: inverse_move,
                        });
                    }
                } else {
                    info!("Already at the beginning of the sequence");
                }
            } else if !move_queue.pending.is_empty() {
                // If no highlight, start at the last position
                let last_index = move_queue.pending.len();
                move_queue.highlight_index = Some(last_index);
                info!("Starting at the end of the sequence");
            } else {
                info!("No moves to go back to");
            }
        }
    }
}

/// System to handle navigation next button clicks
pub fn handle_navigation_next_button_clicks(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<NavigationNextButton>)>,
    mut move_queue: ResMut<MoveQueue>,
    mut move_events: EventWriter<CubeMoveEvent>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed && !move_queue.pending.is_empty() {
            // Execute next move
            if move_queue.current.is_some() {
                info!("Cannot go forward while a move is in progress");
                return;
            }

            // Move highlight to the next position and execute the move at the current position
            if let Some(current_index) = move_queue.highlight_index {
                if current_index < move_queue.pending.len() {
                    // Execute the move at the current position
                    let move_to_execute = move_queue.pending[current_index].clone();
                    move_queue.current = Some(move_to_execute.clone());
                    move_events.send(CubeMoveEvent {
                        notation: move_to_execute,
                    });
                    info!("Executing move at position: {}", current_index);

                    // Move border to next position
                    move_queue.highlight_index = Some(current_index + 1);
                } else {
                    info!("Already at the end of the sequence");
                }
            } else if !move_queue.pending.is_empty() {
                // If no highlight, start at the first position and execute first move
                move_queue.highlight_index = Some(0);
                let move_to_execute = move_queue.pending[0].clone();
                move_queue.current = Some(move_to_execute.clone());
                move_events.send(CubeMoveEvent {
                    notation: move_to_execute,
                });
                info!("Executing first move at position: 0");

                // Move border to next position
                move_queue.highlight_index = Some(1);
            } else {
                info!("No moves to go forward to");
            }
        }
    }
}

/// System to update navigation button states based on move queue
pub fn update_navigation_buttons(
    move_queue: Res<MoveQueue>,
    mut button_queries: ParamSet<(
        Query<(&mut BackgroundColor, &mut BorderColor), With<NavigationPrevButton>>,
        Query<(&mut BackgroundColor, &mut BorderColor), With<NavigationNextButton>>,
    )>,
) {
    let has_moves = !move_queue.pending.is_empty();

    // Update Prev button
    for (mut bg_color, mut border_color) in &mut button_queries.p0() {
        if has_moves {
            *bg_color = BackgroundColor(css::LIGHT_BLUE.into());
            *border_color = BorderColor(css::WHITE.into());
        } else {
            *bg_color = BackgroundColor(css::DARK_GRAY.into());
            *border_color = BorderColor(css::GRAY.into());
        }
    }

    // Update Next button
    for (mut bg_color, mut border_color) in &mut button_queries.p1() {
        if has_moves {
            *bg_color = BackgroundColor(css::LIGHT_GREEN.into());
            *border_color = BorderColor(css::WHITE.into());
        } else {
            *bg_color = BackgroundColor(css::DARK_GRAY.into());
            *border_color = BorderColor(css::GRAY.into());
        }
    }
}
