//! The camera, placed each frame from `World::camera`: position, pitch and
//! yaw in Blitz terms, `CameraRange near, 140` and `CameraZoom`.

use bevy::camera::{PerspectiveProjection, Projection};
use bevy::prelude::*;
use bevy::render::view::Tonemapping;
use bevy::window::PrimaryWindow;

use crate::coords::{rotation, vec, vertical_fov};
use crate::shot::{HEADLESS_SIZE, HeadlessTarget};
use crate::sim::{Sim, SimStep};

/// `CameraRange`'s far plane.
pub const FAR: f32 = 140.0;

#[derive(Component)]
pub struct GameCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, follow_world.after(SimStep));
    }
}

fn spawn_camera(mut commands: Commands, headless: Option<Res<HeadlessTarget>>) {
    let mut camera = commands.spawn((
        Camera3d::default(),
        // Blitz writes colours straight to the screen.
        Tonemapping::None,
        Projection::Perspective(PerspectiveProjection {
            far: FAR,
            ..default()
        }),
        GameCamera,
    ));
    if let Some(t) = headless {
        camera.insert(t.render_target());
    }
}

fn follow_world(
    sim: Res<Sim>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut camera: Query<(&mut Transform, &mut Projection), With<GameCamera>>,
) {
    let c = &sim.world.camera;
    let (hw, hh) = HEADLESS_SIZE;
    let aspect = window
        .single()
        .map_or(hw as f32 / hh as f32, |w| w.width() / w.height().max(1.0));
    for (mut t, mut p) in &mut camera {
        *t = Transform::from_translation(vec(c.position)).with_rotation(rotation(c.pitch, c.yaw));
        if let Projection::Perspective(p) = &mut *p {
            p.fov = vertical_fov(c.zoom, aspect);
            p.near = c.near;
            p.far = FAR;
        }
    }
}
