//! Stamps: a [`Clip`] kept in a file, to put down in any level. A sign in a
//! stamp brings its words, which a clip between levels would otherwise lose.

use crate::reader::Reader;
use crate::signs::{MAX_SIGNS, is_sign};
use crate::{Clip, Editor, FLOOR, MAX_SIZE};

/// The folder stamps are in, under the game folder. Not in `CustomLevels`,
/// which the game lists.
pub const FOLDER: &str = "EditorStamps";
pub const EXTENSION: &str = "STP";

const MAGIC: &[u8; 8] = b"WLSTP01\n";

impl Clip {
    /// The file.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.extend((self.width as u32).to_le_bytes());
        out.extend((self.height as u32).to_le_bytes());
        for code in self.tiles.iter().chain(&self.objects) {
            out.extend(code.to_le_bytes());
        }
        out.extend((self.signs.len() as u32).to_le_bytes());
        for (code, text) in &self.signs {
            out.extend(code.to_le_bytes());
            out.extend((text.len() as u32).to_le_bytes());
            out.extend(text.as_bytes());
        }
        out
    }

    /// Reads a stamp. `None` if it is damaged.
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes.strip_prefix(MAGIC)?);
        let side = |n: u32| Some(n as usize).filter(|n| (1..=MAX_SIZE).contains(n));
        let width = side(r.u32()?)?;
        let height = side(r.u32()?)?;
        let cells = width * height;
        let grid = |r: &mut Reader| (0..cells).map(|_| r.i32()).collect::<Option<Vec<i32>>>();
        let tiles = grid(&mut r)?;
        let objects = grid(&mut r)?;
        let mut signs = vec![];
        for _ in 0..r.u32()? {
            let code = r.i32()?;
            let length = r.u32()? as usize;
            let text = std::str::from_utf8(r.take(length)?).ok()?;
            signs.push((code, text.to_owned()));
        }
        r.done().then_some(Self {
            width,
            height,
            tiles,
            objects,
            signs,
        })
    }
}

impl Editor {
    /// `clip` with each sign on a slot of its own in this level and its
    /// words there, joining the pending edit. A sign with no slot left
    /// becomes floor.
    pub(crate) fn own_signs(&mut self, clip: &Clip) -> Clip {
        let mut own = clip.clone();
        let mut taken = vec![];
        for tile in own.tiles.iter_mut().filter(|t| is_sign(**t)) {
            let free = |slot: &usize| {
                let code = crate::signs::SIGN + *slot as i32;
                !taken.contains(slot) && !self.level.tiles.contains(&code)
            };
            let Some(slot) = (0..MAX_SIGNS).find(free) else {
                *tile = FLOOR;
                continue;
            };
            taken.push(slot);
            let words = clip.signs.iter().find(|(code, _)| code == tile);
            let text = words.map_or(String::new(), |(_, text)| text.clone());
            *tile = crate::signs::SIGN + slot as i32;
            self.set_sign_text(slot, &text);
        }
        own
    }
}

#[cfg(test)]
mod tests {
    use wonderland_formats::catalogue::Layer;

    use super::*;
    use crate::canvas::{Block, Canvas, Tool};
    use crate::signs::SIGN;

    const COIN: i32 = 5;
    const WATER: i32 = 300;

    /// A coin, water and a sign that says something.
    fn clip() -> Clip {
        let mut editor = Editor::new();
        editor.place(2, 2, Layer::Object, COIN);
        editor.place(3, 2, Layer::Tile, WATER);
        let sign = editor.new_sign().unwrap();
        editor.place(4, 2, Layer::Tile, sign);
        editor.set_sign_text(0, "Mind#the gap");
        editor.commit();
        editor.copy(2, 2, 3, 1)
    }

    #[test]
    fn a_stamp_survives_the_file_and_damage_is_refused() {
        let clip = clip();
        assert_eq!(clip.signs, [(SIGN, "Mind#the gap".to_owned())]);
        let bytes = clip.to_bytes();
        assert_eq!(Clip::parse(&bytes), Some(clip));
        assert_eq!(Clip::parse(&bytes[..bytes.len() - 1]), None);
        assert_eq!(Clip::parse(b"not a stamp"), None);
        let mut huge = MAGIC.to_vec();
        huge.extend(1000u32.to_le_bytes());
        huge.extend(1000u32.to_le_bytes());
        assert_eq!(Clip::parse(&huge), None);
    }

    #[test]
    fn stamping_keeps_the_tool_and_undoes_as_one() {
        let mut editor = Editor::new();
        let mut canvas = Canvas::default();
        canvas.pick_stamp(clip());
        assert_eq!(canvas.tool, Tool::Stamp);
        assert_eq!(canvas.stamp_size(), Some((3, 1)));
        canvas.press(&mut editor, (5, 5), false, false);
        canvas.release(&mut editor);
        assert_eq!(canvas.tool, Tool::Stamp);
        assert_eq!(canvas.selection(), None::<Block>);
        assert_eq!(editor.level().object(5, 5), COIN);
        assert_eq!(editor.level().tile(6, 5), WATER);
        editor.undo();
        assert_eq!(editor.level().tile(6, 5), 100);
    }

    #[test]
    fn a_stamped_sign_gets_a_slot_of_its_own_and_its_words() {
        let mut editor = Editor::new();
        let mut canvas = Canvas::default();
        let first = editor.new_sign().unwrap();
        editor.place(1, 1, Layer::Tile, first);
        editor.set_sign_text(0, "Already here");
        editor.commit();
        canvas.pick_stamp(clip());
        canvas.press(&mut editor, (2, 3), false, false);
        canvas.release(&mut editor);
        canvas.press(&mut editor, (2, 4), false, false);
        canvas.release(&mut editor);
        assert_eq!(editor.level().tile(4, 3), SIGN + 1);
        assert_eq!(editor.level().tile(4, 4), SIGN + 2);
        assert_eq!(editor.sign_text(0), "Already here");
        assert_eq!(editor.sign_text(1), "Mind#the gap");
        assert_eq!(editor.sign_text(2), "Mind#the gap");
    }

    #[test]
    fn a_sign_with_no_slot_left_is_floor() {
        let mut editor = Editor::new();
        for x in 0..MAX_SIGNS {
            let sign = editor.new_sign().unwrap();
            editor.place(1 + x % 10, 1 + x / 10, Layer::Tile, sign);
        }
        editor.commit();
        let own = editor.own_signs(&clip());
        assert!(!own.tiles.iter().any(|&t| is_sign(t)));
        assert!(own.tiles.contains(&FLOOR));
    }

    #[test]
    fn a_stamp_too_big_for_the_level_changes_nothing() {
        let mut editor = Editor::new();
        let mut canvas = Canvas::default();
        let mut huge = clip();
        huge.width = MAX_SIZE;
        huge.height = MAX_SIZE;
        huge.tiles = vec![WATER; MAX_SIZE * MAX_SIZE];
        huge.objects = vec![0; MAX_SIZE * MAX_SIZE];
        canvas.pick_stamp(huge);
        canvas.press(&mut editor, (5, 5), false, false);
        canvas.release(&mut editor);
        assert!(!editor.is_dirty());
    }
}
