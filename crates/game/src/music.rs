//! Music: the Impulse Tracker modules the original plays with FMOD's
//! `PlayMusic`, mixed here by `xmrsplayer` and streamed to Bevy's audio.
//! A module loops until it is stopped or replaced.
//!
//! The menus play the title music (`res7.dat`); a level plays its world's
//! track (`StartLevelMusic`), started when play moves into another world
//! and kept across that world's levels. Until the campaign (step 4c) the
//! world comes from [`world_of`], a copy of `WorldLevelNames` in
//! `RTWP/globals.bb`. Custom levels' own tracks (`ExtraLevelMusicTrack`)
//! come with step 4d.

use std::collections::HashMap;
use std::num::NonZero;
use std::time::Duration;

use bevy::audio::{
    AddAudioSource, AudioSink, AudioSinkPlayback, Decodable, Sample, Source, Volume,
};
use bevy::prelude::*;
use xmrs::prelude::Module;
use xmrsplayer::prelude::XmrsPlayer;

use crate::flow::Phase;
use crate::level::{GameData, LevelData};

/// The mixing rate.
const RATE: u32 = 44_100;

/// A loaded module. Modules are leaked once each so a player can borrow
/// one for as long as it plays.
#[derive(Asset, TypePath)]
pub struct Tune {
    module: &'static Module,
}

impl Tune {
    /// Loads the module in `file`.
    pub fn load(data: &GameData, file: &str) -> Result<Self, String> {
        let bytes = data.assets.read(file).map_err(|e| e.to_string())?;
        let module = Module::load(&bytes).map_err(|e| format!("{e:?}"))?;
        Ok(Self {
            module: Box::leak(Box::new(module)),
        })
    }
}

impl Decodable for Tune {
    type Decoder = TuneDecoder;

    fn decoder(&self) -> TuneDecoder {
        // A loop count of 0 loops for ever.
        TuneDecoder(XmrsPlayer::new(self.module, RATE, 0))
    }
}

/// Interleaved stereo samples from the player.
pub struct TuneDecoder(XmrsPlayer<'static>);

impl Iterator for TuneDecoder {
    type Item = Sample;

    fn next(&mut self) -> Option<Sample> {
        self.0.next().map(|s| Sample::from(s) / 32768.0)
    }
}

impl Source for TuneDecoder {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> NonZero<u16> {
        NonZero::new(2).unwrap()
    }

    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(RATE).unwrap()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// A track and how loud it plays relative to the music level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Track {
    pub file: &'static str,
    pub scale: f32,
}

/// The title music, `res7.dat`.
pub const TITLE: Track = Track {
    file: "res7.dat",
    scale: 1.0,
};

/// `StartLevelMusic` for `CurrentWorld = world`. Worlds 8 and 28 (counted
/// from 1) play at half the music level.
pub fn world_track(world: i32) -> Track {
    let file = match world + 1 {
        1 | 5 | 23 | 25 | 12 => "res5.dat",
        3 | 9 | 27 => "res8.dat",
        2 | 6 | 10 | 21 | 24 | 11 => "res3.dat",
        8 | 28 => "res4.dat",
        _ => "res6.dat",
    };
    let scale = if matches!(world + 1, 8 | 28) {
        0.5
    } else {
        1.0
    };
    Track { file, scale }
}

/// RTWP's worlds and their level files (`WorldLevelNames`), until the
/// campaign data replaces it.
const WORLDS: &[(i32, &[&str])] = &[
    (
        0,
        &[
            "intro",
            "reflectors",
            "somereflection",
            "cleartheway",
            "rollingstones",
            "ldrescue",
            "letsgetrolling",
            "relayrace",
        ],
    ),
    (
        1,
        &[
            "hotcoals",
            "chomper",
            "onemeoneyou",
            "cckohmy",
            "boulderpanic",
            "firepit",
            "remotecontrol",
            "chompatbit",
            "repetition",
            "arena",
        ],
    ),
    (
        2,
        &[
            "notinviting",
            "traps",
            "gettingspooky",
            "notsofast",
            "pendulum",
            "chasms",
            "hostile",
            "rooftop",
        ],
    ),
    (
        3,
        &[
            "starlightlounge",
            "closeencounters",
            "multifunctionre",
            "catchme",
            "switchintro",
            "switchbait",
            "floatingplat",
            "sharpshooter",
            "loadingdock",
            "zeta",
        ],
    ),
    (
        4,
        &[
            "gardenentrance",
            "mushrooms",
            "hiddensurprises",
            "foresttrees",
            "whatsinblock",
        ],
    ),
    (
        5,
        &[
            "sinking",
            "pacifist",
            "bridge",
            "binarythinking",
            "guide",
            "littledetour",
            "iceage",
        ],
    ),
    (
        6,
        &[
            "darkcaverns",
            "firingsquad",
            "greedy2",
            "ringoffire",
            "spiral2",
            "dontmove",
            "plumbing",
            "hotstuff",
            "ticktock",
            "shielding",
        ],
    ),
    (
        7,
        &[
            "retropacman",
            "spaceinvaders",
            "pong",
            "digdug",
            "qbert",
            "frogger",
            "centipede",
            "missile",
        ],
    ),
    (
        8,
        &[
            "res10",
            "rimrunner",
            "doorways",
            "serpentine",
            "outoftime",
            "housemirrors",
            "ghostbusting",
            "chainreaction",
            "hopskipjump",
        ],
    ),
    (
        9,
        &[
            "endbegin",
            "marblemaze",
            "longshot",
            "knightsmove",
            "anotherway",
            "beyondtheriver",
            "firestorm",
            "math",
            "522",
            "hurrah",
        ],
    ),
    (
        10,
        &[
            "tut1", "tut2", "tut3", "tut4", "tut5", "tut6", "tut7", "tut8", "tut9", "tut10",
        ],
    ),
    (
        11,
        &[
            "tut11", "tut12", "tut13", "tut14", "tut15", "tut16", "tut17", "tut18", "tut19",
            "tut20",
        ],
    ),
    (
        20,
        &["mystery1", "mystery2", "mystery3", "mystery4", "mystery5"],
    ),
    (
        21,
        &[
            "cave1", "cave2", "cave3", "cave4", "cave5", "cave6", "cave7", "cave8", "cave9",
            "cave10",
        ],
    ),
    (
        22,
        &[
            "qookie1", "qookie2", "qookie3", "qookie4", "qookie5", "qookie6", "qookie7", "qookie8",
        ],
    ),
    (23, &["city1", "city2", "city3", "city4", "city5"]),
    (24, &["three1", "three2", "three3", "three4", "three5"]),
    (
        25,
        &[
            "ufo1", "ufo2", "ufo3", "ufo4", "ufo5", "ufo6", "ufo7", "ufo8", "ufo9",
        ],
    ),
    (
        26,
        &[
            "temple1", "temple2", "temple3", "temple4", "temple5", "temple6", "temple7", "temple8",
        ],
    ),
    (27, &["rainbow"]),
];

/// Every level file the world table names, without its extension.
pub fn world_levels() -> impl Iterator<Item = &'static str> {
    WORLDS.iter().flat_map(|(_, levels)| levels.iter().copied())
}

/// The first world that lists the level file `name` (any folder,
/// extension or case).
pub fn world_of(name: &str) -> Option<i32> {
    level_index(name).map(|l| l / 10)
}

/// `CurrentLevel` for the level file `name`: its world times ten plus its
/// place in the world, from the first world that lists it.
pub fn level_index(name: &str) -> Option<i32> {
    let file = name.rsplit(['\\', '/']).next().unwrap_or(name);
    let stem = file
        .rsplit_once('.')
        .map_or(file, |(s, _)| s)
        .to_lowercase();
    WORLDS.iter().find_map(|(w, levels)| {
        let i = levels.iter().position(|l| *l == stem)?;
        Some(w * 10 + i as i32)
    })
}

/// What the music should do next: `Some(track)` (re)starts it, `None`
/// stops it (`StopChannel Music`).
#[derive(Resource, Default)]
pub struct MusicRequest(pub Option<Option<Track>>);

/// The playing music entity and its track.
#[derive(Component)]
struct Playing(Track);

/// Loaded modules by file, and the world whose track plays in a level.
#[derive(Resource, Default)]
struct Tunes {
    loaded: HashMap<&'static str, Option<Handle<Tune>>>,
    world: Option<i32>,
}

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<Tune>()
            .init_resource::<MusicRequest>()
            .init_resource::<Tunes>()
            .add_systems(OnEnter(Phase::Menu), forget_world)
            .add_systems(
                Update,
                (
                    level_music.run_if(in_state(Phase::Game)),
                    play_music,
                    follow_volume,
                )
                    .chain(),
            );
    }
}

fn forget_world(mut tunes: ResMut<Tunes>) {
    tunes.world = None;
}

/// `StartLevelMusic` when play moves into another world.
fn level_music(level: Res<LevelData>, mut tunes: ResMut<Tunes>, mut request: ResMut<MusicRequest>) {
    if !level.is_changed() && tunes.world.is_some() {
        return;
    }
    let Some(world) = world_of(&level.name) else {
        return;
    };
    if tunes.world != Some(world) {
        tunes.world = Some(world);
        request.0 = Some(Some(world_track(world)));
    }
}

fn play_music(
    mut commands: Commands,
    data: Res<GameData>,
    mut request: ResMut<MusicRequest>,
    mut tunes: ResMut<Tunes>,
    mut assets: ResMut<Assets<Tune>>,
    playing: Query<Entity, With<Playing>>,
) {
    let Some(next) = request.0.take() else {
        return;
    };
    for e in &playing {
        commands.entity(e).despawn();
    }
    let Some(track) = next else {
        return;
    };
    let handle = tunes
        .loaded
        .entry(track.file)
        .or_insert_with(|| match Tune::load(&data, track.file) {
            Ok(t) => Some(assets.add(t)),
            Err(e) => {
                warn!("music {}: {e}", track.file);
                None
            }
        })
        .clone();
    if let Some(handle) = handle {
        commands.spawn((
            AudioPlayer(handle),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(data.music_volume * track.scale)),
            Playing(track),
        ));
    }
}

/// `ChannelVolume Music, OptionsMusicLevel` when the option changes.
fn follow_volume(data: Res<GameData>, mut sinks: Query<(&Playing, &mut AudioSink)>) {
    for (p, mut sink) in &mut sinks {
        let v = Volume::Linear(data.music_volume * p.0.scale);
        if sink.volume() != v {
            sink.set_volume(v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_find_their_world_and_track() {
        assert_eq!(world_of(r"levels\Intro.lev"), Some(0));
        assert_eq!(world_of("tut11.lv5"), Some(11));
        assert_eq!(world_of("custom.lev"), None);
        assert_eq!(level_index("intro"), Some(0));
        assert_eq!(level_index("reflectors.lev"), Some(1));
        assert!(WORLDS.iter().all(|(_, levels)| levels.len() <= 10));
        assert_eq!(world_track(0).file, "res5.dat");
        assert_eq!(
            world_track(7),
            Track {
                file: "res4.dat",
                scale: 0.5
            }
        );
        assert_eq!(world_track(3).file, "res6.dat");
    }
}
