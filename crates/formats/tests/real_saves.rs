//! Round-trips every real player save in `assets/<GAME>/players`. Games whose
//! folder is missing are skipped.

use std::fs;
use std::path::Path;

use wonderland_formats::{Game, PlayerFile};

#[test]
fn every_save_round_trips() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let expected = [
        (Game::Tow, "TOW", 2),
        (Game::Rtwp, "RTWP", 4),
        (Game::Wsw, "WSW", 3),
    ];
    for (game, name, count) in expected {
        let dir = root.join(name).join("players");
        if !dir.is_dir() {
            eprintln!("skipping {name}: {} not found", dir.display());
            continue;
        }
        let mut seen = 0;
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            let ext = path.extension().and_then(|e| e.to_str());
            if !matches!(ext, Some("pla" | "bak")) {
                continue;
            }
            let data = fs::read(&path).unwrap();
            let save = PlayerFile::parse(&data, game)
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            assert!(save.trailing.is_empty(), "{}", path.display());
            assert_eq!(
                (save.levels_completed, save.total_score),
                save.computed_summary(),
                "{}: stored summary differs from recomputed",
                path.display()
            );
            assert_eq!(save.to_bytes(), data, "{}", path.display());
            seen += 1;
        }
        assert_eq!(seen, count, "{name}: save file count");
    }
}

/// Header values read off a hexdump of the real files.
#[test]
fn header_fields_match_hand_checked_values() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let load = |game: &str, file: &str, g: Game| {
        let path = root.join(game).join("players").join(file);
        let data = fs::read(&path).ok()?;
        Some(PlayerFile::parse(&data, g).unwrap())
    };
    if let Some(p) = load("TOW", "noah.pla", Game::Tow) {
        assert_eq!((p.name.as_str(), p.id, p.position), ("NOAH", None, 1));
    }
    if let Some(p) = load("RTWP", "noah.pla", Game::Rtwp) {
        assert_eq!(p.name, "NOAH");
        assert_eq!(p.id, Some(0x000c_da61));
        assert_eq!((p.position, p.levels_completed), (20, 29));
    }
    if let Some(p) = load("WSW", "noah.pla", Game::Wsw) {
        assert_eq!(p.name, "NOAH");
        assert_eq!(p.id, Some(0x085a_cd77));
        assert_eq!((p.position, p.levels_completed), (0x24, 0x2c));
    }
}
