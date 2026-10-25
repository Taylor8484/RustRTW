//! Starts every RTWP level in `assets/RTWP` (gitignored; populate it with
//! `tools/import-assets.sh`) and runs it. Skipped when the folder is missing.

use std::path::Path;

use wonderland_formats::{Game, GameAssets, Level};
use wonderland_logic::{Input, Options, Ruleset, Session, World};

#[test]
fn every_rtwp_level_starts_and_runs() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/RTWP");
    if !dir.is_dir() {
        eprintln!("skipping: {} not found", dir.display());
        return;
    }
    let assets = GameAssets::open(&dir).unwrap();
    let names: Vec<String> = assets
        .pack()
        .unwrap()
        .index()
        .entries()
        .iter()
        .map(|e| e.name.clone())
        .filter(|n| {
            let n = n.to_lowercase();
            n.ends_with(".lev") || n.ends_with(".lv5")
        })
        .collect();
    assert_eq!(names.len(), 85 + 71);

    // One session across all levels, as when playing them in a row.
    let mut session = Session::default();
    for name in names {
        let level = Level::parse(&assets.read(&name).unwrap(), Game::Rtwp).unwrap();
        let mut world = World::start(&level, Ruleset::rtwp(), Options::default(), session)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        let replay = world.clone();
        for t in 0..1000u32 {
            world.step(Input::from_bits((t / 13 % 16) as u8));
        }
        let mut again = replay;
        for t in 0..1000u32 {
            again.step(Input::from_bits((t / 13 % 16) as u8));
        }
        assert_eq!(world, again, "{name}: not deterministic");
        session = world.into_session();
    }
}
