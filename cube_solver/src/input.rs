use crate::components::{RotatingModel, TouchState};
use bevy::prelude::*;

pub fn handle_touch(
    touches: Res<Touches>,
    mut touch_state: ResMut<TouchState>,
    mut rotating_models: Query<&mut Transform, With<RotatingModel>>,
    time: Res<Time>,
    // Query to check if any UI elements are being interacted with
    ui_interaction_query: Query<&Interaction, With<Button>>,
) {
    // Update rotation state timer
    touch_state.update_rotation_state(time.delta_secs());

    // Check if any UI element is currently being interacted with
    let ui_is_active = ui_interaction_query
        .iter()
        .any(|interaction| matches!(interaction, Interaction::Pressed | Interaction::Hovered));

    // Only process touch input for cube rotation if no UI elements are active
    if !ui_is_active {
        if let Some(touch) = touches.iter().next() {
            let current_pos = touch.position();

            if let Some(last_pos) = touch_state.last_touch_pos {
                let delta = current_pos - last_pos;
                let delta_magnitude = delta.length();

                if touch_state.should_rotate(delta_magnitude) {
                    // Mark that we're rotating
                    touch_state.start_rotation();

                    apply_rotation_to_models(delta, &touch_state, &mut rotating_models);

                    debug!(
                        "Applied rotation from touch delta: ({:.1}, {:.1}), magnitude: {:.1}",
                        delta.x, delta.y, delta_magnitude
                    );
                }
            }

            touch_state.last_touch_pos = Some(current_pos);
        } else {
            touch_state.last_touch_pos = None;
        }
    } else {
        // UI is active, clear touch state to prevent cube rotation
        touch_state.last_touch_pos = None;
    }
}

fn apply_rotation_to_models(
    delta: Vec2,
    touch_state: &TouchState,
    rotating_models: &mut Query<&mut Transform, With<RotatingModel>>,
) {
    let sensitivity = touch_state.rotation_sensitivity;

    for mut transform in rotating_models.iter_mut() {
        transform.rotate_y(delta.x * sensitivity);
        transform.rotate_x(delta.y * sensitivity);
    }
}
