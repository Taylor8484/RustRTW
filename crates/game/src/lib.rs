//! The Bevy front end: renders a `logic::World` and feeds it the keyboard.
//!
//! Pure modules (no app state, tested without a GPU):
//! - [`coords`]: Blitz to Bevy coordinates and camera angles.
//! - [`level_mesh`]: `ConstructLevel`'s geometry in Blitz coordinates.
//! - [`style`]: which textures a level style and background use.
//! - [`texture`]: decoding textures with Blitz's flags.
//! - [`lighting`]: Blitz's per-vertex light model.
//!
//! Plugins, one per concern so later steps slot in beside them:
//! - [`sim::SimPlugin`]: runs `World::step` on `logic::Clock` ticks.
//! - [`scene::ScenePlugin`]: the level mesh, backdrop, lights and texture
//!   scrolling.
//! - [`camera::CameraPlugin`]: the camera from `World::camera`.
//! - [`objects::ObjectsPlugin`]: the players, boxes, monsters, items and
//!   the rest, from [`model`]'s models.
//! - [`audio::AudioPlugin`]: the sound effects.
//! - [`particles::ParticlesPlugin`]: explosions, sparkles, trails and
//!   weather.
//! - [`shot::ShotPlugin`]: `WL_SCREENSHOT` saves a frame and quits.
//!
//! Step 3.2 adds the objects, particles and sound as further plugins
//! reading [`sim::Sim`] and its per-frame events; step 3.3 adds the HUD
//! and the start/win/lose flow, replacing [`sim::restart_finished_levels`].

pub mod audio;
pub mod camera;
pub mod coords;
pub mod level;
pub mod level_mesh;
pub mod lighting;
pub mod model;
pub mod objects;
pub mod particles;
pub mod scene;
pub mod shot;
pub mod sim;
pub mod style;
pub mod texture;

use bevy::prelude::*;

/// Everything the game adds to a Bevy app, after `DefaultPlugins`.
pub struct GamePlugins;

impl Plugin for GamePlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            sim::SimPlugin,
            scene::ScenePlugin,
            camera::CameraPlugin,
            objects::ObjectsPlugin,
            audio::AudioPlugin,
            particles::ParticlesPlugin,
            shot::ShotPlugin,
        ));
    }
}
