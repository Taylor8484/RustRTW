//! Replays recordings of the original game and reports where the port
//! differs.
//!
//! `RTWP/recorder.bb` writes the recordings when the game runs with
//! `-record`: the random seed and options at each level start, then per tick
//! the keys held and the state of each object family. [`Recording::parse`]
//! reads one; [`replay`] starts the level with the same seed and options,
//! feeds the keys tick by tick and reports the first tick where each
//! [`Family`] differs. Families whose port is not finished can be left out
//! of the comparison.
//!
//! The format is text, one record per line, fields separated by spaces.
//! Floats are written as their bits (`PokeFloat`/`PeekInt`), so they compare
//! exactly. Booleans are 0 or 1.
//!
//! - `WLREC 1 RTWP`: the header.
//! - `S seed detail sound_exists sound_level_bits no_time_limit
//!   classic_death_sound menu_style menu_bkg language level path`: the top of
//!   `StartLevel`. `seed` is `RndSeed()`; the path is the rest of the line.
//!   Each `S` starts a new run (a level, or a restart of it).
//! - `s seed map_milli_time`: the end of `StartLevel`.
//! - `T level_timer seed_before keys seed_after`: one tick of `UpdateGame`.
//!   `keys` packs the held keys as [`Input::to_bits`], plus bit 6 for the
//!   Alt+X camera cheat and bit 7 for Ctrl (the F-key cheats), which the
//!   port does not model.
//! - The tick's state follows, one line per family: `W` (flow, timers and
//!   lights), `C` (camera), `P` (one line per player), `B` (boxes), `X`
//!   (box factories and transporters), `M` (monsters), `I` (items), `K`
//!   (buttons and laser gates), `R` (electro tiles, spikes, cannons and
//!   fireballs) and `G` (a hash of the tiles). [`Family`] lists their
//!   fields.
//! - `F ticks check_ms`: the end of a frame of `ticks` ticks.
//!   `check_ms` is the `MilliSecs()` the time limit was checked at, or -1
//!   when it was not checked.
//!
//! The original checks the time limit once per frame against the wall
//! clock, where the port checks after every tick against a simulated clock.
//! By default the replay sets the port's clock from `check_ms`, so the
//! seconds tick down on the same ticks as in the recording.

use std::fmt;

use wonderland_formats::Level;

use crate::setup::LoadOptions;
use crate::{BlitzRng, Input, Options, Ruleset, Session, World};

/// A parse failure, with its 1-based line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub msg: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.msg)
    }
}

impl std::error::Error for ParseError {}

/// A recording: one run per level start.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Recording {
    pub runs: Vec<Run>,
}

/// One level from `StartLevel` until the next level starts or the log ends.
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    pub start: Start,
    /// The `s` record. Missing only if the log was cut short.
    pub started: Option<Started>,
    pub frames: Vec<Frame>,
}

/// The `S` record: the state `StartLevel` begins from.
#[derive(Debug, Clone, PartialEq)]
pub struct Start {
    pub seed: i32,
    pub detail_level: i32,
    /// `DoesSoundExist`: a sound device was found.
    pub sound_exists: bool,
    /// `OptionsSoundLevel`.
    pub sound_level: f32,
    pub no_time_limit: bool,
    pub classic_death_sound: i32,
    /// `MenuStyle`, -1 when unset.
    pub menu_style: i32,
    /// `MenuBkg`, -1 when unset.
    pub menu_bkg: i32,
    /// `Language`.
    pub language: i32,
    /// `CurrentLevel`; -999 for a custom level.
    pub level: i32,
    /// The file `LoadLevel` read, as the game names it.
    pub path: String,
}

impl Start {
    pub fn options(&self) -> Options {
        Options {
            detail_level: self.detail_level,
            no_time_limit: self.no_time_limit,
            language: self.language,
            sound: self.sound_exists && self.sound_level > 0.0,
            load: LoadOptions {
                style: (self.menu_style != -1).then_some(self.menu_style),
                background: (self.menu_bkg != -1).then_some(self.menu_bkg),
            },
        }
    }

    pub fn session(&self) -> Session {
        Session {
            rng: BlitzRng::new(self.seed),
            classic_death_sound: self.classic_death_sound,
        }
    }
}

/// The `s` record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Started {
    pub seed: i32,
    /// `MapMilliTime`: `MilliSecs()` when the level started.
    pub map_ms: i64,
}

/// The ticks of one rendered frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Frame {
    pub ticks: Vec<Tick>,
    /// `MilliSecs()` at the time-limit check after the ticks, if it ran.
    pub check_ms: Option<i64>,
}

/// One tick and the state after it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Tick {
    /// `LevelTimer` after the tick.
    pub level_timer: i32,
    pub seed_before: i32,
    pub keys: u8,
    pub seed_after: i32,
    /// The state lines, by tag. The `P` lines are joined in player order.
    lines: [Vec<i32>; TAGS.len()],
}

const TAGS: [char; 10] = ['W', 'C', 'P', 'B', 'X', 'M', 'I', 'K', 'R', 'G'];

/// Fields per `P` line, and how many of them are entity coordinates.
const PLAYER_FIELDS: usize = 19;
const PLAYER_ENTITY: usize = 3;
/// Fields per box on the `B` line.
const BOX_FIELDS: usize = 9;
const BOX_ENTITY: usize = 3;

impl Tick {
    pub fn input(&self) -> Input {
        Input::from_bits(self.keys & 0x3f)
    }

    /// A debug cheat key was held, which the port ignores.
    pub fn cheats(&self) -> bool {
        self.keys & 0xc0 != 0
    }

    fn line(&self, tag: char) -> &[i32] {
        &self.lines[TAGS.iter().position(|&t| t == tag).unwrap()]
    }

    /// The recorded values of `family`, laid out as [`Family::layout`].
    pub fn values(&self, family: Family) -> Vec<i32> {
        match family {
            Family::Rng => vec![self.seed_after],
            Family::World => [&[self.level_timer][..], self.line('W')].concat(),
            Family::Camera => self.line('C').to_vec(),
            Family::Players => {
                let p = self.line('P');
                let mut v = vec![(p.len() / PLAYER_FIELDS) as i32];
                for c in p.chunks(PLAYER_FIELDS) {
                    // Drop the player number.
                    v.extend_from_slice(&c[1..PLAYER_FIELDS - PLAYER_ENTITY]);
                }
                v
            }
            Family::Entities => {
                let p = self.line('P');
                let mut v = vec![(p.len() / PLAYER_FIELDS) as i32];
                for c in p.chunks(PLAYER_FIELDS) {
                    v.extend_from_slice(&c[PLAYER_FIELDS - PLAYER_ENTITY..]);
                }
                let b = self.line('B');
                v.push(b.first().copied().unwrap_or(0));
                for c in b.get(1..).unwrap_or_default().chunks(BOX_FIELDS) {
                    v.extend_from_slice(&c[BOX_FIELDS - BOX_ENTITY..]);
                }
                v
            }
            Family::Boxes => {
                let b = self.line('B');
                let mut v = vec![b.first().copied().unwrap_or(0)];
                for c in b.get(1..).unwrap_or_default().chunks(BOX_FIELDS) {
                    let mut f = c[..BOX_FIELDS - BOX_ENTITY].to_vec();
                    // `CreateBox` leaves a stale timer in the slot (as for
                    // monsters), read only while moving; 0 at rest.
                    if f.get(3) == Some(&0) && f.len() > 4 {
                        f[4] = 0;
                    }
                    v.extend(f);
                }
                v
            }
            Family::Factories => self.line('X').to_vec(),
            Family::Monsters => self.line('M').to_vec(),
            Family::Items => self.line('I').to_vec(),
            Family::Switches => self.line('K').to_vec(),
            Family::Hazards => self.line('R').to_vec(),
            Family::Tiles => self.line('G').to_vec(),
        }
    }
}

impl Recording {
    pub fn parse(text: &str) -> Result<Self, ParseError> {
        let mut rec = Recording::default();
        let mut header = false;
        for (n, line) in text.lines().enumerate() {
            let err = |msg: String| ParseError { line: n + 1, msg };
            let line = line.trim_end();
            if line.is_empty() {
                continue;
            }
            let (tag, rest) = line.split_once(' ').unwrap_or((line, ""));
            if !header {
                if tag != "WLREC" || !rest.starts_with("1 ") {
                    return Err(err(format!("not a version 1 recording: {line:?}")));
                }
                header = true;
                continue;
            }
            let ints = |count: Option<usize>| -> Result<Vec<i32>, ParseError> {
                let v = rest
                    .split_ascii_whitespace()
                    .map(str::parse)
                    .collect::<Result<Vec<i32>, _>>()
                    .map_err(|e| err(format!("{tag}: {e}")))?;
                match count {
                    Some(c) if v.len() != c => {
                        Err(err(format!("{tag}: {} fields, expected {c}", v.len())))
                    }
                    _ => Ok(v),
                }
            };
            if tag == "S" {
                let fields: Vec<&str> = rest.splitn(11, ' ').collect();
                if fields.len() != 11 {
                    return Err(err("S: too few fields".into()));
                }
                let int = |i: usize| -> Result<i32, ParseError> {
                    fields[i]
                        .parse()
                        .map_err(|e| err(format!("S field {i}: {e}")))
                };
                rec.runs.push(Run {
                    start: Start {
                        seed: int(0)?,
                        detail_level: int(1)?,
                        sound_exists: int(2)? != 0,
                        sound_level: f32::from_bits(int(3)? as u32),
                        no_time_limit: int(4)? != 0,
                        classic_death_sound: int(5)?,
                        menu_style: int(6)?,
                        menu_bkg: int(7)?,
                        language: int(8)?,
                        level: int(9)?,
                        path: fields[10].to_string(),
                    },
                    started: None,
                    frames: vec![Frame::default()],
                });
                continue;
            }
            let Some(run) = rec.runs.last_mut() else {
                return Err(err(format!("{tag} before any level start")));
            };
            let frame = run.frames.last_mut().unwrap();
            match tag {
                "s" => {
                    let v = ints(Some(2))?;
                    run.started = Some(Started {
                        seed: v[0],
                        map_ms: i64::from(v[1]),
                    });
                }
                "T" => {
                    let v = ints(Some(4))?;
                    frame.ticks.push(Tick {
                        level_timer: v[0],
                        seed_before: v[1],
                        keys: v[2] as u8,
                        seed_after: v[3],
                        ..Tick::default()
                    });
                }
                "F" => {
                    let v = ints(Some(2))?;
                    if v[0] as usize != frame.ticks.len() {
                        return Err(err(format!(
                            "F: {} ticks, but {} were logged",
                            v[0],
                            frame.ticks.len()
                        )));
                    }
                    frame.check_ms = (v[1] != -1).then_some(i64::from(v[1]));
                    run.frames.push(Frame::default());
                }
                _ => {
                    let Some(i) = TAGS.iter().position(|t| t.to_string() == tag) else {
                        return Err(err(format!("unknown record {tag:?}")));
                    };
                    let Some(tick) = frame.ticks.last_mut() else {
                        return Err(err(format!("{tag} before any tick")));
                    };
                    let count = match tag {
                        "P" => Some(PLAYER_FIELDS),
                        "G" => Some(1),
                        _ => None,
                    };
                    let v = ints(count)?;
                    if tag == "B" && v.len() != 1 + BOX_FIELDS * v[0].max(0) as usize {
                        return Err(err(format!("B: {} fields for {} boxes", v.len(), v[0])));
                    }
                    tick.lines[i].extend(v);
                }
            }
        }
        if !header {
            return Err(ParseError {
                line: 0,
                msg: "empty recording".into(),
            });
        }
        for run in &mut rec.runs {
            if run.frames.last().is_some_and(|f| f.ticks.is_empty()) {
                run.frames.pop();
            }
        }
        Ok(rec)
    }
}

/// A group of state compared together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Family {
    /// `RndSeed()` after the tick: the first tick where the order of
    /// random draws differs.
    Rng,
    /// `LevelTimer`, `LevelCompleted`, the time limit, `BothCompleted`,
    /// sign and focus timers, conveyors, the first-person heading, the
    /// free camera flag, lightning, the final gate and the lights.
    World,
    /// The camera position and angles, the free camera and the swoop.
    Camera,
    /// The players' tile state.
    Players,
    /// Player and box entity positions.
    Entities,
    /// Boxes and other pushables.
    Boxes,
    /// Box factories and transporters.
    Factories,
    Monsters,
    /// Coins left, bonus coins left and the items.
    Items,
    /// Buttons and laser gates.
    Switches,
    /// Electro tiles, spikes, cannons and fireballs.
    Hazards,
    /// A hash of the tiles inside the level.
    Tiles,
}

impl Family {
    pub const ALL: [Family; 12] = [
        Family::Rng,
        Family::World,
        Family::Camera,
        Family::Players,
        Family::Entities,
        Family::Boxes,
        Family::Factories,
        Family::Monsters,
        Family::Items,
        Family::Switches,
        Family::Hazards,
        Family::Tiles,
    ];

    /// The families the port covers so far (all of step 2.2).
    pub const PORTED: [Family; 12] = [
        Family::Rng,
        Family::World,
        Family::Camera,
        Family::Players,
        Family::Entities,
        Family::Boxes,
        Family::Factories,
        Family::Monsters,
        Family::Items,
        Family::Switches,
        Family::Hazards,
        Family::Tiles,
    ];

    /// How the family's values are laid out.
    pub fn layout(self) -> &'static [Segment] {
        use Kind::*;
        use Segment::*;
        match self {
            Family::Rng => &[Fields(&[("seed", Int)])],
            Family::World => &[Fields(&[
                ("level_timer", Int),
                ("completion", Int),
                ("completion_timer", Int),
                ("map_time", Int),
                ("both_completed", Int),
                ("focus_switch_timer", Int),
                ("sign_active", Int),
                ("conveyor_active[0]", Int),
                ("conveyor_active[1]", Int),
                ("view3d_rotation", Int),
                ("view3d_rotation2", Int),
                ("free_camera_active", Int),
                ("lightning_timer", Int),
                ("final_gate_anim", Int),
                ("light.r", Int),
                ("light.g", Int),
                ("light.b", Int),
                ("light_target.r", Int),
                ("light_target.g", Int),
                ("light_target.b", Int),
                ("light_speed", Int),
                ("ambient.r", Int),
                ("ambient.g", Int),
                ("ambient.b", Int),
                ("ambient_target.r", Int),
                ("ambient_target.g", Int),
                ("ambient_target.b", Int),
                ("ambient_speed", Int),
            ])],
            Family::Camera => &[Fields(&[
                ("x", Float),
                ("y", Float),
                ("z", Float),
                ("pitch", Angle),
                ("yaw", Yaw),
                ("free_camera.x", Float),
                ("free_camera.z", Float),
                ("camera_swoop.x", Float),
                ("camera_swoop.z", Float),
            ])],
            Family::Players => &[Group(
                "player",
                1,
                &[
                    ("exists", Int),
                    ("x", Int),
                    ("y", Int),
                    ("x2", Int),
                    ("y2", Int),
                    ("moving", Int),
                    ("moving_timer", Int),
                    ("rotation", Int),
                    ("dying", Int),
                    ("dying_timer", Int),
                    ("dead", Int),
                    ("focus", Int),
                    ("shrink", Int),
                    ("anim_current", Int),
                    ("jumping", Int),
                ],
            )],
            Family::Entities => &[
                Group("player", 1, &[("x", Float), ("y", Float), ("z", Float)]),
                Group("box", 0, &[("x", Float), ("y", Float), ("z", Float)]),
            ],
            Family::Boxes => &[Group(
                "box",
                0,
                &[
                    ("kind", Int),
                    ("x", Int),
                    ("y", Int),
                    ("moving", Int),
                    ("moving_timer", Int),
                    ("dead", Int),
                ],
            )],
            Family::Factories => &[
                Group(
                    "boxer",
                    0,
                    &[("x", Int), ("y", Int), ("kind", Int), ("timer", Int)],
                ),
                Group(
                    "transporter",
                    0,
                    &[
                        ("x", Int),
                        ("y", Int),
                        ("moving", Int),
                        ("moving_timer", Int),
                    ],
                ),
            ],
            Family::Monsters => &[Group(
                "monster",
                0,
                &[
                    ("kind", Int),
                    ("x", Int),
                    ("y", Int),
                    ("moving", Int),
                    ("moving_timer", Int),
                    ("dying", Int),
                    ("dead", Int),
                ],
            )],
            Family::Items => &[
                Fields(&[
                    ("coins_left", Int),
                    ("bonus_coins_left", Int),
                    ("level_score", Int),
                ]),
                Group("item", 0, &[("kind", Int), ("x", Int), ("y", Int)]),
            ],
            Family::Switches => &[
                Group(
                    "button",
                    0,
                    &[("kind", Int), ("active", Int), ("timer", Int)],
                ),
                Repeat("laser_gate_active", 10),
            ],
            Family::Hazards => &[
                Repeat("electro_timer", 4),
                Group("spike", 0, &[("timer", Int)]),
                Group("cannon", 0, &[("timer", Int)]),
                Group("fireball", 0, &[("x", Int), ("y", Int), ("dead", Int)]),
            ],
            Family::Tiles => &[Fields(&[("hash", Int)])],
        }
    }

    /// The port's values for this family, laid out like [`Tick::values`];
    /// `None` while the family is not ported.
    pub fn port_values(self, w: &World) -> Option<Vec<i32>> {
        let b = i32::from;
        let f = |x: f32| x.to_bits() as i32;
        let players = &w.players[1..];
        Some(match self {
            Family::Rng => vec![w.session.rng.seed()],
            Family::World => {
                let l = &w.lights;
                let mut v = vec![
                    w.level_timer,
                    w.completion as i32,
                    w.completion_timer,
                    w.map_time,
                    w.both_completed,
                    w.focus_switch_timer,
                    w.sign_active,
                    b(w.conveyor_active[0]),
                    b(w.conveyor_active[1]),
                    w.view3d_rotation,
                    w.view3d_rotation2,
                    b(w.free_camera_active),
                    w.lightning_timer,
                    w.final_gate_anim,
                ];
                v.extend(l.light);
                v.extend(l.light_target);
                v.push(l.light_speed);
                v.extend(l.ambient);
                v.extend(l.ambient_target);
                v.push(l.ambient_speed);
                v
            }
            Family::Camera => {
                let c = &w.camera;
                [
                    c.position[0],
                    c.position[1],
                    c.position[2],
                    c.pitch,
                    c.yaw,
                    w.free_camera[0],
                    w.free_camera[1],
                    w.camera_swoop[0],
                    w.camera_swoop[1],
                ]
                .map(f)
                .to_vec()
            }
            Family::Players => {
                let mut v = vec![players.len() as i32];
                for p in players {
                    v.extend([
                        b(p.exists),
                        p.x,
                        p.y,
                        p.x2,
                        p.y2,
                        b(p.moving),
                        p.moving_timer,
                        p.rotation,
                        b(p.dying),
                        p.dying_timer,
                        b(p.dead),
                        b(p.focus),
                        p.shrink,
                        p.anim_current,
                        b(p.jumping),
                    ]);
                }
                v
            }
            Family::Entities => {
                let mut v = vec![players.len() as i32];
                for p in players {
                    // The recorder writes zeros for players not in the level.
                    let at = if p.exists { p.position } else { [0.0; 3] };
                    v.extend(at.map(f));
                }
                v.push(w.blocks.len() as i32);
                for bl in &w.blocks {
                    v.extend(bl.position.map(f));
                }
                v
            }
            Family::Boxes => {
                let mut v = vec![w.blocks.len() as i32];
                for bl in &w.blocks {
                    v.extend([
                        bl.kind,
                        bl.x,
                        bl.y,
                        b(bl.moving),
                        if bl.moving { bl.moving_timer } else { 0 },
                        b(bl.dead),
                    ]);
                }
                v
            }
            Family::Factories => {
                let mut v = vec![w.boxers.len() as i32];
                for x in &w.boxers {
                    v.extend([x.x, x.y, x.kind, x.timer]);
                }
                v.push(w.transporters.len() as i32);
                for t in &w.transporters {
                    v.extend([t.x, t.y, b(t.moving), t.moving_timer]);
                }
                v
            }
            Family::Monsters => {
                let mut v = vec![w.monsters.len() as i32];
                for m in &w.monsters {
                    v.extend([
                        m.kind,
                        m.x,
                        m.y,
                        b(m.moving),
                        // The original leaves a stale timer in a new
                        // monster's slot, so both sides log 0 at rest.
                        if m.moving { m.moving_timer } else { 0 },
                        b(m.dying),
                        b(m.dead),
                    ]);
                }
                v
            }
            Family::Items => {
                let mut v = vec![w.coins, w.bonus_coins, w.level_score, w.items.len() as i32];
                for it in &w.items {
                    v.extend([it.kind, it.x, it.y]);
                }
                v
            }
            Family::Switches => {
                let mut v = vec![w.buttons.len() as i32];
                for bt in &w.buttons {
                    v.extend([bt.kind, b(bt.active), bt.timer]);
                }
                v.extend(w.laser_gate_active.map(b));
                v
            }
            Family::Hazards => {
                // The recorder zeroes timers of electro types not in the
                // level, which keep earlier levels' values in the original.
                let mut v: Vec<i32> = w.electros[..4]
                    .iter()
                    .map(|e| if e.exists { e.timer } else { 0 })
                    .collect();
                v.push(w.spikes.len() as i32);
                v.extend(w.spikes.iter().map(|s| s.timer));
                v.push(w.cannons.len() as i32);
                v.extend(w.cannons.iter().map(|c| c.timer));
                v.push(w.fireballs().len() as i32);
                for f in w.fireballs() {
                    v.extend([f.x, f.y, b(f.dead)]);
                }
                v
            }
            Family::Tiles => vec![tile_hash(w)],
        })
    }
}

/// The recorder's `MapTile` hash: `h = h * 31 + tile`, row by row.
pub fn tile_hash(w: &World) -> i32 {
    let g = &w.level.grid;
    let mut h = 0i32;
    for y in 0..g.height() as i32 {
        for x in 0..g.width() as i32 {
            h = h.wrapping_mul(31).wrapping_add(g.get(x, y));
        }
    }
    h
}

/// How a value compares and prints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Int,
    /// Float bits, compared exactly.
    Float,
    /// An angle in degrees. Blitz recomputes `EntityPitch` from the
    /// rotation matrix, so it is compared to within 0.01.
    Angle,
    /// A yaw, compared like [`Kind::Angle`] modulo 360, and skipped when the
    /// pitch (the value before it) is within half a degree of straight
    /// down, where the yaw is undefined.
    Yaw,
}

/// Part of a family's layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Segment {
    /// Fixed fields.
    Fields(&'static [(&'static str, Kind)]),
    /// A count, then that many objects of these fields. The name and the
    /// number of the first object label them.
    Group(&'static str, i32, &'static [(&'static str, Kind)]),
    /// A fixed number of ints of one name.
    Repeat(&'static str, usize),
}

/// The first tick where a family differs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Divergence {
    pub family: Family,
    /// The tick in the run, from 1; 0 for the level start.
    pub tick: usize,
    pub field: String,
    pub recorded: String,
    pub port: String,
}

impl fmt::Display for Divergence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} at tick {}: {} recorded {}, port {}",
            self.family, self.tick, self.field, self.recorded, self.port
        )
    }
}

/// Compares one family's values and names the first difference.
fn compare(family: Family, recorded: &[i32], port: &[i32]) -> Option<(String, String, String)> {
    let show = |v: Option<&i32>, kind: Kind| match (v, kind) {
        (None, _) => "nothing".to_string(),
        (Some(v), Kind::Int) => v.to_string(),
        (Some(&v), _) => format!("{:?}", f32::from_bits(v as u32)),
    };
    let mismatch = |name: String, kind: Kind, at: usize| -> Option<(String, String, String)> {
        let (r, p) = (recorded.get(at), port.get(at));
        let same = match (r, p, kind) {
            (Some(r), Some(p), Kind::Angle | Kind::Yaw) => {
                let (r, p) = (f32::from_bits(*r as u32), f32::from_bits(*p as u32));
                let pitch = at
                    .checked_sub(1)
                    .and_then(|j| recorded.get(j))
                    .map(|&v| f32::from_bits(v as u32));
                if kind == Kind::Yaw && pitch.is_some_and(|p| p.abs() > 89.5) {
                    true
                } else {
                    let d = (r - p).rem_euclid(360.0);
                    d.min(360.0 - d) < 0.01
                }
            }
            (r, p, _) => r == p,
        };
        (!same).then(|| (name, show(r, kind), show(p, kind)))
    };
    let mut i = 0;
    for segment in family.layout() {
        match *segment {
            Segment::Fields(fields) => {
                for &(name, kind) in fields {
                    if let Some(d) = mismatch(name.to_string(), kind, i) {
                        return Some(d);
                    }
                    i += 1;
                }
            }
            Segment::Repeat(name, n) => {
                for k in 0..n {
                    if let Some(d) = mismatch(format!("{name}[{k}]"), Kind::Int, i) {
                        return Some(d);
                    }
                    i += 1;
                }
            }
            Segment::Group(name, first, fields) => {
                if let Some(d) = mismatch(format!("{name} count"), Kind::Int, i) {
                    return Some(d);
                }
                let n = recorded.get(i).copied().unwrap_or(0);
                i += 1;
                for k in 0..n {
                    for &(field, kind) in fields {
                        let label = format!("{name} {}.{field}", k + first);
                        if let Some(d) = mismatch(label, kind, i) {
                            return Some(d);
                        }
                        i += 1;
                    }
                }
            }
        }
    }
    (recorded.len() != port.len()).then(|| {
        (
            "length".to_string(),
            recorded.len().to_string(),
            port.len().to_string(),
        )
    })
}

/// What [`replay`] compares and how it drives the clock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayOptions {
    pub families: Vec<Family>,
    /// Count the time limit down on the recorded frames' clock rather than
    /// the port's simulated one.
    pub recorded_clock: bool,
}

impl Default for ReplayOptions {
    fn default() -> Self {
        Self {
            families: Family::PORTED.to_vec(),
            recorded_clock: true,
        }
    }
}

/// How a run's replay went.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub ticks: usize,
    /// The first difference in each compared family, by tick.
    pub divergences: Vec<Divergence>,
    /// Ticks before which the original drew random numbers outside
    /// `UpdateGame` (a pause menu, say). The port's generator is set to the
    /// recorded seed there.
    pub outside_draws: Vec<usize>,
    /// Ticks with a debug cheat key held.
    pub cheat_ticks: usize,
}

impl Report {
    pub fn passed(&self) -> bool {
        self.divergences.is_empty()
    }
}

/// Drives the port's time limit from the recorded checks.
///
/// The original checks once per frame, after the frame's ticks, against
/// `MilliSecs()`. The port checks after every tick against
/// [`World::clock_ms`], so the clock is held at the last check's time and
/// moves to the frame's check time on its last tick.
struct FrameClock {
    tick_ms: i64,
    /// `MapMilliTime` at level start, which is `tick_ms` on the port's
    /// clock.
    map_ms: i64,
    /// The port's clock at the last check.
    checked: i64,
}

impl FrameClock {
    fn new(run: &Run, tick_ms: i64) -> Self {
        Self {
            tick_ms,
            map_ms: run.started.map_or(0, |s| s.map_ms),
            checked: tick_ms,
        }
    }

    fn before_tick(&mut self, world: &mut World, frame: &Frame, j: usize) {
        if j + 1 == frame.ticks.len()
            && let Some(ms) = frame.check_ms
        {
            self.checked = ms - self.map_ms + self.tick_ms;
        }
        // `World::step` adds a tick before checking.
        world.clock_ms = self.checked - self.tick_ms;
    }
}

/// Replays `run` on `level` and compares each tick.
pub fn replay(
    run: &Run,
    level: &Level,
    ruleset: Ruleset,
    options: &ReplayOptions,
) -> crate::Result<Report> {
    let tick_ms = i64::from(ruleset.tick_ms);
    let mut world = World::start(level, ruleset, run.start.options(), run.start.session())?;
    let mut report = Report::default();
    let diverge = |report: &mut Report, family, tick, (field, recorded, port)| {
        if !report.divergences.iter().any(|d| d.family == family) {
            report.divergences.push(Divergence {
                family,
                tick,
                field,
                recorded,
                port,
            });
        }
    };

    let mut seed = run.start.seed;
    if let Some(started) = run.started {
        seed = started.seed;
        if options.families.contains(&Family::Rng) && world.session.rng.seed() != seed {
            let d = (
                "seed after StartLevel".to_string(),
                seed.to_string(),
                world.session.rng.seed().to_string(),
            );
            diverge(&mut report, Family::Rng, 0, d);
        }
    }
    let mut clock = FrameClock::new(run, tick_ms);

    for frame in &run.frames {
        for (j, tick) in frame.ticks.iter().enumerate() {
            report.ticks += 1;
            let n = report.ticks;
            if tick.seed_before != seed {
                report.outside_draws.push(n);
                world.session.rng = BlitzRng::new(tick.seed_before);
            }
            seed = tick.seed_after;
            if tick.cheats() {
                report.cheat_ticks += 1;
            }
            if options.recorded_clock {
                clock.before_tick(&mut world, frame, j);
            }
            world.step(tick.input());
            for &family in &options.families {
                if report.divergences.iter().any(|d| d.family == family) {
                    continue;
                }
                let Some(port) = family.port_values(&world) else {
                    continue;
                };
                if let Some(d) = compare(family, &tick.values(family), &port) {
                    diverge(&mut report, family, n, d);
                }
            }
        }
    }
    report.divergences.sort_by_key(|d| (d.tick, d.family));
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::tests::level;

    /// A 5x5 floor with player 1 in the middle and a box to its right.
    fn test_level() -> Level {
        let mut objects = [0; 25];
        objects[12] = 1;
        objects[13] = 9;
        level(5, 5, &[100; 25], &objects)
    }

    fn start() -> World {
        World::start(
            &test_level(),
            Ruleset::rtwp(),
            Options::default(),
            Session::default(),
        )
        .unwrap()
    }

    /// Writes ticks the way `recorder.bb` would, from the port itself.
    fn record(world: &mut World, frames: &[&[Input]], start_seed: i32) -> String {
        let mut s = format!(
            "WLREC 1 RTWP\nS {start_seed} 3 1 {} 0 0 -1 -1 0 7 level 7.lev\ns {} 5000\n",
            1.0f32.to_bits() as i32,
            world.session.rng.seed()
        );
        let mut ms = 5000;
        for keys in frames {
            for input in *keys {
                let before = world.session.rng.seed();
                world.step(*input);
                s += &format!(
                    "T {} {before} {} {}\n",
                    world.level_timer,
                    input.to_bits(),
                    world.session.rng.seed()
                );
                s += &line('W', &Family::World.port_values(world).unwrap()[1..]);
                s += &line('C', &Family::Camera.port_values(world).unwrap());
                let p = Family::Players.port_values(world).unwrap();
                let e = Family::Entities.port_values(world).unwrap();
                for k in 0..4 {
                    let mut v = vec![k as i32 + 1];
                    v.extend(&p[1 + 15 * k..16 + 15 * k]);
                    v.extend(&e[1 + 3 * k..4 + 3 * k]);
                    s += &line('P', &v);
                }
                let b = Family::Boxes.port_values(world).unwrap();
                let mut v = vec![b[0]];
                for (k, c) in b[1..].chunks(6).enumerate() {
                    v.extend(c);
                    v.extend(&e[14 + 3 * k..17 + 3 * k]);
                }
                s += &line('B', &v);
                s += &line('X', &Family::Factories.port_values(world).unwrap());
                s += &line('M', &Family::Monsters.port_values(world).unwrap());
                s += &line('I', &Family::Items.port_values(world).unwrap());
                s += &line('K', &Family::Switches.port_values(world).unwrap());
                s += &line('R', &Family::Hazards.port_values(world).unwrap());
                s += &line('G', &[tile_hash(world)]);
            }
            ms += 11 * keys.len() as i32;
            s += &format!("F {} {ms}\n", keys.len());
        }
        s
    }

    fn line(tag: char, v: &[i32]) -> String {
        let mut s = tag.to_string();
        for x in v {
            s += &format!(" {x}");
        }
        s + "\n"
    }

    #[test]
    fn parses_runs_frames_and_ticks() {
        let text = "WLREC 1 RTWP\r\n\
            S 4660 3 1 1065353216 0 2 -1 4 1 12 levels\\a b.lev\r\n\
            s 999 1000\r\n\
            T 1 999 65 1000\r\nW 1 2\r\nP 1 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18\r\n\
            F 1 1011\r\n\
            T 2 1000 1 1001\r\nG 7\r\nF 1 -1\r\n\
            S 1 3 0 0 1 0 -1 -1 0 -999 CustomLevels\\x.lv5\r\n";
        let r = Recording::parse(text).unwrap();
        assert_eq!(r.runs.len(), 2);
        let run = &r.runs[0];
        assert_eq!(run.start.path, r"levels\a b.lev");
        assert_eq!(run.start.sound_level, 1.0);
        assert_eq!(run.start.options().load.background, Some(4));
        assert_eq!(run.start.session().classic_death_sound, 2);
        assert_eq!(run.start.options().language, 1);
        assert_eq!(
            run.started,
            Some(Started {
                seed: 999,
                map_ms: 1000
            })
        );
        assert_eq!(run.frames.len(), 2);
        assert_eq!(run.frames[0].check_ms, Some(1011));
        assert_eq!(run.frames[1].check_ms, None);
        let t = &run.frames[0].ticks[0];
        assert!(t.input().up && t.cheats());
        assert_eq!(t.values(Family::World), [1, 1, 2]);
        assert_eq!(t.values(Family::Players)[..3], [1, 1, 2]);
        assert_eq!(t.values(Family::Entities), [1, 16, 17, 18, 0]);
        assert_eq!(run.frames[1].ticks[0].values(Family::Tiles), [7]);
        let custom = &r.runs[1];
        assert!(!custom.start.options().sound && custom.start.options().no_time_limit);
        assert!(custom.frames.is_empty());
    }

    #[test]
    fn rejects_malformed_lines() {
        assert!(Recording::parse("hello").is_err());
        let e = Recording::parse("WLREC 1 RTWP\nT 1 2 3 4\n").unwrap_err();
        assert_eq!(e.line, 2);
        let e = Recording::parse("WLREC 1 RTWP\nS 1 3 1 0 0 0 -1 -1 0 1 a\nT 1 2 x 4\n");
        assert!(e.unwrap_err().msg.contains("T"));
        let e = Recording::parse("WLREC 1 RTWP\nS 1 3 1 0 0 0 -1 -1 0 1 a\nT 1 2 3 4\nF 2 0\n");
        assert!(e.unwrap_err().msg.contains("2 ticks"));
    }

    #[test]
    fn a_recording_of_the_port_replays_cleanly() {
        let l = test_level();
        let mut w = start();
        let seed = Session::default().rng.seed();
        let right = Input {
            right: true,
            ..Input::default()
        };
        let frames: Vec<Vec<Input>> = (0..300)
            .map(|i| vec![if i % 50 < 20 { right } else { Input::default() }; 1 + i % 3])
            .collect();
        let refs: Vec<&[Input]> = frames.iter().map(Vec::as_slice).collect();
        let text = record(&mut w, &refs, seed);
        let rec = Recording::parse(&text).unwrap();
        let options = ReplayOptions {
            families: Family::ALL.to_vec(),
            recorded_clock: false,
        };
        let r = replay(&rec.runs[0], &l, Ruleset::rtwp(), &options).unwrap();
        assert_eq!(r.ticks, frames.iter().map(Vec::len).sum::<usize>());
        assert!(r.passed(), "{:?}", r.divergences);
        assert!(r.outside_draws.is_empty());
        // The player pushed the box.
        assert_eq!((w.blocks[0].x, w.blocks[0].y), (4, 2));
    }

    #[test]
    fn reports_the_first_tick_each_family_differs() {
        let l = test_level();
        let mut w = start();
        let seed = Session::default().rng.seed();
        let frames = vec![vec![Input::default(); 2]; 20];
        let refs: Vec<&[Input]> = frames.iter().map(Vec::as_slice).collect();
        let text = record(&mut w, &refs, seed);
        // Tick 5 says the map time was 99, and tick 7 that the original drew
        // once more outside the simulation.
        let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
        let ticks: Vec<usize> = (0..lines.len())
            .filter(|&i| lines[i].starts_with("T "))
            .collect();
        let w5 = ticks[4] + 1;
        let mut v: Vec<&str> = lines[w5].split(' ').collect();
        v[3] = "99";
        lines[w5] = v.join(" ");
        let t7: Vec<String> = lines[ticks[6]].split(' ').map(str::to_string).collect();
        let mut rng = BlitzRng::new(t7[2].parse().unwrap());
        rng.rand(0, 1);
        let moved = rng.seed();
        lines[ticks[6]] = format!("T {} {moved} {} {}", t7[1], t7[3], t7[4]);
        let rec = Recording::parse(&lines.join("\n")).unwrap();
        let r = replay(&rec.runs[0], &l, Ruleset::rtwp(), &ReplayOptions::default()).unwrap();
        assert_eq!(r.outside_draws, [7]);
        let d = &r.divergences[0];
        assert_eq!(
            (d.family, d.tick, d.field.as_str()),
            (Family::World, 5, "map_time")
        );
        assert_eq!(d.recorded, "99");
        // The resynced generator draws from a different seed from tick 7 on.
        assert!(
            r.divergences
                .iter()
                .any(|d| d.family == Family::Rng && d.tick == 7)
        );
    }

    #[test]
    fn the_recorded_clock_ticks_the_time_limit_down() {
        let mut w = start();
        let time = w.map_time;
        // 20-tick frames 230 ms apart: a second has passed at the fifth
        // check, on tick 100, where the simulated clock says tick 92.
        let mut s = String::from("WLREC 1 RTWP\nS 0 3 1 0 0 0 -1 -1 0 1 a\ns 0 1000\n");
        for f in 1..=5 {
            s += &"T 0 0 0 0\n".repeat(20);
            s += &format!("F 20 {}\n", 1000 + 230 * f);
        }
        let run = &Recording::parse(&s).unwrap().runs[0];
        let mut clock = FrameClock::new(run, 11);
        let mut n = 0;
        for frame in &run.frames {
            for j in 0..frame.ticks.len() {
                clock.before_tick(&mut w, frame, j);
                w.step(Input::default());
                n += 1;
                let expected = if n == 100 { time - 1 } else { time };
                assert_eq!(w.map_time, expected, "tick {n}");
            }
        }
    }
}
