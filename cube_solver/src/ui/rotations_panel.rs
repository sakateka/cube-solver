use bevy::color::palettes::css;
use bevy::prelude::*;

use crate::cube_moves::CubeMoveEvent;

/// Event sent when a layer rotation animation completes
#[derive(Event)]
pub struct LayerRotationCompletedEvent {
    pub layer_face: crate::layer_components::LayerFace,
    pub move_type: crate::layer_components::LayerMoveType,
}

#[derive(Component)]
pub struct RotationsPanel;

#[derive(Component)]
pub struct RotationItem;

#[derive(Component)]
pub struct LeftSideContainer;

#[derive(Component)]
pub struct RightSideContainer;

#[derive(Component)]
pub struct CenterHighlight;

#[derive(Resource, Default, Clone)]
pub struct MoveQueue {
    pub pending: Vec<String>,
    pub current: Option<String>,
    pub highlight_index: Option<usize>, // Track which position the border is at (can be 0 to len())
}

/// Create a small horizontal panel above the solve button to display rotation steps
pub fn create_rotations_panel(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(90.0), // Above the solve button (at 30px height 50px)
                left: Val::Px(20.0),
                right: Val::Px(20.0),
                height: Val::Px(48.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                padding: UiRect::horizontal(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                overflow: Overflow::clip(), // Clip content that goes outside
                ..default()
            },
            BackgroundColor(Color::from(css::DARK_SLATE_GRAY).with_alpha(0.95)),
            BorderColor(css::WHITE.into()),
            BorderRadius::all(Val::Px(10.0)),
            RotationsPanel,
            Name::new("Rotations Panel"),
        ))
        .with_children(|parent| {
            // Center highlight element as parent (always visible, fixed position)
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0), // Center horizontally
                        top: Val::Px(9.0),        // Center vertically
                        width: Val::Px(0.0),
                        height: Val::Px(30.0),
                        border: UiRect::all(Val::Px(2.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::from(css::BLACK).with_alpha(0.0)),
                    BorderColor(css::LIGHT_BLUE.into()),
                    CenterHighlight,
                    Name::new("Center Highlight"),
                ))
                .with_children(|highlight_parent| {
                    // Left side container for past moves (positioned relative to highlight)
                    highlight_parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            right: Val::Px(5.0), // Position to the left of highlight
                            top: Val::Px(0.0),
                            bottom: Val::Px(0.0),
                            width: Val::Px(200.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::FlexEnd,
                            align_items: AlignItems::Center,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        LeftSideContainer,
                        Name::new("Left Side Container"),
                    ));

                    // Right side container for future moves (positioned relative to highlight)
                    highlight_parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(5.0), // Position to the right of highlight
                            top: Val::Px(0.0),
                            bottom: Val::Px(0.0),
                            width: Val::Px(200.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        RightSideContainer,
                        Name::new("Right Side Container"),
                    ));
                });
        });
}

/// Updates the rotations panel UI to show current and pending moves
pub fn update_rotations_panel_ui(
    move_queue: Res<MoveQueue>,
    left_container_query: Query<Entity, With<LeftSideContainer>>,
    right_container_query: Query<Entity, With<RightSideContainer>>,
    children_query: Query<&Children>,
    item_marker_query: Query<(), With<RotationItem>>,
    mut commands: Commands,
) {
    if !move_queue.is_changed() {
        return;
    }

    // Clear and update left container (past moves)
    if let Ok(left_container) = left_container_query.get_single() {
        if let Ok(children) = children_query.get(left_container) {
            for &child in children.iter() {
                if item_marker_query.get(child).is_ok() {
                    commands.entity(child).despawn_recursive();
                }
            }
        }

        // Add past moves (moves before the highlight position)
        if let Some(highlight_index) = move_queue.highlight_index {
            for i in 0..highlight_index {
                if i < move_queue.pending.len() {
                    commands.entity(left_container).with_children(|parent| {
                        parent.spawn((
                            Text::new(move_queue.pending[i].clone()),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(css::WHITE.into()),
                            Node {
                                margin: UiRect::left(Val::Px(8.0)), // Add space between moves
                                ..default()
                            },
                            RotationItem,
                        ));
                    });
                }
            }
        }
    }

    // Clear and update right container (future moves)
    if let Ok(right_container) = right_container_query.get_single() {
        if let Ok(children) = children_query.get(right_container) {
            for &child in children.iter() {
                if item_marker_query.get(child).is_ok() {
                    commands.entity(child).despawn_recursive();
                }
            }
        }

        // Add future moves (moves at and after the highlight position)
        if let Some(highlight_index) = move_queue.highlight_index {
            for i in highlight_index..move_queue.pending.len() {
                commands.entity(right_container).with_children(|parent| {
                    parent.spawn((
                        Text::new(move_queue.pending[i].clone()),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(css::WHITE.into()),
                        Node {
                            margin: UiRect::right(Val::Px(8.0)), // Add space between moves
                            ..default()
                        },
                        RotationItem,
                    ));
                });
            }
        } else {
            // No highlight, show all moves on right
            for mv in &move_queue.pending {
                commands.entity(right_container).with_children(|parent| {
                    parent.spawn((
                        Text::new(mv.clone()),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(css::WHITE.into()),
                        Node {
                            margin: UiRect::right(Val::Px(8.0)), // Add space between moves
                            ..default()
                        },
                        RotationItem,
                    ));
                });
            }
        }
    }
}

/// Drives the move queue: starts next move when idle and advances after completion
pub fn drive_move_queue(
    mut move_events: EventWriter<CubeMoveEvent>,
    mut move_queue: ResMut<MoveQueue>,
    mut rotation_completed_events: EventReader<LayerRotationCompletedEvent>,
) {
    // Check for rotation completion events
    for _event in rotation_completed_events.read() {
        if move_queue.current.is_some() {
            move_queue.current = None;
        }
    }

    // If idle, start next
    if move_queue.current.is_none()
        && let Some(next) = move_queue.pending.first().cloned()
    {
        move_queue.pending.remove(0);
        move_queue.current = Some(next.clone());
        move_events.send(CubeMoveEvent { notation: next });
    }
}

pub struct RotationsPanelPlugin;

impl Plugin for RotationsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MoveQueue>()
            .add_systems(Startup, create_rotations_panel)
            .add_systems(Update, update_rotations_panel_ui);
        // .add_systems(Update, drive_move_queue.before(LayerRotationSet::Parse)); // Disabled for manual control
    }
}
