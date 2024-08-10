use bevy::math::Vec2;
use bevy::math::*;
use bevy::prelude::*;
use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    render::{mesh::*, render_asset::RenderAssetUsages},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    tasks::ComputeTaskPool,
};
#[derive(Component)]
struct Boid {
    velocity: Vec2,
    position: Vec2,
    acceleration: Vec2,
    max_speed: f32,
    max_force: f32,
    separation_weight: f32,
    alignment_weight: f32,
    cohesion_weight: f32,
    separation_radius: f32,
    alignment_radius: f32,
    cohesion_radius: f32,
}
#[derive(Component)]
pub struct Velocity(Vec2);

#[derive(Component)]
pub struct SpatialEntity;

#[derive(Bundle)]
pub struct BoidBundle {
    mesh: MaterialMesh2dBundle<ColorMaterial>,
    velocity: Velocity,
}

impl Default for BoidBundle {
    fn default() -> Self {
        Self {
            mesh: Default::default(),
            velocity: Velocity(Vec2::default()),
        }
    }
}
