//! The level being edited: what the editor's screen changes, with no Bevy
//! types, so it is unit tested.
//!
//! [`Editor`] always holds a v6 [`Level`], the only version it saves. Every
//! change joins a pending edit, and [`Editor::commit`] makes that edit one
//! undo step, so a drag that paints fifty cells undoes as one. Grid sizes and
//! the new level follow `RTWP/editor.bb` (`ClearLevel`, the map size arrows)
//! and saving does what `SaveLevel` and `checklevel` do, so the files still
//! load in the original game and editor.
//!
//! [`canvas`] turns pointer gestures into these edits. [`proof`] keeps a way
//! to win the level and [`stamp`] keeps blocks of cells to use again.

pub mod canvas;
pub mod links;
pub mod proof;
mod reader;
pub mod settings;
pub mod signs;
pub mod stage;
pub mod stamp;

use std::mem;

use wonderland_formats::Level;
use wonderland_formats::catalogue::{self, Group, Layer};

/// The original editor's limits. The game reads a little more, and a level
/// that size still opens, but it does not grow.
pub const MIN_SIZE: usize = 14;
pub const MAX_SIZE: usize = 99;

/// The longest level file name `SaveLevel`'s prompt takes.
pub const MAX_NAME: usize = 15;

pub(crate) const FLOOR: i32 = 100;
const WALL: i32 = 200;
/// `ClearLevel` leaves `MusicTrack` alone, and it starts at 1.
const MUSIC_TRACK: i32 = 1;
const STINKY: i32 = 1;

/// The name to save `name` under, as the original's prompt takes it: upper
/// case letters and digits, up to [`MAX_NAME`]. `None` if it is no such name.
pub fn level_name(name: &str) -> Option<String> {
    let name = name.trim().to_ascii_uppercase();
    let fits = (1..=MAX_NAME).contains(&name.len());
    (fits && name.bytes().all(|b| b.is_ascii_alphanumeric())).then_some(name)
}

/// An edge of the level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

/// A block of cells taken from a level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clip {
    pub width: usize,
    pub height: usize,
    tiles: Vec<i32>,
    objects: Vec<i32>,
    /// The words of the signs in it, by tile.
    signs: Vec<(i32, String)>,
}

/// One cell's old contents.
#[derive(Debug)]
struct Cell {
    index: usize,
    tile: i32,
    object: i32,
}

/// What undoing an edit puts back. Applying a step leaves the opposite step
/// in its place.
#[derive(Debug)]
enum Step {
    Cells(Vec<Cell>),
    /// For an edit that changed the size or the header.
    Whole(Box<Level>),
}

impl Step {
    fn between(before: Level, after: &Level) -> Self {
        if header(&before) != header(after) {
            return Step::Whole(Box::new(before));
        }
        let cells = before.tiles.iter().zip(&before.objects).enumerate();
        Step::Cells(
            cells
                .filter(|&(i, (&tile, &object))| {
                    tile != after.tiles[i] || object != after.objects[i]
                })
                .map(|(index, (&tile, &object))| Cell {
                    index,
                    tile,
                    object,
                })
                .collect(),
        )
    }

    fn swap(&mut self, level: &mut Level) {
        match self {
            Step::Cells(cells) => {
                for cell in cells {
                    mem::swap(&mut cell.tile, &mut level.tiles[cell.index]);
                    mem::swap(&mut cell.object, &mut level.objects[cell.index]);
                }
            }
            Step::Whole(whole) => mem::swap(whole.as_mut(), level),
        }
    }
}

/// `level` without its grids.
fn header(level: &Level) -> Level {
    Level {
        tiles: vec![],
        objects: vec![],
        ..level.clone()
    }
}

/// The cells `depth` lines in from `side`.
fn line(level: &Level, side: Side, depth: usize) -> Vec<usize> {
    let (width, height) = (level.width, level.height);
    match side {
        Side::Left => (0..height).map(|y| level.index(depth, y)).collect(),
        Side::Right => (0..height)
            .map(|y| level.index(width - 1 - depth, y))
            .collect(),
        Side::Top => (0..width).map(|x| level.index(x, depth)).collect(),
        Side::Bottom => (0..width)
            .map(|x| level.index(x, height - 1 - depth))
            .collect(),
    }
}

/// Gives `level` a new size, with what was at (0, 0) now at (`dx`, `dy`) and
/// wall in any new cell.
fn reframe(level: &mut Level, width: usize, height: usize, dx: isize, dy: isize) {
    let mut tiles = vec![WALL; width * height];
    let mut objects = vec![0; width * height];
    for y in 0..level.height {
        for x in 0..level.width {
            let (Some(new_x), Some(new_y)) = (x.checked_add_signed(dx), y.checked_add_signed(dy))
            else {
                continue;
            };
            if new_x < width && new_y < height {
                let (from, to) = (level.index(x, y), new_y * width + new_x);
                tiles[to] = level.tiles[from];
                objects[to] = level.objects[from];
            }
        }
    }
    level.width = width;
    level.height = height;
    level.tiles = tiles;
    level.objects = objects;
}

/// A level being edited, with its undo history.
#[derive(Debug)]
pub struct Editor {
    level: Level,
    /// The level before the pending edit, if there is one.
    before: Option<Box<Level>>,
    undo: Vec<Step>,
    redo: Vec<Step>,
    /// The level as last saved or opened.
    saved: Level,
    /// Counts changes to the level, for a screen that follows it.
    revision: u64,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    /// `ClearLevel`: a floor inside a wall, in the smallest size.
    pub fn new() -> Self {
        let mut tiles = vec![WALL; MIN_SIZE * MIN_SIZE];
        for y in 1..MIN_SIZE - 1 {
            for x in 1..MIN_SIZE - 1 {
                tiles[y * MIN_SIZE + x] = FLOOR;
            }
        }
        let level = Level {
            magic: String::new(),
            version: 6,
            file_name: "UNTITLED".into(),
            map_id: None,
            description: "Untitled Level".into(),
            houses: None,
            models: None,
            textures: None,
            backgrounds: None,
            time: 300,
            style: 2,
            background: 0,
            width: MIN_SIZE,
            height: MIN_SIZE,
            tiles,
            objects: vec![0; MIN_SIZE * MIN_SIZE],
            signs: vec![],
            music_track: Some(MUSIC_TRACK),
        };
        Self::open(level)
    }

    /// Edits `level`, upgraded to v6 if it is older.
    pub fn open(level: Level) -> Self {
        let level = level.upgraded();
        Self {
            saved: level.clone(),
            level,
            before: None,
            undo: vec![],
            redo: vec![],
            revision: 0,
        }
    }

    pub fn level(&self) -> &Level {
        &self.level
    }

    /// Changes whenever the level may have.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Whether the level differs from the one last saved or opened.
    pub fn is_dirty(&self) -> bool {
        self.level != self.saved
    }

    /// The level, for a change that joins the pending edit.
    fn editing(&mut self) -> &mut Level {
        if self.before.is_none() {
            self.before = Some(Box::new(self.level.clone()));
        }
        self.revision += 1;
        &mut self.level
    }

    /// Ends the pending edit as one undo step. An edit that changed nothing
    /// leaves no step.
    pub fn commit(&mut self) {
        let Some(before) = self.before.take() else {
            return;
        };
        if *before != self.level {
            self.undo.push(Step::between(*before, &self.level));
            self.redo.clear();
        }
    }

    /// Drops the pending edit, as when a drag is called off.
    pub fn cancel(&mut self) {
        if let Some(before) = self.before.take() {
            self.level = *before;
            self.revision += 1;
        }
    }

    /// Takes back the last edit, committing a pending one first. Returns
    /// whether there was one.
    pub fn undo(&mut self) -> bool {
        self.commit();
        let Some(mut step) = self.undo.pop() else {
            return false;
        };
        step.swap(&mut self.level);
        self.revision += 1;
        self.redo.push(step);
        true
    }

    /// Puts back the last edit undone, and returns whether there was one. A
    /// new edit clears these once it is committed.
    pub fn redo(&mut self) -> bool {
        self.commit();
        let Some(mut step) = self.redo.pop() else {
            return false;
        };
        step.swap(&mut self.level);
        self.revision += 1;
        self.undo.push(step);
        true
    }

    /// Puts a piece on a cell. A character is unique, so one placed elsewhere
    /// moves here. An object code of 0 takes the object away.
    pub fn place(&mut self, x: usize, y: usize, layer: Layer, code: i32) {
        let unique = layer == Layer::Object
            && catalogue::find(layer, code).is_some_and(|p| p.group == Group::Character);
        let level = self.editing();
        let index = level.index(x, y);
        match layer {
            Layer::Tile => level.tiles[index] = code,
            Layer::Object => {
                if unique {
                    level.objects.iter_mut().for_each(|o| {
                        if *o == code {
                            *o = 0;
                        }
                    });
                }
                level.objects[index] = code;
            }
        }
    }

    /// What shows on a cell: its object, or else its tile.
    pub fn top(&self, x: usize, y: usize) -> (Layer, i32) {
        match self.level.object(x, y) {
            0 => (Layer::Tile, self.level.tile(x, y)),
            object => (Layer::Object, object),
        }
    }

    /// Takes away what shows on a cell. Ground goes back to floor.
    pub fn erase(&mut self, x: usize, y: usize) {
        match self.top(x, y).0 {
            Layer::Object => self.place(x, y, Layer::Object, 0),
            Layer::Tile => self.place(x, y, Layer::Tile, FLOOR),
        }
    }

    /// Turns a block to bare floor.
    pub fn clear(&mut self, x: usize, y: usize, width: usize, height: usize) {
        for y in y..y + height {
            for x in x..x + width {
                self.place(x, y, Layer::Tile, FLOOR);
                self.place(x, y, Layer::Object, 0);
            }
        }
    }

    /// The block with its corner at (`x`, `y`), which must be in the level.
    pub fn copy(&self, x: usize, y: usize, width: usize, height: usize) -> Clip {
        let rows = |grid: &[i32]| -> Vec<i32> {
            (y..y + height)
                .flat_map(|y| grid[self.level.index(x, y)..][..width].to_vec())
                .collect()
        };
        let tiles = rows(&self.level.tiles);
        let signs = tiles.iter().filter(|&&t| signs::is_sign(t));
        let signs = signs
            .map(|&t| (t, self.sign_text((t - signs::SIGN) as usize).to_owned()))
            .collect();
        Clip {
            width,
            height,
            tiles,
            objects: rows(&self.level.objects),
            signs,
        }
    }

    /// Puts a block with its corner at (`x`, `y`), replacing what is there.
    /// What falls off the level is lost; see [`Editor::fit`].
    pub fn paste(&mut self, x: usize, y: usize, clip: &Clip) {
        let width = clip.width.min(self.level.width.saturating_sub(x));
        let height = clip.height.min(self.level.height.saturating_sub(y));
        for row in 0..height {
            for column in 0..width {
                let from = row * clip.width + column;
                let (to_x, to_y) = (x + column, y + row);
                self.place(to_x, to_y, Layer::Tile, clip.tiles[from]);
                self.place(to_x, to_y, Layer::Object, clip.objects[from]);
            }
        }
    }

    /// Adds a line of cells at `side`, unless the level is at the limit. The
    /// wall moves out with it: as in the original, plain wall on the old edge
    /// becomes floor, but the original's floor overwrites anything else there
    /// and this keeps it.
    pub fn grow(&mut self, side: Side) -> bool {
        let (width, height) = (self.level.width, self.level.height);
        let horizontal = matches!(side, Side::Left | Side::Right);
        if (if horizontal { width } else { height }) >= MAX_SIZE {
            return false;
        }
        let (new_width, new_height, dx, dy) = match side {
            Side::Left => (width + 1, height, 1, 0),
            Side::Right => (width + 1, height, 0, 0),
            Side::Top => (width, height + 1, 0, 1),
            Side::Bottom => (width, height + 1, 0, 0),
        };
        let level = self.editing();
        reframe(level, new_width, new_height, dx, dy);
        let seam = line(level, side, 1);
        for &index in seam.iter().skip(1).take(seam.len().saturating_sub(2)) {
            if level.tiles[index] == WALL {
                level.tiles[index] = FLOOR;
            }
        }
        true
    }

    /// Removes the line of cells at `side` and everything on it, unless the
    /// level is at the limit, and makes the new edge wall.
    pub fn shrink(&mut self, side: Side) -> bool {
        let (width, height) = (self.level.width, self.level.height);
        let horizontal = matches!(side, Side::Left | Side::Right);
        if (if horizontal { width } else { height }) <= MIN_SIZE {
            return false;
        }
        let (new_width, new_height, dx, dy) = match side {
            Side::Left => (width - 1, height, -1, 0),
            Side::Right => (width - 1, height, 0, 0),
            Side::Top => (width, height - 1, 0, -1),
            Side::Bottom => (width, height - 1, 0, 0),
        };
        let level = self.editing();
        reframe(level, new_width, new_height, dx, dy);
        for index in line(level, side, 0) {
            level.tiles[index] = WALL;
        }
        true
    }

    /// Grows the level until a block with its corner at (`x`, `y`), which may
    /// be off the level, stands inside the outer wall. Returns how far the
    /// level's cells moved right and down, or `None`, changing nothing, if
    /// that would pass [`MAX_SIZE`].
    pub fn fit(
        &mut self,
        x: isize,
        y: isize,
        width: usize,
        height: usize,
    ) -> Option<(usize, usize)> {
        let (level_width, level_height) = (self.level.width as isize, self.level.height as isize);
        let left = (1 - x).max(0) as usize;
        let top = (1 - y).max(0) as usize;
        let right = (x + width as isize + 1 - level_width).max(0) as usize;
        let bottom = (y + height as isize + 1 - level_height).max(0) as usize;
        let across = left + right;
        let down = top + bottom;
        if (across > 0 && self.level.width + across > MAX_SIZE)
            || (down > 0 && self.level.height + down > MAX_SIZE)
        {
            return None;
        }
        for (side, count) in [
            (Side::Left, left),
            (Side::Right, right),
            (Side::Top, top),
            (Side::Bottom, bottom),
        ] {
            for _ in 0..count {
                self.grow(side);
            }
        }
        Some((left, top))
    }

    /// Sets the name stored in the file, which the original also takes as
    /// the file's name.
    pub fn rename(&mut self, name: &str) {
        self.editing().file_name = name.to_owned();
    }

    /// Commits any pending edit, tidies the level as `checklevel` does and
    /// returns the file. `map_id` should be new: the player's scores for a
    /// custom level are kept by it, and `SaveLevel` makes one from the clock
    /// on every save.
    ///
    /// Tidying can change the level (it only does for one opened with two of
    /// a character, or with none), and that is an undo step of its own.
    pub fn save(&mut self, map_id: i32) -> Vec<u8> {
        self.commit();
        self.tidy();
        self.commit();
        self.level.map_id = Some(map_id);
        self.saved = self.level.clone();
        self.level.to_bytes()
    }

    /// The level to test play: tidied as [`Editor::save`] tidies, without
    /// touching the level being edited or its history.
    pub fn playable(&self) -> Level {
        let mut copy = Self::open(self.level.clone());
        copy.tidy();
        copy.level
    }

    /// [`Editor::playable`] with the first character found moved to (`x`,
    /// `y`), to test a level from there.
    pub fn playable_from(&self, x: usize, y: usize) -> Level {
        let mut copy = Self::open(self.playable());
        let first = copy.level.objects.iter().copied().find(|&code| {
            catalogue::find(Layer::Object, code).is_some_and(|p| p.group == Group::Character)
        });
        if let Some(code) = first {
            copy.place(x, y, Layer::Object, code);
        }
        copy.level
    }

    /// Whether the level has a character of its own, before tidying.
    pub fn has_character(&self) -> bool {
        self.level.objects.iter().any(|&code| {
            catalogue::find(Layer::Object, code).is_some_and(|p| p.group == Group::Character)
        })
    }

    /// Sets the title the custom level menu shows.
    pub fn set_title(&mut self, title: &str) {
        self.editing().description = title.to_owned();
    }

    /// Keeps the first of each character, and puts Stinky at (1, 1) if there
    /// are none.
    fn tidy(&mut self) {
        let characters: Vec<i32> = catalogue::PIECES
            .iter()
            .filter(|p| p.group == Group::Character)
            .flat_map(|p| p.base..p.base + p.count())
            .collect();
        let extra = |objects: &[i32]| {
            let mut seen = vec![];
            objects.iter().enumerate().find_map(|(i, code)| {
                if !characters.contains(code) {
                    return None;
                }
                if seen.contains(code) {
                    return Some(i);
                }
                seen.push(*code);
                None
            })
        };
        while let Some(index) = extra(&self.level.objects) {
            self.editing().objects[index] = 0;
        }
        if !self.level.objects.iter().any(|o| characters.contains(o)) {
            let index = self.level.width + 1;
            if let Some(object) = self.editing().objects.get_mut(index) {
                *object = STINKY;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wonderland_formats::Game;

    const COIN: i32 = 5;

    #[test]
    fn a_new_level_matches_clear_level() {
        let editor = Editor::new();
        let level = editor.level();
        assert_eq!((level.width, level.height), (14, 14));
        assert_eq!((level.tile(0, 0), level.tile(13, 5)), (200, 200));
        assert_eq!((level.tile(1, 1), level.tile(12, 12)), (100, 100));
        assert_eq!(level.signs.len(), 20);
        assert_eq!(level.version, 6);
        assert_eq!(level.music_track, Some(1));
        assert!(!editor.is_dirty());
    }

    #[test]
    fn a_stroke_is_one_undo_step() {
        let mut editor = Editor::new();
        let start = editor.level().clone();
        editor.place(1, 1, Layer::Tile, 300);
        editor.place(2, 1, Layer::Tile, 300);
        editor.commit();
        editor.place(3, 1, Layer::Object, COIN);
        editor.commit();
        let done = editor.level().clone();

        assert!(editor.undo());
        assert_eq!(editor.level().object(3, 1), 0);
        assert_eq!(editor.level().tile(2, 1), 300);
        assert!(editor.undo());
        assert_eq!(*editor.level(), start);
        assert!(!editor.undo());
        assert!(editor.redo());
        assert!(editor.redo());
        assert!(!editor.redo());
        assert_eq!(*editor.level(), done);
    }

    #[test]
    fn nothing_changed_leaves_no_step() {
        let mut editor = Editor::new();
        editor.place(1, 1, Layer::Tile, 100);
        editor.commit();
        assert!(!editor.undo());
    }

    #[test]
    fn a_new_edit_clears_redo() {
        let mut editor = Editor::new();
        editor.place(1, 1, Layer::Tile, 300);
        editor.commit();
        editor.undo();
        editor.place(2, 2, Layer::Tile, 300);
        editor.commit();
        assert!(!editor.redo());
    }

    #[test]
    fn cancel_drops_the_pending_edit() {
        let mut editor = Editor::new();
        let start = editor.level().clone();
        editor.place(1, 1, Layer::Tile, 300);
        editor.grow(Side::Left);
        editor.cancel();
        assert_eq!(*editor.level(), start);
        assert!(!editor.undo());
    }

    #[test]
    fn characters_are_unique() {
        let mut editor = Editor::new();
        editor.place(1, 1, Layer::Object, 2);
        editor.place(3, 3, Layer::Object, 2);
        assert_eq!(editor.level().object(1, 1), 0);
        assert_eq!(editor.level().object(3, 3), 2);
        editor.place(1, 1, Layer::Object, COIN);
        editor.place(2, 1, Layer::Object, COIN);
        assert_eq!(editor.level().object(1, 1), COIN);
    }

    #[test]
    fn erase_takes_the_top_piece() {
        let mut editor = Editor::new();
        editor.place(1, 1, Layer::Tile, 300);
        editor.place(1, 1, Layer::Object, COIN);
        assert_eq!(editor.top(1, 1), (Layer::Object, COIN));
        editor.erase(1, 1);
        assert_eq!(editor.top(1, 1), (Layer::Tile, 300));
        editor.erase(1, 1);
        assert_eq!(editor.top(1, 1), (Layer::Tile, 100));
    }

    #[test]
    fn copy_and_paste() {
        let mut editor = Editor::new();
        editor.place(2, 3, Layer::Tile, 300);
        editor.place(3, 3, Layer::Object, COIN);
        let clip = editor.copy(2, 3, 2, 1);
        editor.paste(5, 6, &clip);
        assert_eq!(editor.level().tile(5, 6), 300);
        assert_eq!(editor.level().object(6, 6), COIN);
        // Off the edge, the rest is lost.
        editor.paste(13, 13, &clip);
        assert_eq!(editor.level().tile(13, 13), 300);
    }

    #[test]
    fn grow_moves_the_wall_out() {
        for (side, inner, outer) in [
            (Side::Left, (1, 5), (0, 5)),
            (Side::Right, (13, 5), (14, 5)),
            (Side::Top, (5, 1), (5, 0)),
            (Side::Bottom, (5, 13), (5, 14)),
        ] {
            let mut editor = Editor::new();
            assert!(editor.grow(side));
            let level = editor.level();
            assert_eq!(level.tile(inner.0, inner.1), FLOOR, "{side:?}");
            assert_eq!(level.tile(outer.0, outer.1), WALL, "{side:?}");
            assert_eq!(level.tiles.len(), 15 * 14);
            // The corners of the new line and of the old one are wall.
            for x in [0, level.width - 1] {
                assert_eq!(level.tile(x, 0), WALL);
                assert_eq!(level.tile(x, level.height - 1), WALL);
            }
            assert_eq!(level.tile(1, 0), WALL);
            assert_eq!(level.tile(0, 1), WALL);
        }
    }

    #[test]
    fn grow_keeps_what_was_on_the_edge() {
        let mut editor = Editor::new();
        editor.place(0, 5, Layer::Tile, 300);
        editor.place(0, 6, Layer::Object, COIN);
        editor.grow(Side::Left);
        assert_eq!(editor.level().tile(1, 5), 300);
        assert_eq!(editor.level().object(1, 6), COIN);
    }

    #[test]
    fn shrink_walls_the_new_edge() {
        let mut editor = Editor::new();
        assert!(!editor.shrink(Side::Left));
        editor.grow(Side::Left);
        editor.grow(Side::Top);
        editor.place(3, 3, Layer::Object, COIN);
        assert!(editor.shrink(Side::Left));
        assert!(editor.shrink(Side::Top));
        assert_eq!(*editor.level().tiles, *Editor::new().level().tiles);
        assert_eq!(editor.level().object(2, 2), COIN);
    }

    #[test]
    fn size_stops_at_the_limit() {
        let mut editor = Editor::new();
        while editor.grow(Side::Right) {}
        assert_eq!(editor.level().width, MAX_SIZE);
        assert_eq!(editor.fit(MAX_SIZE as isize, 1, 1, 1), None);
        assert_eq!(editor.fit(-1, 1, 1, 1), None);
        assert_eq!(editor.fit(1, -1, 1, 1), Some((0, 2)));
    }

    #[test]
    fn a_level_past_the_limit_opens_but_does_not_grow() {
        let mut level = Editor::new().level().clone();
        level.width = 102;
        level.height = 1;
        level.tiles = vec![WALL; 102];
        level.objects = vec![0; 102];
        let mut editor = Editor::open(level);
        assert!(!editor.grow(Side::Left));
    }

    #[test]
    fn fit_grows_around_the_block() {
        let mut editor = Editor::new();
        editor.place(1, 1, Layer::Object, COIN);
        assert_eq!(editor.fit(1, 1, 3, 3), Some((0, 0)));
        assert_eq!(editor.fit(-2, 1, 1, 1), Some((3, 0)));
        assert_eq!(editor.level().width, 17);
        assert_eq!(editor.level().object(4, 1), COIN);
        assert_eq!(editor.level().tile(0, 5), WALL);
        assert_eq!(editor.level().tile(1, 5), FLOOR);
        assert_eq!(editor.fit(16, 16, 1, 1), Some((0, 0)));
        assert_eq!(editor.level().width, 18);
        assert_eq!(editor.level().height, 18);
    }

    #[test]
    fn save_tidies_like_checklevel() {
        let mut editor = Editor::new();
        let bytes = editor.save(1234);
        let saved = Level::parse(&bytes, Game::Rtwp).unwrap();
        assert_eq!(saved.object(1, 1), STINKY);
        assert_eq!(saved.map_id, Some(1234));
        assert_eq!(saved.version, 6);
        assert_eq!(saved, *editor.level());
        assert!(!editor.is_dirty());

        let mut level = editor.level().clone();
        let spare = level.index(5, 5);
        level.objects[spare] = STINKY;
        let mut editor = Editor::open(level);
        editor.save(1);
        assert_eq!(editor.level().object(1, 1), STINKY);
        assert_eq!(editor.level().object(5, 5), 0);
        assert!(editor.undo());
        assert_eq!(editor.level().object(5, 5), STINKY);
    }

    #[test]
    fn playable_tidies_a_copy() {
        let mut editor = Editor::new();
        editor.place(3, 3, Layer::Object, COIN);
        let level = editor.playable();
        assert_eq!(level.object(1, 1), STINKY);
        assert_eq!(level.object(3, 3), COIN);
        assert_eq!(editor.level().object(1, 1), 0);
        // The pending edit is still pending, and still undoes as one.
        editor.commit();
        assert!(editor.undo());
        assert_eq!(editor.level().object(3, 3), 0);
        assert!(!editor.undo());
    }

    #[test]
    fn playable_from_moves_the_first_character() {
        let mut editor = Editor::new();
        editor.place(2, 2, Layer::Object, 2);
        let level = editor.playable_from(5, 6);
        assert_eq!(level.object(5, 6), 2);
        assert_eq!(level.object(2, 2), 0);
        assert_eq!(editor.level().object(2, 2), 2);
        assert_eq!(Editor::new().playable_from(5, 6).object(5, 6), STINKY);
    }

    #[test]
    fn names_follow_the_prompt() {
        assert_eq!(level_name(" my1 ").as_deref(), Some("MY1"));
        assert_eq!(level_name(""), None);
        assert_eq!(level_name("A/B"), None);
        assert_eq!(level_name("TWO WORDS"), None);
        assert_eq!(level_name(&"A".repeat(16)), None);
    }

    #[test]
    fn revision_follows_every_change() {
        let mut editor = Editor::new();
        let start = editor.revision();
        editor.place(1, 1, Layer::Tile, 300);
        let placed = editor.revision();
        assert_ne!(placed, start);
        editor.undo();
        assert_ne!(editor.revision(), placed);
    }

    #[test]
    fn dirty_follows_the_level() {
        let mut editor = Editor::new();
        editor.place(1, 1, Layer::Object, STINKY);
        assert!(editor.is_dirty());
        editor.save(1);
        assert!(!editor.is_dirty());
        editor.rename("MINE");
        assert!(editor.is_dirty());
        editor.undo();
        assert!(!editor.is_dirty());
    }
}
