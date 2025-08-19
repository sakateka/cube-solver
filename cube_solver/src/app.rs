use bevy::color::palettes::css;
use bevy::{asset::load_internal_binary_asset, prelude::*};

use crate::camera::setup_camera_and_lighting;
use crate::cube::create_cube;
use crate::cube_moves::CubeMoveEvent;
use crate::input::handle_touch;
use crate::layer_rotation::LayerRotationPlugin;
use crate::selection::{SelectionPlugin, detect_touch_selection};
use crate::solver_integration::{
    CubeSolverResource, lightweight_validation_on_recolor,
    lightweight_validation_on_rotation_complete, update_solver_state,
};
use crate::ui::color_panel::{
    create_ui_color_panel, handle_color_button_clicks, update_color_button_selection,
    update_color_count_labels, update_color_text_colors,
};
use crate::ui::move_test::MoveTestPlugin;
use crate::ui::navigation::{
    handle_navigation_next_button_clicks, handle_navigation_prev_button_clicks,
    update_navigation_buttons,
};
use crate::ui::rotations_panel::RotationsPanelPlugin;
use crate::ui::solve::{
    create_solve_button, handle_solution_move_completion, handle_solve_button_clicks,
    update_solve_button,
};

/// Create the Bevy app with common configuration
pub fn create_app() -> App {
    log::info!("3x3x3 Cube Solver is starting");

    let mut app = App::new();

    // Add full 3D plugins with PBR
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "3x3x3 Cube Solver".into(),
                    resolution: (800.0, 600.0).into(),
                    present_mode: bevy::window::PresentMode::Fifo,
                    ..default()
                }),
                ..default()
            })
            .set(bevy::log::LogPlugin {
                level: bevy::log::Level::DEBUG,
                filter: "cube_solver=debug,wgpu=error,naga=error".to_string(),
                ..default()
            }),
    )
    .add_plugins(RotationsPanelPlugin)
    .add_plugins(MoveTestPlugin)
    .add_plugins(SelectionPlugin);

    // Add color manager and solver resources
    app.init_resource::<crate::components::ColorManager>();
    app.init_resource::<CubeSolverResource>();

    // Add cube move events
    app.add_event::<CubeMoveEvent>();

    // Dark background
    app.insert_resource(ClearColor(css::MIDNIGHT_BLUE.into()));

    // Add our 3D systems and UI systems
    app.add_systems(
        Startup,
        (
            crate::colors::initialize_placeholder_material,
            setup_camera_and_lighting,
            create_cube,
            create_ui_color_panel,
            create_solve_button,
        )
            .chain(),
    );

    // Add debug system to create facelet dots
    app.add_systems(
        Update,
        (
            // UI systems run first to process interactions
            (
                handle_color_button_clicks,
                handle_solve_button_clicks,
                handle_navigation_next_button_clicks,
                handle_navigation_prev_button_clicks,
                update_color_button_selection,
                update_color_count_labels,
                update_color_text_colors,
            ),
            // Button update systems run separately to avoid query conflicts
            (update_solve_button, update_navigation_buttons),
            // 3D input systems and others
            handle_touch.before(detect_touch_selection),
            update_solver_state,
            lightweight_validation_on_recolor,
            lightweight_validation_on_rotation_complete,
            handle_solution_move_completion,
        ),
    )
    .add_plugins(LayerRotationPlugin);

    // This needs to happen after `DefaultPlugins` is added.
    load_internal_binary_asset!(
        app,
        Handle::default(),
        "../assets/fonts/Roboto-Regular.ttf",
        |bytes: &[u8], _path: String| { Font::try_from_bytes(bytes.to_vec()).unwrap() }
    );

    app
}
