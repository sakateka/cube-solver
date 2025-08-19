use bevy::prelude::*;

/// Sets up the 3D camera and lighting for the scene
pub fn setup_camera_and_lighting(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Very dim ambient light to complement emissive glow
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
    });

    // TouchState is initialized by SelectionPlugin
}
