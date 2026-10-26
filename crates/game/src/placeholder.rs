//! Coloured boxes where the players are, until step 3.2 draws the models.

use bevy::prelude::*;

use crate::coords::vec;
use crate::scene::LevelEntity;
use crate::sim::{Sim, SimStep};

/// Stands in for player `id` (1–4).
#[derive(Component)]
pub struct PlayerMarker(pub usize);

pub struct PlaceholderPlugin;

impl Plugin for PlaceholderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_markers)
            .add_systems(Update, move_markers.after(SimStep));
    }
}

fn spawn_markers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(0.5, 0.8, 0.5));
    let colours = [
        Color::srgb(0.2, 0.8, 0.2),
        Color::srgb(0.9, 0.5, 0.1),
        Color::srgb(0.9, 0.8, 0.2),
        Color::srgb(0.3, 0.5, 1.0),
    ];
    for (k, colour) in colours.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: colour,
                unlit: true,
                ..default()
            })),
            Transform::default(),
            Visibility::Hidden,
            PlayerMarker(k + 1),
            LevelEntity,
        ));
    }
}

fn move_markers(
    sim: Res<Sim>,
    mut markers: Query<(&PlayerMarker, &mut Transform, &mut Visibility)>,
) {
    for (m, mut t, mut v) in &mut markers {
        let Some(p) = sim.world.players.get(m.0) else {
            continue;
        };
        *v = if p.exists && !p.hidden {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        // The entity position is the model's origin at its feet.
        t.translation = vec(p.position) + Vec3::Y * 0.4;
    }
}
