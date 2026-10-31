//! Checks the piece catalogue against level setup: each piece's choices must
//! mean what the loader does with its code.

use wonderland_formats::Level;
use wonderland_formats::catalogue::{Layer, PIECES, Piece};
use wonderland_logic::Ruleset;
use wonderland_logic::setup::{LoadOptions, MonsterDir, Setup, Spawn};

const NO_OPTIONS: LoadOptions = LoadOptions {
    style: None,
    background: None,
};

/// A v6 level of floor with one piece at (1, 1).
fn setup(layer: Layer, code: i32) -> Setup {
    let mut tiles = vec![100; 9];
    let mut objects = vec![0; 9];
    match layer {
        Layer::Tile => tiles[4] = code,
        Layer::Object => objects[4] = code,
    }
    let level = Level {
        magic: "Stinky & Loof Level File v6".into(),
        version: 6,
        file_name: "a.lv5".into(),
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
        signs: vec![String::new(); 20],
        music_track: Some(1),
    };
    Setup::load(&level, &Ruleset::rtwp(), NO_OPTIONS).unwrap()
}

/// The one thing `code` spawns, if any.
fn spawned(layer: Layer, code: i32) -> Option<Spawn> {
    let spawns = setup(layer, code).spawns;
    assert!(spawns.len() <= 1, "{code}: {spawns:?}");
    spawns.first().copied()
}

fn piece(name: &str) -> &'static Piece {
    PIECES.iter().find(|p| p.name == name).unwrap()
}

/// Calls `check` with every code of a piece and its choices.
fn each(name: &str, mut check: impl FnMut(i32, Vec<usize>, Option<Spawn>)) {
    let p = piece(name);
    for code in p.base..p.base + p.count() {
        check(code, p.choices(code), spawned(p.layer, code));
    }
}

fn step(dir: usize) -> (i32, i32) {
    [(0, -1), (1, 0), (0, 1), (-1, 0)][dir]
}

#[test]
fn cannons_fire_in_their_direction() {
    each("Cannon", |_, c, s| {
        let (dx, dy) = step(c[1]);
        let speed = c[0] as i32 + 1;
        assert_eq!(
            s,
            Some(Spawn::Cannon {
                x: 1,
                y: 1,
                speed,
                dx,
                dy
            })
        );
    });
}

#[test]
fn gates_buttons_and_spikes() {
    each("Final gate", |_, c, s| {
        let dir = c[0] as i32;
        let gate = Spawn::LaserGate {
            x: 1,
            y: 1,
            col: 0,
            dir,
        };
        assert_eq!(s, Some(gate));
    });
    each("Gate", |_, c, s| {
        let gate = Spawn::LaserGate {
            x: 1,
            y: 1,
            col: c[1] as i32 + 1,
            dir: c[0] as i32,
        };
        assert_eq!(s, Some(gate));
    });
    each("Button", |_, c, s| {
        let kind = c[1] as i32 * 3 + c[0] as i32;
        assert_eq!(s, Some(Spawn::Button { x: 1, y: 1, kind }));
    });
    each("Spikes", |_, c, s| {
        let speed = c[0] as i32;
        assert_eq!(s, Some(Spawn::Spike { x: 1, y: 1, speed }));
    });
}

#[test]
fn boxes_keep_one_numbering() {
    each("Box factory", |_, c, s| {
        let kind = c[0] as i32;
        assert_eq!(s, Some(Spawn::Boxer { x: 1, y: 1, kind }));
    });
    let factory = piece("Box factory").axes[0].values;
    for (name, kind) in [
        ("Wood box", 0),
        ("Prism", 7),
        ("Sticky cube", 8),
        ("Blue link sphere", 12),
    ] {
        let p = piece(name);
        let want = Spawn::Box { x: 1, y: 1, kind };
        assert_eq!(spawned(p.layer, p.base), Some(want), "{name}");
    }
    assert_eq!(factory[7], "Prism");
    assert_eq!(factory[12], "Blue sphere");
    each("Push cannon", |code, _, s| {
        let kind = 100 + code - 1900;
        assert_eq!(s, Some(Spawn::Box { x: 1, y: 1, kind }));
    });
}

#[test]
fn monsters_keep_their_hand_and_direction() {
    // Left is the lower code of each pair; the kinds differ by family.
    for (name, left, right) in [("Z-bot", 1, 2), ("Bomber", 4, 3), ("Saucer", 5, 6)] {
        each(name, |_, c, s| {
            let kind = if c[0] == 0 { left } else { right };
            let want = Spawn::Monster {
                x: 1,
                y: 1,
                kind,
                dir: MonsterDir::Fixed(c[1] as i32),
            };
            assert_eq!(s, Some(want), "{name}");
        });
    }
    for (name, first) in [("Shadow stinker", 13), ("Fire stinker", 17)] {
        each(name, |_, c, s| {
            let want = Spawn::Monster {
                x: 1,
                y: 1,
                kind: first + c[0] as i32,
                dir: MonsterDir::Fixed(0),
            };
            assert_eq!(s, Some(want), "{name}");
        });
    }
    each("Warp gate", |_, c, s| {
        let kind = c[0] as i32;
        assert_eq!(s, Some(Spawn::Warper { x: 1, y: 1, kind }));
    });
}

#[test]
fn players_items_and_scenery() {
    each("Player", |_, c, s| {
        let id = c[0] as i32 + 1;
        assert_eq!(s, Some(Spawn::Player { id, x: 1, y: 1 }));
    });
    for (name, kind) in [("Coin", 0), ("Exit", 1), ("Clock", 2), ("Bonus coin", 3)] {
        let p = piece(name);
        let want = Spawn::Item { x: 1, y: 1, kind };
        assert_eq!(spawned(p.layer, p.base), Some(want), "{name}");
    }
    for (name, kind) in [
        ("Flowers", 22),
        ("Stinky statue", 23),
        ("Snowman", 25),
        ("Green statue", 30),
    ] {
        let p = piece(name);
        let want = Spawn::Pillar { x: 1, y: 1, kind };
        assert_eq!(spawned(p.layer, p.base), Some(want), "{name}");
    }
}

#[test]
fn settings_apply_to_the_level() {
    let code = |name| piece(name).base;
    assert!(setup(Layer::Tile, code("3D view")).view3d);
    assert!(setup(Layer::Object, code("Snow")).weather.snow);
    assert!(setup(Layer::Object, code("Rain")).weather.rain);
    assert!(setup(Layer::Object, code("Lightning")).weather.lightning);
    assert!(setup(Layer::Object, code("Colour flashes")).weather.psyche);
    let tint = setup(Layer::Object, code("Tint") + 3).channels;
    assert!(tint.red && tint.green && !tint.blue);
}
