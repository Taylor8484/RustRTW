//! Checks that every texture, sound and music file the games ship decodes
//! with the crates the game crate will use: `image` for bmp/jpg textures,
//! `hound` for wav (what Bevy's audio decodes), and `xmrs`/`xmrsplayer` for
//! the Impulse Tracker music. Covers pack entries by extension and the loose
//! `res*.dat` files in each game folder by their magic bytes. Games whose
//! folder is missing are skipped.

use std::fs;
use std::path::{Path, PathBuf};

use wonderland_formats::GameAssets;
use xmrs::prelude::*;
use xmrsplayer::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Bmp,
    Jpg,
    Wav,
    Music,
}

fn kind_from_magic(data: &[u8]) -> Option<Kind> {
    match data {
        [b'B', b'M', ..] => Some(Kind::Bmp),
        [0xFF, 0xD8, ..] => Some(Kind::Jpg),
        [b'R', b'I', b'F', b'F', ..] => Some(Kind::Wav),
        [b'I', b'M', b'P', b'M', ..] => Some(Kind::Music),
        _ => None,
    }
}

fn kind_from_name(name: &str) -> Option<Kind> {
    let name = name.to_lowercase();
    let ext = name.rsplit_once('.')?.1;
    match ext {
        "bmp" => Some(Kind::Bmp),
        "jpg" => Some(Kind::Jpg),
        "wav" => Some(Kind::Wav),
        _ => None,
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every media file of one game: pack entries by extension, then loose
/// `res*.dat` by magic.
fn media(game: &str) -> Option<Vec<(String, Kind, Vec<u8>)>> {
    let dir = repo_root().join("assets").join(game);
    if !dir.is_dir() {
        eprintln!("skipping {game}: {} not found", dir.display());
        return None;
    }
    let assets = GameAssets::open(&dir).unwrap();
    let pack = assets.pack().unwrap();
    let mut files = vec![];
    for e in pack.index().entries() {
        if let Some(kind) = kind_from_name(&e.name) {
            let data = pack.entry_data(e).to_vec();
            assert_eq!(
                kind_from_magic(&data),
                Some(kind),
                "{game}: {} does not match its extension",
                e.name
            );
            files.push((e.name.clone(), kind, data));
        }
    }
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_lowercase();
        if !name.starts_with("res") || !name.ends_with(".dat") {
            continue;
        }
        let data = fs::read(&path).unwrap();
        if let Some(kind) = kind_from_magic(&data) {
            files.push((name, kind, data));
        }
    }
    Some(files)
}

fn count(files: &[(String, Kind, Vec<u8>)], kind: Kind) -> usize {
    files.iter().filter(|f| f.1 == kind).count()
}

#[test]
fn every_texture_decodes() {
    for game in ["TOW", "RTWP", "WSW"] {
        let Some(files) = media(game) else { continue };
        for (name, kind, data) in &files {
            let format = match kind {
                Kind::Bmp => image::ImageFormat::Bmp,
                Kind::Jpg => image::ImageFormat::Jpeg,
                _ => continue,
            };
            let img = image::load_from_memory_with_format(data, format)
                .unwrap_or_else(|e| panic!("{game}: {name}: {e}"));
            assert!(img.width() > 0 && img.height() > 0, "{game}: {name}");
        }
        assert!(count(&files, Kind::Bmp) + count(&files, Kind::Jpg) > 0);
    }
}

#[test]
fn every_sound_is_pcm_wav() {
    for game in ["TOW", "RTWP", "WSW"] {
        let Some(files) = media(game) else { continue };
        for (name, _, data) in files.iter().filter(|f| f.1 == Kind::Wav) {
            let mut reader = hound::WavReader::new(data.as_slice())
                .unwrap_or_else(|e| panic!("{game}: {name}: {e}"));
            let spec = reader.spec();
            assert_eq!(spec.sample_format, hound::SampleFormat::Int, "{name}");
            assert!(matches!(spec.bits_per_sample, 8 | 16), "{name}: {spec:?}");
            let samples = match spec.bits_per_sample {
                8 => reader
                    .samples::<i8>()
                    .collect::<Result<Vec<_>, _>>()
                    .map(|s| s.len()),
                _ => reader
                    .samples::<i16>()
                    .collect::<Result<Vec<_>, _>>()
                    .map(|s| s.len()),
            }
            .unwrap_or_else(|e| panic!("{game}: {name}: {e}"));
            assert!(samples > 0, "{game}: {name}: no samples");
        }
    }
}

/// Each module loads and renders its first second of audio, which is not all
/// silence.
#[test]
fn every_music_module_plays() {
    const RATE: u32 = 44_100;
    let expected = [("TOW", 5), ("RTWP", 6), ("WSW", 6)];
    for (game, n) in expected {
        let Some(files) = media(game) else { continue };
        assert_eq!(count(&files, Kind::Music), n, "{game}: module count");
        for (name, _, data) in files.iter().filter(|f| f.1 == Kind::Music) {
            let module = Module::load(data).unwrap_or_else(|e| panic!("{game}: {name}: {e:?}"));
            let mut player = XmrsPlayer::new(&module, RATE, 0);
            player.set_max_loop_count(1);
            let loud = player
                .by_ref()
                .take(RATE as usize * 2)
                .filter(|s| *s != 0)
                .count();
            assert!(loud > 0, "{game}: {name}: silent");
        }
    }
}
