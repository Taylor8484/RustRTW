//! `GamePhase` (menus or play), level start, win and loss: what
//! `MainGameLoop` does when `LevelCompleted` becomes 2 (won and faded
//! out) or 3 (lost).
//!
//! A lost level restarts, carrying the [`Session`] (the random generator
//! and the death-sound count) over as `StartLevel` does. A won level goes
//! back to the level selector with the next level (`CurrentLevel+1`)
//! under the cursor.
//!
//! Levels started from the menus are the player's: each starts with the
//! player's best score for `CurrentLevel` ([`Menu::level`]) as its
//! record, and a won level's score is kept and the player saved
//! (`SavePlayer`). The player is the menus' [`Menu::profile`].
//!
//! A custom level (`CurrentLevel=-999`) goes back to the custom level
//! menu when won, with the cursor on the next level if it is in a folder,
//! which is how a pack is played. Its record is the player's score for its map id, and a
//! win is kept only in English, as `ControlCoins` has no code for it in
//! the other languages; the player is saved either way.
//!
//! `--level` plays without the menus, through a [`Playlist`]: a won level
//! goes on to the next in its pack's order (RTWP's, or TOW's for a level
//! only TOW has), and nothing is recorded.
//!
//! TOW's playground levels (400 on) go back to the playground's selector
//! instead, which puts the cursor on its first level again.
//!
//! Esc opens the in-game menu ([`GameMenu`]) over the level. With no
//! player (`--level`), selecting a level goes to the main menu too.
//!
//! A test play from the editor ([`Editing`]) records nothing, and a win or
//! leaving through the in-game menu goes back to [`Phase::Editor`]. A lost
//! test play starts afresh, with the session it began with, so that its keys
//! can be played again.
//!
//! [`Menu::level`]: crate::menu::Menu::level
//! [`Menu::profile`]: crate::menu::Menu::profile

use bevy::prelude::*;
use wonderland_campaign::custom;
use wonderland_campaign::worlds::Content;
use wonderland_editor::proof::{Recording, fair};
use wonderland_formats::Level;
use wonderland_logic::{Completion, Event, Options, Session};

use crate::audio::MenuSounds;
use crate::editor_scene::Editing;
use crate::game_menu::{Choice, GameMenu};
use crate::level::{GameData, LevelData};
use crate::menu::{CUSTOM_LEVELS, MAIN, Menu, MenuSound};
use crate::menu_scene::{MenuState, read_keys};
use crate::objects::clear_pools;
use crate::pad::{Held, Pad};
use crate::scene::{LevelEntity, spawn_level};
use crate::sim::{Sim, SimStep};

/// `GamePhase`: 0 runs `MenuLoop`, 1 `MainGameLoop`.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Phase {
    #[default]
    Menu,
    Game,
    /// The level editor, between test plays.
    Editor,
}

/// Whether a level is on show, with the [`Sim`] that draws it.
pub fn level_shown(phase: Res<State<Phase>>, sim: Option<Res<Sim>>) -> bool {
    *phase.get() != Phase::Menu && sim.is_some()
}

/// `--level`'s levels to play in order, and which is being played.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct Playlist {
    /// The pack's game.
    pub content: Content,
    pub names: Vec<String>,
    pub index: usize,
}

impl Playlist {
    /// `content`'s pack's levels, starting at `name`.
    pub fn from_pack(data: &GameData, content: Content, name: &str) -> Self {
        let names = data.level_names(content);
        Self {
            content,
            index: names.iter().position(|n| n == name).unwrap_or(0),
            names,
        }
    }

    /// The level after the current one, wrapping round.
    pub fn advance(&mut self) -> Option<&str> {
        if self.names.is_empty() {
            return None;
        }
        self.index = (self.index + 1) % self.names.len();
        Some(&self.names[self.index])
    }
}

/// The player's best score for the level being played, when it is
/// theirs. For a custom level only English shows the record line.
pub fn record(menu: &Menu) -> Option<i32> {
    let profile = menu.profile.as_ref()?;
    if menu.level == custom::LEVEL {
        let file = menu.custom_file()?;
        let r = profile.custom_record(file.id)?;
        return (menu.language == 0).then_some(r.score);
    }
    profile.record(menu.level)
}

pub struct FlowPlugin;

impl Plugin for FlowPlugin {
    fn build(&self, app: &mut App) {
        // `main` may have started in another phase already.
        if !app.world().contains_resource::<State<Phase>>() {
            app.init_state::<Phase>();
        }
        app.add_systems(OnEnter(Phase::Game), spawn_level)
            .add_systems(OnExit(Phase::Game), despawn_level)
            .add_systems(OnExit(Phase::Editor), despawn_level)
            .add_systems(
                Update,
                (record_wins, finish_level, game_menu)
                    .chain()
                    .after(SimStep)
                    .run_if(in_state(Phase::Game)),
            );
    }
}

/// Loads the level at `path` in `content`'s folder and starts it: the
/// resources [`Phase::Game`] plays.
pub fn begin(
    data: &GameData,
    content: Content,
    path: &str,
    session: Session,
    now_ms: i64,
    record: Option<i32>,
) -> Result<(LevelData, Sim), String> {
    let level = data.load_content_level(content, path)?;
    let sim = start(data, &level, data.options, session, now_ms, record)?;
    Ok((level, sim))
}

/// Starts `level` from memory, under [`fair`] options as its keys may be
/// played again: the resources [`Phase::Game`] plays.
pub fn begin_memory(
    data: &GameData,
    level: Level,
    now_ms: i64,
) -> Result<(LevelData, Sim), String> {
    let level = data.memory_level(level)?;
    let options = fair(data.options);
    let sim = start(data, &level, options, Session::default(), now_ms, None)?;
    Ok((level, sim))
}

/// `StartLevel`, with the player's best score for the level.
fn start(
    data: &GameData,
    level: &LevelData,
    options: Options,
    session: Session,
    now_ms: i64,
    record: Option<i32>,
) -> Result<Sim, String> {
    let mut sim = Sim::start(data, level, options, session, now_ms)?;
    sim.world.record = record;
    Ok(sim)
}

/// A won level of the player's: the better score and the perfect flag go
/// into the profile, which is then saved.
fn record_wins(
    sim: Res<Sim>,
    playlist: Option<Res<Playlist>>,
    editing: Option<Res<Editing>>,
    state: Option<ResMut<MenuState>>,
) {
    let Some(mut state) = state.filter(|_| playlist.is_none() && editing.is_none()) else {
        return;
    };
    let menu = &mut state.menu;
    let file = menu.custom_file().map(|f| (f.id, f.id2));
    let Some(profile) = &mut menu.profile else {
        return;
    };
    for e in &sim.events {
        if let Event::LevelWon { score, perfect } = *e {
            match file {
                Some((id, id2)) if menu.language == 0 => {
                    profile.record_custom_win(id, id2, score, perfect)
                }
                Some(_) => {}
                None => profile.record_win(menu.level, score, perfect),
            }
            if let Some(players) = &menu.players
                && let Err(e) = players.save(profile)
            {
                warn!("cannot save player {}: {e}", profile.name);
            }
        }
    }
}

fn despawn_level(mut commands: Commands, entities: Query<Entity, With<LevelEntity>>) {
    for e in &entities {
        commands.entity(e).despawn();
    }
    commands.queue(clear_pools);
}

fn now_ms(time: &Time<Real>) -> i64 {
    time.elapsed().as_millis() as i64
}

/// Despawns the level's entities and builds the scene for `level` again.
fn rebuild(commands: &mut Commands, entities: &Query<Entity, With<LevelEntity>>) {
    for e in entities {
        commands.entity(e).despawn();
    }
    commands.queue(clear_pools);
    commands.run_system_cached(spawn_level);
}

#[allow(clippy::too_many_arguments)]
fn finish_level(
    mut commands: Commands,
    mut sim: ResMut<Sim>,
    data: Res<GameData>,
    mut level: ResMut<LevelData>,
    playlist: Option<ResMut<Playlist>>,
    editing: Option<Res<Editing>>,
    state: Option<ResMut<MenuState>>,
    mut next: ResMut<NextState<Phase>>,
    time: Res<Time<Real>>,
    entities: Query<Entity, With<LevelEntity>>,
) {
    let session = || -> Session { sim.world.session.clone() };
    let testing = editing.is_some();
    let mut state = state.filter(|_| playlist.is_none() && !testing);
    let record = state.as_ref().and_then(|s| record(&s.menu));
    match sim.world.completion {
        Completion::Restart => {
            // `StartLevel(CurrentLevel)`.
            let options = sim.world.options;
            let again = if testing {
                Session::default()
            } else {
                session()
            };
            match start(&data, &level, options, again, now_ms(&time), record) {
                Ok(mut new) => {
                    new.recording = sim.recording.as_ref().map(|_| Recording::default());
                    *sim = new;
                }
                Err(e) => error!("{e}"),
            }
            rebuild(&mut commands, &entities);
        }
        Completion::Done if testing => next.set(Phase::Editor),
        Completion::Done if state.is_some() => {
            // `CurrentLevel=CurrentLevel+1`, `StartMenu(4)` (TOW's
            // `StartMenu(8)` in the playground), or `StartMenu(14)` after
            // a custom level.
            if let Some(state) = &mut state {
                let menu = &mut state.menu;
                if menu.level == custom::LEVEL {
                    menu.next_custom();
                    menu.start(CUSTOM_LEVELS);
                } else {
                    menu.level += 1;
                    menu.start(menu.selector_menu());
                }
            }
            next.set(Phase::Menu);
        }
        Completion::Done => {
            let content = playlist.as_ref().map_or(Content::Rtwp, |p| p.content);
            let next = playlist
                .and_then(|mut p| p.advance().map(str::to_owned))
                .unwrap_or_else(|| level.name.clone());
            let new_level = match data.load_content_level(content, &next) {
                Ok(l) => l,
                Err(e) => {
                    error!("{e}");
                    level.clone()
                }
            };
            match start(
                &data,
                &new_level,
                data.options,
                session(),
                now_ms(&time),
                None,
            ) {
                Ok(new) => {
                    info!("level {}", new_level.name);
                    *sim = new;
                    *level = new_level;
                }
                Err(e) => error!("{e}"),
            }
            rebuild(&mut commands, &entities);
        }
        _ => {}
    }
}

/// Esc during play (`StartMenu(5)`), and `MenuLoop5` while it is open.
#[allow(clippy::too_many_arguments)]
fn game_menu(
    keys: Res<ButtonInput<KeyCode>>,
    pad: Res<Pad>,
    mut sim: ResMut<Sim>,
    mut menu: ResMut<MenuState>,
    playlist: Option<Res<Playlist>>,
    editing: Option<Res<Editing>>,
    mut next: ResMut<NextState<Phase>>,
    mut sounds: ResMut<MenuSounds>,
) {
    if let Some(ending) = &mut sim.ending {
        if ending.leave(read_keys(&keys, pad.held, true)) {
            sim.ending = None;
            menu.menu.hold_keys();
            menu.menu.end();
            menu.menu.start(MAIN);
            next.set(Phase::Menu);
        }
        return;
    }
    let Some(open) = &mut sim.menu else {
        if (keys.just_pressed(KeyCode::Escape) || pad.pressed.has(Held::MENU))
            && !matches!(sim.world.completion, Completion::Done | Completion::Restart)
        {
            sounds.0.push(MenuSound::Move);
            sim.world.stop_signs_for_menu();
            sim.menu = Some(GameMenu::new());
        }
        return;
    };
    let (moved, choice) = open.frame(read_keys(&keys, pad.held, true), menu.menu.language);
    if moved {
        sounds.0.push(MenuSound::Move);
    }
    let Some(choice) = choice else {
        return;
    };
    sim.menu = None;
    match choice {
        Choice::Restart => sim.world.kill_players(),
        Choice::Resume => sim.world.check_signs(),
        Choice::SelectLevel | Choice::Exit if editing.is_some() => next.set(Phase::Editor),
        Choice::SelectLevel | Choice::Exit => {
            let menu = &mut menu.menu;
            menu.hold_keys();
            let selector = if menu.level == custom::LEVEL {
                CUSTOM_LEVELS
            } else {
                menu.selector_menu()
            };
            let own = menu.profile.is_some() && playlist.is_none();
            menu.start(if choice == Choice::SelectLevel && own {
                selector
            } else {
                MAIN
            });
            next.set(Phase::Menu);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playlist_wraps() {
        let mut p = Playlist {
            content: Content::Rtwp,
            names: vec!["a".into(), "b".into()],
            index: 1,
        };
        assert_eq!(p.advance(), Some("a"));
        assert_eq!(p.advance(), Some("b"));
        let mut empty = Playlist {
            content: Content::Rtwp,
            names: vec![],
            index: 0,
        };
        assert_eq!(empty.advance(), None);
    }

    #[test]
    fn custom_records_show_only_in_english() {
        use wonderland_campaign::custom::FileEntry;
        use wonderland_campaign::profile::Profile;
        let mut menu = Menu::default();
        let mut p = Profile::new("ZED", 0);
        p.record_custom_win(7, 0, 900, false);
        p.scores[3] = 400;
        menu.profile = Some(p);
        menu.level = 3;
        assert_eq!(record(&menu), Some(400));
        menu.files = vec![FileEntry {
            name: "A.LEV".into(),
            title: "A".into(),
            id: 7,
            id2: 0,
            score: 900,
            perfect: 0,
        }];
        menu.level = custom::LEVEL;
        assert_eq!(record(&menu), Some(900));
        menu.language = 1;
        assert_eq!(record(&menu), None);
        menu.language = 0;
        menu.files[0].id = 8;
        assert_eq!(record(&menu), None);
    }
}
