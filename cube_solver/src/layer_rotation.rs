use crate::cube_moves::{CubeMoveEvent, CubeMoveTarget, parse_move_notation};
use crate::layer_components::{
    CubeLayer, LayerFace, LayerMoveType, LayerRotationAnimation, cube_belongs_to_layer,
};
use bevy::prelude::*;

/// Marker that the layer pivot has been prepared (children reparented) for the current animation
#[derive(Component)]
pub struct RotationPrepared;

fn snap_vec3_to_grid(position: Vec3) -> Vec3 {
    // Keep cube centers snapped to the 3x3x3 grid used at creation (step = 2/3)
    const STEP: f32 = 2.0 / 3.0;
    fn snap_axis(value: f32) -> f32 {
        let n = (value / STEP).round() as i32;
        let n = n.clamp(-1, 1);
        (n as f32) * STEP
    }
    Vec3::new(
        snap_axis(position.x),
        snap_axis(position.y),
        snap_axis(position.z),
    )
}

fn snap_rotation_to_axis_aligned(q: Quat) -> Quat {
    // Snap an orientation to the nearest 90-degree axis-aligned basis relative to the parent
    let m = Mat3::from_quat(q);
    // Column vectors represent the rotated local axes in parent space
    let mut x = Vec3::new(m.x_axis.x, m.x_axis.y, m.x_axis.z);
    let mut y = Vec3::new(m.y_axis.x, m.y_axis.y, m.y_axis.z);

    // Helper to snap a direction to nearest cardinal axis
    fn snap_dir(v: Vec3) -> Vec3 {
        let ax = v.x.abs();
        let ay = v.y.abs();
        let az = v.z.abs();
        if ax >= ay && ax >= az {
            Vec3::new(v.x.signum(), 0.0, 0.0)
        } else if ay >= az {
            Vec3::new(0.0, v.y.signum(), 0.0)
        } else {
            Vec3::new(0.0, 0.0, v.z.signum())
        }
    }

    x = snap_dir(x);
    y = snap_dir(y);

    // Ensure orthonormal right-handed basis
    let mut z = x.cross(y);
    if z.length_squared() < 0.5 {
        // If x and y snapped to the same axis, derive y from another axis
        if x.x.abs() > 0.5 {
            y = Vec3::new(0.0, 0.0, 1.0);
        } else {
            y = Vec3::new(1.0, 0.0, 0.0);
        }
        z = x.cross(y);
    }
    y = z.cross(x);
    x = x.normalize();
    y = y.normalize();
    z = z.normalize();

    let snapped = Mat3::from_cols(x, y, z);
    Quat::from_mat3(&snapped)
}

/// Prepare a layer pivot for rotation by temporarily reparenting member cubes under the pivot
pub fn prepare_layer_rotation(
    mut commands: Commands,
    mut layer_query: Query<
        (Entity, &Transform, &CubeLayer, Option<&Children>),
        (
            With<CubeLayer>,
            With<LayerRotationAnimation>,
            Without<RotationPrepared>,
            Without<CubeMoveTarget>,
        ),
    >,
    parent_of_layer: Query<&Parent, With<CubeLayer>>, // parent is the root cube entity
    mut cubes_query: Query<
        (Entity, &GlobalTransform, &mut Transform),
        (With<CubeMoveTarget>, Without<CubeLayer>),
    >,
    globals: Query<&GlobalTransform>,
) {
    for (layer_entity, _layer_transform, cube_layer, maybe_children) in &mut layer_query {
        // Ensure pivot has no stale children: move any existing children back to the root parent first
        if let Some(children) = maybe_children
            && let Ok(parent) = parent_of_layer.get(layer_entity)
        {
            let root_entity = parent.get();
            // Root global for relative conversion
            let Ok(root_global) = globals.get(root_entity) else {
                continue;
            };
            for &child in children.iter() {
                if let (Ok(child_global), Ok(triple)) =
                    (globals.get(child), cubes_query.get_mut(child))
                {
                    let (_, _, mut child_local) = triple;
                    let relative = root_global.affine().inverse() * child_global.affine();
                    let (scale, rotation, translation) = relative.to_scale_rotation_translation();
                    child_local.translation = snap_vec3_to_grid(translation);
                    child_local.rotation = rotation;
                    child_local.scale = scale;
                }
                commands.entity(root_entity).add_child(child);
            }
        }

        // Collect cubes that belong to this layer by position and
        // reparent them under the pivot while preserving world pose
        let Ok(pivot_global) = globals.get(layer_entity) else {
            continue;
        };
        if let Ok(parent) = parent_of_layer.get(layer_entity) {
            let root_entity = parent.get();
            // Iterate all cubes (children are already back under root) and
            // compute membership in root-local space so whole-cube rotation is respected
            let Ok(root_global) = globals.get(root_entity) else {
                continue;
            };
            for (cube_entity, cube_global, mut cube_local) in &mut cubes_query {
                let relative_to_root = root_global.affine().inverse() * cube_global.affine();
                let (_s, _r, translation_root_local) =
                    relative_to_root.to_scale_rotation_translation();
                if cube_belongs_to_layer(translation_root_local, cube_layer.face) {
                    // Compute local relative to pivot
                    let relative = pivot_global.affine().inverse() * cube_global.affine();
                    let (scale, rotation, translation) = relative.to_scale_rotation_translation();
                    cube_local.translation = translation;
                    cube_local.rotation = rotation;
                    cube_local.scale = scale;
                    commands.entity(layer_entity).add_child(cube_entity);
                } else {
                    // Ensure cube stays under root with correct local (relative to root)
                    let relative = root_global.affine().inverse() * cube_global.affine();
                    let (scale, rotation, translation) = relative.to_scale_rotation_translation();
                    cube_local.translation = snap_vec3_to_grid(translation);
                    cube_local.rotation = rotation;
                    cube_local.scale = scale;
                    commands.entity(root_entity).add_child(cube_entity);
                }
            }
        }

        // Mark as prepared to avoid repeating
        commands.entity(layer_entity).insert(RotationPrepared);
    }
}

/// System to handle layer rotation animations
pub fn layer_rotation_system(
    mut commands: Commands,
    mut rotation_completed_events: EventWriter<
        crate::ui::rotations_panel::LayerRotationCompletedEvent,
    >,
    time: Res<Time>,
    mut layer_query: Query<
        (
            Entity,
            &mut Transform,
            &mut LayerRotationAnimation,
            &CubeLayer,
        ),
        (
            With<CubeLayer>,
            With<RotationPrepared>,
            Without<CubeMoveTarget>,
        ),
    >,
    parent_of_layer: Query<&Parent, With<CubeLayer>>,
    children_query: Query<&Children>,
    globals: Query<&GlobalTransform>,
    mut cube_transforms: Query<&mut Transform, (With<CubeMoveTarget>, Without<CubeLayer>)>,
) {
    for (layer_entity, mut layer_transform, mut animation, cube_layer) in &mut layer_query {
        animation.elapsed += time.delta_secs();

        if animation.is_complete() {
            // Finalize: bake current world transforms of children, move them back to root, and reset pivot
            let Ok(parent) = parent_of_layer.get(layer_entity) else {
                continue;
            };
            let root_entity = parent.get();

            if let Ok(children) = children_query.get(layer_entity) {
                for &child in children.iter() {
                    // Get child's world transform and apply it to its local Transform when reparented to root
                    if let (Ok(child_global), Ok(root_global), Ok(mut local)) = (
                        globals.get(child),
                        globals.get(root_entity),
                        cube_transforms.get_mut(child),
                    ) {
                        let relative = root_global.affine().inverse() * child_global.affine();
                        let (scale, rotation, translation) =
                            relative.to_scale_rotation_translation();
                        local.translation = snap_vec3_to_grid(translation);
                        local.rotation = snap_rotation_to_axis_aligned(rotation);
                        local.scale = scale;
                    }
                    // Reparent back to root
                    commands.entity(root_entity).add_child(child);
                }
            }

            // Reset pivot and remove animation + prepared marker
            layer_transform.rotation = animation.initial_transform.rotation;
            commands
                .entity(layer_entity)
                .remove::<LayerRotationAnimation>();
            commands.entity(layer_entity).remove::<RotationPrepared>();

            // Send completion event with layer info
            rotation_completed_events.send(
                crate::ui::rotations_panel::LayerRotationCompletedEvent {
                    layer_face: cube_layer.face,
                    move_type: animation.move_type,
                },
            );
            info!("Layer rotation completed for entity {:?}", layer_entity);
        } else {
            // Update layer rotation for visual feedback (doesn't affect cubes during animation)
            let current_angle = animation.current_angle();
            let current_rotation = Quat::from_axis_angle(animation.axis, current_angle);
            layer_transform.rotation = animation.initial_transform.rotation * current_rotation;
        }
    }
}

/// Start a layer rotation animation
pub fn start_layer_rotation(
    commands: &mut Commands,
    layer_entity: Entity,
    layer_transform: Transform,
    layer_face: LayerFace,
    move_type: LayerMoveType,
) {
    // Make double ("2") rotations slower for better readability
    let duration = match move_type {
        LayerMoveType::Double => 1.2,
        _ => 0.7,
    }; // seconds for the animation
    let axis = layer_face.rotation_axis();
    let direction = layer_face.rotation_direction();
    let move_angle = move_type.rotation_angle();
    let target_angle = move_angle * direction;

    info!(
        "Starting rotation for {:?}: axis={:?}, direction={}, move_angle={}, target_angle={}",
        layer_face, axis, direction, move_angle, target_angle
    );

    let animation =
        LayerRotationAnimation::new(target_angle, duration, axis, layer_transform, move_type);

    commands.entity(layer_entity).insert(animation);
}

/// Function to get entities that should be rotated for a given face move (updated for layers)
pub fn get_layer_entities(
    layer_query: &Query<(Entity, &Transform, &CubeLayer)>,
    face: LayerFace,
) -> Option<(Entity, Transform)> {
    layer_query
        .iter()
        .find(|(_, _, layer)| layer.face == face)
        .map(|(entity, transform, _)| (entity, *transform))
}

/// Extended move notation parser that supports middle layer moves
pub fn parse_extended_move_notation(notation: &str) -> Option<(LayerFace, LayerMoveType)> {
    if notation.is_empty() {
        return None;
    }

    // Handle standard face moves first
    if let Some((face, move_type)) = parse_move_notation(notation) {
        let layer_face = LayerFace::from_cube_face(face);
        let layer_move_type = LayerMoveType::from_move_type(move_type);
        return Some((layer_face, layer_move_type));
    }

    // Handle middle layer moves (M, E, S)
    let base_char = notation.chars().next()?;
    let layer_face = match base_char {
        'M' => LayerFace::MiddleX, // Middle slice (between L and R)
        'E' => LayerFace::MiddleY, // Equatorial slice (between U and D)
        'S' => LayerFace::MiddleZ, // Standing slice (between F and B)
        _ => return None,
    };

    let move_type = if notation.len() > 1 {
        match &notation[1..] {
            "'" => LayerMoveType::CounterClockwise,
            "2" => LayerMoveType::Double,
            _ => return None,
        }
    } else {
        LayerMoveType::Clockwise
    };

    Some((layer_face, move_type))
}

/// System to handle extended move commands (including middle layers)
pub fn handle_extended_move_commands(
    mut commands: Commands,
    layer_query: Query<(Entity, &Transform, &CubeLayer)>,
    animating_any: Query<Entity, With<LayerRotationAnimation>>,
    mut move_events: EventReader<CubeMoveEvent>,
) {
    let rotation_in_progress = animating_any.iter().next().is_some();
    for event in move_events.read() {
        if rotation_in_progress {
            // Drop events while a rotation is in progress to avoid overlapping reparent/baking
            continue;
        }

        if let Some((layer_face, move_type)) = parse_extended_move_notation(&event.notation) {
            // Find the layer entity for this face
            if let Some((layer_entity, layer_transform, _)) = layer_query
                .iter()
                .find(|(_, _, layer)| layer.face == layer_face)
            {
                start_layer_rotation(
                    &mut commands,
                    layer_entity,
                    *layer_transform,
                    layer_face,
                    move_type,
                );

                info!(
                    "Start rotation: {} ({:?} {:?})",
                    event.notation, layer_face, move_type
                );
            } else {
                warn!("Could not find layer for face: {:?}", layer_face);
            }
        } else {
            warn!("Invalid extended move notation: {}", event.notation);
        }
    }
}

// Bevy-idiomatic plugin and sets to clearly order layer rotation pipeline
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum LayerRotationSet {
    Parse,
    Prepare,
    Animate,
}

pub struct LayerRotationPlugin;

impl Plugin for LayerRotationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<crate::ui::rotations_panel::LayerRotationCompletedEvent>()
            .configure_sets(
                Update,
                (
                    LayerRotationSet::Parse,
                    LayerRotationSet::Prepare,
                    LayerRotationSet::Animate,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                handle_extended_move_commands.in_set(LayerRotationSet::Parse),
            )
            .add_systems(
                Update,
                prepare_layer_rotation.in_set(LayerRotationSet::Prepare),
            )
            .add_systems(
                Update,
                layer_rotation_system.in_set(LayerRotationSet::Animate),
            );
    }
}
