//! The worlds and their levels, and the rules the level selector
//! (`StartMenu(4)`, `MenuLoop4`) applies to them.
//!
//! RTWP's table (`WorldLevelNames` in `RTWP/globals.bb`) is kept in
//! `data/rtwp.ron`, TOW's in `data/tow.ron`. World `w` holds levels
//! `w*10` to `w*10+9`; a level's number is `CurrentLevel` and indexes the
//! profile's scores. Worlds come in packs of ten (`WhichWorldPack`): 0
//! classic, 1 tutorial, 2 deluxe, 3 kids. The game's table,
//! [`Worlds::game`], puts TOW's ten worlds in pack 3, as the original
//! never reaches the kids pack, and TOW's playground at level 400
//! (`PlaygroundLevel`), where RTWP lists the same files without shipping
//! them.

use serde::Deserialize;
use wonderland_formats::Game;

/// `PlaygroundLevel`: the first level of the playground.
pub const PLAYGROUND_LEVEL: i32 = 400;

/// Levels in the table (`Restore WorldLevelNames` reads 500).
pub const LEVELS: usize = 500;

/// Levels in a world.
pub const WORLD_SIZE: usize = 10;

/// `WorldName$` holds worlds 0 to 40.
const NAMED_WORLDS: usize = 41;

/// The world pack that plays TOW's worlds.
pub const TOW_PACK: i32 = 3;

/// The data, embedded so the game needs no file beside it.
const RTWP_RON: &str = include_str!("../../../data/rtwp.ron");
const TOW_RON: &str = include_str!("../../../data/tow.ron");

/// Which game's folder a level file is in. Each game's files are read
/// the way that game reads them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Content {
    #[default]
    Rtwp,
    Tow,
}

impl Content {
    pub fn game(self) -> Game {
        match self {
            Content::Rtwp => Game::Rtwp,
            Content::Tow => Game::Tow,
        }
    }
}

/// One world: its name (in English, German and French; RTWP names only
/// English), its level files (`""` for an empty slot) and their game.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct World {
    pub names: Vec<String>,
    pub levels: Vec<String>,
    pub content: Content,
}

#[derive(Deserialize)]
struct RtwpWorld {
    name: String,
    levels: Vec<String>,
}

#[derive(Deserialize)]
struct RtwpFile {
    worlds: Vec<RtwpWorld>,
}

#[derive(Deserialize)]
struct TowWorld {
    names: Vec<String>,
    levels: Vec<String>,
}

#[derive(Deserialize)]
struct TowFile {
    worlds: Vec<TowWorld>,
    playground: Vec<String>,
    ending: String,
}

/// The world table: `WorldName$` and `WorldLevelName$`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Worlds {
    worlds: Vec<World>,
}

impl Worlds {
    /// RTWP's worlds, from `data/rtwp.ron`.
    pub fn rtwp() -> Self {
        Self::parse(RTWP_RON).expect("data/rtwp.ron is checked by the tests")
    }

    /// The game's worlds: RTWP's, with TOW's worlds as pack 3 (worlds 30
    /// to 39) and TOW's playground and ending stage (worlds 40 and 41).
    pub fn game() -> Self {
        let mut w = Self::rtwp();
        w.add_tow(TOW_RON)
            .expect("data/tow.ron is checked by the tests");
        w
    }

    /// Reads a world table in the `rtwp.ron` layout.
    pub fn parse(text: &str) -> Result<Self, String> {
        let file: RtwpFile = ron::from_str(text).map_err(|e| e.to_string())?;
        let worlds = file
            .worlds
            .into_iter()
            .map(|w| World {
                names: vec![w.name],
                levels: w.levels,
                content: Content::Rtwp,
            })
            .collect();
        Self::check(worlds)
    }

    fn check(worlds: Vec<World>) -> Result<Self, String> {
        if worlds.len() * WORLD_SIZE > LEVELS {
            return Err(format!("{} worlds", worlds.len()));
        }
        if let Some((i, _)) = (worlds.iter().enumerate()).find(|(_, w)| w.levels.len() > 10) {
            return Err(format!("world {i} has more than {WORLD_SIZE} levels"));
        }
        Ok(Self { worlds })
    }

    /// Puts a table in the `tow.ron` layout in pack [`TOW_PACK`] and the
    /// playground.
    fn add_tow(&mut self, text: &str) -> Result<(), String> {
        let file: TowFile = ron::from_str(text).map_err(|e| e.to_string())?;
        if file.worlds.len() > WORLD_SIZE {
            return Err(format!("{} TOW worlds", file.worlds.len()));
        }
        let first = (TOW_PACK * 10) as usize;
        let playground = PLAYGROUND_LEVEL as usize / WORLD_SIZE;
        let mut worlds = std::mem::take(&mut self.worlds);
        worlds.resize_with(worlds.len().max(playground + 2), World::default);
        let tow = |names, levels| World {
            names,
            levels,
            content: Content::Tow,
        };
        for i in 0..WORLD_SIZE {
            worlds[first + i] = match file.worlds.get(i) {
                Some(w) => tow(w.names.clone(), w.levels.clone()),
                None => tow(Vec::new(), Vec::new()),
            };
        }
        worlds[playground] = tow(Vec::new(), file.playground);
        worlds[playground + 1] = tow(Vec::new(), vec![file.ending]);
        *self = Self::check(worlds)?;
        Ok(())
    }

    /// The game whose folder holds `level`'s file.
    pub fn content(&self, level: i32) -> Content {
        usize::try_from(level)
            .ok()
            .and_then(|l| self.worlds.get(l / WORLD_SIZE))
            .map_or_else(Content::default, |w| w.content)
    }

    fn world(&self, world: i32) -> Option<&World> {
        self.worlds.get(usize::try_from(world).ok()?)
    }

    /// `WorldLevelName$(level)`, or `None` for an empty slot.
    pub fn level_file(&self, level: i32) -> Option<&str> {
        let l = usize::try_from(level).ok()?;
        let name = self
            .worlds
            .get(l / WORLD_SIZE)?
            .levels
            .get(l % WORLD_SIZE)?;
        (!name.is_empty()).then_some(name.as_str())
    }

    /// Where `StartLevel` loads `level` from: `levels\<file>`, except
    /// `res10.dat`, which sits beside the levels folder.
    pub fn level_path(&self, level: i32) -> Option<String> {
        let file = self.level_file(level)?;
        Some(if file == "res10.dat" {
            file.to_owned()
        } else {
            format!("levels\\{file}")
        })
    }

    /// `CurrentWorldLevelN`: the levels before the world's first empty
    /// slot.
    pub fn level_count(&self, world: i32) -> i32 {
        (0..WORLD_SIZE as i32)
            .take_while(|&i| self.level_file(world * 10 + i).is_some())
            .count() as i32
    }

    /// `WorldName$(world, language)`. RTWP reads only the English names,
    /// so its worlds show an empty line in other languages; TOW has all
    /// three.
    pub fn name(&self, world: i32, language: i32) -> &str {
        match (self.world(world), usize::try_from(language)) {
            (Some(w), Ok(l)) if (world as usize) < NAMED_WORLDS => {
                w.names.get(l).map_or("", String::as_str)
            }
            _ => "",
        }
    }

    /// The first level in `content`'s folder whose file is `name` (any
    /// folder, case or extension).
    pub fn index_of(&self, content: Content, name: &str) -> Option<i32> {
        let want = stem(name);
        (0..LEVELS as i32).find(|&l| {
            self.content(l) == content && self.level_file(l).is_some_and(|f| stem(f) == want)
        })
    }

    /// Every level file in the table, in order (with repeats).
    pub fn files(&self) -> impl Iterator<Item = &str> {
        self.worlds
            .iter()
            .flat_map(|w| w.levels.iter())
            .filter(|f| !f.is_empty())
            .map(String::as_str)
    }

    /// The forward gate on world `world`'s level selector is open
    /// (`NextWorldFlag=0`): every level from the pack's first world to
    /// this one has a score. With `LockWorlds` (all worlds unlocked) it is
    /// open anyway up to world `9 - pack` of the pack, or up to world 9 of
    /// TOW's (`CurrentWorld<9` in TOW).
    pub fn gate_open(&self, scores: &[i32], pack: i32, world: i32, lock_worlds: i32) -> bool {
        let unwon = (pack * 10..=world).any(|j| {
            (0..WORLD_SIZE as i32).any(|i| {
                let l = j * 10 + i;
                self.level_file(l).is_some() && score(scores, l) == 0
            })
        });
        let last = if pack == TOW_PACK { 9 } else { 9 - pack };
        !unwon || (world % 10 < last && lock_worlds == 1)
    }
}

fn stem(name: &str) -> String {
    let file = name.rsplit(['\\', '/']).next().unwrap_or(name);
    file.rsplit_once('.')
        .map_or(file, |(s, _)| s)
        .to_lowercase()
}

/// `CurrentPlayerLevelScore(level)`, 0 outside the table.
pub fn score(scores: &[i32], level: i32) -> i32 {
    usize::try_from(level)
        .ok()
        .and_then(|l| scores.get(l))
        .copied()
        .unwrap_or(0)
}

/// `WorldAdjust`: the first world of `world`'s pack (0, 10, 20 or 30).
pub fn pack_start(world: i32) -> i32 {
    match world {
        ..10 => 0,
        10..20 => 10,
        20..30 => 20,
        _ => 30,
    }
}

/// The selector's "TOTAL SCORE": levels `adjust*10` to `adjust*10+99`,
/// where `adjust` is [`pack_start`].
pub fn total_score(scores: &[i32], adjust: i32) -> i32 {
    (adjust * 10..=adjust * 10 + 99)
        .map(|l| score(scores, l))
        .filter(|&s| s > 0)
        .sum()
}

/// A level title split into lines at `#`, as `StartMenu(4)` does: a `#`
/// that is the last character left is not a split.
pub fn title_lines(title: &str) -> Vec<String> {
    let mut ex: Vec<char> = title.chars().collect();
    let mut lines = Vec::new();
    let mut j = 1;
    while j < ex.len() {
        if ex[j - 1] == '#' {
            lines.push(ex[..j - 1].iter().collect());
            ex.drain(..j);
            j = 0;
        }
        j += 1;
    }
    lines.push(ex.into_iter().collect());
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtwp_table_matches_world_level_names() {
        let w = Worlds::rtwp();
        assert_eq!(w.level_file(0), Some("intro.lev"));
        assert_eq!(w.level_file(8), None);
        assert_eq!(w.level_file(99), Some("hurrah.lev"));
        assert_eq!(w.level_file(PLAYGROUND_LEVEL + 9), Some("bonus5.lev"));
        assert_eq!(w.level_path(80).as_deref(), Some("res10.dat"));
        assert_eq!(w.level_path(1).as_deref(), Some("levels\\reflectors.lev"));
        assert_eq!(w.level_count(0), 8);
        assert_eq!(w.level_count(1), 10);
        assert_eq!(w.level_count(13), 0);
        assert_eq!(w.level_count(99), 0);
        assert_eq!(w.name(5, 0), "ATLANTIS");
        assert_eq!(w.name(5, 1), "");
        assert_eq!(w.name(40, 0), "");
        assert_eq!(w.index_of(Content::Rtwp, "levels\\TUT11.lv5"), Some(110));
        // The first world that lists it.
        assert_eq!(w.index_of(Content::Rtwp, "intro"), Some(0));
        assert_eq!(w.files().filter(|f| *f == "intro.lev").count(), 5);
    }

    #[test]
    fn tow_worlds_take_the_kids_pack() {
        let w = Worlds::game();
        assert_eq!(w.level_file(300), Some("tut1.lev"));
        assert_eq!(w.level_file(305), None);
        assert_eq!(w.level_file(399), Some("lastone.lev"));
        assert_eq!(w.level_path(310).as_deref(), Some("levels\\castleb.lev"));
        assert_eq!(w.content(310), Content::Tow);
        assert_eq!(w.content(110), Content::Rtwp);
        assert_eq!(w.content(-999), Content::Rtwp);
        assert_eq!(w.index_of(Content::Tow, "tut1"), Some(300));
        assert_eq!(w.index_of(Content::Rtwp, "tut1"), Some(100));
        let counts: Vec<i32> = (30..40).map(|i| w.level_count(i)).collect();
        assert_eq!(counts, [5, 8, 10, 9, 5, 10, 8, 5, 10, 10]);
        assert_eq!(counts.iter().sum::<i32>(), 80);
        assert_eq!(w.name(32, 0), "WINTER IN WONDERLAND");
        assert_eq!(w.name(32, 1), "WINTER IM WONDERLAND");
        assert_eq!(w.name(39, 2), "LE TEMPLE DE JADE");
        assert_eq!(w.name(5, 1), "");
        assert_eq!(w.name(40, 0), "");
        assert_eq!(w.level_file(PLAYGROUND_LEVEL + 5), Some("bonus1.lev"));
        assert_eq!(w.content(PLAYGROUND_LEVEL + 10), Content::Tow);
        assert_eq!(w.level_file(PLAYGROUND_LEVEL + 10), Some("theend.lev"));
        // RTWP's own worlds are unchanged.
        let rtwp = Worlds::rtwp();
        assert!((0..300).all(|l| w.level_file(l) == rtwp.level_file(l)));
    }

    #[test]
    fn tow_gates_unlock_up_to_its_last_world() {
        let w = Worlds::game();
        let scores = vec![0; LEVELS];
        assert!(!w.gate_open(&scores, TOW_PACK, 30, 0));
        assert!(w.gate_open(&scores, TOW_PACK, 38, 1));
        assert!(!w.gate_open(&scores, TOW_PACK, 39, 1));
        let mut scores = scores;
        scores[300..305].fill(1);
        assert!(w.gate_open(&scores, TOW_PACK, 30, 0));
        // Pack 3 counts from world 30: RTWP's packs do not hold it back.
        assert!(!w.gate_open(&scores, TOW_PACK, 31, 0));
    }

    #[test]
    fn the_gate_opens_when_the_pack_so_far_is_won() {
        let w = Worlds::rtwp();
        let mut scores = vec![0; LEVELS];
        assert!(!w.gate_open(&scores, 0, 0, 0));
        // All worlds unlocked: up to world 8 of the classic pack.
        assert!(w.gate_open(&scores, 0, 8, 1));
        assert!(!w.gate_open(&scores, 0, 9, 1));
        assert!(w.gate_open(&scores, 1, 17, 1));
        assert!(!w.gate_open(&scores, 1, 18, 1));
        for s in &mut scores[..8] {
            *s = 1;
        }
        assert!(w.gate_open(&scores, 0, 0, 0));
        // Every world from the pack's first counts.
        assert!(!w.gate_open(&scores, 0, 1, 0));
        scores[1] = 0;
        assert!(!w.gate_open(&scores, 0, 0, 0));
        // Tutorial pack: worlds before 10 do not count.
        for s in &mut scores[100..110] {
            *s = 5;
        }
        assert!(w.gate_open(&scores, 1, 10, 0));
    }

    #[test]
    fn total_score_counts_the_whole_pack() {
        let mut scores = vec![0; LEVELS];
        scores[3] = 10;
        scores[99] = 5;
        scores[100] = 7;
        scores[50] = -3;
        assert_eq!(total_score(&scores, pack_start(4)), 15);
        assert_eq!(total_score(&scores, pack_start(11)), 7);
    }

    #[test]
    fn titles_split_at_hashes() {
        assert_eq!(title_lines("ONE"), ["ONE"]);
        assert_eq!(title_lines("ONE#TWO#THREE"), ["ONE", "TWO", "THREE"]);
        assert_eq!(title_lines("#A"), ["", "A"]);
        // The last character is never a split.
        assert_eq!(title_lines("A#"), ["A#"]);
        assert_eq!(title_lines("A##"), ["A", "#"]);
        assert_eq!(title_lines(""), [""]);
    }
}
