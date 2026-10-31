//! What connects the level's pieces, which the format keeps as matching
//! numbers: a button works the gates or conveyors of its target, and the two
//! teleporters of a colour lead to each other. [`links`] reads these so they
//! can be drawn, [`Editor::link`] sets them by connecting two pieces, and
//! [`warnings`] finds what will not work.

use wonderland_formats::Level;
use wonderland_formats::catalogue::{self, Group, Layer};

use crate::Editor;
use crate::signs::MAX_SIGNS;

const FINAL_GATE: std::ops::Range<i32> = 1000..1002;
const GATE: std::ops::Range<i32> = 1002..1016;
const BUTTON: std::ops::Range<i32> = 1100..1127;
const TELEPORTER: std::ops::Range<i32> = 1200..1208;
const CONVEYOR: std::ops::Range<i32> = 500..516;
const SIGN: std::ops::Range<i32> = 1300..1320;
const EXIT: i32 = 6;
/// A button's choices: three actions for each target.
const ACTIONS: i32 = 3;
/// The two conveyor sets come before the gates in a button's targets.
const CONVEYOR_SETS: i32 = 2;
/// `MAX_BOXES`, `MAX_MONSTERS` in `logic`, the sizes of the game's arrays.
const MAX_BOXES: usize = 400;
const MAX_MONSTERS: usize = 600;

/// The cell a link starts or ends on.
pub type Spot = (usize, usize);

/// A connection between two cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Link {
    pub from: Spot,
    pub to: Spot,
    /// A pair of teleporters, as opposed to a button and what it works.
    pub teleport: bool,
    /// A button and a conveyor. There are often many of these, so a screen
    /// may want them only on request.
    pub conveyor: bool,
}

/// Something in the level that will not work as it stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    pub message: String,
    pub cell: Option<Spot>,
}

/// A button's target, as the catalogue's Works axis counts.
fn button_target(code: i32) -> i32 {
    (code - BUTTON.start) / ACTIONS
}

/// The target a button needs to work gates of `colour` (1 up).
fn gate_target(colour: i32) -> i32 {
    colour - 1 + CONVEYOR_SETS
}

fn gate_colour(code: i32) -> i32 {
    (code - FINAL_GATE.start) / 2
}

/// Which set a conveyor tile is in.
fn conveyor_set(code: i32) -> i32 {
    (code - CONVEYOR.start) / 8
}

fn cells(level: &Level) -> impl Iterator<Item = (Spot, i32)> + '_ {
    let width = level.width;
    level
        .tiles
        .iter()
        .enumerate()
        .map(move |(i, &t)| ((i % width, i / width), t))
}

/// Every link in the level.
pub fn links(level: &Level) -> Vec<Link> {
    let mut out = vec![];
    let tiles: Vec<(Spot, i32)> = cells(level).collect();
    for &(from, code) in &tiles {
        if BUTTON.contains(&code) {
            let target = button_target(code);
            for &(to, other) in &tiles {
                let works = if target < CONVEYOR_SETS {
                    CONVEYOR.contains(&other) && conveyor_set(other) == target
                } else {
                    GATE.contains(&other) && gate_target(gate_colour(other)) == target
                };
                if works {
                    out.push(Link {
                        from,
                        to,
                        teleport: false,
                        conveyor: target < CONVEYOR_SETS,
                    });
                }
            }
        } else if TELEPORTER.contains(&code) {
            for &(to, other) in &tiles {
                if other == code && from < to {
                    out.push(Link {
                        from,
                        to,
                        teleport: true,
                        conveyor: false,
                    });
                }
            }
        }
    }
    out
}

impl Editor {
    /// Connects the pieces on two cells, if they can be, as one undo step.
    /// A button takes on what the other piece needs. A teleporter takes on
    /// the colour of a lone one, or both take a colour nobody has.
    pub fn link(&mut self, from: Spot, to: Spot) -> bool {
        let linked = self.linked(from, to) || self.linked(to, from);
        self.commit();
        linked
    }

    fn linked(&mut self, from: Spot, to: Spot) -> bool {
        let (a, b) = (self.level.tile(from.0, from.1), self.level.tile(to.0, to.1));
        if BUTTON.contains(&a) {
            let target = if GATE.contains(&b) {
                gate_target(gate_colour(b))
            } else if CONVEYOR.contains(&b) {
                conveyor_set(b)
            } else {
                return false;
            };
            let action = (a - BUTTON.start) % ACTIONS;
            let code = BUTTON.start + action + target * ACTIONS;
            self.place(from.0, from.1, Layer::Tile, code);
            return true;
        }
        if !(TELEPORTER.contains(&a) && TELEPORTER.contains(&b)) || from == to {
            return false;
        }
        let count = |code| self.level.tiles.iter().filter(|&&t| t == code).count();
        if a == b && count(a) == 2 {
            return true;
        }
        if count(a) == 1 {
            self.place(to.0, to.1, Layer::Tile, a);
            return true;
        }
        if count(b) == 1 {
            self.place(from.0, from.1, Layer::Tile, b);
            return true;
        }
        let Some(free) = TELEPORTER.clone().find(|&c| count(c) == 0) else {
            return false;
        };
        self.place(from.0, from.1, Layer::Tile, free);
        self.place(to.0, to.1, Layer::Tile, free);
        true
    }
}

/// What will not work in `level`.
pub fn warnings(level: &Level) -> Vec<Warning> {
    let mut out = vec![];
    let mut warn = |message: String, cell| out.push(Warning { message, cell });
    let tiles: Vec<(Spot, i32)> = cells(level).collect();
    let count = |want: &dyn Fn(i32) -> bool| tiles.iter().filter(|(_, t)| want(*t)).count();

    if !level.objects.contains(&EXIT) {
        warn("There is no exit".to_owned(), None);
    }
    let named = |code: i32| catalogue::find(Layer::Tile, code).map_or("Piece", |p| p.name);
    for code in TELEPORTER {
        let found: Vec<Spot> = tiles
            .iter()
            .filter(|(_, t)| *t == code)
            .map(|(at, _)| *at)
            .collect();
        if found.len() == 1 {
            warn("This teleporter has no partner".to_owned(), Some(found[0]));
        } else if found.len() > 2 {
            let message = "Only two teleporters of a colour work as a pair".to_owned();
            warn(message, Some(found[2]));
        }
    }
    for &(at, code) in &tiles {
        if BUTTON.contains(&code) {
            let target = button_target(code);
            let works = links(level)
                .iter()
                .any(|l| !l.teleport && l.from == at && (l.to != at));
            if !works {
                let what = if target < CONVEYOR_SETS {
                    "conveyors"
                } else {
                    "gates"
                };
                warn(format!("This button has no {what} to work"), Some(at));
            }
        } else if GATE.contains(&code) {
            let target = gate_target(gate_colour(code));
            let opened = tiles
                .iter()
                .any(|(_, t)| BUTTON.contains(t) && button_target(*t) == target);
            if !opened {
                warn(format!("No button works this {}", named(code)), Some(at));
            }
        } else if SIGN.contains(&code) {
            let slot = (code - SIGN.start) as usize;
            if level.signs.get(slot).is_none_or(String::is_empty) {
                warn("This sign has no text".to_owned(), Some(at));
            }
        }
    }

    let group = |layer: Layer, code: i32, group: Group| {
        catalogue::find(layer, code).is_some_and(|p| p.group == group)
    };
    let objects = |g: Group| {
        level
            .objects
            .iter()
            .filter(|&&o| group(Layer::Object, o, g))
            .count()
    };
    // Box factories make theirs as the level runs.
    let boxes = objects(Group::Box) + count(&|t| (1900..2104).contains(&t));
    if boxes > MAX_BOXES {
        warn(
            format!("{boxes} boxes is over the game's limit of {MAX_BOXES}"),
            None,
        );
    }
    let monsters = objects(Group::Monster) + count(&|t| (2400..2408).contains(&t));
    if monsters > MAX_MONSTERS {
        warn(
            format!("{monsters} monsters is over the game's limit of {MAX_MONSTERS}"),
            None,
        );
    }
    let signs = count(&|t| SIGN.contains(&t));
    if signs > MAX_SIGNS {
        warn(
            format!("{signs} signs is over the limit of {MAX_SIGNS}"),
            None,
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED_TELEPORT: i32 = 1200;
    const YELLOW_TELEPORT: i32 = 1201;
    /// Stays down, works magenta gates.
    const MAGENTA_BUTTON: i32 = 1106;
    const MAGENTA_GATE: i32 = 1002;
    const GREEN_GATE: i32 = 1006;

    fn with(pieces: &[(Spot, i32)]) -> Editor {
        let mut editor = Editor::new();
        for &((x, y), code) in pieces {
            editor.place(x, y, Layer::Tile, code);
        }
        editor.commit();
        editor
    }

    fn says(editor: &Editor, text: &str) -> bool {
        warnings(editor.level())
            .iter()
            .any(|w| w.message.contains(text))
    }

    #[test]
    fn a_button_links_to_the_gates_of_its_target() {
        let editor = with(&[
            ((1, 1), MAGENTA_BUTTON),
            ((3, 1), MAGENTA_GATE),
            ((5, 1), GREEN_GATE),
        ]);
        assert_eq!(
            links(editor.level()),
            [Link {
                from: (1, 1),
                to: (3, 1),
                teleport: false,
                conveyor: false
            }]
        );
    }

    #[test]
    fn a_conveyor_button_links_to_its_set() {
        // Set 2 starts at 508.
        let editor = with(&[((1, 1), 1103), ((2, 1), 508), ((3, 1), 500)]);
        let found = links(editor.level());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].to, (2, 1));
    }

    #[test]
    fn teleporters_pair_by_colour_once() {
        let editor = with(&[((1, 1), RED_TELEPORT), ((3, 1), RED_TELEPORT)]);
        assert_eq!(links(editor.level()).len(), 1);
    }

    #[test]
    fn connecting_a_button_keeps_its_action() {
        // Pops up, works conveyors 1.
        let mut editor = with(&[((1, 1), 1101), ((3, 1), GREEN_GATE)]);
        assert!(editor.link((1, 1), (3, 1)));
        let code = editor.level().tile(1, 1);
        assert_eq!((code - 1100) % 3, 1);
        assert_eq!(links(editor.level()).len(), 1);
        assert!(editor.undo());
        assert_eq!(editor.level().tile(1, 1), 1101);
    }

    #[test]
    fn a_link_may_be_drawn_from_the_gate_end() {
        let mut editor = with(&[((1, 1), 1100), ((3, 1), MAGENTA_GATE)]);
        assert!(editor.link((3, 1), (1, 1)));
        assert_eq!(editor.level().tile(1, 1), MAGENTA_BUTTON);
    }

    #[test]
    fn a_lone_teleporter_joins_another() {
        let mut editor = with(&[((1, 1), RED_TELEPORT), ((3, 1), YELLOW_TELEPORT)]);
        assert!(editor.link((1, 1), (3, 1)));
        assert_eq!(editor.level().tile(3, 1), RED_TELEPORT);
        assert!(!says(&editor, "no partner"));
    }

    #[test]
    fn teleporters_of_two_pairs_take_a_new_colour() {
        let mut editor = with(&[
            ((1, 1), RED_TELEPORT),
            ((2, 1), RED_TELEPORT),
            ((1, 2), YELLOW_TELEPORT),
            ((2, 2), YELLOW_TELEPORT),
        ]);
        assert!(editor.link((1, 1), (1, 2)));
        assert_eq!(editor.level().tile(1, 1), 1202);
        assert_eq!(editor.level().tile(1, 2), 1202);
    }

    #[test]
    fn unrelated_pieces_do_not_link() {
        let mut editor = with(&[((1, 1), 100), ((3, 1), GREEN_GATE)]);
        let before = editor.level().clone();
        assert!(!editor.link((1, 1), (3, 1)));
        assert_eq!(*editor.level(), before);
    }

    #[test]
    fn warns_of_what_will_not_work() {
        let editor = with(&[
            ((1, 1), RED_TELEPORT),
            ((2, 1), MAGENTA_BUTTON),
            ((3, 1), GREEN_GATE),
            ((4, 1), 1300),
        ]);
        assert!(says(&editor, "no exit"));
        assert!(says(&editor, "no partner"));
        assert!(says(&editor, "no gates"));
        assert!(says(&editor, "No button works"));
        assert!(says(&editor, "no text"));
    }

    #[test]
    fn a_working_level_has_no_warnings() {
        let mut editor = with(&[
            ((1, 1), RED_TELEPORT),
            ((5, 5), RED_TELEPORT),
            ((2, 1), MAGENTA_BUTTON),
            ((3, 1), MAGENTA_GATE),
            ((4, 1), 1300),
        ]);
        editor.set_sign_text(0, "HI");
        editor.place(6, 6, Layer::Object, EXIT);
        editor.commit();
        assert_eq!(warnings(editor.level()), []);
    }

    #[test]
    fn warns_past_the_game_limits() {
        let mut editor = Editor::new();
        editor.fit(40, 40, 1, 1);
        for y in 1..30 {
            for x in 1..30 {
                editor.place(x, y, Layer::Object, 9);
            }
        }
        for y in 30..40 {
            for x in 1..10 {
                editor.place(x, y, Layer::Object, 10);
            }
        }
        assert!(says(&editor, "limit of 400"));
    }
}
