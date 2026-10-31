//! Checks `Level::upgraded` against level setup: an old level must set up the
//! same as a v6 file. Setup mirrors the game's loaders, so this holds the
//! upgrade to what the game does with the old file.

use std::path::Path;

use wonderland_formats::catalogue::{self, Layer};
use wonderland_formats::{Game, GameAssets, Level};
use wonderland_logic::Ruleset;
use wonderland_logic::setup::{LoadOptions, Setup};

const NO_OPTIONS: LoadOptions = LoadOptions {
    style: None,
    background: None,
};

/// The setup, without what a v6 file cannot keep. Button tiles are renumbered
/// and only their class is read once the buttons exist.
fn setup(level: &Level, ruleset: &Ruleset) -> Setup {
    let mut setup = Setup::load(level, ruleset, NO_OPTIONS).unwrap();
    setup.version = 0;
    setup.map_id = None;
    setup.music_track = None;
    let cells: Vec<_> = setup.grid.cells().collect();
    for (x, y) in cells {
        if setup.grid.get(x, y) / 100 == 11 {
            setup.grid.set(x, y, 1100);
        }
    }
    setup
}

fn assert_same(level: &Level, ruleset: &Ruleset, what: &str) {
    let upgraded = level.upgraded();
    let bytes = upgraded.to_bytes();
    assert_eq!(
        Level::parse(&bytes, Game::Rtwp).unwrap(),
        upgraded,
        "{what}"
    );
    assert_eq!(setup(level, ruleset), setup(&upgraded, ruleset), "{what}");
}

/// A level of floor with one tile and one object at (1, 1).
fn level(version: u8, tile: i32, object: i32) -> Level {
    let mut tiles = vec![100; 9];
    let mut objects = vec![0; 9];
    tiles[4] = tile;
    objects[4] = object;
    Level {
        magic: format!("Stinky & Loof Level File v{version}"),
        version,
        file_name: "a.lev".into(),
        map_id: Some(1),
        description: "Test".into(),
        houses: None,
        models: None,
        textures: None,
        backgrounds: None,
        time: 300,
        style: 2,
        background: 0,
        width: 3,
        height: 3,
        tiles,
        objects,
        signs: vec![String::new(); 5],
        music_track: (version >= 4).then_some(1),
    }
}

#[test]
fn every_old_code_keeps_its_meaning() {
    let rtwp = Ruleset::rtwp();
    for version in 1..=4 {
        for object in -1..60 {
            let what = format!("v{version} object {object}");
            assert_same(&level(version, 100, object), &rtwp, &what);
        }
        // Up to the classes old levels do not have, and the buttons' end.
        for tile in (-1..1197).chain(1200..1600) {
            let what = format!("v{version} tile {tile}");
            assert_same(&level(version, tile, 0), &rtwp, &what);
        }
    }
}

#[test]
fn the_hidden_rainbow_world_stays() {
    let mut old = level(4, 100, 0);
    old.background = 10;
    assert_same(&old, &Ruleset::rtwp(), "background 10");
    assert_eq!(setup(&old.upgraded(), &Ruleset::rtwp()).background, 10);
}

#[test]
fn every_shipped_level_upgrades() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    for (folder, game, ruleset, count) in [
        ("TOW", Game::Tow, Ruleset::tow(), 87),
        ("RTWP", Game::Rtwp, Ruleset::rtwp(), 85 + 71),
        ("WSW", Game::Wsw, Ruleset::rtwp(), 110),
    ] {
        let dir = root.join(folder);
        if !dir.is_dir() {
            eprintln!("skipping: {} not found", dir.display());
            continue;
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
        let mut upgraded = 0;
        for name in names {
            let Ok(old) = Level::parse(&assets.read(&name).unwrap(), game) else {
                continue;
            };
            assert_same(&old, &ruleset, &name);
            let new = old.upgraded();
            for &code in &new.tiles {
                assert!(
                    catalogue::find(Layer::Tile, code).is_some(),
                    "{name}: tile {code}"
                );
            }
            for &code in new.objects.iter().filter(|&&c| c != 0) {
                assert!(
                    catalogue::find(Layer::Object, code).is_some(),
                    "{name}: object {code}"
                );
            }
            upgraded += 1;
        }
        assert_eq!(upgraded, count, "{folder}");
    }
}

#[test]
fn a_real_v6_level_is_left_alone() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/HOLIDAYA.LV6");
    if !path.is_file() {
        eprintln!("skipping: {} not found", path.display());
        return;
    }
    let data = std::fs::read(path).unwrap();
    let level = Level::parse(&data, Game::Rtwp).unwrap();
    assert_eq!(level.version, 6);
    assert_eq!(level.to_bytes(), data);
    assert_eq!(level.upgraded(), level);
    assert_same(&level, &Ruleset::rtwp(), "HOLIDAYA");
}
