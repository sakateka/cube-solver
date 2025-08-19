use bevy::prelude::bevy_main;

/// Android app entry point
#[bevy_main]
pub fn main() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("cube_solver")
            .with_filter(
                android_logger::FilterBuilder::new()
                    .filter_module("cube_android", log::LevelFilter::Debug)
                    .filter_module("cube_solver", log::LevelFilter::Debug)
                    .filter_level(log::LevelFilter::Info) // Allow info and above from other modules
                    .build(),
            ),
    );

    log::info!("3x3x3 Cube Solver Android App is starting");

    let mut app = cube_solver::app::create_app();
    app.run();
}
