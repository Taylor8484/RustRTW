//! RTWP's worlds and their levels (`WorldLevelNames` in `RTWP/globals.bb`),
//! kept in `data/rtwp.ron`, and the rules the level selector
//! (`StartMenu(4)`, `MenuLoop4`) applies to them.
//!
//! World `w` holds levels `w*10` to `w*10+9`; a level's number is
//! `CurrentLevel` and indexes the profile's scores. Worlds come in packs
//! of ten (`WhichWorldPack`): 0 classic, 1 tutorial, 2 deluxe, 3 kids.
//! Levels 400 on are the playground (`PlaygroundLevel`), which no world
//! pack reaches.

use serde::Deserialize;

/// `PlaygroundLevel`: the first level of the playground.
pub const PLAYGROUND_LEVEL: i32 = 400;

/// Levels in the table (`Restore WorldLevelNames` reads 500).
pub const LEVELS: usize = 500;

/// Levels in a world.
pub const WORLD_SIZE: usize = 10;

/// `WorldName$` holds worlds 0 to 40.
const NAMED_WORLDS: usize = 41;

/// The data, embedded so the game needs no file beside it.
const RTWP_RON: &str = include_str!("../../../data/rtwp.ron");

/// One world: its name and level files (`""` for an empty slot).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct World {
    pub name: String,
    pub levels: Vec<String>,
}

#[derive(Deserialize)]
struct File {
    worlds: Vec<World>,
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

    /// Reads a world table in the `rtwp.ron` layout.
    pub fn parse(text: &str) -> Result<Self, String> {
        let file: File = ron::from_str(text).map_err(|e| e.to_string())?;
        if file.worlds.len() * WORLD_SIZE > LEVELS {
            return Err(format!("{} worlds", file.worlds.len()));
        }
        if let Some((i, _)) = (file.worlds.iter().enumerate()).find(|(_, w)| w.levels.len() > 10) {
            return Err(format!("world {i} has more than {WORLD_SIZE} levels"));
        }
        Ok(Self {
            worlds: file.worlds,
        })
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

    /// `WorldName$(world, language)`. Only the English names are read in,
    /// so other languages get an empty line.
    pub fn name(&self, world: i32, language: i32) -> &str {
        match self.world(world) {
            Some(w) if language == 0 && (world as usize) < NAMED_WORLDS => &w.name,
            _ => "",
        }
    }

    /// The first level whose file is `name` (any folder, case or
    /// extension).
    pub fn index_of(&self, name: &str) -> Option<i32> {
        let want = stem(name);
        (0..LEVELS as i32).find(|&l| self.level_file(l).is_some_and(|f| stem(f) == want))
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
    /// open anyway up to world `9 - pack` of the pack.
    pub fn gate_open(&self, scores: &[i32], pack: i32, world: i32, lock_worlds: i32) -> bool {
        let unwon = (pack * 10..=world).any(|j| {
            (0..WORLD_SIZE as i32).any(|i| {
                let l = j * 10 + i;
                self.level_file(l).is_some() && score(scores, l) == 0
            })
        });
        !unwon || (world % 10 < 9 - pack && lock_worlds == 1)
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
        assert_eq!(w.index_of("levels\\TUT11.lv5"), Some(110));
        // The first world that lists it.
        assert_eq!(w.index_of("intro"), Some(0));
        assert_eq!(w.files().filter(|f| *f == "intro.lev").count(), 5);
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
