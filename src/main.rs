mod boid;

use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use std::sync::Arc;
use bevy_spatial::{kdtree::KDTree2, SpatialAccess};
use bevy_spatial::{AutomaticUpdate, SpatialStructure};
use boid::*;
use halton::Sequence;
use rand::prelude::*;
use std::time::Duration;
pub const CLEAR: Color = Color::srgb(0.0, 0.0, 0.0);

pub const BOUNDS: Vec2 = Vec2::new(800.0, 400.0);

use boid::SpatialEntity;
use boid::*;

#[derive(Resource, Copy, Clone)]
pub struct Values {
    /// Number of boids to spawn
    boid_count: i32,
    /// Size of the boids
    boid_size: f32,
    /// Speed of the boids
    boid_speed: f32,
    ///Maximum number of neighbors a boid can have
    max_neighbors: usize,
    /// Visibility range of the boids, determines how far away a boid can see another boid
    boid_vis_range: f32,
    /// Protection range of the boids, determines how far away a boid can see a predator/obstacle
    /// or a boid that is determined to be "too close"
    boid_prot_range: f32,
    /// Factor/amount that the boids want to center around the center of mass of the boids
    boid_centering_factor: f32,
    /// Factor/amount that the boids want to avoid each other
    boid_avoidance_factor: f32,
    /// Factor/amount that the boids want to match the velocity of the boids around them
    boid_matching_factor: f32,
    /// Minimum speed of the boids
    boid_min_speed: f32,
    /// Maximum speed of the boids
    boid_max_speed: f32,
    ///boid field of view
    boid_fov: f32,

    vis_range_sq: f32,
    prot_range_sq: f32,

    boid_mouse_chase_factor: f32,
    boid_bound_size: f32,
    boid_turn_factor: f32,

    is_toroidal: bool,
}

impl Default for Values {
    fn default() -> Self {
        Self {
            boid_count: 1000,
            boid_size: 0.27,
            boid_speed: 5.,
            max_neighbors: 100,
            boid_vis_range: 35.0, 
            boid_fov: 120.0 * std::f32::consts::PI / 180.0,
            boid_bound_size: 98.0,
            boid_turn_factor: 0.5,
            boid_prot_range: 10.0,
            boid_centering_factor: 0.0008,
            boid_mouse_chase_factor: 0.0006,
            boid_avoidance_factor: 0.05,
            boid_matching_factor: 0.05,
            boid_min_speed: 5.,
            boid_max_speed: 10.,

            vis_range_sq: 35.0 * 35.0,
            prot_range_sq: 10.0 * 10.0,
            is_toroidal: true,
        }
    }
}

fn main() {
    (App::new()
            .add_plugins((
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
        .add_systems(FixedUpdate, (velo_system, movement_system, flocking_system))
        .run();
}
