use crate::colors::CubeColors;
use crate::components::{Orientation, RotatingModel};
use crate::cube_moves::CubeMoveTarget;
use crate::layer_components::{CubeLayer, LayerFace, LayersCube, get_position_in_layer};
use crate::selection::Selectable;
use bevy::prelude::*;
use std::collections::HashMap;

/// Creates a complete Rubik's cube with proper layer hierarchy
/// Each layer contains 9 cubes organized as a cohesive group
pub fn create_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    placeholder_material: Res<crate::colors::PlaceholderMaterial>,
) {
    info!("Creating Rubik's cube with layer-based hierarchy");

    // Neutral cube material (dark gray)
    let cube_material = materials.add(StandardMaterial {
        base_color: CubeColors::base_color(),
        metallic: 0.6,
        perceptual_roughness: 0.5,
        ..default()
    });

    let small_cube_size = 2.0 / 3.0; // 1/3 of original cube size
    let spacing = small_cube_size; // Space between cube centers
    let face_thickness = 0.02; // Thin planes for faces

    // Create the parent Rubik's cube entity
    let parent_cube = commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            RotatingModel,
            Name::new("Rubik's Cube"),
        ))
        .id();

    let cube_mesh = meshes.add(Cuboid::new(
        small_cube_size,
        small_cube_size,
        small_cube_size,
    ));
    let face_mesh_x = meshes.add(Cuboid::new(
        face_thickness,
        small_cube_size * 0.9,
        small_cube_size * 0.9,
    )); // YZ plane
    let face_mesh_y = meshes.add(Cuboid::new(
        small_cube_size * 0.9,
        face_thickness,
        small_cube_size * 0.9,
    )); // XZ plane
    let face_mesh_z = meshes.add(Cuboid::new(
        small_cube_size * 0.9,
        small_cube_size * 0.9,
        face_thickness,
    )); // XY plane

    // Create layer entities for all 9 possible layers
    let mut layer_entities: HashMap<LayerFace, Entity> = HashMap::new();

    // Create all 9 layer entities
    let all_layers = [
        LayerFace::Right,
        LayerFace::MiddleX,
        LayerFace::Left,
        LayerFace::Up,
        LayerFace::MiddleY,
        LayerFace::Down,
        LayerFace::Front,
        LayerFace::MiddleZ,
        LayerFace::Back,
    ];

    for layer_face in all_layers {
        let layer_entity = commands
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                CubeLayer {
                    face: layer_face,
                    layer_index: layer_face.layer_index(),
                },
                Name::new(format!("Layer {:?}", layer_face)),
            ))
            .id();

        // Make layer a child of the parent cube
        commands.entity(parent_cube).add_child(layer_entity);
        layer_entities.insert(layer_face, layer_entity);

        info!(
            "Created layer entity for {:?}: {:?}",
            layer_face, layer_entity
        );
    }

    let mut cube_index = 0;

    // Create 3x3x3 grid (27 positions) but skip center (26 cubes)
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                // Skip the center cube (it's hidden inside)
                if x == 0 && y == 0 && z == 0 {
                    continue;
                }

                let position =
                    Vec3::new(x as f32 * spacing, y as f32 * spacing, z as f32 * spacing);

                // Create each small cube
                let small_cube = commands
                    .spawn((
                        Mesh3d(cube_mesh.clone()),
                        MeshMaterial3d(cube_material.clone()),
                        Transform::from_translation(position).with_scale(Vec3::splat(0.9)),
                        CubeMoveTarget {
                            face: CubeMoveTarget::determine_face_from_position(&position),
                            layer: 0, // Will be updated based on layer membership
                        },
                        Name::new(format!("Small Cube {}", cube_index + 1)),
                    ))
                    .id();

                // Determine which layers this cube belongs to based on its coordinates
                let x_layer = if position.x > 0.5 {
                    LayerFace::Right
                } else if position.x < -0.5 {
                    LayerFace::Left
                } else {
                    LayerFace::MiddleX
                };

                let y_layer = if position.y > 0.5 {
                    LayerFace::Up
                } else if position.y < -0.5 {
                    LayerFace::Down
                } else {
                    LayerFace::MiddleY
                };

                let z_layer = if position.z > 0.5 {
                    LayerFace::Front
                } else if position.z < -0.5 {
                    LayerFace::Back
                } else {
                    LayerFace::MiddleZ
                };

                // Make the cube a child of only ONE layer to avoid transform conflicts
                // Choose the most "outer" layer (prioritize faces over middle layers)
                let primary_layer = if position.z.abs() > 0.5 {
                    z_layer
                } else if position.x.abs() > 0.5 {
                    x_layer
                } else if position.y.abs() > 0.5 {
                    y_layer
                } else {
                    x_layer
                }; // For center cubes, default to x_layer

                if let Some(&layer_entity) = layer_entities.get(&primary_layer) {
                    commands.entity(layer_entity).add_child(small_cube);
                }

                // Add LayerCube components for ALL layers this cube belongs to (for tracking)
                for layer_face in [x_layer, y_layer, z_layer] {
                    let position_in_layer = get_position_in_layer(position, layer_face);
                    commands.entity(small_cube).insert(LayersCube {
                        layer_face,
                        position_in_layer,
                    });
                }

                // Add faces on outer surfaces
                let face_offset = small_cube_size * 0.505 + face_thickness * 0.5;
                let face_configs = [
                    (
                        x == 1,
                        Orientation::Right,
                        Vec3::X * (x as f32 * face_offset),
                        &face_mesh_x,
                        "right",
                    ),
                    (
                        x == -1,
                        Orientation::Left,
                        Vec3::X * (x as f32 * face_offset),
                        &face_mesh_x,
                        "left",
                    ),
                    (
                        y == 1,
                        Orientation::Up,
                        Vec3::Y * (y as f32 * face_offset),
                        &face_mesh_y,
                        "top",
                    ),
                    (
                        y == -1,
                        Orientation::Down,
                        Vec3::Y * (y as f32 * face_offset),
                        &face_mesh_y,
                        "bottom",
                    ),
                    (
                        z == 1,
                        Orientation::Front,
                        Vec3::Z * (z as f32 * face_offset),
                        &face_mesh_z,
                        "front",
                    ),
                    (
                        z == -1,
                        Orientation::Back,
                        Vec3::Z * (z as f32 * face_offset),
                        &face_mesh_z,
                        "back",
                    ),
                ];

                for (should_spawn, orientation, face_pos, mesh, name) in face_configs {
                    if should_spawn {
                        let face = commands
                            .spawn((
                                Mesh3d(mesh.clone()),
                                MeshMaterial3d(placeholder_material.0.clone()),
                                Transform::from_translation(face_pos).with_scale(Vec3::splat(0.9)),
                                Selectable::with_id(0.5, format!("{}_{}", name, cube_index)),
                                Name::new(format!(
                                    "{} Face {}",
                                    orientation.as_str(),
                                    cube_index + 1
                                )),
                                crate::components::Face {
                                    parent_cube: small_cube,
                                },
                            ))
                            .id();
                        commands.entity(small_cube).add_child(face);
                    }
                }
                cube_index += 1;
            }
        }
    }

    info!("Rubik's cube creation completed with layer hierarchy");
}
