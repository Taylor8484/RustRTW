//! The camera, placed each frame from `World::camera`: position, pitch and
//! yaw in Blitz terms, `CameraRange near, 140` and `CameraZoom`.
//!
//! The original only ran in 4:3 and 5:4 modes, and Blitz keeps the
//! horizontal view fixed, so on a wider screen it would lose the top and
//! bottom of the picture. Here a screen wider than 4:3 keeps the height a
//! 4:3 screen sees and shows more at the sides ([`fov`]); the text stays
//! on the original's grid in the centred 4:3 area ([`view_area`]). With
//! [`Pillarbox`] the camera draws only into that area, with black bars at
//! the sides, as the original's picture was.

use bevy::camera::{PerspectiveProjection, Projection, Viewport};
use bevy::prelude::*;
use bevy::render::view::Tonemapping;
use bevy::window::PrimaryWindow;

use crate::coords::{rotation, vec, vertical_fov};
use crate::flow::Phase;
use crate::shot::{HeadlessTarget, headless_size};
use crate::sim::{Sim, SimStep};

/// `CameraRange`'s far plane.
pub const FAR: f32 = 140.0;
const ENDING_FAR: f32 = crate::tow_ending::FAR;

#[derive(Component)]
pub struct GameCamera;

/// `--pillarbox`: draw only in the centred 4:3 area on wide screens.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Pillarbox;

/// The original's 4:3 picture on a `size` screen: all of it up to 4:3, a
/// centred 4:3 area on wider screens. Returns the position and size.
pub fn view_area(size: UVec2) -> (UVec2, UVec2) {
    let w = (size.y * 4).div_ceil(3);
    if size.x > w {
        (UVec2::new((size.x - w) / 2, 0), UVec2::new(w, size.y))
    } else {
        (UVec2::ZERO, size)
    }
}

/// The width over height of what `camera` draws.
pub fn view_aspect(camera: &Camera) -> f32 {
    let (hw, hh) = headless_size();
    camera
        .logical_viewport_size()
        .filter(|s| s.y > 0.0)
        .map_or(hw as f32 / hh as f32, |s| s.x / s.y)
}

/// The vertical field of view for `CameraZoom zoom` on a view of
/// `aspect`: Blitz's fixed horizontal view up to 4:3, then the height of
/// a 4:3 view with more shown at the sides.
pub fn fov(zoom: f32, aspect: f32) -> f32 {
    vertical_fov(zoom, aspect.min(4.0 / 3.0))
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(PreUpdate, fit_viewport)
            .add_systems(
                Update,
                follow_world.after(SimStep).run_if(in_state(Phase::Game)),
            );
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

/// With [`Pillarbox`], keeps the camera's viewport at [`view_area`] of
/// the window.
fn fit_viewport(
    window: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<GameCamera>>,
    pillarbox: Option<Res<Pillarbox>>,
) {
    let Ok(w) = window.single() else {
        return;
    };
    let size = UVec2::new(w.physical_width(), w.physical_height());
    if size.x == 0 || size.y == 0 || pillarbox.is_none() {
        return;
    }
    let (physical_position, physical_size) = view_area(size);
    for mut c in &mut cameras {
        let same = c.viewport.as_ref().is_some_and(|v| {
            v.physical_position == physical_position && v.physical_size == physical_size
        });
        if !same {
            c.viewport = Some(Viewport {
                physical_position,
                physical_size,
                ..default()
            });
        }
    }
}

fn follow_world(
    sim: Res<Sim>,
    mut camera: Query<(&Camera, &mut Transform, &mut Projection), With<GameCamera>>,
) {
    let c = &sim.world.camera;
    for (cam, mut t, mut p) in &mut camera {
        let aspect = view_aspect(cam);
        *t = Transform::from_translation(vec(c.position)).with_rotation(rotation(c.pitch, c.yaw));
        if let Projection::Perspective(p) = &mut *p {
            p.fov = fov(c.zoom, aspect);
            p.near = c.near;
            p.far = if sim.ending.is_some() {
                ENDING_FAR
            } else {
                FAR
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_screens_get_a_centred_four_by_three_view() {
        let (at, size) = view_area(UVec2::new(1920, 1080));
        assert_eq!((at, size), (UVec2::new(240, 0), UVec2::new(1440, 1080)));
        // 4:3 and 5:4 use the whole screen, as the original's modes did.
        assert_eq!(view_area(UVec2::new(1024, 768)).1, UVec2::new(1024, 768));
        assert_eq!(view_area(UVec2::new(1280, 1024)).1, UVec2::new(1280, 1024));
    }

    #[test]
    fn wide_views_keep_the_four_by_three_height() {
        let four_three = fov(2.0, 4.0 / 3.0);
        assert_eq!(fov(2.0, 21.0 / 9.0), four_three);
        // 5:4 is taller, as in the original's 1280x1024 mode.
        assert!(fov(2.0, 1.25) > four_three);
    }
}
