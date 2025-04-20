use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Number of marbles along each axis (total = GRID^2)
const GRID: usize = 16;
const MARBLE_RADIUS: f32 = 1.0;
const MARBLE_SPACING: f32 = 5.0;
const CURSOR_GRAVITY: f32 = 50000.0;
const CURSOR_MASS: f32 = 10000.0;
const MARBLE_MASS: f32 = 1.0;
const MAX_MARBLE_VELOCITY: f32 = 500.0;

const HALF_WIDTH: f32 = (GRID as f32 * MARBLE_SPACING) / 2.0 + 700.0;
const HALF_HEIGHT: f32 = (GRID as f32 * MARBLE_SPACING) / 2.0 + 700.0;

const MAX_ACCELERATION: f32 = 4000.0;
const INTER_MARBLE_GRAVITY: f32 = 400000.0;

#[derive(PhysicsLayer, Default)]
enum GameLayer {
    #[default]
    Marble,
    Wall,
}

#[derive(Component)]
struct Marble;

fn main() {
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
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .add_systems(Startup, setup)
        .add_systems(Update, nbody_and_cursor_gravity)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let wall_layers = CollisionLayers::new(GameLayer::Wall, [GameLayer::Marble]);
    let wall_color = Color::srgb(0.4, 0.45, 0.6);
    let wall_thickness = 50.0;
    let wall_friction = Friction::new(0.98);
    let wall_bounce = Restitution::new(0.0);

    // Pre-create mesh & material
    let h_mesh = meshes.add(Rectangle::new(2.0 * HALF_WIDTH, wall_thickness));
    let v_mesh = meshes.add(Rectangle::new(wall_thickness, 2.0 * HALF_HEIGHT));
    let wall_material = materials.add(wall_color);

    // Ceiling
    commands.spawn((
        Mesh2d(h_mesh.clone().into()),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(0.0, HALF_HEIGHT, 0.0),
        RigidBody::Static,
        Collider::rectangle(2.0 * HALF_WIDTH, wall_thickness),
        wall_friction,
        wall_bounce,
        wall_layers,
        Name::new("Ceiling"),
    ));
    // Floor
    commands.spawn((
        Mesh2d(h_mesh.clone().into()),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(0.0, -HALF_HEIGHT, 0.0),
        RigidBody::Static,
        Collider::rectangle(2.0 * HALF_WIDTH, wall_thickness),
        wall_friction,
        wall_bounce,
        wall_layers,
        Name::new("Floor"),
    ));
    // Left wall
    commands.spawn((
        Mesh2d(v_mesh.clone().into()),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(-HALF_WIDTH, 0.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(wall_thickness, 2.0 * HALF_HEIGHT),
        wall_friction,
        wall_bounce,
        wall_layers,
        Name::new("LeftWall"),
    ));
    // Right wall
    commands.spawn((
        Mesh2d(v_mesh.clone().into()),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(HALF_WIDTH, 0.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(wall_thickness, 2.0 * HALF_HEIGHT),
        wall_friction,
        wall_bounce,
        wall_layers,
        Name::new("RightWall"),
    ));

    // Marbles
    let marble_mesh = meshes.add(Circle::new(MARBLE_RADIUS));
    let marble_material = materials.add(Color::srgb(0.2, 0.8, 1.0));
    let marble_layers = CollisionLayers::new(GameLayer::Marble, [GameLayer::Wall]);
    let x0 = -(GRID as f32) * MARBLE_SPACING / 2.0 + MARBLE_SPACING / 2.0;
    let y0 = -(GRID as f32) * MARBLE_SPACING / 2.0 + MARBLE_SPACING / 2.0;
    for i in 0..GRID {
        for j in 0..GRID {
            let x = x0 + i as f32 * MARBLE_SPACING;
            let y = y0 + j as f32 * MARBLE_SPACING;
            commands.spawn((
                Mesh2d(marble_mesh.clone().into()),
                MeshMaterial2d(marble_material.clone()),
                Transform::from_xyz(x, y, 1.0),
                RigidBody::Dynamic,
                Collider::circle(MARBLE_RADIUS),
                marble_layers,
                LinearVelocity::ZERO,
                Marble,
                Name::new(format!("Marble_{i}_{j}")),
            ));
        }
    }
}

fn attract_marbles_to_cursor(
    time: Res<Time>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut q_marbles: Query<(&GlobalTransform, &mut LinearVelocity), With<Marble>>,
) {
    if !mouse.pressed(MouseButton::Left) {
        return;
    }
    let window = q_windows.single();
    let (camera, camera_transform) = q_camera.single();

    if let Some(cursor_screen_pos) = window.cursor_position() {
        if let Ok(cursor_world) = camera.viewport_to_world_2d(camera_transform, cursor_screen_pos) {
            let dt = time.delta_secs();
            for (transform, mut velocity) in q_marbles.iter_mut() {
                let pos = transform.translation().truncate();
                let to_cursor = cursor_world - pos;
                let r2 = to_cursor.length_squared().max(1.0);
                let accel_mag = CURSOR_GRAVITY * CURSOR_MASS / r2;
                let accel =
                    (to_cursor.normalize_or_zero() * accel_mag).clamp_length_max(MAX_ACCELERATION);

                velocity.x += accel.x * dt;
                velocity.y += accel.y * dt;

                let speed2 = velocity.x * velocity.x + velocity.y * velocity.y;
                if speed2 > MAX_MARBLE_VELOCITY * MAX_MARBLE_VELOCITY {
                    let scale = MAX_MARBLE_VELOCITY / speed2.sqrt();
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
    mut q_marbles: Query<(&GlobalTransform, &mut LinearVelocity), With<Marble>>,
) {
    let (camera, camera_transform) = q_camera.single();

    // Get world cursor position (if available)
    let cursor_world = q_windows
        .single()
        .cursor_position()
        .and_then(|pos| camera.viewport_to_world_2d(camera_transform, pos).ok());

    // Gather all marble positions
    let marble_positions: Vec<Vec2> = q_marbles
        .iter_mut()
        .map(|(transform, _)| transform.translation().truncate())
        .collect();

    // Now for each marble, sum forces
    let dt = time.delta_secs();
    let mut i = 0;
    for (_, mut velocity) in q_marbles.iter_mut() {
        let pos_i = marble_positions[i];
        let mut accel = Vec2::ZERO;

        // Add attraction to every other marble
        for (j, pos_j) in marble_positions.iter().enumerate() {
            if i == j {
                continue;
            }
            let delta = *pos_j - pos_i;
            let r2 = delta.length_squared().max(1.0);
            let a = INTER_MARBLE_GRAVITY * MARBLE_MASS * MARBLE_MASS / r2;
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
        accel = accel.clamp_length_max(MAX_ACCELERATION);

        velocity.x += accel.x * dt;
        velocity.y += accel.y * dt;
        let speed2 = velocity.x * velocity.x + velocity.y * velocity.y;
        if speed2 > MAX_MARBLE_VELOCITY * MAX_MARBLE_VELOCITY {
            let scale = MAX_MARBLE_VELOCITY / speed2.sqrt();
            velocity.x *= scale;
            velocity.y *= scale;
        }
        i += 1;
    }
}
