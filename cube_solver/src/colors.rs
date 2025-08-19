use bevy::color::palettes::css;
use bevy::prelude::*;

/// Resource containing the standard Rubik's cube face colors and helpers.
#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct CubeColors {
    pub colors: Vec<Color>,
}

impl Default for CubeColors {
    fn default() -> Self {
        Self {
            // Keep consistent with UI palette
            colors: vec![
                Color::srgb(1.0, 1.0, 1.0), // White (index 0)
                Color::srgb(1.0, 1.0, 0.0), // Yellow (index 1)
                Color::srgb(1.0, 0.0, 0.0), // Red (index 2)
                Color::srgb(1.0, 0.5, 0.0), // Orange (index 3)
                Color::srgb(0.0, 0.0, 1.0), // Blue (index 4)
                Color::srgb(0.0, 1.0, 0.0), // Green (index 5)
            ],
        }
    }
}

impl CubeColors {
    /// Placeholder gray color for unassigned cube faces
    pub fn placeholder_color() -> Color {
        css::DIM_GRAY.into()
    }

    pub fn base_color() -> Color {
        Color::srgb(0.1, 0.1, 0.1) // Dark gray
    }

    /// Safe get by index with default fallback
    pub fn get(&self, index: usize) -> Color {
        self.colors.get(index).copied().unwrap_or(Color::WHITE)
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }
    pub fn as_slice(&self) -> &[Color] {
        &self.colors
    }
}

/// Resource containing the placeholder material for uncolored cube faces
#[derive(Resource)]
pub struct PlaceholderMaterial(pub Handle<StandardMaterial>);

/// System to initialize the placeholder material resource
pub fn initialize_placeholder_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let placeholder_color = CubeColors::placeholder_color();
    let linear_color = placeholder_color.to_linear();
    let emissive_color = bevy::color::LinearRgba::new(
        linear_color.red * 0.3,
        linear_color.green * 0.3,
        linear_color.blue * 0.3,
        linear_color.alpha,
    );

    let material = materials.add(StandardMaterial {
        base_color: placeholder_color,
        emissive: emissive_color,
        metallic: 0.3,
        perceptual_roughness: 0.8,
        ..default()
    });

    commands.insert_resource(PlaceholderMaterial(material));
    info!("Initialized placeholder material resource");
}
