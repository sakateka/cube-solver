use crate::colors::CubeColors;
use crate::components::{ColorManager, ColorSquare};
use bevy::color::palettes::css;
use bevy::prelude::*;

pub const COLOR_NAMES: [&str; 6] = ["White", "Yellow", "Red", "Orange", "Blue", "Green"];

#[derive(Component)]
pub struct ColorCountLabel {
    pub color_index: usize,
}

/// Creates a UI-based color selector panel with text labels
pub fn create_ui_color_panel(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    color_manager: Res<ColorManager>,
    cube_colors: Res<CubeColors>,
) {
    info!("Creating UI color palette panel");

    // Create the main UI container positioned at the top of the screen
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(60.0),
                left: Val::Px(10.0),
                right: Val::Px(10.0),
                height: Val::Px(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                column_gap: Val::Px(5.0),
                ..default()
            },
            BackgroundColor(Color::from(css::MIDNIGHT_BLUE).with_alpha(0.8)),
            Name::new("Color Selector Panel"),
        ))
        .with_children(|parent| {
            // Create each color button with label
            for (i, (color, name)) in cube_colors
                .as_slice()
                .iter()
                .zip(COLOR_NAMES.iter())
                .enumerate()
            {
                parent
                    .spawn((
                        Node {
                            width: Val::Px(60.0),
                            height: Val::Px(80.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        Name::new(format!("{} Button Container", name)),
                    ))
                    .with_children(|container| {
                        // Color count label text (X/9 format)
                        container.spawn((
                            Text::new(color_manager.get_usage_info(i)),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(css::WHITE.into()),
                            Node {
                                margin: UiRect::bottom(Val::Px(5.0)),
                                ..default()
                            },
                            ColorCountLabel { color_index: i },
                        ));

                        // Color button
                        container.spawn((
                            Button,
                            Node {
                                width: Val::Px(50.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(*color),
                            BorderColor(css::WHITE.into()),
                            ColorSquare::new(i),
                            Name::new(format!("{} Button", name)),
                        ));
                    });
            }
        });

    info!("UI color palette panel creation completed");
}

/// System to handle color button clicks in the UI
pub fn handle_color_button_clicks(
    mut interaction_query: Query<
        (&Interaction, &ColorSquare, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut color_manager: ResMut<ColorManager>,
) {
    for (interaction, color_square, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Select the color (now allows any color, even at limit)
                match color_manager.try_select_color(color_square.color_index) {
                    Ok(()) => {
                        // Successfully selected color
                        *border_color = BorderColor(css::MAGENTA.into());
                        info!(
                            "Selected color: {} (index {})",
                            COLOR_NAMES[color_square.color_index], color_square.color_index
                        );
                    }
                    Err(err) => {
                        // Color selection failed (shouldn't happen now, but keep for safety)
                        *border_color = BorderColor(css::GRAY.into());
                        info!(
                            "Cannot select color {}: {}",
                            COLOR_NAMES[color_square.color_index], err
                        );
                    }
                }
            }
            Interaction::Hovered => {
                // Slightly highlight when hovered
                *border_color = BorderColor(css::GAINSBORO.into());
            }
            Interaction::None => {
                // Default border color
                *border_color = BorderColor(css::WHITE.into());
            }
        }
    }
}

/// System to update text color based on selection and limit states
pub fn update_color_text_colors(
    color_manager: Res<ColorManager>,
    mut text_query: Query<(&ColorCountLabel, &mut TextColor)>,
) {
    let selected_color = color_manager.get_selected_color();

    for (label, mut text_color) in &mut text_query {
        if color_manager.is_at_limit(label.color_index) {
            // Color is at limit - gray text
            *text_color = TextColor(css::GRAY.into());
        } else if Some(label.color_index) == selected_color {
            // Color is selected - magenta text
            *text_color = TextColor(css::MAGENTA.into());
        } else {
            // Default - white text
            *text_color = TextColor(css::WHITE.into());
        }
    }
}

/// System to update color count labels when colors are applied
pub fn update_color_count_labels(
    color_manager: Res<ColorManager>,
    mut label_query: Query<(&ColorCountLabel, &mut Text)>,
) {
    if !color_manager.is_changed() {
        return;
    }

    for (label, mut text) in &mut label_query {
        text.0 = color_manager.get_usage_info(label.color_index);
    }
}

/// System to update button borders based on current selection
pub fn update_color_button_selection(
    color_manager: Res<ColorManager>,
    mut button_query: Query<(&ColorSquare, &mut BorderColor, &mut BackgroundColor), With<Button>>,
    cube_colors: Res<CubeColors>,
) {
    // Run every frame to ensure visual feedback is always up to date
    // This ensures we catch the exact moment when limits are reached

    let selected_color = color_manager.get_selected_color();

    for (color_square, mut border_color, mut bg_color) in &mut button_query {
        let original_color = cube_colors.as_slice()[color_square.color_index];

        if Some(color_square.color_index) == selected_color {
            // Color is selected - show with magenta border regardless of limit
            *border_color = BorderColor(css::MAGENTA.into());
            if color_manager.is_at_limit(color_square.color_index) {
                // Selected but at limit - show faded but with selection border
                let s = original_color.to_srgba();
                let faded = Color::srgba(s.red * 0.5, s.green * 0.5, s.blue * 0.5, 0.8);
                *bg_color = BackgroundColor(faded);
            } else {
                // Selected and available - full brightness
                *bg_color = BackgroundColor(original_color);
            }
        } else if color_manager.is_at_limit(color_square.color_index) {
            // Color is at limit but not selected - show faded with gray border
            let s = original_color.to_srgba();
            let faded = Color::srgba(s.red * 0.3, s.green * 0.3, s.blue * 0.3, 0.6);
            *bg_color = BackgroundColor(faded);
            *border_color = BorderColor(css::GRAY.into());
        } else {
            // Color is available and not selected - normal appearance
            *border_color = BorderColor(css::WHITE.into());
            *bg_color = BackgroundColor(original_color);
        }
    }
}
