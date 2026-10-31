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
//! - [`music::MusicPlugin`]: the title and world music.
//! - [`particles::ParticlesPlugin`]: explosions, sparkles, trails and
//!   weather.
//! - [`shot::ShotPlugin`]: `WL_SCREENSHOT` saves a frame and quits.
//!
//! - [`hud::HudPlugin`]: draws [`hud::Screen`]: the coin and time display
//!   and the signs, or the menu text.
//! - [`flow::FlowPlugin`]: the [`flow::Phase`] (menus, play or editor),
//!   restarts a lost level, moves on from a won one.
//! - [`editor_scene::EditorPlugin`]: the level editor's canvas, and test play;
//!   [`editor_ui::EditorUiPlugin`]: its panels.
//! - [`menu_scene::MenuScenePlugin`]: runs [`menu`] and draws its 3D
//!   background.
//! - [`ending_scene::EndingScenePlugin`]: draws the endings.
//!
//! The level plugins run in [`flow::Phase::Game`] only, and need the
//! [`sim::Sim`] and [`level::LevelData`] resources [`flow::begin`] makes.

pub mod audio;
pub mod camera;
pub mod coords;
pub mod editor_check;
pub mod editor_icons;
pub mod editor_scene;
pub mod editor_ui;
pub mod ending;
pub mod ending_scene;
pub mod flow;
pub mod game_menu;
pub mod hud;
pub mod level;
pub mod level_mesh;
pub mod lighting;
pub mod menu;
pub mod menu_scene;
pub mod model;
pub mod music;
pub mod objects;
pub mod pad;
pub mod particles;
pub mod scene;
pub mod shot;
pub mod sim;
pub mod style;
pub mod texture;
pub mod tow_ending;

use bevy::prelude::*;

/// Everything the game adds to a Bevy app, after `DefaultPlugins`.
pub struct GamePlugins;

impl Plugin for GamePlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_egui::EguiPlugin::default());
        app.add_plugins((
            pad::PadPlugin,
            sim::SimPlugin,
            scene::ScenePlugin,
            camera::CameraPlugin,
            objects::ObjectsPlugin,
            audio::AudioPlugin,
            music::MusicPlugin,
            particles::ParticlesPlugin,
            hud::HudPlugin,
            flow::FlowPlugin,
            editor_scene::EditorPlugin,
            editor_ui::EditorUiPlugin,
            menu_scene::MenuScenePlugin,
            ending_scene::EndingScenePlugin,
            shot::ShotPlugin,
        ));
    }
}
