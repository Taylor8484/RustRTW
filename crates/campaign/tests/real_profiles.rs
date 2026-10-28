//! Loads every real RTWP player file in `assets/RTWP/players` as
//! `LoadPlayer` does. Saving a full file again (`SavePlayer`) gives back
//! the same bytes. Skipped when the folder is missing.

use std::fs;
use std::path::Path;

use wonderland_campaign::profile::{Players, Profile};
use wonderland_formats::save::Progress;
use wonderland_formats::{Game, PlayerFile};

#[test]
fn real_profiles_load_and_save_unchanged() {
    let game = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/RTWP");
    let players = Players::new(&game);
    if !players.dir.is_dir() {
        eprintln!("skipping: {} not found", players.dir.display());
        return;
    }
    let mut seen = 0;
    for entry in fs::read_dir(&players.dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("pla") {
            continue;
        }
        let data = fs::read(&path).unwrap();
        let mut profile =
            Profile::load(&data).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let file = PlayerFile::parse(&data, Game::Rtwp).unwrap();
        assert_eq!(profile.name, file.name);
        let full = matches!(&file.progress, Progress::Rtwp(r) if r.perfect.is_some());
        if full {
            assert_eq!(profile.save_bytes(), data, "{}", path.display());
        }
        seen += 1;
    }
    assert!(seen > 0);
}
