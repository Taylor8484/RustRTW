//! A longer random run of every RTWP and TOW level, looking for panics
//! (reads off the map, emptied lists). TOW's levels are parsed as TOW reads
//! them and play under `Ruleset::tow()`. Ignored by default because it takes
//! minutes; run it with `cargo test --release -p wonderland-logic --test
//! soak -- --ignored`. `WL_SOAK_SEEDS` sets how many input seeds to try
//! (default 4).

use std::path::Path;

use wonderland_formats::{Game, GameAssets, Level};
use wonderland_logic::{BlitzRng, Completion, Input, Options, Ruleset, Session, World};

/// Every level in the pack, plus the given loose files.
fn level_names(assets: &GameAssets, loose: &[&str]) -> Vec<String> {
    assets
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
        .chain(loose.iter().map(|n| (*n).to_owned()))
        .collect()
}

#[test]
#[ignore]
fn every_rtwp_level_survives_random_play() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/RTWP");
    if !dir.is_dir() {
        eprintln!("skipping: {} not found", dir.display());
        return;
    }
    let assets = GameAssets::open(&dir).unwrap();
    let names = level_names(&assets, &[]);
    soak(&assets, &names, Game::Rtwp, Ruleset::rtwp());
}

#[test]
#[ignore]
fn every_tow_level_survives_random_play() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/TOW");
    if !dir.is_dir() {
        eprintln!("skipping: {} not found", dir.display());
        return;
    }
    let assets = GameAssets::open(&dir).unwrap();
    // The bonus levels came as a separate download, beside the pack.
    let bonus = [
        "bonus1.lev",
        "bonus2.lev",
        "bonus3.lev",
        "bonus4.lev",
        "bonus5.lev",
    ];
    let names = level_names(&assets, &bonus);
    soak(&assets, &names, Game::Tow, Ruleset::tow());
}

fn soak(assets: &GameAssets, names: &[String], game: Game, ruleset: Ruleset) {
    let seeds: i32 = std::env::var("WL_SOAK_SEEDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);

    let (mut won, mut lost) = (0, 0);
    for seed in 1..=seeds {
        let mut keys = BlitzRng::new(seed * 7919);
        let mut session = Session::default();
        for name in names {
            let level = Level::parse(&assets.read(name).unwrap(), game).unwrap();
            let options = Options {
                detail_level: 1 + seed % 5,
                ..Options::default()
            };
            let mut world = World::start(&level, ruleset.clone(), options, session)
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            // Held keys that change every 1 to 30 ticks.
            let mut ticks = 0;
            'play: while ticks < 20_000 {
                let bits =
                    keys.rand(0, 15) as u8 | (u8::from(keys.rand(0, 19) == 0) << keys.rand(4, 5));
                let input = Input::from_bits(bits);
                for _ in 0..keys.rand(1, 30) {
                    world.step(input);
                    ticks += 1;
                    if matches!(world.completion, Completion::Done | Completion::Restart) {
                        break 'play;
                    }
                }
            }
            match world.completion {
                Completion::Done => won += 1,
                Completion::Restart => lost += 1,
                _ => {}
            }
            session = world.into_session();
        }
    }
    eprintln!(
        "{game:?}: {} runs: {won} won, {lost} lost, {} timed out",
        seeds as usize * names.len(),
        seeds as usize * names.len() - won - lost
    );
}
