use bevy::prelude::bevy_main;

/// iOS app entry point
#[bevy_main]
pub fn main() {
    // Initialize iOS logging using oslog
    oslog::OsLogger::new("com.example.cube-solver")
        .level_filter(log::LevelFilter::Debug)
        .init()
        .expect("Failed to initialize logger");

    log::info!("3x3x3 Cube Solver iOS App is starting");

    let mut app = cube_solver::app::create_app();
    app.run();
}

// Export symbols needed by iOS
#[unsafe(no_mangle)]
pub extern "C" fn start_bevy_app() {
    main();
}
