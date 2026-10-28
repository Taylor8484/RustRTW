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
//! `--level` plays without the menus, through a [`Playlist`]: a won level
//! goes on to the next in pack order, and nothing is recorded.
//!
//! Esc goes back to the level selector (or the main menu with no player),
//! where the original opens its in-game menu (step 4f).
//!
//! [`Menu::level`]: crate::menu::Menu::level
//! [`Menu::profile`]: crate::menu::Menu::profile

use bevy::prelude::*;
use wonderland_logic::{Completion, Event, Session};

use crate::audio::MenuSounds;
use crate::level::{GameData, LevelData};
use crate::menu::{LEVEL_SELECT, MAIN, Menu, MenuSound};
use crate::menu_scene::MenuState;
use crate::objects::clear_pools;
use crate::scene::{LevelEntity, spawn_level};
use crate::sim::{Sim, SimStep};

/// `GamePhase`: 0 runs `MenuLoop`, 1 `MainGameLoop`.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Phase {
    #[default]
    Menu,
    Game,
}

/// `--level`'s levels to play in order, and which is being played.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct Playlist {
    pub names: Vec<String>,
    pub index: usize,
}

impl Playlist {
    /// The pack's levels, starting at `name`.
    pub fn from_pack(data: &GameData, name: &str) -> Self {
        let names = data.level_names();
        Self {
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
/// theirs.
pub fn record(menu: &Menu) -> Option<i32> {
    menu.profile.as_ref()?.record(menu.level)
}

pub struct FlowPlugin;

impl Plugin for FlowPlugin {
    fn build(&self, app: &mut App) {
        // `main` may have started in `Phase::Game` already.
        if !app.world().contains_resource::<State<Phase>>() {
            app.init_state::<Phase>();
        }
        app.add_systems(OnEnter(Phase::Game), spawn_level)
            .add_systems(OnExit(Phase::Game), despawn_level)
            .add_systems(
                Update,
                (record_wins, finish_level, to_menu)
                    .chain()
                    .after(SimStep)
                    .run_if(in_state(Phase::Game)),
            );
    }
}

/// Loads the level at `path` and starts it: the resources [`Phase::Game`]
/// plays.
pub fn begin(
    data: &GameData,
    path: &str,
    session: Session,
    now_ms: i64,
    record: Option<i32>,
) -> Result<(LevelData, Sim), String> {
    let level = data.load_level(path)?;
    let sim = start(data, &level, session, now_ms, record)?;
    Ok((level, sim))
}

/// `StartLevel`, with the player's best score for the level.
fn start(
    data: &GameData,
    level: &LevelData,
    session: Session,
    now_ms: i64,
    record: Option<i32>,
) -> Result<Sim, String> {
    let mut sim = Sim::start(data, level, session, now_ms)?;
    sim.world.record = record;
    Ok(sim)
}

/// A won level of the player's: the better score and the perfect flag go
/// into the profile, which is then saved.
fn record_wins(sim: Res<Sim>, playlist: Option<Res<Playlist>>, state: Option<ResMut<MenuState>>) {
    let Some(mut state) = state.filter(|_| playlist.is_none()) else {
        return;
    };
    let menu = &mut state.menu;
    let Some(profile) = &mut menu.profile else {
        return;
    };
    for e in &sim.events {
        if let Event::LevelWon { score, perfect } = *e {
            profile.record_win(menu.level, score, perfect);
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
    state: Option<ResMut<MenuState>>,
    mut next: ResMut<NextState<Phase>>,
    time: Res<Time<Real>>,
    entities: Query<Entity, With<LevelEntity>>,
) {
    let session = || -> Session { sim.world.session.clone() };
    let mut state = state.filter(|_| playlist.is_none());
    let record = state.as_ref().and_then(|s| record(&s.menu));
    match sim.world.completion {
        Completion::Restart => {
            // `StartLevel(CurrentLevel)`.
            match start(&data, &level, session(), now_ms(&time), record) {
                Ok(new) => *sim = new,
                Err(e) => error!("{e}"),
            }
            rebuild(&mut commands, &entities);
        }
        Completion::Done if state.is_some() => {
            // `CurrentLevel=CurrentLevel+1`, `StartMenu(4)`.
            if let Some(state) = &mut state {
                state.menu.level += 1;
                state.menu.start(LEVEL_SELECT);
            }
            next.set(Phase::Menu);
        }
        Completion::Done => {
            let next = playlist
                .and_then(|mut p| p.advance().map(str::to_owned))
                .unwrap_or_else(|| level.name.clone());
            let new_level = match data.load_level(&next) {
                Ok(l) => l,
                Err(e) => {
                    error!("{e}");
                    level.clone()
                }
            };
            match start(&data, &new_level, session(), now_ms(&time), None) {
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

/// Esc during play: the menu sound, then the level selector, or the main
/// menu when no player is playing.
fn to_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<MenuState>,
    playlist: Option<Res<Playlist>>,
    mut next: ResMut<NextState<Phase>>,
    mut sounds: ResMut<MenuSounds>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        sounds.0.push(MenuSound::Move);
        let menu = &mut menu.menu;
        // Esc is still down: the menu must not take it as a press.
        menu.hold_keys();
        if menu.profile.is_some() && playlist.is_none() {
            menu.start(LEVEL_SELECT);
        } else {
            menu.start(MAIN);
        }
        next.set(Phase::Menu);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playlist_wraps() {
        let mut p = Playlist {
            names: vec!["a".into(), "b".into()],
            index: 1,
        };
        assert_eq!(p.advance(), Some("a"));
        assert_eq!(p.advance(), Some("b"));
        let mut empty = Playlist {
            names: vec![],
            index: 0,
        };
        assert_eq!(empty.advance(), None);
    }
}
