
mod boid;

use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_spatial::{kdtree::KDTree2, SpatialAccess};
use bevy_spatial::{AutomaticUpdate, SpatialStructure};
use boid::*;
use halton::Sequence;
use rand::prelude::*;
use std::time::Duration;
pub const CLEAR: Color = Color::srgb(0.0, 0.0, 0.0);

pub struct nums {
    boid_count: i32,
    boid_speed: f32,
    boid_vis_range: f32,
    boid_prot_range: f32,
    boid_centering_factor: f32,
    boid_avoidance_factor: f32,
    boid_matching_factor: f32,

    boid_min_speed: f32,
    boid_max_speed: f32,

}
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Track boids in the KD-Tree
        .add_plugins(
            AutomaticUpdate::<SpatialEntity>::new()
                // TODO: check perf of other tree types
                .with_spatial_ds(SpatialStructure::KDTree2)
                .with_frequency(Duration::from_millis(16)),
        )
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
 commands.spawn(Camera2dBundle::default());

    let mut rng = rand::thread_rng();

    let seq = halton::Sequence::new(2).zip(Sequence::new(3)).zip(1..BOID_COUNT)
}
