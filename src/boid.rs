use crate::Values;
use crate::BOUNDS;
use bevy::math::Vec2;
use bevy::math::Vec3;
use bevy::math::*;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy::tasks::ComputeTaskPool;
use bevy_egui::egui::epaint::color;
use bevy_spatial::kdtree::KDTree2;
use bevy_spatial::SpatialAccess;
use halton::Sequence;
use rand::Rng;

#[derive(Component)]
pub struct Velocity(Vec2);

#[derive(Component)]
pub struct SpatialEntity;

#[derive(Component)]
pub struct SimpleColor(Vec3); // Stored as a vec3 cause it's lighter than a Color object

impl Default for SimpleColor {
    fn default() -> Self {
        SimpleColor(Vec3::ZERO)
    }
}

impl SimpleColor {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        SimpleColor((360. * rng.gen::<f32>(), rng.gen(), 0.7).into())
    }
}
#[derive(Bundle)]
pub struct BoidBundle {
    mesh: MaterialMesh2dBundle<ColorMaterial>,
    velocity: Velocity,
    start_color: SimpleColor,
}

#[derive(Event)]
pub struct DvEvent(Entity, Vec2);

#[derive(Event)]
pub struct ColorEvent(Entity, Vec3);

impl Default for BoidBundle {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            mesh: Default::default(),
            velocity: Velocity(Vec2::default()),
            start_color: SimpleColor::random(),
        }
    }
}

pub fn boid_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    values: Res<Values>,
) {
    commands.spawn(Camera2dBundle::default());
    let mut rng = rand::thread_rng();
    let seq = halton::Sequence::new(2)
        .zip(Sequence::new(3))
        .zip(1..values.boid_count);

    for ((x, y), _) in seq {
        let spawn_x = (x as f32 * BOUNDS.x) - BOUNDS.x / 2.0;
        let spawn_y = (y as f32 * BOUNDS.y) - BOUNDS.y / 2.0;
        let mut transform =
            Transform::from_xyz(spawn_x, spawn_y, 0.0).with_scale(Vec3::splat(values.boid_size)); // Fixed typo here
        transform.rotate_z(0.0);

        let velocity = Velocity(
            Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)) * values.boid_speed,
        );

        let start = SimpleColor::default();
        let color = Color::hsl(start.0.x, start.0.y, start.0.z);
        commands.spawn((
            BoidBundle {
                mesh: MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Circle { radius: 4.0 })),
                    //material: materials.add(Color::hsl(360. * rng.gen::<f32>(), rng.gen(), 0.7)),
                    material: materials.add(color),
                    transform,
                    ..default()
                },
                velocity,
                start_color: start,
            },
            SpatialEntity,
        ));
    }
}

/**
* @param kdtree: KDTree2<SpatialEntity> - The KDTree of all boids
* @param boid_query: Query<(Entity, &Velocity, &Transform), With<SpatialEntity>> - Query of all
* boids
* @param camera: Query<(&Camera, &GlobalTransform)> - Query of the camera
* @param window: Query<&Window> - Query of the window
* @param boid: &Entity - The entity of the boid
* @param t0: &&Transform - The transform of the boid
* @param values: &Res<Values> - The values resource
* @return Vec2 - The delta velocity
* @description Get the delta velocity for a boid, this is where all the real logic of the boids and
* stuff takes place
*
*/
fn get_dv(
    kdtree: &Res<KDTree2<SpatialEntity>>,
    boid_query: &Query<
        (
            Entity,
            &Velocity,
            &Transform,
            &Handle<ColorMaterial>,
            &SimpleColor,
        ),
        With<SpatialEntity>,
    >,
    camera: &Query<(&Camera, &GlobalTransform)>,
    window: &Query<&Window>,
    boid: &Entity,
    t0: &&Transform,
    values: &Res<Values>,
) -> (Vec2, Vec3) {
    let mut dv = Vec2::default();
    let mut vec_away = Vec2::default();
    let mut avg_position = Vec2::default();
    let mut avg_velocity = Vec2::default();
    let mut neighboring_boids = 0;
    let mut close_boids = 0;
    let mut total_color = Vec3::ZERO;

    let (_, _, _, _, start_color) = boid_query.get(*boid).unwrap();
    let mut final_color = start_color.0;

    for (_, entity) in kdtree.k_nearest_neighbour(t0.translation.xy(), values.max_neighbors) {
        let Ok((other, v1, t1, _, other_color)) = boid_query.get(entity.unwrap()) else {
            continue;
        };

        if *boid == other {
            continue;
        }

        let vec_to = (t1.translation - t0.translation).xy();
        let dist_sq = vec_to.length_squared();

        if dist_sq > values.vis_range_sq {
            continue;
        }

        if let Some(vec_to_norm) = vec_to.try_normalize() {
            if t0
                .rotation
                .angle_between(Quat::from_rotation_arc_2d(Vec2::X, vec_to_norm))
                > values.boid_fov
            {
                continue;
            }
        }

        if dist_sq < values.prot_range_sq {
            vec_away -= vec_to;
            close_boids += 1;
        } else {
            avg_position += vec_to;
            avg_velocity += v1.0;
            neighboring_boids += 1;
            total_color += other_color.0;
        }
    }

    if neighboring_boids > 0 {
        let neighbors = neighboring_boids as f32;
        dv += avg_position / neighbors * values.boid_centering_factor;
        dv += avg_velocity / neighbors * values.boid_matching_factor;

        // Color blending
        let avg_color = total_color / neighbors;
        final_color = final_color.lerp(avg_color, 0.1); // Adjust 0.1 to control blending speed
    } else {
        // Revert to start color when alone
        final_color = final_color.lerp(start_color.0, 0.05); // Adjust 0.05 to control reversion speed
    }

    if close_boids > 0 {
        let close = close_boids as f32;
        dv += vec_away / close * values.boid_avoidance_factor;
    }

    // Mouse chasing logic (unchanged)
    let (camera, t_camera) = camera.single();
    if let Some(c_window) = window.single().cursor_position() {
        if let Some(c_world) = camera.viewport_to_world_2d(t_camera, c_window) {
            let to_cursor = c_world - t0.translation.xy();
            if !values.modes.mouse_predator {
                dv += to_cursor * values.boid_mouse_chase_factor;
            } else {
                dv -= to_cursor * values.boid_mouse_chase_factor;
            }
        };
    };

    (dv, final_color)
}
/**
* @param boid_query: Query<(Entity, &Velocity, &Transform), With<SpatialEntity>> - Query of all
* boids
* @param kdtree: Res<KDTree2<SpatialEntity> - The KDTree of all boids
* @param dv_event_writer: EventWriter<DvEvent> - The event writer for the delta velocity events
* @param camera: Query<(&Camera, &GlobalTransform)> - Query of the camera
* @param window: Query<&Window> - Query of the window
* @param values: Res<Values> - The values resource
* @description The "parent" system for the boids, this is where the boids are updated as well as where
* the threads are spawned/managed
*
*/
pub fn flocking_system(
    boid_query: Query<
        (
            Entity,
            &Velocity,
            &Transform,
            &Handle<ColorMaterial>,
            &SimpleColor,
        ),
        With<SpatialEntity>,
    >,
    kdtree: Res<KDTree2<SpatialEntity>>,
    mut dv_event_writer: EventWriter<DvEvent>,
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window>,
    values: Res<Values>,
) {
    let pool = ComputeTaskPool::get();
    let boids = boid_query.iter().collect::<Vec<_>>();
    let boids_per_thread = (boids.len() + pool.thread_num() - 1) / pool.thread_num();

    // https://docs.rs/bevy/latest/bevy/tasks/struct.ComputeTaskPool.html
    // https://github.com/kvietcong/rusty-boids
    for batch in pool.scope(|s| {
        for chunk in boids.chunks(boids_per_thread) {
            let kdtree = &kdtree;
            let boid_query = &boid_query;
            let camera = &camera;
            let window = &window;
            let values = &values;

            s.spawn(async move {
                let mut dv_batch: Vec<DvEvent> = vec![];
                let mut color_batch: Vec<ColorEvent> = vec![];
                for (boid, _, t0, _, _) in chunk {
                    //dv_batch.push(DvEvent(
                    //    *boid,
                    //    get_dv(kdtree, boid_query, camera, window, boid, t0, values),
                    let (dv, new_color) =
                        get_dv(kdtree, boid_query, camera, window, boid, t0, values);

                    dv_batch.push(DvEvent(*boid, dv));
                    color_batch.push(ColorEvent(*boid, new_color)); //Jeez this is uggly
                }
                dv_batch
            });
        }
    }) {
        dv_event_writer.send_batch(batch);
    }
}

pub fn velo_system(
    mut events: EventReader<DvEvent>,
    mut boids: Query<(&mut Velocity, &mut Transform)>,
    window: Query<&Window>,
    values: Res<Values>,
) {
    for DvEvent(boid, dv) in events.read() {
        let Ok((mut velocity, transform)) = boids.get_mut(*boid) else {
            todo!()
        };

        velocity.0 += *dv;

        let window = window.single();
        let width = ((window.width() - values.boid_bound_size) / 2.) as i32;
        let height = ((window.height() - values.boid_bound_size) / 2.) as i32;

        if values.modes.toroidal {
            // TODO:
        } else {
            let pos_x = transform.translation.x as i32;
            let pos_y = transform.translation.y as i32;
            // Steer back into visible region
            if pos_x < -width {
                velocity.0.x += values.boid_turn_factor;
            }
            if pos_x > width {
                velocity.0.x -= values.boid_turn_factor;
            }
            if pos_y < -height {
                velocity.0.y += values.boid_turn_factor;
            }
            if pos_y > height {
                velocity.0.y -= values.boid_turn_factor;
            }
        }

        // Clamp speed
        let speed = velocity.0.length();

        if speed < values.boid_min_speed {
            velocity.0 *= values.boid_min_speed / speed;
        } else {
            velocity.0 *= values.boid_max_speed / speed;
        }
    }
}
pub fn movement_system(mut query: Query<(&mut Velocity, &mut Transform)>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.rotation = Quat::from_axis_angle(Vec3::Z, steer_to(Vec2::ZERO, velocity.0));
        transform.translation.x += velocity.0.x;
        transform.translation.y += velocity.0.y;
    }
}
fn steer_to(a: Vec2, b: Vec2) -> f32 {
    // https://stackoverflow.com/a/68929139
    let dir = b - a;
    dir.y.atan2(dir.x)
}

// function that when called, will update the color of the current boid to be a bit closer to the
// average color of the boids in it's neighborhood, if the neighborhood is less than a certain
// size, it will slowly start to revert that change back to it's start_color

pub fn setup_bounds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Create a rectangular border
    let border_thickness = 2.0;

    // Top border
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(BOUNDS.x, border_thickness)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, BOUNDS.y / 2.0, 0.0)),
        ..default()
    });

    // Bottom border
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(BOUNDS.x, border_thickness)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, -BOUNDS.y / 2.0, 0.0)),
        ..default()
    });

    // Left border
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(border_thickness, BOUNDS.y)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(-BOUNDS.x / 2.0, 0.0, 0.0)),
        ..default()
    });

    // Right border
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(border_thickness, BOUNDS.y)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(BOUNDS.x / 2.0, 0.0, 0.0)),
        ..default()
    });
}

pub fn color_change_system(
    mut color_events: EventReader<ColorEvent>,
    mut boids: Query<(&mut SimpleColor, &Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for ColorEvent(boid, new_color) in color_events.read() {
        if let Ok((mut simple_color, material_handle)) = boids.get_mut(*boid) {
            simple_color.0 = *new_color;
            if let Some(material) = materials.get_mut(material_handle) {
                material.color = Color::hsl(new_color.x, new_color.y, new_color.z);
            }
        }
    }
}
