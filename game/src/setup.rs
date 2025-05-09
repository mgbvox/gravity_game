use crate::{GameLayer, Particle, Physics, HALF_HEIGHT, HALF_WIDTH, PARTICLE_RADIUS, PARTICLE_SPAWN_SPACING, SPAWN_GRID_WIDTH};
use avian2d::collision::{Collider, CollisionLayers};
use avian2d::prelude::{Friction, LinearVelocity, Restitution, RigidBody};
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::core::Name;
use bevy::prelude::{Camera2d, Circle, ColorMaterial, Commands, Mesh, Mesh2d, MeshMaterial2d, Query, Rectangle, Res, ResMut, Transform, Window, With};
use bevy::window::PrimaryWindow;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    physics_constants: Res<Physics>,
) {
    commands.spawn(Camera2d);

    let window = q_windows.single();

    crate::hud::spawn_hud(&mut commands, &window, physics_constants);

    // let wall_layers = CollisionLayers::new(GameLayer::Wall, [GameLayer::Particle]);
    // let wall_color = Color::srgb(0.4, 0.45, 0.6);
    // let wall_thickness = 50.0;
    // let wall_friction = Friction::new(0.98);
    // let wall_bounce = Restitution::new(0.0);
    // 
    // // Pre-create mesh & material
    // let h_mesh = meshes.add(Rectangle::new(2.0 * HALF_WIDTH, wall_thickness));
    // let v_mesh = meshes.add(Rectangle::new(wall_thickness, 2.0 * HALF_HEIGHT));
    // let wall_material = materials.add(wall_color);

    // // Ceiling
    // commands.spawn((
    //     Mesh2d(h_mesh.clone().into()),
    //     MeshMaterial2d(wall_material.clone()),
    //     Transform::from_xyz(0.0, HALF_HEIGHT, 0.0),
    //     RigidBody::Static,
    //     Collider::rectangle(2.0 * HALF_WIDTH, wall_thickness),
    //     wall_friction,
    //     wall_bounce,
    //     wall_layers,
    //     Name::new("Ceiling"),
    // ));
    // // Floor
    // commands.spawn((
    //     Mesh2d(h_mesh.clone().into()),
    //     MeshMaterial2d(wall_material.clone()),
    //     Transform::from_xyz(0.0, -HALF_HEIGHT, 0.0),
    //     RigidBody::Static,
    //     Collider::rectangle(2.0 * HALF_WIDTH, wall_thickness),
    //     wall_friction,
    //     wall_bounce,
    //     wall_layers,
    //     Name::new("Floor"),
    // ));
    // // Left wall
    // commands.spawn((
    //     Mesh2d(v_mesh.clone().into()),
    //     MeshMaterial2d(wall_material.clone()),
    //     Transform::from_xyz(-HALF_WIDTH, 0.0, 0.0),
    //     RigidBody::Static,
    //     Collider::rectangle(wall_thickness, 2.0 * HALF_HEIGHT),
    //     wall_friction,
    //     wall_bounce,
    //     wall_layers,
    //     Name::new("LeftWall"),
    // ));
    // // Right wall
    // commands.spawn((
    //     Mesh2d(v_mesh.clone().into()),
    //     MeshMaterial2d(wall_material.clone()),
    //     Transform::from_xyz(HALF_WIDTH, 0.0, 0.0),
    //     RigidBody::Static,
    //     Collider::rectangle(wall_thickness, 2.0 * HALF_HEIGHT),
    //     wall_friction,
    //     wall_bounce,
    //     wall_layers,
    //     Name::new("RightWall"),
    // ));

    // Marbles
    let marble_mesh = meshes.add(Circle::new(PARTICLE_RADIUS));
    let marble_material = materials.add(Color::srgb(0.2, 0.8, 1.0));
    let marble_layers = CollisionLayers::new(GameLayer::Particle, [GameLayer::Wall]);
    let x0 = -(SPAWN_GRID_WIDTH as f32) * PARTICLE_SPAWN_SPACING / 2.0 + PARTICLE_SPAWN_SPACING / 2.0;
    let y0 = -(SPAWN_GRID_WIDTH as f32) * PARTICLE_SPAWN_SPACING / 2.0 + PARTICLE_SPAWN_SPACING / 2.0;
    for i in 0..SPAWN_GRID_WIDTH {
        for j in 0..SPAWN_GRID_WIDTH {
            let x = x0 + i as f32 * PARTICLE_SPAWN_SPACING;
            let y = y0 + j as f32 * PARTICLE_SPAWN_SPACING;
            commands.spawn((
                Mesh2d(marble_mesh.clone().into()),
                MeshMaterial2d(marble_material.clone()),
                Transform::from_xyz(x, y, 1.0),
                RigidBody::Dynamic,
                Collider::circle(PARTICLE_RADIUS),
                marble_layers,
                LinearVelocity::ZERO,
                Particle,
                Name::new(format!("Marble_{i}_{j}")),
            ));
        }
    }
}