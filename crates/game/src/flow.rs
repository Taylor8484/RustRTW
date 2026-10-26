//! Level start, win and loss: what `MainGameLoop` does when
//! `LevelCompleted` becomes 2 (won and faded out) or 3 (lost).
//!
//! Until the menus and campaign arrive (step 4), a won level goes on to
//! the next in [`Playlist`] and a lost one restarts, carrying the
//! [`Session`] (the random generator and the death-sound count) over as
//! `StartLevel` does. Esc quits, where the original opens its in-game
//! menu.

use bevy::app::AppExit;
use bevy::prelude::*;
use wonderland_logic::{Completion, Session};

use crate::level::{GameData, LevelData};
use crate::objects::clear_pools;
use crate::scene::{LevelEntity, spawn_level};
use crate::sim::{Sim, SimStep};

/// The levels to play in order, and which is being played.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct Playlist {
    pub names: Vec<String>,
    pub index: usize,
}

impl Playlist {
    /// The level after the current one, wrapping round.
    pub fn advance(&mut self) -> Option<&str> {
        if self.names.is_empty() {
            return None;
        }
        self.index = (self.index + 1) % self.names.len();
        Some(&self.names[self.index])
    }
}

pub struct FlowPlugin;

impl Plugin for FlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (finish_level, quit).after(SimStep));
    }
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
    time: Res<Time<Real>>,
    entities: Query<Entity, With<LevelEntity>>,
) {
    let session = || -> Session { sim.world.session.clone() };
    match sim.world.completion {
        Completion::Restart => {
            // `StartLevel(CurrentLevel)`.
            match Sim::start(&data, &level, session(), now_ms(&time)) {
                Ok(new) => *sim = new,
                Err(e) => error!("{e}"),
            }
            rebuild(&mut commands, &entities);
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
            match Sim::start(&data, &new_level, session(), now_ms(&time)) {
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

fn quit(keys: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
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
