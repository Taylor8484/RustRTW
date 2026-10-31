//! The palette's entries, and a level holding one of each, which the game
//! photographs for their icons: game art cannot ship with the source, so the
//! icons are made on the player's machine from their own files.

use wonderland_formats::Level;
use wonderland_formats::catalogue::{Group, Layer, PIECES, Piece};

use crate::Editor;

/// Cells between one piece and the next across, and down, which needs more
/// as tall pieces reach up the picture.
pub const PITCH: usize = 5;
pub const ROW_PITCH: usize = 7;
const COLUMNS: usize = 16;
const EMPTY: i32 = 0;
/// The axis a palette shows as separate entries.
const EXPANDED: &str = "Character";

/// One thing to pick in the palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub piece: &'static Piece,
    pub choices: Vec<usize>,
}

impl Entry {
    pub fn code(&self) -> i32 {
        self.piece.code(&self.choices)
    }

    /// The name to show: a character's own, where the piece has one.
    pub fn name(&self) -> String {
        let axis = self.piece.axes.iter().position(|a| a.name == EXPANDED);
        match axis {
            Some(axis) if self.piece.group == Group::Character => {
                self.piece.axes[axis].values[self.choices[axis]].to_owned()
            }
            Some(axis) => format!(
                "{} ({})",
                self.piece.name, self.piece.axes[axis].values[self.choices[axis]]
            ),
            None => self.piece.name.to_owned(),
        }
    }
}

/// Whether `axis` of a piece is shown as separate entries.
pub fn expanded(axis: &str) -> bool {
    axis == EXPANDED
}

/// `piece`'s entries: one for each character it comes as, or else just one.
pub fn entries_of(piece: &'static Piece) -> Vec<Entry> {
    let zeroes = vec![0; piece.axes.len()];
    let axis = piece.axes.iter().position(|a| expanded(a.name));
    let Some(axis) = axis else {
        return vec![Entry {
            piece,
            choices: zeroes,
        }];
    };
    (0..piece.axes[axis].values.len())
        .map(|value| {
            let mut choices = zeroes.clone();
            choices[axis] = value;
            Entry { piece, choices }
        })
        .collect()
}

/// A level with every entry worth a picture on its own, and their cells.
pub fn stage() -> (Level, Vec<(Layer, i32, usize, usize)>) {
    let entries: Vec<Entry> = PIECES
        .iter()
        .filter(|p| p.group != Group::Setting && !(p.layer == Layer::Tile && p.base == EMPTY))
        .flat_map(entries_of)
        .collect();
    let rows = entries.len().div_ceil(COLUMNS);
    let (width, height) = (COLUMNS * PITCH, rows * ROW_PITCH);
    let mut level = Editor::new().level().clone();
    level.width = width;
    level.height = height;
    level.tiles = vec![EMPTY; width * height];
    level.objects = vec![0; width * height];
    let cells = entries.iter().enumerate().map(|(i, entry)| {
        let (x, y) = (
            i % COLUMNS * PITCH + PITCH / 2,
            i / COLUMNS * ROW_PITCH + ROW_PITCH / 2,
        );
        let code = entry.code();
        let index = level.index(x, y);
        match entry.piece.layer {
            Layer::Tile => level.tiles[index] = code,
            Layer::Object => level.objects[index] = code,
        }
        (entry.piece.layer, code, x, y)
    });
    let cells = cells.collect();
    (level, cells)
}

#[cfg(test)]
mod tests {
    use wonderland_logic::{Input, Options, Ruleset, Session, World};

    use super::*;

    #[test]
    fn each_player_is_an_entry_of_its_own() {
        let player = PIECES.iter().find(|p| p.group == Group::Character).unwrap();
        let entries = entries_of(player);
        let codes: Vec<i32> = entries.iter().map(Entry::code).collect();
        assert_eq!(codes, [1, 2, 3, 4]);
        assert_eq!(entries[1].name(), "Loof");
    }

    #[test]
    fn the_stage_starts_and_shows_everything_but_settings() {
        let (level, cells) = stage();
        let mut seen = std::collections::HashSet::new();
        for &(layer, code, x, y) in &cells {
            assert!(seen.insert((layer == Layer::Tile, code)));
            let at = match layer {
                Layer::Tile => level.tile(x, y),
                Layer::Object => level.object(x, y),
            };
            assert_eq!(at, code);
        }
        assert!(cells.len() > PIECES.len() - 10);
        let session = Session::default();
        let mut world = World::start(&level, Ruleset::rtwp(), Options::default(), session).unwrap();
        world.step(Input::default());
    }
}
