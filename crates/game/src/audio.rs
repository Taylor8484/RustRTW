//! Sound effects: plays the `logic` sound events with the files
//! `LoadSounds` loads (`RTWP/sound.bb`).
//!
//! `PlayMySound` without an entity plays at the sound level; with one,
//! `logic` has already worked out the distance volume ([`Event::SoundAt`]).
//! `SoundPitch` sets a sample rate in Hz, which becomes Bevy's playback
//! speed relative to the file's own rate. Voices in `CustomVoices\` replace
//! the shipped ones when present, as in the original.
//!
//! The menus queue their sounds in [`MenuSounds`], played in any phase.
//!
//! Music is in [`crate::music`].

use std::collections::HashMap;
use std::sync::Arc;

use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;
use wonderland_logic::world::LoseSound;
use wonderland_logic::{Event, Sound};

use crate::flow::Phase;
use crate::level::GameData;
use crate::menu::MenuSound;
use crate::sim::{Sim, SimStep};

/// The file for `sound`, and the pitch it plays at (`None`: as recorded).
pub fn sound_file(sound: Sound, custom: &dyn Fn(&str) -> bool) -> (String, Option<i32>) {
    let s = |name: &str| (format!("sound\\{name}.wav"), None);
    let n = |name: &str| (format!("newsounds\\{name}.wav"), None);
    // `CustomVoices\<name>` if the file is there.
    let voice = |name: &str, fallback: (String, Option<i32>)| {
        let path = format!("CustomVoices\\{name}.wav");
        if custom(&path) {
            (path, None)
        } else {
            fallback
        }
    };
    let player = |k: usize, names: [&str; 4]| {
        let name = names[k.clamp(1, 4) - 1];
        if name.contains('\\') {
            (format!("{name}.wav"), None)
        } else {
            s(name)
        }
    };
    match sound {
        Sound::Talk(t) => match t {
            0 => voice("start1", s("getready")),
            1 => voice("start1", s("herewego")),
            2 => voice("start1", s("cmonletsgo")),
            3 => voice("start1", s("letsdoit")),
            4 => voice("start1", n("gitready")),
            5 => s("stinkyno"),
            6 => voice("start2", n("mynameisqookie2")),
            7 => voice("start3", n("peeguespell3")),
            11 => voice("stinky", n("charstinky")),
            12 => voice("loof", n("charloof")),
            13 => voice("qookie", n("charqookie2")),
            _ => voice("peegue", n("charpeegue")),
        },
        Sound::Death => n("sldeathquiet"),
        Sound::Teleport => s("teleport"),
        Sound::Ice => s("ice"),
        Sound::SlideIce(i) => (format!("sound\\slice{}.wav", i + 1), Some(19000)),
        Sound::Convey(k) => player(
            k,
            [
                "stinkyconvey",
                "loofconvey",
                "newsounds\\qookiecoin5",
                "newsounds\\peegueconvey1",
            ],
        ),
        Sound::Transporter => ("res50.dat".into(), None),
        Sound::Trampoline => ("res47.dat".into(), None),
        Sound::Bounce(k, i) => (
            format!(
                "res{}.dat",
                31 + (k.clamp(1, 4) as i32 - 1) * 4 + (i.clamp(1, 4) - 1)
            ),
            None,
        ),
        Sound::BreakAway => s("breakaway"),
        Sound::BoxPush => s("boxpush"),
        Sound::BoulderPush => s("boulderpush"),
        Sound::BoulderHit => s("boulderhit"),
        Sound::BoxSplosh => s("splosh"),
        Sound::Lava => s("blub"),
        Sound::Sticky => ("res48.dat".into(), None),
        Sound::BoxExplode => s("boxexplo"),
        Sound::Single(i) => match i {
            0 => s("electro"),
            1 => s("fireshot"),
            2 => s("fireexplo"),
            3 => s("saucershot"),
            4 => s("saucerfly"),
            5 => s("spikeup"),
            6 => s("spikedown"),
            7 => s("reflector"),
            8..=11 => s("chomper1"),
            12 => n("prism1"),
            13 => n("boxgen1"),
            14 => n("mother1"),
            15 => n("spirit1"),
            _ => ("res49.dat".into(), None),
        },
        Sound::BallBounce(pitch) => (s("bounce").0, Some(pitch)),
        Sound::CoilyDeath => s("coilydeath"),
        Sound::BombExplode => s("bombexplo"),
        Sound::SaucerDie(pitch) => (s("SaucerDie").0, Some(pitch)),
        Sound::Time => s("sltime"),
        Sound::Count(i) => s(&format!("sl{i}")),
        Sound::Lightning(i) => s(&format!("lightning{}", i + 1)),
        Sound::Lose(l) => match l {
            LoseSound::Classic => s("death2"),
            LoseSound::Variant(c) => n(&format!("death-2{c}")),
            LoseSound::Spanish => n("death2-spa"),
            LoseSound::German => n("death2-ger"),
            LoseSound::French => n("death2-fra"),
        },
        Sound::Button => s("button"),
        Sound::TimerButton => s("windup"),
        Sound::Hmmm => s("slhmmm"),
        Sound::Coin(k) => player(
            k,
            [
                "stinkycoin",
                "loofcoin",
                "newsounds\\qookiecoin3",
                "newsounds\\peeguecoin3",
            ],
        ),
        Sound::LastCoin => s("lastcoin"),
        Sound::Clock(k) => player(
            k,
            [
                "stinkyclock",
                "loofclock",
                "newsounds\\qookietime1",
                "newsounds\\peeguetime1",
            ],
        ),
        Sound::Bonus(k) => player(
            k,
            [
                "stinkybonus",
                "loofbonus",
                "newsounds\\qookiecoin4",
                "newsounds\\peeguetime2",
            ],
        ),
        Sound::Exit(k) => {
            let (name, fallback) = match k {
                1 => ("stinkyend", s("stinkyend")),
                2 => ("loofend", s("loofend")),
                3 => ("qookieend", n("qookieend1")),
                _ => ("peegueend", n("peegueend1")),
            };
            voice(name, fallback)
        }
    }
}

/// A WAV file's sample rate (bytes 24–27 of the `fmt ` chunk header).
fn wav_rate(bytes: &[u8]) -> Option<u32> {
    bytes
        .get(24..28)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .filter(|&r| r > 0)
}

/// Loaded sounds by path, with their sample rates. A missing file is
/// remembered as missing, like `DoesSoundExist`'s per-sound silence.
#[derive(Resource, Default)]
struct SoundBank {
    loaded: HashMap<String, Option<(Handle<AudioSource>, u32)>>,
}

impl SoundBank {
    /// The sound in `path` and its sample rate, loaded once.
    fn get(
        &mut self,
        path: &str,
        data: &GameData,
        sources: &mut Assets<AudioSource>,
    ) -> Option<(Handle<AudioSource>, u32)> {
        self.loaded
            .entry(path.to_owned())
            .or_insert_with(|| match data.assets.read(path) {
                Ok(bytes) => {
                    let rate = wav_rate(&bytes).unwrap_or(22050);
                    let source = AudioSource {
                        bytes: Arc::from(bytes.as_ref()),
                    };
                    Some((sources.add(source), rate))
                }
                Err(err) => {
                    warn!("sound {path}: {err}");
                    None
                }
            })
            .clone()
    }
}

/// Sounds from outside the level to play this frame: the menus' and the
/// exit sound.
#[derive(Resource, Default)]
pub struct MenuSounds(pub Vec<MenuSound>);

/// The file for a menu sound and whether it plays through `PlayMySound`
/// (at the sound level, and not at all with sound off) or `PlaySound`
/// (at full volume).
pub fn menu_sound_file(sound: MenuSound) -> (&'static str, bool) {
    match sound {
        MenuSound::Move => ("sound\\menumove.wav", true),
        MenuSound::Type => ("sound\\type.wav", true),
        MenuSound::LastCoin => ("sound\\lastcoin.wav", true),
        MenuSound::No => ("sound\\stinkyno.wav", true),
        MenuSound::SecretSong => ("newsounds\\song1c.wav", false),
        MenuSound::Exit(n) => match n {
            3 => ("newsounds\\exit3.wav", true),
            4 => ("newsounds\\exit4.wav", true),
            5 => ("newsounds\\exit5.wav", true),
            6 => ("newsounds\\exit6.wav", true),
            _ => ("newsounds\\exit8.wav", true),
        },
    }
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoundBank>()
            .init_resource::<MenuSounds>()
            .add_systems(
                Update,
                (
                    play_sounds.after(SimStep).run_if(in_state(Phase::Game)),
                    play_menu_sounds,
                ),
            );
    }
}

fn play_menu_sounds(
    mut commands: Commands,
    data: Res<GameData>,
    mut queue: ResMut<MenuSounds>,
    mut bank: ResMut<SoundBank>,
    mut sources: ResMut<Assets<AudioSource>>,
) {
    for sound in std::mem::take(&mut queue.0) {
        let (path, scaled) = menu_sound_file(sound);
        let volume = if scaled { data.sound_volume } else { 1.0 };
        if volume <= 0.0 {
            continue;
        }
        if let Some((handle, _)) = bank.get(path, &data, &mut sources) {
            commands.spawn((
                AudioPlayer::new(handle),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)),
            ));
        }
    }
}

fn play_sounds(
    mut commands: Commands,
    sim: Res<Sim>,
    data: Res<GameData>,
    mut bank: ResMut<SoundBank>,
    mut sources: ResMut<Assets<AudioSource>>,
) {
    if !data.options.sound || data.sound_volume <= 0.0 {
        return;
    }
    for e in &sim.events {
        let (sound, volume) = match *e {
            Event::Sound(s) => (s, 1.0),
            Event::SoundAt(s, v) => (s, v),
            _ => continue,
        };
        let (path, mut pitch) = sound_file(sound, &|p| data.assets.exists(p));
        if sound == Sound::Single(8) && sim.world.chomp_pitch > 0 {
            pitch = Some(sim.world.chomp_pitch);
        }
        let Some((handle, rate)) = bank.get(&path, &data, &mut sources) else {
            continue;
        };
        let speed = pitch.map_or(1.0, |p| p as f32 / rate as f32);
        commands.spawn((
            AudioPlayer::new(handle),
            PlaybackSettings::DESPAWN
                .with_volume(Volume::Linear(data.sound_volume * volume))
                .with_speed(speed),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_voices_replace_the_shipped_ones() {
        let none = |_: &str| false;
        let all = |_: &str| true;
        assert_eq!(sound_file(Sound::Talk(2), &none).0, "sound\\cmonletsgo.wav");
        assert_eq!(
            sound_file(Sound::Talk(2), &all).0,
            "CustomVoices\\start1.wav"
        );
        assert_eq!(
            sound_file(Sound::Exit(3), &none).0,
            "newsounds\\qookieend1.wav"
        );
        assert_eq!(sound_file(Sound::Bounce(2, 3), &none).0, "res37.dat");
        assert_eq!(
            sound_file(Sound::Coin(4), &none).0,
            "newsounds\\peeguecoin3.wav"
        );
        assert_eq!(
            sound_file(Sound::SlideIce(0), &none),
            ("sound\\slice1.wav".into(), Some(19000))
        );
    }

    #[test]
    fn every_sound_file_exists() {
        let dir = crate::level::default_assets_dir();
        let Ok(assets) = wonderland_formats::GameAssets::open(&dir) else {
            eprintln!("skipping: no game data at {}", dir.display());
            return;
        };
        let exists = |p: &str| assets.exists(p);
        let mut sounds = vec![
            Sound::Death,
            Sound::Teleport,
            Sound::Ice,
            Sound::Transporter,
            Sound::Trampoline,
            Sound::BreakAway,
            Sound::BoxPush,
            Sound::BoulderPush,
            Sound::BoulderHit,
            Sound::BoxSplosh,
            Sound::Lava,
            Sound::Sticky,
            Sound::BoxExplode,
            Sound::BallBounce(22050),
            Sound::CoilyDeath,
            Sound::BombExplode,
            Sound::SaucerDie(16000),
            Sound::Time,
            Sound::Button,
            Sound::TimerButton,
            Sound::Hmmm,
            Sound::LastCoin,
            Sound::Lose(LoseSound::Classic),
            Sound::Lose(LoseSound::Variant('b')),
            Sound::Lose(LoseSound::Variant('c')),
            Sound::Lose(LoseSound::Variant('d')),
            Sound::Lose(LoseSound::Spanish),
            Sound::Lose(LoseSound::German),
            Sound::Lose(LoseSound::French),
        ];
        sounds.extend([0, 1, 2, 3, 4, 5, 6, 7, 11, 12, 13, 14].map(Sound::Talk));
        sounds.extend((0..5).map(Sound::SlideIce));
        sounds.extend((0..=16).map(Sound::Single));
        sounds.extend((0..=10).map(Sound::Count));
        sounds.extend((0..4).map(Sound::Lightning));
        for k in 1..=4 {
            sounds.extend([
                Sound::Convey(k),
                Sound::Coin(k),
                Sound::Clock(k),
                Sound::Bonus(k),
                Sound::Exit(k),
            ]);
            sounds.extend((1..=4).map(|i| Sound::Bounce(k, i)));
        }
        for s in sounds {
            let (path, _) = sound_file(s, &exists);
            assert!(assets.exists(&path), "{s:?}: {path}");
        }
    }
}
