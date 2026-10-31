//! [`Phase::Editor`]: the level being edited, and test play.
//!
//! There is no canvas yet, only a prompt: F5 plays the level as it stands in
//! memory, with nothing saved, and Esc leaves for the main menu. While
//! [`Editing`] exists, [`crate::flow`] brings a won or abandoned level back
//! here and records nothing.

use bevy::prelude::*;
use wonderland_editor::Editor;
use wonderland_logic::Session;

use crate::flow::{Phase, begin_memory};
use crate::hud::{Screen, ScreenFilled};
use crate::level::GameData;
use crate::menu::{MAIN, Text};
use crate::menu_scene::MenuState;

/// The level being edited. Removed on leaving for the menus.
#[derive(Resource, Debug, Default)]
pub struct Editing(pub Editor);

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Phase::Editor), show_prompt)
            .add_systems(
                Update,
                editor_keys
                    .before(ScreenFilled)
                    .run_if(in_state(Phase::Editor)),
            );
    }
}

fn show_prompt(mut screen: ResMut<Screen>) {
    let mut text = Text::default();
    text.center("F5 - Test Play   Esc - Main Menu", 18.0, 1.0, 0);
    text.activate(|| None);
    screen.images.clear();
    screen.letters = text.shown();
}

fn editor_keys(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    editing: Res<Editing>,
    data: Res<GameData>,
    mut menu: ResMut<MenuState>,
    mut next: ResMut<NextState<Phase>>,
    time: Res<Time<Real>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.remove_resource::<Editing>();
        menu.menu.hold_keys();
        menu.menu.start(MAIN);
        next.set(Phase::Menu);
    } else if keys.just_pressed(KeyCode::F5) {
        let now = time.elapsed().as_millis() as i64;
        match begin_memory(&data, editing.0.playable(), Session::default(), now) {
            Ok((level, sim)) => {
                commands.insert_resource(level);
                commands.insert_resource(sim);
                next.set(Phase::Game);
            }
            Err(e) => error!("{e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use bevy::state::app::StatesPlugin;
    use wonderland_campaign::profile::Profile;
    use wonderland_logic::{Completion, Event};

    use super::*;
    use crate::audio::MenuSounds;
    use crate::flow::FlowPlugin;
    use crate::game_menu::GameMenu;
    use crate::level::LevelData;
    use crate::menu::Menu;
    use crate::pad::Pad;
    use crate::scene::TextureCache;
    use crate::sim::Sim;

    fn phase(app: &App) -> Phase {
        *app.world().resource::<State<Phase>>().get()
    }

    /// Taps `key` and lets the phase change.
    fn tap(app: &mut App, key: KeyCode) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        app.update();
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(key);
        keys.clear();
        // A menu chooses on release, and the phase changes a frame later.
        app.update();
        app.update();
    }

    /// An app with no window, in the editor, with a player to record for.
    fn app(data: GameData) -> App {
        let mut menu = Menu::default();
        menu.profile = Some(Profile::new("ZED", 0));
        let menu = MenuState::of(menu, &data);
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin, AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_asset::<StandardMaterial>()
            .init_asset::<Image>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Pad>()
            .init_resource::<MenuSounds>()
            .init_resource::<Screen>()
            .init_resource::<TextureCache>()
            .init_resource::<Editing>()
            .insert_resource(data)
            .insert_resource(menu)
            .insert_state(Phase::Editor)
            .add_plugins((FlowPlugin, EditorPlugin));
        app.update();
        app
    }

    #[test]
    fn test_play_comes_back_to_the_editor() {
        let dir = std::env::var_os("WL_ASSETS")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets"))
            .join("RTWP");
        if !dir.is_dir() {
            eprintln!("skipping: no RTWP assets");
            return;
        }
        let mut app = app(GameData::open(&dir).unwrap());
        assert!(!app.world().resource::<Screen>().letters.is_empty());

        tap(&mut app, KeyCode::F5);
        assert_eq!(phase(&app), Phase::Game);
        let level = app.world().resource::<LevelData>();
        assert_eq!(level.level.object(1, 1), 1, "tidied: Stinky is added");
        let editing = app.world().resource::<Editing>();
        assert_eq!(editing.0.level().object(1, 1), 0, "the edit is untouched");

        // A lost level restarts from memory.
        let mut sim = app.world_mut().resource_mut::<Sim>();
        sim.world.completion = Completion::Restart;
        app.update();
        assert_eq!(phase(&app), Phase::Game);
        let sim = app.world().resource::<Sim>();
        assert_eq!(sim.world.completion, Completion::Playing);

        // A win records nothing and comes back.
        let mut sim = app.world_mut().resource_mut::<Sim>();
        sim.world.completion = Completion::Done;
        sim.events.push(Event::LevelWon {
            score: 900,
            perfect: true,
        });
        app.update();
        app.update();
        assert_eq!(phase(&app), Phase::Editor);
        let menu = &app.world().resource::<MenuState>().menu;
        assert_eq!(menu.profile.as_ref().unwrap().scores[0], 0);
        assert!(!app.world().resource::<Screen>().letters.is_empty());

        // So does leaving through the in-game menu (Select Level, Enter).
        tap(&mut app, KeyCode::F5);
        assert_eq!(phase(&app), Phase::Game);
        let mut sim = app.world_mut().resource_mut::<Sim>();
        sim.menu = Some(GameMenu::new());
        app.update();
        tap(&mut app, KeyCode::ArrowDown);
        tap(&mut app, KeyCode::Enter);
        assert_eq!(phase(&app), Phase::Editor);

        tap(&mut app, KeyCode::Escape);
        assert_eq!(phase(&app), Phase::Menu);
        assert!(!app.world().contains_resource::<Editing>());
    }
}
