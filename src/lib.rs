use bevy::prelude::*;
use wasm_bindgen::prelude::*;
pub mod boid;

pub const CLEAR: Color = Color::srgb(0.0, 0.0, 0.0);

#[cfg(not(target_arch = "wasm32"))]
pub const BOUNDS: Vec2 = Vec2::new(800.0, 400.0);

#[cfg(target_arch = "wasm32")]
pub const BOUNDS: Vec2 = Vec2::new(400.0, 200.0);

#[derive(Resource, Copy, Clone)]
pub struct Values {
    /// Number of boids to spawn
    pub boid_count: i32,
    /// Size of the boids
    pub boid_size: f32,
    /// Speed of the boids
    pub boid_speed: f32,
    ///Maximum number of neighbors a boid can have
    pub max_neighbors: usize,
    /// Visibility range of the boids, determines how far away a boid can see another boid
    pub boid_vis_range: f32,
    /// Protection range of the boids, determines how far away a boid can see a predator/obstacle
    /// or a boid that is determined to be "too close"
    pub boid_prot_range: f32,
    /// Factor/amount that the boids want to center around the center of mass of the boids
    pub boid_centering_factor: f32,
    /// Factor/amount that the boids want to avoid each other
    pub boid_avoidance_factor: f32,
    /// Factor/amount that the boids want to match the velocity of the boids around them
    pub boid_matching_factor: f32,
    /// Minimum speed of the boids
    pub boid_min_speed: f32,
    /// Maximum speed of the boids
    pub boid_max_speed: f32,
    ///boid field of view
    pub boid_fov: f32,

    pub vis_range_sq: f32,
    pub prot_range_sq: f32,

    pub boid_mouse_chase_factor: f32,
    pub boid_bound_size: f32,
    pub boid_turn_factor: f32,

    pub is_toroidal: bool,
}

impl Default for Values {
    #[cfg(not(target_arch = "wasm32"))]
    fn default() -> Self {
        Self {
            boid_count: 500,
            boid_size: 0.4,
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
            is_toroidal: false,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn default() -> Self {
        Self {
            boid_count: 50,
            boid_size: 0.7,
            boid_speed: 2.5,
            max_neighbors: 50,
            boid_vis_range: 25.0,
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
            vis_range_sq: 25.0 * 25.0, // Updated to match new boid_vis_range
            prot_range_sq: 10.0 * 10.0,
            is_toroidal: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct FpsCounter {
    fps: f64,
}

pub fn update_fps_counter(time: Res<Time>, mut fps_counter: ResMut<FpsCounter>) {
    fps_counter.fps = 1.0 / time.delta_seconds_f64();
}

pub fn display_fps(
    mut commands: Commands,
    fps_counter: Res<FpsCounter>,
    mut query: Query<Entity, With<FpsText>>,
) {
    let fps_text = format!("FPS: {:.2}", fps_counter.fps);

    if let Ok(entity) = query.get_single_mut() {
        commands.entity(entity).despawn();
    }

    commands.spawn((
        TextBundle::from_section(
            fps_text,
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        FpsText,
    ));
}

#[derive(Component)]
pub struct FpsText;
