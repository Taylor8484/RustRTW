//! Music: the Impulse Tracker modules the original plays with FMOD's
//! `PlayMusic`, mixed here by `xmrsplayer` and streamed to Bevy's audio.
//! A module loops until it is stopped or replaced.
//!
//! The menus play the title music (`res7.dat`). The level selector starts
//! a world's track (`StartLevelMusic`) when it is entered from the pack
//! menu or moves to another world, and the track plays on through that
//! world's levels and back in the selector. A custom level starts its own
//! track (`ExtraLevelMusicTrack`) when it names another one than the
//! last, and the track plays on in the custom level menu.

use std::collections::HashMap;
use std::num::NonZero;
use std::time::Duration;

use bevy::audio::{
    AddAudioSource, AudioSink, AudioSinkPlayback, Decodable, Sample, Source, Volume,
};
use bevy::prelude::*;
use xmrs::prelude::Module;
use xmrsplayer::prelude::XmrsPlayer;

use crate::level::GameData;

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

/// The classic ending's music, `res4.dat`.
pub const ENDING: Track = Track {
    file: "res4.dat",
    scale: 1.0,
};

/// `StartLevelMusic` for `CurrentWorld = world`. Worlds 8 and 28 (counted
/// from 1) play at half the music level. TOW's worlds (30 on) and the
/// playground play TOW's choice, [`tow_world_track`].
pub fn world_track(world: i32) -> Track {
    if world >= 30 {
        return tow_world_track(world - 30);
    }
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

/// TOW's `StartLevelMusic` for its world `world` (the full version; the
/// playground is TOW's world 40). Its world 8 plays at half the music
/// level. TOW's modules are the same files as RTWP's.
pub fn tow_world_track(world: i32) -> Track {
    let file = match world + 1 {
        4 | 9 => "res6.dat",
        2 | 6 | 10 => "res5.dat",
        8 => "res4.dat",
        _ => "res3.dat",
    };
    let scale = if world + 1 == 8 { 0.5 } else { 1.0 };
    Track { file, scale }
}

/// The music for `ExtraLevelMusicTrack = track`, as the custom level
/// menu starts it. Tracks 1 and 5 play at half the music level.
pub fn extra_track(track: i32) -> Track {
    let file = match track {
        1 => "res4.dat",
        2 => "res3.dat",
        3 => "res5.dat",
        4 => "res6.dat",
        5 => "res7.dat",
        _ => "res8.dat",
    };
    let scale = if matches!(track, 1 | 5) { 0.5 } else { 1.0 };
    Track { file, scale }
}

/// What the music should do next: `Some(track)` (re)starts it, `None`
/// stops it (`StopChannel Music`).
#[derive(Resource, Default)]
pub struct MusicRequest(pub Option<Option<Track>>);

/// The playing music entity and its track.
#[derive(Component)]
struct Playing(Track);

/// Loaded modules by file.
#[derive(Resource, Default)]
struct Tunes {
    loaded: HashMap<&'static str, Option<Handle<Tune>>>,
}

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<Tune>()
            .init_resource::<MusicRequest>()
            .init_resource::<Tunes>()
            .add_systems(Update, (play_music, follow_volume).chain());
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
    fn custom_levels_name_their_track() {
        assert_eq!(
            extra_track(1),
            Track {
                file: "res4.dat",
                scale: 0.5
            }
        );
        assert_eq!(extra_track(5).scale, 0.5);
        assert_eq!(extra_track(2).file, "res3.dat");
        // Anything else, 0 included, is `res8.dat`.
        assert_eq!(extra_track(0).file, "res8.dat");
        assert_eq!(extra_track(9).file, "res8.dat");
    }

    #[test]
    fn worlds_find_their_track() {
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
