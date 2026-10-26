//! Starts every RTWP level in `assets/RTWP` (gitignored; populate it with
//! `tools/import-assets.sh`) and runs it. Skipped when the folder is missing.

use std::path::Path;

use wonderland_formats::{Game, GameAssets, Level};
use wonderland_logic::{BlitzRng, Input, Options, Player, Ruleset, Session, World};

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

    // Held keys that change every 13 ticks: arrows, and now and then the
    // free camera or a player switch.
    let mut keys = BlitzRng::new(7);
    let inputs: Vec<Input> = (0..3000 / 13 + 1)
        .flat_map(|_| {
            let bits = keys.rand(0, 15) as u8 | (u8::from(keys.rand(0, 9) == 0) << keys.rand(4, 5));
            [Input::from_bits(bits); 13]
        })
        .take(3000)
        .collect();

    // One session across all levels, as when playing them in a row.
    let mut session = Session::default();
    let mut walked = 0;
    let (mut with_monsters, mut monsters_moved) = (0, 0);
    let (mut with_exit, mut took_items) = (0, 0);
    for name in names {
        let level = Level::parse(&assets.read(&name).unwrap(), Game::Rtwp).unwrap();
        let mut world = World::start(&level, Ruleset::rtwp(), Options::default(), session)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        let start = world.players.clone();
        let monsters = world.monsters.clone();
        let items = world.items.len();
        if world.items.iter().any(|it| it.kind == 1) {
            with_exit += 1;
        }
        let replay = world.clone();
        for &input in &inputs {
            world.step(input);
        }
        let mut again = replay;
        for &input in &inputs {
            again.step(input);
        }
        assert_eq!(world, again, "{name}: not deterministic");
        let moved = |(a, b): (&Player, &Player)| (a.x, a.y) != (b.x, b.y);
        if world.players.iter().zip(&start).any(moved) {
            walked += 1;
        }
        if !monsters.is_empty() {
            with_monsters += 1;
            let still = world.monsters.len() == monsters.len()
                && world
                    .monsters
                    .iter()
                    .zip(&monsters)
                    .all(|(a, b)| (a.x, a.y) == (b.x, b.y));
            if !still {
                monsters_moved += 1;
            }
        }
        if world.items.len() < items || world.both_completed != 0 {
            took_items += 1;
        }
        session = world.into_session();
    }
    eprintln!("{walked} levels saw a player move");
    eprintln!("{monsters_moved} of {with_monsters} levels with monsters saw them move");
    eprintln!("{took_items} levels saw an item taken; {with_exit} have an exit");
    assert!(walked > 120, "only {walked} levels saw a player move");
    assert!(
        took_items > 30,
        "only {took_items} levels saw an item taken"
    );
    // The rest are shut in by boxes or by gates that need buttons (2.2e).
    assert!(
        monsters_moved > 80,
        "only {monsters_moved} levels saw monsters move"
    );
}

#[test]
fn player_models_have_the_frames_the_animations_assume() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/RTWP");
    if !dir.is_dir() {
        eprintln!("skipping: {} not found", dir.display());
        return;
    }
    let assets = GameAssets::open(&dir).unwrap();
    for name in [r"graphics\stinky.md2", r"graphics\peegueb.md2"] {
        let model = wonderland_formats::mesh::md2::parse(&assets.read(name).unwrap()).unwrap();
        assert_eq!(
            model.frames.len() as i32,
            wonderland_logic::player::FRAMES,
            "{name}"
        );
    }
}
