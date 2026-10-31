//! The canvas's gestures, with no Bevy types: what pressing, moving and
//! releasing the pointer on a cell does to an [`Editor`].
//!
//! A cell may be off the level. A piece put on or past the outer wall grows
//! the level around it, which moves every cell; [`Canvas::take_shift`] reports
//! that so a screen can keep the picture still. Each gesture is one undo step.

use wonderland_formats::catalogue::{self, Group, Layer};

use crate::links::Spot;
use crate::{Clip, Editor, FLOOR};

/// A cell, which may be off the level.
pub type Cell = (isize, isize);

/// A piece, with its choices made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Brush {
    pub layer: Layer,
    pub code: i32,
}

/// What the left button does on the canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tool {
    #[default]
    Select,
    Brush(Brush),
    Erase,
    /// Draws a connection from one piece to another.
    Link,
}

/// A block of cells inside the level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Block {
    pub fn contains(&self, (x, y): Cell) -> bool {
        let (left, top) = (self.x as isize, self.y as isize);
        (left..left + self.width as isize).contains(&x)
            && (top..top + self.height as isize).contains(&y)
    }
}

/// A gesture under way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gesture {
    /// Painting, or erasing with no brush.
    Stroke { brush: Option<Brush>, last: Cell },
    /// Dragging out a selection.
    Band { start: Cell, end: Cell },
    /// Dragging a placed piece, which moves on release.
    Piece { from: Cell, to: Cell, brush: Brush },
    /// Dragging the selection, which moves on release.
    Move { from: Cell, to: Cell },
    /// Drawing a connection, which is made on release.
    Link { from: Cell, to: Cell },
}

/// `brush` turned one step, for the first choice that is a direction, or
/// else the first choice of any kind.
pub fn turned(brush: Brush) -> Brush {
    let Some(piece) = catalogue::find(brush.layer, brush.code) else {
        return brush;
    };
    let Some(axis) = piece
        .axes
        .iter()
        .position(|a| matches!(a.name, "Direction" | "Facing"))
        .or((!piece.axes.is_empty() && piece.group != Group::Sign).then_some(0))
    else {
        return brush;
    };
    let mut choices = piece.choices(brush.code);
    choices[axis] = (choices[axis] + 1) % piece.axes[axis].values.len();
    Brush {
        code: piece.code(&choices),
        ..brush
    }
}

/// Whether `brush` is a sign, whose slot [`Editor::new_sign`] gives.
fn is_sign(brush: Brush) -> bool {
    catalogue::find(brush.layer, brush.code).is_some_and(|p| p.group == Group::Sign)
}

fn spot(editor: &Editor, (x, y): Cell) -> Option<Spot> {
    let level = editor.level();
    let x = usize::try_from(x).ok().filter(|&x| x < level.width)?;
    let y = usize::try_from(y).ok().filter(|&y| y < level.height)?;
    Some((x, y))
}

/// The cells from `from` to `to`, leaving out `from`.
fn line(from: Cell, to: Cell) -> Vec<Cell> {
    let steps = (to.0 - from.0).abs().max((to.1 - from.1).abs());
    (1..=steps)
        .map(|i| {
            let along = |a: isize, b: isize| {
                let scaled = (b - a) as f64 * i as f64 / steps as f64;
                a + scaled.round() as isize
            };
            (along(from.0, to.0), along(from.1, to.1))
        })
        .collect()
}

/// Bare ground, which a drag on selects rather than moves.
fn bare(code: i32) -> bool {
    (FLOOR..FLOOR + 4).contains(&code)
}

/// The state of the canvas.
#[derive(Debug, Default)]
pub struct Canvas {
    pub tool: Tool,
    gesture: Option<Gesture>,
    selection: Option<Block>,
    clip: Option<Clip>,
    /// How far growing has moved the cells, since last taken.
    shift: (usize, usize),
    /// The part of `shift` the pending edit made.
    pending: (usize, usize),
    /// The part of `shift` a called-off gesture took back.
    back: (usize, usize),
}

impl Canvas {
    pub fn gesture(&self) -> Option<Gesture> {
        self.gesture
    }

    pub fn selection(&self) -> Option<Block> {
        self.selection
    }

    pub fn has_clip(&self) -> bool {
        self.clip.is_some()
    }

    /// How far the cells have moved right and down since the last call.
    pub fn take_shift(&mut self) -> (isize, isize) {
        let (shift, back) = (self.shift, self.back);
        self.shift = (0, 0);
        self.back = (0, 0);
        (
            shift.0 as isize - back.0 as isize,
            shift.1 as isize - back.1 as isize,
        )
    }

    fn moved(&mut self, (dx, dy): (usize, usize)) {
        if (dx, dy) == (0, 0) {
            return;
        }
        self.shift = (self.shift.0 + dx, self.shift.1 + dy);
        self.pending = (self.pending.0 + dx, self.pending.1 + dy);
        let shift = |c: &mut Cell| *c = (c.0 + dx as isize, c.1 + dy as isize);
        match &mut self.gesture {
            Some(Gesture::Stroke { last, .. }) => shift(last),
            Some(Gesture::Band { start, end }) => {
                shift(start);
                shift(end);
            }
            Some(
                Gesture::Piece { from, to, .. }
                | Gesture::Move { from, to }
                | Gesture::Link { from, to },
            ) => {
                shift(from);
                shift(to);
            }
            None => {}
        }
        if let Some(block) = &mut self.selection {
            block.x += dx;
            block.y += dy;
        }
    }

    /// Puts a piece on a cell, growing the level to hold it. Plain wall on
    /// the outer wall changes nothing. Returns whether the cell was reached,
    /// and refuses past the size limit.
    fn put(&mut self, editor: &mut Editor, (x, y): Cell, brush: Brush) -> bool {
        let level = editor.level();
        let (width, height) = (level.width as isize, level.height as isize);
        let on_level = (0..width).contains(&x) && (0..height).contains(&y);
        let walled = brush.layer == Layer::Tile
            && catalogue::find(brush.layer, brush.code).is_some_and(|p| p.group == Group::Wall);
        let inside = (1..width - 1).contains(&x) && (1..height - 1).contains(&y);
        if on_level && !inside && walled {
            return true;
        }
        let Some(moved) = editor.fit(x, y, 1, 1) else {
            return false;
        };
        if moved != (0, 0) {
            editor.tuck_settings();
        }
        self.moved(moved);
        let (x, y) = (x + moved.0 as isize, y + moved.1 as isize);
        editor.place(x as usize, y as usize, brush.layer, brush.code);
        true
    }

    /// One cell of a stroke. Erasing stops at the edges of the level's
    /// inside.
    fn stroke(&mut self, editor: &mut Editor, brush: Option<Brush>, (x, y): Cell) {
        let Some(brush) = brush else {
            let level = editor.level();
            let (width, height) = (level.width as isize, level.height as isize);
            if (1..width - 1).contains(&x) && (1..height - 1).contains(&y) {
                editor.erase(x as usize, y as usize);
            }
            return;
        };
        self.put_new(editor, (x, y), brush);
    }

    /// [`Canvas::put`] for a piece new to the level, which a sign needs a
    /// slot for. Returns false if there is none.
    fn put_new(&mut self, editor: &mut Editor, cell: Cell, brush: Brush) -> bool {
        if !is_sign(brush) {
            return self.put(editor, cell, brush);
        }
        if spot(editor, cell).is_some_and(|(x, y)| {
            is_sign(Brush {
                layer: Layer::Tile,
                code: editor.level().tile(x, y),
            })
        }) {
            return true;
        }
        match editor.new_sign() {
            Some(code) => self.put(editor, cell, Brush { code, ..brush }),
            None => false,
        }
    }

    /// The pointer went down on `cell`. `erase` is the erasing button, and
    /// `band` always drags out a selection.
    pub fn press(&mut self, editor: &mut Editor, cell: Cell, erase: bool, band: bool) {
        if self.gesture.is_some() {
            return;
        }
        let tool = if erase { Tool::Erase } else { self.tool };
        let brush = match tool {
            Tool::Brush(brush) => Some(brush),
            Tool::Erase => None,
            Tool::Link => {
                self.gesture = Some(Gesture::Link {
                    from: cell,
                    to: cell,
                });
                return;
            }
            Tool::Select => {
                self.gesture = Some(self.select_gesture(editor, cell, band));
                return;
            }
        };
        self.gesture = Some(Gesture::Stroke { brush, last: cell });
        self.stroke(editor, brush, cell);
    }

    fn select_gesture(&self, editor: &Editor, cell: Cell, band: bool) -> Gesture {
        let level = editor.level();
        let (x, y) = cell;
        let on_level =
            (0..level.width as isize).contains(&x) && (0..level.height as isize).contains(&y);
        if !band && self.selection.is_some_and(|s| s.contains(cell)) {
            return Gesture::Move {
                from: cell,
                to: cell,
            };
        }
        if !band && on_level {
            let (layer, code) = editor.top(x as usize, y as usize);
            if !(layer == Layer::Tile && bare(code)) && !self.outer(editor, cell) {
                return Gesture::Piece {
                    from: cell,
                    to: cell,
                    brush: Brush { layer, code },
                };
            }
        }
        Gesture::Band {
            start: cell,
            end: cell,
        }
    }

    fn outer(&self, editor: &Editor, (x, y): Cell) -> bool {
        let level = editor.level();
        !((1..level.width as isize - 1).contains(&x) && (1..level.height as isize - 1).contains(&y))
    }

    /// The pointer moved to `cell`.
    pub fn hover(&mut self, editor: &mut Editor, cell: Cell) {
        match self.gesture {
            Some(Gesture::Stroke { brush, last }) => {
                let mut offset = (0, 0);
                for (x, y) in line(last, cell) {
                    let shift = self.pending;
                    self.stroke(editor, brush, (x + offset.0, y + offset.1));
                    offset.0 += (self.pending.0 - shift.0) as isize;
                    offset.1 += (self.pending.1 - shift.1) as isize;
                }
                self.gesture = Some(Gesture::Stroke {
                    brush,
                    last: (cell.0 + offset.0, cell.1 + offset.1),
                });
            }
            Some(Gesture::Band { start, .. }) => {
                self.gesture = Some(Gesture::Band { start, end: cell });
            }
            Some(Gesture::Piece { from, brush, .. }) => {
                self.gesture = Some(Gesture::Piece {
                    from,
                    to: cell,
                    brush,
                });
            }
            Some(Gesture::Move { from, .. }) => {
                self.gesture = Some(Gesture::Move { from, to: cell });
            }
            Some(Gesture::Link { from, .. }) => {
                self.gesture = Some(Gesture::Link { from, to: cell });
            }
            None => {}
        }
    }

    /// The pointer came up, which finishes the gesture.
    pub fn release(&mut self, editor: &mut Editor) {
        match self.gesture {
            Some(Gesture::Band { start, end }) => {
                self.selection = self.band(editor, start, end);
                self.gesture = None;
            }
            Some(Gesture::Piece { from, to, brush }) => {
                if from == to {
                    self.selection = self.band(editor, from, to);
                } else {
                    let (x, y) = (from.0 as usize, from.1 as usize);
                    match brush.layer {
                        Layer::Object => editor.place(x, y, Layer::Object, 0),
                        Layer::Tile => editor.place(x, y, Layer::Tile, FLOOR),
                    }
                    if !self.put(editor, to, brush) {
                        return self.cancel(editor);
                    }
                }
                self.gesture = None;
            }
            Some(Gesture::Move { from, to }) => {
                self.gesture = None;
                if let Some(block) = self.selection.filter(|_| from != to) {
                    let clip = editor.copy(block.x, block.y, block.width, block.height);
                    editor.clear(block.x, block.y, block.width, block.height);
                    let at = (
                        block.x as isize + to.0 - from.0,
                        block.y as isize + to.1 - from.1,
                    );
                    if !self.drop_clip(editor, at, &clip) {
                        return self.cancel(editor);
                    }
                }
            }
            Some(Gesture::Link { from, to }) => {
                self.gesture = None;
                if let (Some(from), Some(to)) = (spot(editor, from), spot(editor, to)) {
                    editor.link(from, to);
                }
            }
            Some(Gesture::Stroke { .. }) | None => self.gesture = None,
        }
        self.finish(editor);
    }

    /// Ends the pending edit as one undo step.
    fn finish(&mut self, editor: &mut Editor) {
        self.pending = (0, 0);
        editor.commit();
    }

    /// Calls off the gesture and whatever it changed.
    pub fn cancel(&mut self, editor: &mut Editor) {
        self.gesture = None;
        editor.cancel();
        let pending = std::mem::take(&mut self.pending);
        self.back = (self.back.0 + pending.0, self.back.1 + pending.1);
        if let Some(block) = &mut self.selection {
            block.x -= pending.0.min(block.x);
            block.y -= pending.1.min(block.y);
        }
    }

    /// Lets go of the selection.
    pub fn deselect(&mut self) {
        self.selection = None;
    }

    /// The level's inside, within the block from `a` to `b`.
    fn band(&self, editor: &Editor, a: Cell, b: Cell) -> Option<Block> {
        let level = editor.level();
        let left = a.0.min(b.0).max(1);
        let top = a.1.min(b.1).max(1);
        let right = a.0.max(b.0).min(level.width as isize - 2);
        let bottom = a.1.max(b.1).min(level.height as isize - 2);
        (left <= right && top <= bottom).then_some(Block {
            x: left as usize,
            y: top as usize,
            width: (right - left + 1) as usize,
            height: (bottom - top + 1) as usize,
        })
    }

    /// A piece dragged from the palette was dropped on `cell`.
    pub fn drop_piece(&mut self, editor: &mut Editor, cell: Cell, brush: Brush) {
        self.put_new(editor, cell, brush);
        self.finish(editor);
    }

    /// Puts `clip` with its corner at `at` and selects it, growing the level
    /// to hold it. Leaves the edit pending.
    fn drop_clip(&mut self, editor: &mut Editor, at: Cell, clip: &Clip) -> bool {
        let Some(moved) = editor.fit(at.0, at.1, clip.width, clip.height) else {
            return false;
        };
        self.moved(moved);
        let (x, y) = (
            (at.0 + moved.0 as isize) as usize,
            (at.1 + moved.1 as isize) as usize,
        );
        editor.paste(x, y, clip);
        editor.split_signs();
        editor.tuck_settings();
        self.selection = Some(Block {
            x,
            y,
            width: clip.width,
            height: clip.height,
        });
        true
    }

    /// Keeps the selection for [`Canvas::paste`].
    pub fn copy(&mut self, editor: &Editor) -> bool {
        let Some(block) = self.selection else {
            return false;
        };
        self.clip = Some(editor.copy(block.x, block.y, block.width, block.height));
        true
    }

    /// Clears the selection.
    pub fn delete(&mut self, editor: &mut Editor) {
        if let Some(block) = self.selection.take() {
            editor.clear(block.x, block.y, block.width, block.height);
            self.finish(editor);
        }
    }

    /// [`Canvas::copy`], then [`Canvas::delete`].
    pub fn cut(&mut self, editor: &mut Editor) {
        if self.copy(editor) {
            self.delete(editor);
        }
    }

    /// Puts what was copied with its corner at `cell`. Returns whether it fit.
    pub fn paste(&mut self, editor: &mut Editor, cell: Cell) -> bool {
        let Some(clip) = self.clip.clone() else {
            return false;
        };
        let done = self.drop_clip(editor, cell, &clip);
        self.finish(editor);
        done
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MIN_SIZE;

    const WATER: Brush = Brush {
        layer: Layer::Tile,
        code: 300,
    };
    const COIN: Brush = Brush {
        layer: Layer::Object,
        code: 5,
    };
    const WALL: Brush = Brush {
        layer: Layer::Tile,
        code: 200,
    };

    fn painting(brush: Brush) -> (Editor, Canvas) {
        let canvas = Canvas {
            tool: Tool::Brush(brush),
            ..Canvas::default()
        };
        (Editor::new(), canvas)
    }

    #[test]
    fn a_fast_stroke_leaves_no_gaps_and_undoes_as_one() {
        let (mut editor, mut canvas) = painting(WATER);
        canvas.press(&mut editor, (2, 2), false, false);
        canvas.hover(&mut editor, (7, 4));
        canvas.release(&mut editor);
        for x in 2..=7 {
            assert!(
                (2..=4).any(|y| editor.level().tile(x, y) == 300),
                "column {x}"
            );
        }
        assert!(editor.undo());
        assert_eq!(editor.level().tile(2, 2), FLOOR);
        assert!(!editor.undo());
    }

    #[test]
    fn the_other_button_erases_whatever_the_tool() {
        let (mut editor, mut canvas) = painting(WATER);
        editor.place(3, 3, Layer::Object, COIN.code);
        editor.commit();
        canvas.press(&mut editor, (3, 3), true, false);
        canvas.release(&mut editor);
        assert_eq!(editor.level().object(3, 3), 0);
        assert_eq!(editor.level().tile(3, 3), FLOOR);
    }

    #[test]
    fn erasing_leaves_the_outer_wall_alone() {
        let (mut editor, mut canvas) = painting(WATER);
        canvas.press(&mut editor, (0, 3), true, false);
        canvas.hover(&mut editor, (-3, 3));
        canvas.release(&mut editor);
        assert_eq!(editor.level().tile(0, 3), 200);
        assert_eq!(editor.level().width, MIN_SIZE);
    }

    #[test]
    fn painting_past_the_edge_grows_the_level_and_keeps_the_stroke_on_track() {
        let (mut editor, mut canvas) = painting(WATER);
        canvas.press(&mut editor, (2, 5), false, false);
        canvas.hover(&mut editor, (-1, 5));
        canvas.release(&mut editor);
        let level = editor.level();
        assert_eq!(level.width, MIN_SIZE + 2);
        assert_eq!(canvas.take_shift(), (2, 0));
        assert_eq!(canvas.take_shift(), (0, 0));
        for x in 1..=4 {
            assert_eq!(level.tile(x, 5), 300, "cell {x}");
        }
        assert_eq!(level.tile(0, 5), 200);
    }

    #[test]
    fn a_called_off_stroke_puts_the_size_and_the_picture_back() {
        let (mut editor, mut canvas) = painting(WATER);
        canvas.press(&mut editor, (2, 5), false, false);
        canvas.hover(&mut editor, (-1, 5));
        assert_eq!(canvas.take_shift(), (2, 0));
        canvas.cancel(&mut editor);
        assert_eq!(canvas.take_shift(), (-2, 0));
        assert_eq!(*editor.level(), *Editor::new().level());
        assert!(canvas.gesture().is_none());
    }

    #[test]
    fn wall_on_the_outer_wall_does_not_grow_the_level() {
        let (mut editor, mut canvas) = painting(WALL);
        canvas.press(&mut editor, (0, 5), false, false);
        canvas.release(&mut editor);
        assert_eq!(editor.level().width, MIN_SIZE);
    }

    #[test]
    fn dropping_from_the_palette_is_one_step() {
        let (mut editor, mut canvas) = (Editor::new(), Canvas::default());
        canvas.drop_piece(&mut editor, (4, 4), COIN);
        assert_eq!(editor.level().object(4, 4), COIN.code);
        assert!(editor.undo());
        assert_eq!(editor.level().object(4, 4), 0);
    }

    #[test]
    fn a_placed_piece_drags_to_a_new_cell() {
        let (mut editor, mut canvas) = (Editor::new(), Canvas::default());
        canvas.drop_piece(&mut editor, (4, 4), COIN);
        canvas.press(&mut editor, (4, 4), false, false);
        canvas.hover(&mut editor, (6, 7));
        assert_eq!(editor.level().object(4, 4), COIN.code, "moves on release");
        canvas.release(&mut editor);
        assert_eq!(editor.level().object(4, 4), 0);
        assert_eq!(editor.level().object(6, 7), COIN.code);
        assert!(editor.undo());
        assert_eq!(editor.level().object(4, 4), COIN.code);
    }

    #[test]
    fn a_dragged_tile_leaves_floor() {
        let (mut editor, mut canvas) = (Editor::new(), Canvas::default());
        canvas.drop_piece(&mut editor, (4, 4), WATER);
        canvas.press(&mut editor, (4, 4), false, false);
        canvas.hover(&mut editor, (5, 4));
        canvas.release(&mut editor);
        assert_eq!(editor.level().tile(4, 4), FLOOR);
        assert_eq!(editor.level().tile(5, 4), 300);
    }

    #[test]
    fn a_click_on_a_piece_selects_it() {
        let (mut editor, mut canvas) = (Editor::new(), Canvas::default());
        canvas.drop_piece(&mut editor, (4, 4), COIN);
        canvas.press(&mut editor, (4, 4), false, false);
        canvas.release(&mut editor);
        let block = canvas.selection().unwrap();
        assert_eq!((block.x, block.y, block.width), (4, 4, 1));
    }

    #[test]
    fn a_band_stays_inside_the_outer_wall() {
        let (mut editor, mut canvas) = (Editor::new(), Canvas::default());
        canvas.press(&mut editor, (-5, 3), false, false);
        canvas.hover(&mut editor, (4, 60));
        canvas.release(&mut editor);
        assert_eq!(
            canvas.selection(),
            Some(Block {
                x: 1,
                y: 3,
                width: 4,
                height: 10
            })
        );
    }

    #[test]
    fn a_band_on_the_outer_wall_selects_nothing() {
        let (mut editor, mut canvas) = (Editor::new(), Canvas::default());
        canvas.press(&mut editor, (0, 0), false, false);
        canvas.release(&mut editor);
        assert_eq!(canvas.selection(), None);
    }

    #[test]
    fn the_selection_drags_as_a_block() {
        let (mut editor, mut canvas) = (Editor::new(), Canvas::default());
        canvas.drop_piece(&mut editor, (2, 2), COIN);
        canvas.drop_piece(&mut editor, (3, 3), WATER);
        canvas.press(&mut editor, (2, 2), false, true);
        canvas.hover(&mut editor, (3, 3));
        canvas.release(&mut editor);
        canvas.press(&mut editor, (3, 3), false, false);
        canvas.hover(&mut editor, (6, 5));
        canvas.release(&mut editor);
        let level = editor.level();
        assert_eq!(level.object(5, 4), COIN.code);
        assert_eq!(level.tile(6, 5), 300);
        assert_eq!(level.object(2, 2), 0);
        assert_eq!(level.tile(3, 3), FLOOR);
        assert_eq!(canvas.selection().map(|b| (b.x, b.y)), Some((5, 4)));
        assert!(editor.undo());
        assert_eq!(editor.level().object(2, 2), COIN.code);
    }

    #[test]
    fn dragging_a_block_off_the_level_grows_it_and_moves_the_selection() {
        let (mut editor, mut canvas) = (Editor::new(), Canvas::default());
        canvas.drop_piece(&mut editor, (2, 2), COIN);
        canvas.press(&mut editor, (2, 2), false, true);
        canvas.release(&mut editor);
        canvas.press(&mut editor, (2, 2), false, false);
        canvas.hover(&mut editor, (-2, 2));
        canvas.release(&mut editor);
        assert_eq!(canvas.take_shift(), (3, 0));
        assert_eq!(editor.level().object(1, 2), COIN.code);
        assert_eq!(canvas.selection().map(|b| b.x), Some(1));
    }

    #[test]
    fn copy_paste_cut_and_delete() {
        let (mut editor, mut canvas) = (Editor::new(), Canvas::default());
        assert!(!canvas.paste(&mut editor, (1, 1)));
        canvas.drop_piece(&mut editor, (2, 2), COIN);
        canvas.press(&mut editor, (2, 2), false, true);
        canvas.release(&mut editor);
        assert!(canvas.copy(&editor));
        assert!(canvas.paste(&mut editor, (5, 5)));
        assert_eq!(editor.level().object(5, 5), COIN.code);
        assert_eq!(canvas.selection().map(|b| (b.x, b.y)), Some((5, 5)));
        canvas.cut(&mut editor);
        assert_eq!(editor.level().object(5, 5), 0);
        assert!(canvas.has_clip());
        assert_eq!(canvas.selection(), None);
        // Past the edge, the level grows to hold it.
        assert!(canvas.paste(&mut editor, (13, 5)));
        assert_eq!(editor.level().width, MIN_SIZE + 1);
        assert_eq!(editor.level().object(13, 5), COIN.code);
    }

    #[test]
    fn a_painted_sign_takes_the_next_slot_and_a_moved_one_keeps_its_own() {
        const SIGN: Brush = Brush {
            layer: Layer::Tile,
            code: 1300,
        };
        let (mut editor, mut canvas) = painting(SIGN);
        canvas.press(&mut editor, (2, 2), false, false);
        canvas.hover(&mut editor, (3, 2));
        canvas.release(&mut editor);
        assert_eq!(editor.level().tile(2, 2), 1300);
        assert_eq!(editor.level().tile(3, 2), 1301);
        // Going over a sign leaves it.
        canvas.press(&mut editor, (3, 2), false, false);
        canvas.release(&mut editor);
        assert_eq!(editor.level().tile(3, 2), 1301);

        canvas.tool = Tool::Select;
        canvas.press(&mut editor, (3, 2), false, false);
        canvas.hover(&mut editor, (5, 5));
        canvas.release(&mut editor);
        assert_eq!(editor.level().tile(5, 5), 1301);
    }

    #[test]
    fn a_pasted_sign_gets_a_slot_and_the_words() {
        let mut canvas = Canvas::default();
        let mut editor = Editor::new();
        editor.place(2, 2, Layer::Tile, 1300);
        editor.set_sign_text(0, "HI");
        editor.commit();
        canvas.press(&mut editor, (2, 2), false, true);
        canvas.release(&mut editor);
        canvas.copy(&editor);
        canvas.paste(&mut editor, (6, 6));
        assert_eq!(editor.level().tile(6, 6), 1301);
        assert_eq!(editor.sign_text(1), "HI");
    }

    #[test]
    fn drawing_a_link_connects_two_pieces_as_one_step() {
        let mut canvas = Canvas {
            tool: Tool::Link,
            ..Canvas::default()
        };
        let mut editor = Editor::new();
        editor.place(2, 2, Layer::Tile, 1100);
        editor.place(5, 5, Layer::Tile, 1002);
        editor.commit();
        canvas.press(&mut editor, (2, 2), false, false);
        canvas.hover(&mut editor, (5, 5));
        canvas.release(&mut editor);
        assert_eq!(editor.level().tile(2, 2), 1106);
        assert!(editor.undo());
        assert_eq!(editor.level().tile(2, 2), 1100);
    }

    #[test]
    fn turning_uses_the_direction_or_else_the_first_choice() {
        let cannon = Brush {
            layer: Layer::Tile,
            code: 800,
        };
        assert_eq!(turned(cannon).code, 804);
        let spikes = Brush {
            layer: Layer::Tile,
            code: 1400,
        };
        assert_eq!(turned(spikes).code, 1401);
        assert_eq!(turned(WATER), WATER);
        // Wraps round.
        let last = Brush {
            layer: Layer::Tile,
            code: 812,
        };
        assert_eq!(turned(last).code, 800);
    }

    #[test]
    fn a_piece_too_far_off_a_full_level_is_refused() {
        let (mut editor, mut canvas) = painting(WATER);
        while editor.grow(crate::Side::Right) {}
        editor.commit();
        let width = editor.level().width as isize;
        canvas.press(&mut editor, (width, 3), false, false);
        canvas.release(&mut editor);
        assert_eq!(editor.level().width, width as usize);
        assert_eq!(editor.level().tile(width as usize - 1, 3), 200);
    }
}
