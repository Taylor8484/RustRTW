//! The in-game menu (`MenuLoop5`): Esc during play stops the level and
//! lists what to do next, over the frozen level. Saving and loading the
//! game state are commented out in the original.
//!
//! This module has no Bevy state, so it is tested headless; `flow` opens
//! it and carries out the [`Choice`]. The original waits for Enter or Esc
//! to be let go before it acts, so a key still held cannot reach the game
//! or the next menu.

use crate::menu::{MenuKeys, Text};

/// What the player chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// Every player dies, and the level restarts as after any death.
    Restart,
    /// Back to the level selector, or the custom levels.
    SelectLevel,
    /// Back to the main menu.
    Exit,
    Resume,
}

/// The options in order, by `CurrentMenuOption`.
const CHOICES: [Choice; 4] = [
    Choice::Restart,
    Choice::SelectLevel,
    Choice::Exit,
    Choice::Resume,
];

fn lines(language: i32) -> [&'static str; 4] {
    match language {
        1 => [
            "LEVEL NEU STARTEN",
            "NEUES LEVEL",
            "SPIEL BEENDEN",
            "SPIEL FORTSETZEN",
        ],
        2 => [
            "RECOMMENCER CE NIVEAU",
            "CHOISIR UN AUTRE NIVEAU",
            "SORTIE DU JEU",
            "CONTINUER A JOUER",
        ],
        _ => [
            "RESTART THIS LEVEL",
            "SELECT NEW LEVEL",
            "EXIT GAME",
            "RESUME GAME",
        ],
    }
}

/// The open menu.
#[derive(Debug, Clone, Default)]
pub struct GameMenu {
    pub text: Text,
    option: usize,
    /// `CursorKeyPressed`, which the keys that started the level left set.
    cursor_key: bool,
    /// Esc, still held from opening the menu, does not count yet.
    opening: bool,
    /// Waiting for Enter or Esc to be let go, then this.
    chosen: Option<Choice>,
}

impl GameMenu {
    /// `StartMenu(5)`, with Esc still down.
    pub fn new() -> Self {
        Self {
            cursor_key: true,
            opening: true,
            ..Self::default()
        }
    }

    /// One frame of `MenuLoop5`: lays out the text, then reads the keys.
    /// Returns whether the cursor moved, and the choice once made.
    pub fn frame(&mut self, keys: MenuKeys, language: i32) -> (bool, Option<Choice>) {
        for (i, line) in lines(language).into_iter().enumerate() {
            let (row, tag) = (7 + i as i32, 52 + i as i32);
            if i == self.option {
                let line = format!("== {line} ==");
                self.text.center(&line, row as f32, 1.0, tag);
            } else {
                self.text.center(line, row as f32, 1.0, tag);
            }
        }
        self.text.activate(|| None);

        if let Some(choice) = self.chosen {
            return (false, (!keys.enter && !keys.escape).then_some(choice));
        }
        self.opening &= keys.escape;
        let mut moved = false;
        if keys.up || keys.down {
            if !self.cursor_key {
                self.option = (self.option + if keys.up { 3 } else { 1 }) % 4;
                self.cursor_key = true;
                moved = true;
            }
        } else if keys.enter {
            self.chosen = Some(CHOICES[self.option]);
        } else if keys.escape && !self.opening {
            self.chosen = Some(Choice::Resume);
        } else {
            self.cursor_key = false;
        }
        (moved, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(m: &mut GameMenu, keys: MenuKeys) -> (bool, Option<Choice>) {
        m.frame(keys, 0)
    }

    fn text(m: &GameMenu) -> String {
        m.text.shown().iter().map(|l| l.ascii as char).collect()
    }

    #[test]
    fn the_cursor_wraps_over_four_lines() {
        let mut m = GameMenu::new();
        let up = MenuKeys {
            up: true,
            ..MenuKeys::default()
        };
        run(&mut m, MenuKeys::default());
        assert!(text(&m).contains("== RESTART THIS LEVEL =="));
        assert_eq!(run(&mut m, up), (true, None));
        assert_eq!(run(&mut m, up), (false, None));
        run(&mut m, MenuKeys::default());
        run(&mut m, MenuKeys::default());
        assert!(text(&m).contains("== RESUME GAME =="));
    }

    #[test]
    fn enter_acts_when_let_go() {
        let mut m = GameMenu::new();
        run(&mut m, MenuKeys::default());
        let down = MenuKeys {
            down: true,
            ..MenuKeys::default()
        };
        run(&mut m, down);
        run(&mut m, MenuKeys::default());
        let enter = MenuKeys {
            enter: true,
            ..MenuKeys::default()
        };
        assert_eq!(run(&mut m, enter), (false, None));
        assert_eq!(run(&mut m, enter), (false, None));
        assert_eq!(
            run(&mut m, MenuKeys::default()),
            (false, Some(Choice::SelectLevel))
        );
    }

    #[test]
    fn the_esc_that_opened_it_does_not_close_it() {
        let mut m = GameMenu::new();
        let esc = MenuKeys {
            escape: true,
            ..MenuKeys::default()
        };
        assert_eq!(run(&mut m, esc), (false, None));
        assert_eq!(run(&mut m, esc), (false, None));
        assert_eq!(run(&mut m, MenuKeys::default()), (false, None));
        assert_eq!(run(&mut m, esc), (false, None));
        assert_eq!(
            run(&mut m, MenuKeys::default()),
            (false, Some(Choice::Resume))
        );
    }

    #[test]
    fn german_and_french_lines() {
        let mut m = GameMenu::new();
        m.frame(MenuKeys::default(), 1);
        assert!(text(&m).contains("== LEVEL NEU STARTEN =="));
        let mut m = GameMenu::new();
        m.frame(MenuKeys::default(), 2);
        assert!(text(&m).contains("CONTINUER A JOUER"));
    }
}
