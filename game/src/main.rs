mod hud;
mod setup;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;
use std::ops::AddAssign;

use game_macros::{FieldIter, key_name};

/// Number of particles along each axis (total = GRID^2)
const SPAWN_GRID_WIDTH: usize = 16;
const PARTICLE_RADIUS: f32 = 2.0;
const PARTICLE_SPAWN_SPACING: f32 = 5.0;
const CURSOR_GRAVITY: f32 = 50000.0;
const CURSOR_MASS: f32 = 10000.0;
const PARTICLE_MASS: f32 = 1.0;
const MAX_PARTICLE_VELOCITY: f32 = 500.0;

const HALF_WIDTH: f32 = (SPAWN_GRID_WIDTH as f32 * PARTICLE_SPAWN_SPACING) / 2.0 + 700.0;
const HALF_HEIGHT: f32 = (SPAWN_GRID_WIDTH as f32 * PARTICLE_SPAWN_SPACING) / 2.0 + 700.0;

#[derive(Debug, Eq, PartialEq, Hash)]
enum PName {
    MAX_ACCELERATION,
    INTER_PARTICLE_GRAVITY,
}

struct PhysicsManipulable {
    key_increase: KeyCode,
    key_decrease: KeyCode,
    delta: f32,
    value: f32,
    name: PName,
}

impl PhysicsManipulable {
    pub fn handle_keys(&mut self, keys: &Res<ButtonInput<KeyCode>>) {
        if keys.pressed(self.key_increase) {
            self.value += self.delta;
        }
        if keys.pressed(self.key_decrease) {
            self.value -= self.delta;
        }
    }
}

#[derive(Resource)]
struct Physics(HashMap<PName, PhysicsManipulable>);

impl Physics {
    pub fn get_value(&self, name: PName) -> Option<f32> {
        if let Some(e) = self.0.get(&name) {
            Some(e.value)
        } else {
            None
        }
    }
    pub fn get(&self, name: PName) -> Option<&PhysicsManipulable> {
        self.0.get(&name)
    }
    pub fn get_mut(&mut self, name: PName) -> Option<&mut PhysicsManipulable> {
        self.0.get_mut(&name)
    }
}

impl Default for Physics {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(
            PName::MAX_ACCELERATION,
            PhysicsManipulable {
                name: PName::MAX_ACCELERATION,
                value: 4000.0,
                key_increase: KeyCode::KeyM,
                key_decrease: KeyCode::KeyN,
                delta: 100.0,
            },
        );
        map.insert(
            PName::INTER_PARTICLE_GRAVITY,
            PhysicsManipulable {
                name: PName::INTER_PARTICLE_GRAVITY,
                value: 400000.0,
                key_increase: KeyCode::KeyG,
                key_decrease: KeyCode::KeyF,
                delta: 10000.0,
            },
        );
        Self(map)
    }
}

trait UpdateHud {
    fn update_hud(&self, hud_text: &mut Text);
}

impl UpdateHud for Physics {
    fn update_hud(&self, hud_text: &mut Text) {
        **hud_text = self
            .0
            .iter()
            .map(|kv| {
                let (_, val) = kv;
                format!(
                    "{:?}: {}\n{:?} to increase, {:?} to decrease",
                    val.name, val.value, val.key_increase, val.key_decrease
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
    }
}

#[derive(PhysicsLayer, Default)]
enum GameLayer {
    #[default]
    Particle,
    Wall,
}

#[derive(Component)]
struct Particle;

#[derive(FieldIter)]
struct Foo {
    bar: String,
    baz: i8,
}

impl Default for Foo {
    fn default() -> Self {
        Self {
            bar: "Heyo".to_string(),
            baz: 0,
        }
    }
}

fn main() {
    // let foo = Foo::default();
    // for asdf in foo.iter_fields() {
    //     println!("{:?}", asdf);
    // }
    //

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Particle Attraction Simulation".into(),
                    resolution: (1600., 1000.).into(),
                    ..default()
                }),
                ..default()
            }),
            // PhysicsDebugPlugin::default(),
            PhysicsPlugins::default()
                .with_length_unit(10.0)
                .set(PhysicsInterpolationPlugin::interpolate_all()),
        ))
        .insert_resource(Gravity::ZERO)
        .insert_resource(Physics::default())
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .add_systems(Startup, setup::setup)
        .add_systems(
            Update,
            (
                nbody_and_cursor_gravity,
                modify_physics_constants,
                hud::update_hud,
            ),
        )
        .run();
}

impl AddAssign<f32> for &mut PhysicsManipulable {
    fn add_assign(&mut self, rhs: f32) {
        self.value += rhs;
    }
}

fn modify_physics_constants(
    keys: Res<ButtonInput<KeyCode>>,
    mut physics_constants: ResMut<Physics>,
) {
    physics_constants
        .0
        .iter_mut()
        .for_each(|(_, p)| p.handle_keys(&keys))
}

fn attract_particles_to_cursor(
    time: Res<Time>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    physics_constants: ResMut<Physics>,
    mut q_particles: Query<(&GlobalTransform, &mut LinearVelocity), With<Particle>>,
) {
    if !mouse.pressed(MouseButton::Left) {
        return;
    }
    let window = q_windows.single();
    let (camera, camera_transform) = q_camera.single();

    if let Some(cursor_screen_pos) = window.cursor_position() {
        if let Ok(cursor_world) = camera.viewport_to_world_2d(camera_transform, cursor_screen_pos) {
            let dt = time.delta_secs();
            for (transform, mut velocity) in q_particles.iter_mut() {
                let pos = transform.translation().truncate();
                let to_cursor = cursor_world - pos;
                let r2 = to_cursor.length_squared().max(1.0);
                let accel_mag = CURSOR_GRAVITY * CURSOR_MASS / r2;
                let accel = (to_cursor.normalize_or_zero() * accel_mag)
                    .clamp_length_max(physics_constants.get_value(PName::MAX_ACCELERATION).unwrap());

                velocity.x += accel.x * dt;
                velocity.y += accel.y * dt;

                let speed2 = velocity.x * velocity.x + velocity.y * velocity.y;
                if speed2 > MAX_PARTICLE_VELOCITY * MAX_PARTICLE_VELOCITY {
                    let scale = MAX_PARTICLE_VELOCITY / speed2.sqrt();
                    velocity.x *= scale;
                    velocity.y *= scale;
                }
            }
        }
    }
}

fn nbody_and_cursor_gravity(
    time: Res<Time>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    physics_constants: ResMut<Physics>,
    mut q_particles: Query<(&GlobalTransform, &mut LinearVelocity), With<Particle>>,
) {
    let (camera, camera_transform) = q_camera.single();

    // Get world cursor position (if available)
    let cursor_world = q_windows
        .single()
        .cursor_position()
        .and_then(|pos| camera.viewport_to_world_2d(camera_transform, pos).ok());

    // Gather all particle positions
    let particle_positions: Vec<Vec2> = q_particles
        .iter_mut()
        .map(|(transform, _)| transform.translation().truncate())
        .collect();

    // Now for each particle, sum forces
    let dt = time.delta_secs();
    let mut i = 0;
    for (_, mut velocity) in q_particles.iter_mut() {
        let pos_i = particle_positions[i];
        let mut accel = Vec2::ZERO;

        // Add attraction to every other particle
        for (j, pos_j) in particle_positions.iter().enumerate() {
            if i == j {
                continue;
            }
            let delta = *pos_j - pos_i;
            let r2 = delta.length_squared().max(1.0);
            let a = physics_constants.get(PName::INTER_PARTICLE_GRAVITY).unwrap().value * PARTICLE_MASS * PARTICLE_MASS / r2;
            accel += delta.normalize_or_zero() * a;
        }

        // Add attraction to cursor (if present)
        if let Some(cursor_world) = cursor_world {
            let to_cursor = cursor_world - pos_i;
            let r2 = to_cursor.length_squared().max(1.0);
            let a = CURSOR_GRAVITY * CURSOR_MASS / r2;
            accel += to_cursor.normalize_or_zero() * a;
        }

        // Clamp acceleration and velocity for stability
        accel = accel.clamp_length_max(physics_constants.get_value(PName::MAX_ACCELERATION).unwrap());

        velocity.x += accel.x * dt;
        velocity.y += accel.y * dt;
        let speed2 = velocity.x * velocity.x + velocity.y * velocity.y;
        if speed2 > MAX_PARTICLE_VELOCITY * MAX_PARTICLE_VELOCITY {
            let scale = MAX_PARTICLE_VELOCITY / speed2.sqrt();
            velocity.x *= scale;
            velocity.y *= scale;
        }
        i += 1;
    }
}
