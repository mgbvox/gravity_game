use crate::{Physics, UpdateHud};
use bevy::prelude::{Commands, Component, Query, Res, Text, Transform, Window, With};

#[derive(Component)]
pub struct HUD;

pub fn spawn_hud(commands: &mut Commands, window: &Window, physics_constants: Res<Physics>) {
    commands.spawn((
        Text::new(""),
        Transform::from_xyz(0.0, window.height() / 2.0 - 20.0, 0.0),
        HUD,
    ));
}

pub fn update_hud(mut q_hud: Query<&mut Text, With<HUD>>, q_physics_constants: Res<Physics>) {
    let mut hud = q_hud.single_mut();
    q_physics_constants.update_hud(&mut hud)
}
