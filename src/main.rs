use bevy::prelude::*;
use bevy_spatial::{AutomaticUpdate, SpatialStructure};
use std::time::Duration;
use Boids::boid::*;
use Boids::display_fps;
use Boids::update_fps_counter;
use Boids::Values;
use Boids::BOUNDS;
//
// NOTE: The below code is ALSO really important for a rust-wasm binary to work. I am stupid and
// did not realize this
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    (App::new().add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#bevy_boids_canvas".into()),
                resizable: true,
                ..default()
            }),
            ..default()
        }),
        // Track boids in the KD-Tree
        AutomaticUpdate::<SpatialEntity>::new()
            .with_spatial_ds(SpatialStructure::KDTree2)
            .with_frequency(Duration::from_millis(16)),
    )))
    .insert_resource(Values::default())
    .insert_resource(Time::<Fixed>::from_hz(60.0))
    .add_event::<DvEvent>()
    .add_systems(Startup, boid_setup)
    .add_systems(
        FixedUpdate,
        (velo_system, movement_system, flocking_system, display_fps),
    )
    .add_systems(Update, update_fps_counter)
    //.add_systems(Update, ui_system)
    .run();
}
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    //eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    (App::new().add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#bevy_boids_canvas".into()),
                resizable: true,
                ..default()
            }),
            ..default()
        }),
        // Track boids in the KD-Tree
        AutomaticUpdate::<SpatialEntity>::new()
            .with_spatial_ds(SpatialStructure::KDTree2)
            .with_frequency(Duration::from_millis(16)),
    )))
    .insert_resource(Values::default())
    .insert_resource(Time::<Fixed>::from_hz(60.0))
    .add_event::<DvEvent>()
    .add_systems(Startup, boid_setup)
    //.add_systems(Update, update_fps_counter)
    .add_systems(
        FixedUpdate,
        (
            velo_system,
            movement_system,
            flocking_system,
        ),
    )
    //.add_systems(Update, ui_system)
    .run();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn start() {
    main();
}
