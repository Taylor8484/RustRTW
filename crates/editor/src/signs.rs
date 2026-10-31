//! Sign texts. A sign tile holds the number of its text, of which a level has
//! 20; here slots are handed out as signs are placed and nobody sees them.

use wonderland_formats::catalogue::Layer;

use crate::Editor;

pub(crate) const SIGN: i32 = 1300;
/// `SignText$`.
pub const MAX_SIGNS: usize = 20;
/// `EditSignLoop`: four lines, stored with `#` between them.
const MAX_LINES: usize = 4;

/// The stored text as lines of a text box.
pub fn sign_lines(text: &str) -> String {
    text.replace('#', "\n")
}

/// Lines of a text box as stored.
pub fn stored_sign(lines: &str) -> String {
    let lines: Vec<String> = lines
        .split('\n')
        .take(MAX_LINES)
        .map(|l| l.replace('#', ""))
        .collect();
    lines.join("#")
}

pub(crate) fn is_sign(code: i32) -> bool {
    (SIGN..SIGN + MAX_SIGNS as i32).contains(&code)
}

impl Editor {
    /// A slot no sign in the level uses.
    pub fn free_sign(&self) -> Option<usize> {
        let used = |slot: &usize| self.level.tiles.contains(&(SIGN + *slot as i32));
        (0..MAX_SIGNS).find(|slot| !used(slot))
    }

    /// The tile for a new sign, with its old text gone, if a slot is free.
    pub(crate) fn new_sign(&mut self) -> Option<i32> {
        let slot = self.free_sign()?;
        self.set_sign_text(slot, "");
        Some(SIGN + slot as i32)
    }

    pub fn sign_text(&self, slot: usize) -> &str {
        self.level.signs.get(slot).map_or("", String::as_str)
    }

    /// Changes a sign's text, joining the pending edit.
    pub fn set_sign_text(&mut self, slot: usize, text: &str) {
        if self.sign_text(slot) == text {
            return;
        }
        let signs = &mut self.editing().signs;
        if signs.len() <= slot {
            signs.resize(MAX_SIGNS, String::new());
        }
        signs[slot] = text.to_owned();
    }

    /// Gives every copy of a sign a slot of its own and the same text, so a
    /// pasted sign keeps its words. A copy with no slot left goes back to
    /// floor.
    pub(crate) fn split_signs(&mut self) {
        let mut seen = vec![];
        for index in 0..self.level.tiles.len() {
            let code = self.level.tiles[index];
            if !is_sign(code) {
                continue;
            }
            if !seen.contains(&code) {
                seen.push(code);
                continue;
            }
            let (x, y) = (index % self.level.width, index / self.level.width);
            let text = self.sign_text((code - SIGN) as usize).to_owned();
            match self.free_sign() {
                Some(slot) => {
                    let tile = SIGN + slot as i32;
                    self.set_sign_text(slot, &text);
                    self.place(x, y, Layer::Tile, tile);
                    seen.push(tile);
                }
                None => self.place(x, y, Layer::Tile, crate::FLOOR),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_are_handed_out_in_turn() {
        let mut editor = Editor::new();
        let first = editor.new_sign().unwrap();
        editor.place(1, 1, Layer::Tile, first);
        assert_eq!(first, 1300);
        assert_eq!(editor.new_sign(), Some(1301));
    }

    #[test]
    fn the_twenty_first_sign_has_no_slot() {
        let mut editor = Editor::new();
        for slot in 0..MAX_SIGNS {
            editor.place(
                1 + slot % 12,
                1 + slot / 12,
                Layer::Tile,
                SIGN + slot as i32,
            );
        }
        assert_eq!(editor.new_sign(), None);
    }

    #[test]
    fn a_reused_slot_starts_empty() {
        let mut editor = Editor::new();
        editor.set_sign_text(0, "OLD");
        assert_eq!(editor.new_sign(), Some(1300));
        assert_eq!(editor.sign_text(0), "");
    }

    #[test]
    fn text_goes_to_four_lines_with_hashes() {
        assert_eq!(stored_sign("a\nb#c"), "a#bc");
        assert_eq!(stored_sign("1\n2\n3\n4\n5"), "1#2#3#4");
        assert_eq!(sign_lines("a#b"), "a\nb");
        assert_eq!(stored_sign(&sign_lines("a#")), "a#");
    }

    #[test]
    fn copies_of_a_sign_keep_the_words_in_slots_of_their_own() {
        let mut editor = Editor::new();
        editor.place(1, 1, Layer::Tile, SIGN);
        editor.set_sign_text(0, "HELLO");
        editor.place(2, 1, Layer::Tile, SIGN);
        editor.split_signs();
        assert_eq!(editor.level().tile(1, 1), SIGN);
        assert_eq!(editor.level().tile(2, 1), SIGN + 1);
        assert_eq!(editor.sign_text(1), "HELLO");
    }
}
