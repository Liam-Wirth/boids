use crate::Values;
use crate::BOUNDS;
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy::tasks::ComputeTaskPool;
use bevy_spatial::kdtree::KDTree2;
use bevy_spatial::SpatialAccess;
use halton::Sequence;
use rand::Rng;

#[derive(Component)]
pub struct Velocity(Vec2);

#[derive(Component)]
pub struct SpatialEntity;

#[derive(Bundle)]
pub struct BoidBundle {
    mesh: MaterialMesh2dBundle<ColorMaterial>,
    velocity: Velocity,
}

#[derive(Event)]
pub struct DvEvent(Entity, Vec2);

impl Default for BoidBundle {
    fn default() -> Self {
        Self {
            mesh: Default::default(),
            velocity: Velocity(Vec2::default()),
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

        commands.spawn((
            BoidBundle {
                mesh: MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Circle { radius: 4.0 })),
                    material: materials.add(Color::hsl(360. * rng.gen::<f32>(), rng.gen(), 0.7)),
                    transform,
                    ..default()
                },
                velocity,
            },
            SpatialEntity,
        ));
    }
}

fn get_dv(
    kdtree: &Res<KDTree2<SpatialEntity>>,
    boid_query: &Query<(Entity, &Velocity, &Transform), With<SpatialEntity>>,
    camera: &Query<(&Camera, &GlobalTransform)>,
    window: &Query<&Window>,
    boid: &Entity,
    t0: &&Transform,
    values: &Res<Values>,
) -> Vec2 {
    // https://vanhunteradams.com/Pico/Animal_Movement/Boids-algorithm.html
    let mut dv = Vec2::default();
    let mut vec_away = Vec2::default();
    let mut avg_position = Vec2::default();
    let mut avg_velocity = Vec2::default();
    let mut neighboring_boids = 0;
    let mut close_boids = 0;

    for (_, entity) in kdtree.k_nearest_neighbour(t0.translation.xy(), values.max_neighbors) {
        let Ok((other, v1, t1)) = boid_query.get(entity.unwrap()) else {
            todo!()
        };

        // Don't evaluate against itself
        if *boid == other {
            continue;
        }

        let vec_to = (t1.translation - t0.translation).xy();
        let dist_sq = vec_to.x * vec_to.x + vec_to.y * vec_to.y;

        // Don't evaluate boids out of range
        if dist_sq > values.vis_range_sq {
            continue;
        }

        // Don't evaluate boids behind
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
            // separation
            vec_away -= vec_to;
            close_boids += 1;
        } else {
            // cohesion
            avg_position += vec_to;
            // alignment
            avg_velocity += v1.0;
            neighboring_boids += 1;
        }
    }

    if neighboring_boids > 0 {
        let neighbors = neighboring_boids as f32;
        dv += avg_position / neighbors * values.boid_centering_factor;
        dv += avg_velocity / neighbors * values.boid_matching_factor;
    }

    if close_boids > 0 {
        let close = close_boids as f32;
        dv += vec_away / close * values.boid_avoidance_factor;
    }

    // Chase the mouse
    let (camera, t_camera) = camera.single();
    if let Some(c_window) = window.single().cursor_position() {
        if let Some(c_world) = camera.viewport_to_world_2d(t_camera, c_window) {
            let to_cursor = c_world - t0.translation.xy();
            dv += to_cursor * values.boid_mouse_chase_factor;
        };
    };

    dv
}

pub fn flocking_system(
    boid_query: Query<(Entity, &Velocity, &Transform), With<SpatialEntity>>,
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

                for (boid, _, t0) in chunk {
                    dv_batch.push(DvEvent(
                        *boid,
                        get_dv(kdtree, boid_query, camera, window, boid, t0, values),
                    ));
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

        let res = &window.single().resolution;

        let width = (res.width() - values.boid_bound_size) / 2.;
        let height = (res.height() - values.boid_bound_size) / 2.;

        if values.is_toroidal {
            // TODO:
        } else {
            // Steer back into visible region
            if transform.translation.x < -width {
                velocity.0.x += values.boid_turn_factor;
            }
            if transform.translation.x > width {
                velocity.0.x -= values.boid_turn_factor;
            }
            if transform.translation.y < -height {
                velocity.0.y += values.boid_turn_factor;
            }
            if transform.translation.y > height {
                velocity.0.y -= values.boid_turn_factor;
            }
        }

        // Clamp speed
        let speed = velocity.0.length();

        if speed < values.boid_min_speed {
            velocity.0 *= values.boid_min_speed / speed;
        }
        if speed > values.boid_max_speed {
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
