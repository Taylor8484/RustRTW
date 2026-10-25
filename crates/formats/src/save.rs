//! Player save files (`players/<name>.pla`, and their `.bak` byte copies).
//!
//! Ported from `SavePlayer`, `LoadPlayer` and `CreateNewPlayer` in each game's
//! `menu.bb`, plus the player listing (which reads only the header). All three
//! games write a magic string, the player name, a few header ints and then
//! progress data, but the layouts differ, so parsing takes the [`Game`]:
//!
//! - TOW: `"Stinky & Loof Player File"`, name, world, levels completed, total
//!   score, then 400 level scores. No player id. The magic is not checked.
//! - RTWP: `"Stinky & Loof Player File v2"`, name, id, world, levels
//!   completed, total score, 400 level scores, the extra (custom) levels as
//!   `(id, id2, score)` triples, 400 perfect flags, one perfect flag per extra
//!   level and 4 world pack positions.
//! - WSW: `"Stinky & Loof WSW Player File"`, name, id, current level, levels
//!   completed, total score, 141 world map entries `(status, score, type)`,
//!   five item counts and 20 story flags.
//!
//! RTWP and WSW refuse a file with the wrong magic (the menu renames it to
//! `.old`), and so does [`PlayerFile::parse`].
//!
//! [`PlayerFile`] holds the values as stored, so `parse` then `to_bytes`
//! reproduces the file. In particular the levels-completed and total-score
//! fields are the stored ones; `SavePlayer` recomputes them on every save and
//! [`PlayerFile::computed_summary`] does the same. Load-time fix-ups belong to
//! profile loading, not here; the one that exists is RTWP's `LoadPlayer`
//! replacing a stored world pack of `0` with `i * 10` for packs 1 to 3.
//!
//! RTWP's `CreateNewPlayer` writes a shorter file than `SavePlayer`: it has no
//! perfect flags (see [`RtwpProgress::perfect`]). `LoadPlayer` on such a file
//! reads the perfect flags from the world pack bytes and the rest past the end
//! of the file, where Blitz returns 0, so the world packs load as 0 (then
//! fixed up as above). That quirk is not modelled; the file is represented as
//! it is stored.

use crate::blitz::{BlitzReader, BlitzWriter};
use crate::{Error, Game, Result};

/// Levels in the main game; the size of the per-level arrays
/// (`PlayGroundLevel`, the same in all three games).
pub const LEVEL_COUNT: usize = 400;

/// WSW world map entries (`For i=0 To 140`).
pub const WSW_MAP_ENTRIES: usize = 141;

/// WSW story flags (`storytold`).
pub const WSW_STORY_FLAGS: usize = 20;

/// A player save file, as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerFile {
    /// The first string, e.g. `Stinky & Loof Player File v2`.
    pub magic: String,
    /// The player name as typed (the file name is lower case).
    pub name: String,
    /// `CurrentPlayerID`. Absent in TOW, whose files have none.
    pub id: Option<i32>,
    /// TOW and RTWP: `CurrentWorld`. WSW: `CurrentLevel`.
    pub position: i32,
    /// Stored count of levels with a score above 0.
    pub levels_completed: i32,
    /// Stored sum of the level scores.
    pub total_score: i32,
    pub progress: Progress,
    /// Bytes after the last value the loader reads. The games never write any,
    /// so this is normally empty; kept so writing round-trips.
    pub trailing: Vec<u8>,
}

/// The game-specific part of a save.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Progress {
    /// TOW: the 400 level scores.
    Tow(Vec<i32>),
    Rtwp(RtwpProgress),
    Wsw(WswProgress),
}

/// RTWP progress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtwpProgress {
    /// `CurrentPlayerLevelScore`, 400 entries.
    pub scores: Vec<i32>,
    /// Custom levels played, in file order.
    pub extra_levels: Vec<ExtraLevel>,
    /// Perfect flags. `None` in the shorter file `CreateNewPlayer` writes,
    /// which stops after the extra level count and the world packs.
    pub perfect: Option<Perfect>,
    /// `CurrentWorldPack(0..=3)`, as stored (see the module docs for the fix-up).
    pub world_packs: [i32; 4],
}

/// A custom level's record (`CurrentPlayerExtraLevelID`, `ID2` and `Score`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraLevel {
    pub id: i32,
    pub id2: i32,
    pub score: i32,
}

/// RTWP perfect flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Perfect {
    /// `CurrentPlayerLevelPerfect`, 400 entries.
    pub levels: Vec<i32>,
    /// `CurrentPlayerExtraLevelPerfect`, one per extra level.
    pub extra: Vec<i32>,
}

/// WSW progress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WswProgress {
    /// `WorldMapLevel*` for indices 0 to 140.
    pub map: Vec<WorldMapEntry>,
    pub gold_stars: i32,
    pub rainbow_stars: i32,
    pub peegues_pass: i32,
    pub qookies_map: i32,
    pub forest_map: i32,
    /// `storytold`, 20 flags.
    pub story_told: Vec<i32>,
}

/// One WSW world map level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldMapEntry {
    pub status: i32,
    pub score: i32,
    pub kind: i32,
}

fn expected_magic(game: Game) -> Option<&'static str> {
    match game {
        Game::Tow => None,
        Game::Rtwp => Some("Stinky & Loof Player File v2"),
        Game::Wsw => Some("Stinky & Loof WSW Player File"),
    }
}

fn read_ints(r: &mut BlitzReader, n: usize) -> Result<Vec<i32>> {
    (0..n).map(|_| r.read_int()).collect()
}

impl PlayerFile {
    /// Parses a save the way `game` loads it. Bytes after the last value read
    /// are kept in [`PlayerFile::trailing`].
    pub fn parse(data: &[u8], game: Game) -> Result<Self> {
        let mut r = BlitzReader::new(data);
        let magic = r.read_string()?;
        if let Some(expected) = expected_magic(game)
            && magic != expected
        {
            return Err(Error::Invalid(format!(
                "not a {game:?} player file: {magic:?}"
            )));
        }
        let name = r.read_string()?;
        let id = (game != Game::Tow).then(|| r.read_int()).transpose()?;
        let position = r.read_int()?;
        let levels_completed = r.read_int()?;
        let total_score = r.read_int()?;

        let progress = match game {
            Game::Tow => Progress::Tow(read_ints(&mut r, LEVEL_COUNT)?),
            Game::Rtwp => Progress::Rtwp(parse_rtwp(&mut r, data.len())?),
            Game::Wsw => Progress::Wsw(parse_wsw(&mut r)?),
        };
        let trailing = r.read_bytes(data.len() - r.position())?.to_vec();
        Ok(Self {
            magic,
            name,
            id,
            position,
            levels_completed,
            total_score,
            progress,
            trailing,
        })
    }

    /// Serialises the way `SavePlayer` writes the file.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut w = BlitzWriter::new();
        w.write_string(&self.magic);
        w.write_string(&self.name);
        if let Some(id) = self.id {
            w.write_int(id);
        }
        w.write_int(self.position);
        w.write_int(self.levels_completed);
        w.write_int(self.total_score);
        match &self.progress {
            Progress::Tow(scores) => scores.iter().for_each(|&s| w.write_int(s)),
            Progress::Rtwp(p) => {
                p.scores.iter().for_each(|&s| w.write_int(s));
                w.write_int(p.extra_levels.len() as i32);
                for e in &p.extra_levels {
                    w.write_int(e.id);
                    w.write_int(e.id2);
                    w.write_int(e.score);
                }
                if let Some(perfect) = &p.perfect {
                    perfect.levels.iter().for_each(|&v| w.write_int(v));
                    perfect.extra.iter().for_each(|&v| w.write_int(v));
                }
                p.world_packs.iter().for_each(|&v| w.write_int(v));
            }
            Progress::Wsw(p) => {
                for e in &p.map {
                    w.write_int(e.status);
                    w.write_int(e.score);
                    w.write_int(e.kind);
                }
                w.write_int(p.gold_stars);
                w.write_int(p.rainbow_stars);
                w.write_int(p.peegues_pass);
                w.write_int(p.qookies_map);
                w.write_int(p.forest_map);
                p.story_told.iter().for_each(|&v| w.write_int(v));
            }
        }
        w.write_bytes(&self.trailing);
        w.into_bytes()
    }

    /// The levels-completed count and total score `SavePlayer` would write for
    /// the current progress: levels with a score above 0, and the sum of those
    /// scores (RTWP adds the extra level scores; WSW skips map entry 0).
    /// Wrapping arithmetic matches Blitz's 32-bit ints.
    pub fn computed_summary(&self) -> (i32, i32) {
        let count_scores = |scores: &mut dyn Iterator<Item = i32>| {
            scores
                .filter(|&s| s > 0)
                .fold((0i32, 0i32), |(n, sum), s| (n + 1, sum.wrapping_add(s)))
        };
        match &self.progress {
            Progress::Tow(scores) => count_scores(&mut scores.iter().copied()),
            Progress::Rtwp(p) => {
                let (n, sum) = count_scores(&mut p.scores.iter().copied());
                let extra = p
                    .extra_levels
                    .iter()
                    .fold(0i32, |acc, e| acc.wrapping_add(e.score));
                (n, sum.wrapping_add(extra))
            }
            Progress::Wsw(p) => count_scores(&mut p.map.iter().skip(1).map(|e| e.score)),
        }
    }
}

fn parse_rtwp(r: &mut BlitzReader, len: usize) -> Result<RtwpProgress> {
    let scores = read_ints(r, LEVEL_COUNT)?;
    let count = r.read_int()?;
    let count = usize::try_from(count)
        .map_err(|_| Error::Invalid(format!("negative extra level count {count}")))?;
    let extra_levels = (0..count)
        .map(|_| {
            Ok(ExtraLevel {
                id: r.read_int()?,
                id2: r.read_int()?,
                score: r.read_int()?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    // CreateNewPlayer omits the perfect flags, leaving only the 4 world packs.
    let perfect = if len - r.position() == 16 {
        None
    } else {
        Some(Perfect {
            levels: read_ints(r, LEVEL_COUNT)?,
            extra: read_ints(r, count)?,
        })
    };
    let world_packs = [r.read_int()?, r.read_int()?, r.read_int()?, r.read_int()?];
    Ok(RtwpProgress {
        scores,
        extra_levels,
        perfect,
        world_packs,
    })
}

fn parse_wsw(r: &mut BlitzReader) -> Result<WswProgress> {
    let map = (0..WSW_MAP_ENTRIES)
        .map(|_| {
            Ok(WorldMapEntry {
                status: r.read_int()?,
                score: r.read_int()?,
                kind: r.read_int()?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(WswProgress {
        map,
        gold_stars: r.read_int()?,
        rainbow_stars: r.read_int()?,
        peegues_pass: r.read_int()?,
        qookies_map: r.read_int()?,
        forest_map: r.read_int()?,
        story_told: read_ints(r, WSW_STORY_FLAGS)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ints(v: &[i32]) -> Vec<u8> {
        v.iter().flat_map(|i| i.to_le_bytes()).collect()
    }

    fn string(s: &str) -> Vec<u8> {
        let mut out = (s.len() as u32).to_le_bytes().to_vec();
        out.extend(s.bytes());
        out
    }

    fn rtwp_head() -> Vec<u8> {
        let mut d = string("Stinky & Loof Player File v2");
        d.extend(string("Zed"));
        d.extend(ints(&[77, 10, 2, 300])); // id, world, completed, score
        let mut scores = vec![0; LEVEL_COUNT];
        scores[0] = 100;
        scores[5] = 200;
        d.extend(ints(&scores));
        d
    }

    #[test]
    fn tow_layout_has_no_id() {
        let mut d = string("Stinky & Loof Player File");
        d.extend(string("Ann"));
        d.extend(ints(&[3, 1, 50]));
        let mut scores = vec![0; LEVEL_COUNT];
        scores[2] = 50;
        d.extend(ints(&scores));
        let p = PlayerFile::parse(&d, Game::Tow).unwrap();
        assert_eq!(p.name, "Ann");
        assert_eq!(p.id, None);
        assert_eq!(p.position, 3);
        assert_eq!(p.computed_summary(), (1, 50));
        assert_eq!(p.to_bytes(), d);
    }

    #[test]
    fn rtwp_full_with_extra_levels() {
        let mut d = rtwp_head();
        d.extend(ints(&[2, 11, 12, 13, 21, 22, 23])); // two extra levels
        let mut perfect = vec![0; LEVEL_COUNT];
        perfect[5] = 1;
        d.extend(ints(&perfect));
        d.extend(ints(&[1, 0])); // extra perfect
        d.extend(ints(&[0, 0, 20, 30])); // world packs, stored as is
        let p = PlayerFile::parse(&d, Game::Rtwp).unwrap();
        assert_eq!(p.id, Some(77));
        let Progress::Rtwp(rp) = &p.progress else {
            panic!()
        };
        assert_eq!(rp.extra_levels[1].score, 23);
        assert_eq!(rp.perfect.as_ref().unwrap().extra, [1, 0]);
        assert_eq!(rp.world_packs, [0, 0, 20, 30]);
        assert_eq!(p.computed_summary(), (2, 100 + 200 + 13 + 23));
        assert_eq!(p.to_bytes(), d);
    }

    #[test]
    fn rtwp_new_player_shape_has_no_perfect_flags() {
        let mut d = rtwp_head();
        d.extend(ints(&[0])); // no extra levels
        d.extend(ints(&[0, 10, 20, 30]));
        let p = PlayerFile::parse(&d, Game::Rtwp).unwrap();
        let Progress::Rtwp(rp) = &p.progress else {
            panic!()
        };
        assert!(rp.perfect.is_none());
        assert_eq!(rp.world_packs, [0, 10, 20, 30]);
        assert_eq!(p.to_bytes(), d);
    }

    #[test]
    fn trailing_bytes_are_kept() {
        let mut d = rtwp_head();
        d.extend(ints(&[0]));
        d.extend(ints(&[0; LEVEL_COUNT]));
        d.extend(ints(&[0, 10, 20, 30]));
        d.extend([1, 2, 3]);
        let p = PlayerFile::parse(&d, Game::Rtwp).unwrap();
        assert_eq!(p.trailing, [1, 2, 3]);
        assert_eq!(p.to_bytes(), d);
    }

    #[test]
    fn wsw_layout() {
        let mut d = string("Stinky & Loof WSW Player File");
        d.extend(string("Bo"));
        d.extend(ints(&[9, 4, 1, 25]));
        for i in 0..WSW_MAP_ENTRIES as i32 {
            d.extend(ints(&[i, if i == 3 { 25 } else { 0 }, 2]));
        }
        d.extend(ints(&[1, 2, 3, 4, 5]));
        d.extend(ints(&[0; WSW_STORY_FLAGS]));
        let p = PlayerFile::parse(&d, Game::Wsw).unwrap();
        let Progress::Wsw(w) = &p.progress else {
            panic!()
        };
        assert_eq!((p.id, p.position), (Some(9), 4));
        assert_eq!(w.map[3].score, 25);
        assert_eq!((w.gold_stars, w.forest_map), (1, 5));
        assert_eq!(p.computed_summary(), (1, 25));
        assert_eq!(p.to_bytes(), d);
    }

    #[test]
    fn rejects_wrong_magic_and_truncation() {
        let d = rtwp_head();
        assert!(PlayerFile::parse(&d, Game::Wsw).is_err());
        assert!(PlayerFile::parse(&d, Game::Rtwp).is_err()); // truncated
        let mut old = string("Stinky & Loof Player File");
        old.extend(string("x"));
        assert!(PlayerFile::parse(&old, Game::Rtwp).is_err());
    }
}
