//! Round-trips the real config files in `assets/<GAME>` (gitignored). Missing
//! games or files are skipped.

use std::fs;
use std::path::{Path, PathBuf};

use wonderland_formats::Game;
use wonderland_formats::config::{ExtraCfg, Options, StoryDat};

fn read(game: &str, file: &str) -> Option<Vec<u8>> {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets")
        .join(game)
        .join(file);
    match fs::read(&path) {
        Ok(b) => Some(b),
        Err(_) => {
            eprintln!("skipping {}", path.display());
            None
        }
    }
}

#[test]
fn options_cfg_round_trips() {
    for (name, game) in [("TOW", Game::Tow), ("RTWP", Game::Rtwp), ("WSW", Game::Wsw)] {
        let Some(bytes) = read(name, "options.cfg") else {
            continue;
        };
        let o = Options::parse(game, &bytes).unwrap();
        assert_eq!(o.to_bytes(), bytes, "{name}");
    }
}

#[test]
fn extra_cfg_round_trips() {
    if let Some(bytes) = read("TOW", "extra.cfg") {
        assert_eq!(ExtraCfg::parse(&bytes).unwrap().to_bytes(), bytes);
    }
}

#[test]
fn story_dat_round_trips() {
    if let Some(bytes) = read("WSW", "story.dat") {
        assert_eq!(StoryDat::parse(&bytes).unwrap().to_bytes(), bytes);
    }
}
