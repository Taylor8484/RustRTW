//! Level setup: what `LoadLevel`, `LoadLevelOldVersion` and `LoadLevelv5` in
//! `RTWP/level.bb` do with a level's data after reading it.
//!
//! The loaders rewrite some tiles while scanning (signs become 1300, the
//! first-person marker 2300 becomes floor, electro, spike and other
//! object-like tiles become floor 100) and create game objects from tiles and
//! from the object layer. Object creation is recorded as a list of
//! [`Spawn`]s in the order the original calls its `Create*` functions, since
//! that order fixes each object's array index and the order of any random
//! draws during creation.

use wonderland_formats::Level;

use crate::{Error, Grid, Result, Ruleset};

/// One `Create*` call made while loading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spawn {
    /// `CreatePlayer(id, x, y)`, id 1–4.
    Player { id: i32, x: i32, y: i32 },
    /// `CreateCannon(x, y, speed, dx, dy)`.
    Cannon {
        x: i32,
        y: i32,
        speed: i32,
        dx: i32,
        dy: i32,
    },
    /// `CreateLaserGate(x, y, col, dir)`.
    LaserGate { x: i32, y: i32, col: i32, dir: i32 },
    /// `CreateButton(x, y, kind)`.
    Button { x: i32, y: i32, kind: i32 },
    /// `CreateTransporter(x, y)`.
    Transporter { x: i32, y: i32 },
    /// `CreateSpike(x, y, speed)`.
    Spike { x: i32, y: i32, speed: i32 },
    /// `CreateBoxer(x, y, kind)`: a box factory; its tile stays.
    Boxer { x: i32, y: i32, kind: i32 },
    /// `CreateWarper(x, y, kind)`: a warp gate; its tile stays.
    Warper { x: i32, y: i32, kind: i32 },
    /// `CreateBox(x, y, kind)`: boxes, push cannons (100+), sticky cubes
    /// (8) and link spheres (9+).
    Box { x: i32, y: i32, kind: i32 },
    /// `CreateMonster(x, y, kind, dir)`.
    Monster {
        x: i32,
        y: i32,
        kind: i32,
        dir: MonsterDir,
    },
    /// `CreateItem(x, y, kind)`: 0 coin, 1 gate, 2 clock, 3 bonus coin.
    Item { x: i32, y: i32, kind: i32 },
    /// `CreateFish(x, y)`.
    Fish { x: i32, y: i32 },
    /// `CreateFireball(x, y, dx, dy)`.
    Fireball { x: i32, y: i32, dx: i32, dy: i32 },
    /// `CreatePillar(x, y, kind)`.
    Pillar { x: i32, y: i32, kind: i32 },
}

/// A monster's starting direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterDir {
    Fixed(i32),
    /// The broken z-bot's `Rand(0, 3)`, drawn when the monster is created.
    Random,
}

/// A sign tile (13xx): where sign `i` stands and its text. Unused signs sit
/// at (-1, -1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sign {
    pub x: i32,
    pub y: i32,
    pub text: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Weather {
    pub snow: bool,
    pub rain: bool,
    pub lightning: bool,
    /// Random colour flashes.
    pub psyche: bool,
}

/// Which light channels the level uses (`LevelRed` and friends); objects
/// 80–83 tint the level by switching some off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Channels {
    pub red: bool,
    pub green: bool,
    pub blue: bool,
}

/// Menu settings that change how a level loads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadOptions {
    /// `MenuStyle`: replaces the level's style when set.
    pub style: Option<i32>,
    /// `MenuBkg`: replaces the level's background when set.
    pub background: Option<i32>,
}

/// A level as the loaders leave it, before any object exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setup {
    pub version: u8,
    pub map_id: Option<i32>,
    /// `MapName$`: the level title.
    pub name: String,
    /// `MapTime`: the time limit in seconds.
    pub time: i32,
    pub style: i32,
    pub background: i32,
    pub grid: Grid,
    /// `ElectroMap`: each tile's electro type, or -1, row by row. The
    /// electro tiles themselves become floor.
    pub electros: Vec<i32>,
    /// Always 20. Levels before v5 only store the first 5 texts; the others
    /// keep the previous level's text in the original and are empty here.
    pub signs: Vec<Sign>,
    /// Tile 2300 makes the level first-person.
    pub view3d: bool,
    pub weather: Weather,
    pub channels: Channels,
    pub spawns: Vec<Spawn>,
    /// `ExtraLevelMusicTrack` (v4+). Older levels keep the previous level's
    /// track in the original.
    pub music_track: Option<i32>,
}

impl Setup {
    /// Sets up `level` as RTWP's loaders do.
    pub fn load(level: &Level, ruleset: &Ruleset, options: LoadOptions) -> Result<Self> {
        if level.width > ruleset.map_size || level.height > ruleset.map_size {
            return Err(Error::Level(format!(
                "{}x{} does not fit a {} grid",
                level.width, level.height, ruleset.map_size
            )));
        }
        let v5 = level.version >= 5;
        let mut grid = Grid::new(ruleset.map_size, level.width, level.height);
        let mut spawns = Vec::new();
        let mut signs: Vec<Sign> = (0..20)
            .map(|i| Sign {
                x: -1,
                y: -1,
                text: level.signs.get(i).cloned().unwrap_or_default(),
            })
            .collect();
        let mut view3d = false;

        let mut background = options.background.unwrap_or(level.background);
        if v5 && background == 10 {
            // 9 and 10 are both the rainbow world.
            background = 11;
        }

        for (x, y) in grid.cells() {
            let mut t = level.tile(x as usize, y as usize);
            match t / 100 {
                8 => {
                    let dir = match (t - 800) / 4 {
                        0 => Some((0, -1)),
                        1 => Some((1, 0)),
                        2 => Some((0, 1)),
                        3 => Some((-1, 0)),
                        _ => None,
                    };
                    if let Some((dx, dy)) = dir {
                        let speed = t % 4 + 1;
                        spawns.push(Spawn::Cannon {
                            x,
                            y,
                            speed,
                            dx,
                            dy,
                        });
                    }
                }
                10 => spawns.push(Spawn::LaserGate {
                    x,
                    y,
                    col: (t % 100) / 2,
                    dir: t % 2,
                }),
                11 => {
                    let kind = t - 1100;
                    // Old levels have no timer buttons, so their colours
                    // skip a block of three.
                    let kind = if v5 || kind < 3 { kind } else { kind + 3 };
                    spawns.push(Spawn::Button { x, y, kind });
                }
                13 => {
                    if let Some(sign) = usize::try_from(t - 1300)
                        .ok()
                        .and_then(|i| signs.get_mut(i))
                    {
                        (sign.x, sign.y) = (x, y);
                    }
                    t = 1300;
                }
                17 if v5 => spawns.push(Spawn::Transporter { x, y }),
                _ => {}
            }
            if v5 && t == 2300 {
                view3d = true;
                t = 100;
            }
            grid.set(x, y, t);
        }

        // CreateElectro runs before the electro tiles become floor.
        let electros = grid
            .cells()
            .map(|(x, y)| {
                let t = grid.get(x, y);
                if t / 100 == 7 { t % 100 } else { -1 }
            })
            .collect();
        let floor = |grid: &mut Grid, class: i32| {
            for (x, y) in grid.cells() {
                if grid.get(x, y) / 100 == class {
                    grid.set(x, y, 100);
                }
            }
        };
        floor(&mut grid, 7);

        // Then each object-like tile class in turn, in separate scans.
        let mut push =
            |grid: &mut Grid, class: i32, to_floor: bool, f: &dyn Fn(i32, i32, i32) -> Spawn| {
                for (x, y) in grid.cells() {
                    let t = grid.get(x, y);
                    if t / 100 == class {
                        spawns.push(f(x, y, t));
                        if to_floor {
                            grid.set(x, y, 100);
                        }
                    }
                }
            };
        push(&mut grid, 14, true, &|x, y, t| Spawn::Spike {
            x,
            y,
            speed: t % 100,
        });
        if v5 {
            push(&mut grid, 16, false, &|x, y, t| Spawn::Boxer {
                x,
                y,
                kind: t % 100,
            });
            push(&mut grid, 22, false, &|x, y, t| Spawn::Warper {
                x,
                y,
                kind: t % 100,
            });
            push(&mut grid, 19, true, &|x, y, t| Spawn::Box {
                x,
                y,
                kind: 100 + t % 100,
            });
            push(&mut grid, 20, true, &|x, y, _| Spawn::Box { x, y, kind: 8 });
            push(&mut grid, 21, true, &|x, y, t| Spawn::Box {
                x,
                y,
                kind: 9 + t - 2100,
            });
            push(&mut grid, 24, true, &|x, y, t| Spawn::Monster {
                x,
                y,
                kind: 13 + t - 2400,
                dir: MonsterDir::Fixed(0),
            });
        }

        let mut weather = Weather::default();
        let mut channels = Channels {
            red: true,
            green: true,
            blue: true,
        };
        let objects = match level.version {
            1 => object_v1,
            2..=4 => object_v2,
            _ => object_v5,
        };
        for (x, y) in grid.cells() {
            let obj = level.object(x as usize, y as usize);
            match objects(obj, x, y) {
                Object::Spawn(s) => spawns.push(s),
                Object::Channels(r, g, b) => {
                    channels = Channels {
                        red: r,
                        green: g,
                        blue: b,
                    }
                }
                Object::Snow => weather.snow = true,
                Object::Rain => weather.rain = true,
                Object::Lightning => weather.lightning = true,
                Object::Psyche => weather.psyche = true,
                Object::HiddenRainbow => {
                    if background == 9 {
                        background = 10;
                    }
                }
                Object::None => {}
            }
        }

        Ok(Self {
            version: level.version,
            map_id: level.map_id,
            name: level.description.clone(),
            time: level.time,
            style: options.style.unwrap_or(level.style),
            background,
            grid,
            electros,
            signs,
            view3d,
            weather,
            channels,
            spawns,
            music_track: level.music_track,
        })
    }
}

/// What one object code does.
enum Object {
    None,
    Spawn(Spawn),
    Channels(bool, bool, bool),
    Snow,
    Rain,
    Lightning,
    Psyche,
    /// Object 88: the hidden rainbow world (background 9 becomes 10).
    HiddenRainbow,
}

fn monster(x: i32, y: i32, kind: i32, dir: i32) -> Object {
    Object::Spawn(Spawn::Monster {
        x,
        y,
        kind,
        dir: MonsterDir::Fixed(dir),
    })
}

fn spawn(s: Spawn) -> Object {
    Object::Spawn(s)
}

/// `LoadLevelv5`'s object codes.
fn object_v5(obj: i32, x: i32, y: i32) -> Object {
    match obj {
        1..=4 => spawn(Spawn::Player { id: obj, x, y }),
        5..=8 => spawn(Spawn::Item {
            x,
            y,
            kind: obj - 5,
        }),
        9..=16 => spawn(Spawn::Box {
            x,
            y,
            kind: obj - 9,
        }),
        17 => monster(x, y, 0, 0),
        18..=25 => monster(x, y, obj % 2 + 1, (obj - 18) / 2),
        26 => spawn(Spawn::Monster {
            x,
            y,
            kind: 11,
            dir: MonsterDir::Random,
        }),
        27..=34 => monster(x, y, obj % 2 + 3, (obj - 27) / 2),
        35..=42 => monster(x, y, (obj + 1) % 2 + 5, (obj - 35) / 2),
        43 => monster(x, y, 7, 0),
        44 => monster(x, y, 8, 0),
        45 => monster(x, y, 9, 0),
        46 => monster(x, y, 10, 0),
        47 => monster(x, y, 12, 0),
        48 => spawn(Spawn::Fish { x, y }),
        49 => spawn(Spawn::Fireball { x, y, dx: 0, dy: 0 }),
        50..=79 => spawn(Spawn::Pillar {
            x,
            y,
            kind: obj - 50,
        }),
        80 => Object::Channels(true, false, false),
        81 => Object::Channels(false, true, false),
        82 => Object::Channels(false, false, true),
        83 => Object::Channels(true, true, false),
        84 => Object::Snow,
        85 => Object::Rain,
        86 => Object::Lightning,
        87 => Object::Psyche,
        88 => Object::HiddenRainbow,
        89 => spawn(Spawn::Pillar { x, y, kind: 30 }),
        _ => Object::None,
    }
}

/// `LoadLevelOldVersion`'s codes for v1 files.
fn object_v1(obj: i32, x: i32, y: i32) -> Object {
    match obj {
        1 | 2 => spawn(Spawn::Player { id: obj, x, y }),
        3 => spawn(Spawn::Box { x, y, kind: 0 }),
        4 => spawn(Spawn::Box { x, y, kind: 1 }),
        5 => spawn(Spawn::Item { x, y, kind: 1 }),
        6 => spawn(Spawn::Item { x, y, kind: 0 }),
        7 => monster(x, y, 0, 0),
        8..=15 => monster(x, y, obj % 2 + 1, (obj - 8) / 2),
        16..=23 => monster(x, y, (obj + 1) % 2 + 3, (obj - 16) / 2),
        24 => spawn(Spawn::Item { x, y, kind: 2 }),
        25 => spawn(Spawn::Item { x, y, kind: 3 }),
        26 => spawn(Spawn::Fish { x, y }),
        27 => Object::Snow,
        28..=30 => spawn(Spawn::Pillar {
            x,
            y,
            kind: obj - 28,
        }),
        31..=38 => monster(x, y, (1 - obj % 2) + 5, (obj - 31) / 2),
        39 => monster(x, y, 7, 0),
        40 => monster(x, y, 8, 0),
        41 => monster(x, y, 9, 0),
        42..=45 => spawn(Spawn::Box {
            x,
            y,
            kind: obj - 40,
        }),
        46 => spawn(Spawn::Pillar { x, y, kind: 20 }),
        47 => spawn(Spawn::Pillar { x, y, kind: 16 }),
        48 => Object::Rain,
        49 => Object::Lightning,
        50 => Object::Psyche,
        _ => Object::None,
    }
}

/// `LoadLevelOldVersion`'s codes for v2–v4 files.
fn object_v2(obj: i32, x: i32, y: i32) -> Object {
    match obj {
        1 | 2 => spawn(Spawn::Player { id: obj, x, y }),
        3..=6 => spawn(Spawn::Item {
            x,
            y,
            kind: obj - 3,
        }),
        7..=12 => spawn(Spawn::Box {
            x,
            y,
            kind: obj - 7,
        }),
        13 => monster(x, y, 0, 0),
        14..=21 => monster(x, y, obj % 2 + 1, (obj - 14) / 2),
        22..=29 => monster(x, y, (obj + 1) % 2 + 3, (obj - 22) / 2),
        30..=37 => monster(x, y, obj % 2 + 5, (obj - 30) / 2),
        38 => monster(x, y, 7, 0),
        39 => monster(x, y, 8, 0),
        40 => monster(x, y, 9, 0),
        41 => spawn(Spawn::Fish { x, y }),
        42..=44 => spawn(Spawn::Pillar {
            x,
            y,
            kind: obj - 42,
        }),
        45 => spawn(Spawn::Pillar { x, y, kind: 16 }),
        46 => spawn(Spawn::Pillar { x, y, kind: 20 }),
        47 => Object::Snow,
        48 => Object::Rain,
        49 => Object::Lightning,
        50 => Object::Psyche,
        _ => Object::None,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A level with the given tile and object layers, row by row.
    pub(crate) fn level(version: u8, width: usize, tiles: &[i32], objects: &[i32]) -> Level {
        assert_eq!(tiles.len(), objects.len());
        Level {
            magic: format!("Stinky & Loof Level File v{version}"),
            version,
            file_name: "test.lv5".into(),
            map_id: Some(1),
            description: "Test".into(),
            houses: None,
            models: None,
            textures: None,
            backgrounds: None,
            time: 120,
            style: 1,
            background: 10,
            width,
            height: tiles.len() / width,
            tiles: tiles.to_vec(),
            objects: objects.to_vec(),
            signs: (0..if version >= 5 { 20 } else { 5 })
                .map(|i| format!("sign {i}"))
                .collect(),
            music_track: (version >= 4).then_some(2),
        }
    }

    fn load(l: &Level) -> Setup {
        let options = LoadOptions {
            style: None,
            background: None,
        };
        Setup::load(l, &Ruleset::rtwp(), options).unwrap()
    }

    #[test]
    fn v5_rewrites_tiles_and_spawns_in_load_order() {
        let tiles = [
            1302, 2300, 702, 1405, //
            1903, 2102, 805, 1601, //
        ];
        let objects = [
            26, 1, 84, 80, //
            9, 88, 0, 0, //
        ];
        let s = load(&level(5, 4, &tiles, &objects));
        let row = |y| (0..4).map(|x| s.grid.get(x, y)).collect::<Vec<_>>();
        assert_eq!(row(0), [1300, 100, 100, 100]);
        assert_eq!(row(1), [100, 100, 805, 1601]);
        assert_eq!((s.signs[2].x, s.signs[2].y), (0, 0));
        assert_eq!(s.signs[19].text, "sign 19");
        assert!(s.view3d);
        assert_eq!(s.electros[..4], [-1, -1, 2, -1]);
        // 10 becomes 11 on load, so object 88 finds no rainbow world.
        assert_eq!(s.background, 11);
        assert!(s.weather.snow && !s.weather.rain);
        assert_eq!((s.channels.red, s.channels.green), (true, false));
        assert_eq!(
            s.spawns,
            [
                Spawn::Cannon {
                    x: 2,
                    y: 1,
                    speed: 2,
                    dx: 1,
                    dy: 0
                },
                Spawn::Spike {
                    x: 3,
                    y: 0,
                    speed: 5
                },
                Spawn::Boxer {
                    x: 3,
                    y: 1,
                    kind: 1
                },
                Spawn::Box {
                    x: 0,
                    y: 1,
                    kind: 103
                },
                Spawn::Box {
                    x: 1,
                    y: 1,
                    kind: 11
                },
                Spawn::Monster {
                    x: 0,
                    y: 0,
                    kind: 11,
                    dir: MonsterDir::Random
                },
                Spawn::Player { id: 1, x: 1, y: 0 },
                Spawn::Box {
                    x: 0,
                    y: 1,
                    kind: 0
                },
            ]
        );
    }

    #[test]
    fn old_versions_map_objects_and_buttons_differently() {
        let tiles = [1104, 2300, 1601];
        let v1 = load(&level(1, 3, &tiles, &[1, 6, 27]));
        assert_eq!(v1.grid.get(1, 0), 2300);
        assert!(!v1.view3d);
        assert_eq!(v1.background, 10);
        assert_eq!(v1.signs[5].text, "");
        assert!(v1.weather.snow);
        assert_eq!(
            v1.spawns,
            [
                Spawn::Button {
                    x: 0,
                    y: 0,
                    kind: 7
                },
                Spawn::Player { id: 1, x: 0, y: 0 },
                Spawn::Item {
                    x: 1,
                    y: 0,
                    kind: 0
                },
            ]
        );

        let v4 = load(&level(4, 3, &tiles, &[2, 6, 47]));
        assert!(v4.weather.snow);
        assert_eq!(
            v4.spawns[1..],
            [
                Spawn::Player { id: 2, x: 0, y: 0 },
                Spawn::Item {
                    x: 1,
                    y: 0,
                    kind: 3
                },
            ]
        );
    }

    #[test]
    fn overrides_and_size_limit() {
        let options = LoadOptions {
            style: Some(4),
            background: Some(9),
        };
        let s = Setup::load(&level(5, 1, &[100], &[88]), &Ruleset::rtwp(), options).unwrap();
        assert_eq!((s.style, s.background), (4, 10));

        let mut big = level(5, 1, &[100], &[0]);
        big.width = 103;
        assert!(Setup::load(&big, &Ruleset::rtwp(), options).is_err());
    }
}
