//! Level files (`.lev`, `.lv5`), format versions 1–6.
//!
//! Ported from the data-reading half of `LoadLevel`, `LoadLevelOldVersion` and
//! `LoadLevelv5` in `RTWP/level.bb`, plus the TOW and WSW loaders. A file is a
//! magic string whose last character gives the version, a header, two
//! `width * height` grids of ints (tiles, then objects; row by row), the sign
//! texts and, from v4, a music track.
//!
//! The games disagree on the header, so parsing takes the [`Game`] that loads
//! the file:
//!
//! - TOW ignores the version and never reads a map id. Its levels are all v1.
//! - RTWP takes the version from the magic (`2`–`6`, anything else is v1) and
//!   always reads a map id, so TOW's levels do not load in RTWP.
//! - WSW only loads v5 and leaves every other version unread.
//!
//! [`Level`] holds the values as stored. The loaders also rewrite some of them
//! while building the level (sign, electro, spike and other tiles become
//! floor, background 10 becomes 11), and that belongs to level setup, not here.

use crate::blitz::BlitzReader;
use crate::{Error, Game, Result};

/// A level file's contents, as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level {
    /// The first string, e.g. `Stinky & Loof Level File v4`.
    pub magic: String,
    /// The format version the loading game derived from `magic`.
    pub version: u8,
    /// The file name the editor saved under (unused by the games).
    pub file_name: String,
    /// Absent in TOW, which does not read it.
    pub map_id: Option<i32>,
    /// The level title shown in game.
    pub description: String,
    /// v5+: custom house textures (`customhouses\<dir>`).
    pub houses: Option<CustomDir>,
    /// v5+: custom pillar models (`custommodels\<dir>`).
    pub models: Option<CustomDir>,
    /// v6: custom textures. RTWP reads but ignores it.
    pub textures: Option<CustomDir>,
    /// v6: custom backgrounds. RTWP reads but ignores it.
    pub backgrounds: Option<CustomDir>,
    /// Time limit.
    pub time: i32,
    pub style: i32,
    pub background: i32,
    pub width: usize,
    pub height: usize,
    /// Tile codes, `width * height`, indexed by [`Level::index`].
    pub tiles: Vec<i32>,
    /// Object codes, same layout as `tiles`.
    pub objects: Vec<i32>,
    /// 5 sign texts before v5, 20 from v5.
    pub signs: Vec<String>,
    /// v4+: `ExtraLevelMusicTrack`.
    pub music_track: Option<i32>,
}

/// A custom asset folder in a v5+ header: an on/off flag and a folder name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomDir {
    pub flag: i32,
    pub dir: String,
}

impl CustomDir {
    /// The games only use the folder when the flag is exactly 1.
    pub fn enabled(&self) -> bool {
        self.flag == 1
    }
}

impl Level {
    /// Parses a level the way `game` loads it. Every byte must be consumed.
    pub fn parse(data: &[u8], game: Game) -> Result<Self> {
        let mut r = BlitzReader::new(data);
        let magic = r.read_string()?;
        let version = match game {
            Game::Tow => 1,
            Game::Rtwp => match magic.chars().last() {
                Some(c @ '2'..='6') => c as u8 - b'0',
                _ => 1,
            },
            Game::Wsw if magic.ends_with('5') => 5,
            Game::Wsw => {
                return Err(Error::Invalid(format!(
                    "WSW only loads v5 levels, found {magic:?}"
                )));
            }
        };

        let file_name = r.read_string()?;
        let map_id = (game != Game::Tow).then(|| r.read_int()).transpose()?;
        let description = r.read_string()?;

        let mut custom_dir = |present: bool| -> Result<Option<CustomDir>> {
            if !present {
                return Ok(None);
            }
            let flag = r.read_int()?;
            let dir = r.read_string()?;
            Ok(Some(CustomDir { flag, dir }))
        };
        let houses = custom_dir(version >= 5)?;
        let models = custom_dir(version >= 5)?;
        let textures = custom_dir(version == 6)?;
        let backgrounds = custom_dir(version == 6)?;

        let time = r.read_int()?;
        let style = r.read_int()?;
        let background = r.read_int()?;
        let width = r.read_int()?;
        let height = r.read_int()?;
        let max = max_dimension(game);
        let dimension = |n: i32| {
            usize::try_from(n)
                .ok()
                .filter(|&n| n <= max)
                .ok_or_else(|| Error::Invalid(format!("level size {width}x{height} (max {max})")))
        };
        let (width, height) = (dimension(width)?, dimension(height)?);

        let mut grid =
            || -> Result<Vec<i32>> { (0..width * height).map(|_| r.read_int()).collect() };
        let tiles = grid()?;
        let objects = grid()?;

        let sign_count = if version >= 5 { 20 } else { 5 };
        let signs = (0..sign_count)
            .map(|_| r.read_string())
            .collect::<Result<_>>()?;
        let music_track = (version >= 4).then(|| r.read_int()).transpose()?;

        if !r.is_eof() {
            return Err(Error::Invalid(format!(
                "{} unread bytes after the level data",
                data.len() - r.position()
            )));
        }

        Ok(Self {
            magic,
            version,
            file_name,
            map_id,
            description,
            houses,
            models,
            textures,
            backgrounds,
            time,
            style,
            background,
            width,
            height,
            tiles,
            objects,
            signs,
            music_track,
        })
    }

    /// Index of tile `(x, y)` in `tiles` and `objects`.
    pub fn index(&self, x: usize, y: usize) -> usize {
        debug_assert!(x < self.width && y < self.height);
        y * self.width + x
    }

    pub fn tile(&self, x: usize, y: usize) -> i32 {
        self.tiles[self.index(x, y)]
    }

    pub fn object(&self, x: usize, y: usize) -> i32 {
        self.objects[self.index(x, y)]
    }
}

/// The largest width or height that fits the game's `MapTile` array
/// (`Dim MapTile(n, n)` has n+1 entries per side).
fn max_dimension(game: Game) -> usize {
    match game {
        Game::Rtwp => 102,
        Game::Tow | Game::Wsw => 101,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Writer(Vec<u8>);

    impl Writer {
        fn int(&mut self, v: i32) -> &mut Self {
            self.0.extend(v.to_le_bytes());
            self
        }

        fn string(&mut self, s: &str) -> &mut Self {
            self.0.extend((s.len() as u32).to_le_bytes());
            self.0.extend(s.as_bytes());
            self
        }
    }

    /// A 2x1 level; `header` writes whatever sits between the magic and the
    /// time limit.
    fn level(magic: &str, header: impl Fn(&mut Writer), signs: usize, music: bool) -> Vec<u8> {
        let mut w = Writer(vec![]);
        w.string(magic);
        header(&mut w);
        w.int(300).int(2).int(4).int(2).int(1);
        w.int(100).int(1301).int(1).int(5);
        for i in 0..signs {
            w.string(&format!("sign {i}"));
        }
        if music {
            w.int(7);
        }
        w.0
    }

    #[test]
    fn tow_reads_no_map_id_or_version() {
        let data = level(
            "Stinky & Loof Level File",
            |w| {
                w.string("a.lev").string("First");
            },
            5,
            false,
        );
        let l = Level::parse(&data, Game::Tow).unwrap();
        assert_eq!((l.version, l.map_id, l.music_track), (1, None, None));
        assert_eq!(l.description, "First");
        assert_eq!((l.time, l.style, l.background), (300, 2, 4));
        assert_eq!((l.width, l.height), (2, 1));
        assert_eq!((l.tile(1, 0), l.object(0, 0)), (1301, 1));
        assert_eq!(l.signs.len(), 5);
        assert!(Level::parse(&data, Game::Rtwp).is_err());
    }

    #[test]
    fn rtwp_v4_has_map_id_and_music() {
        let data = level(
            "Stinky & Loof Level File v4",
            |w| {
                w.string("a.lev").int(42).string("Second");
            },
            5,
            true,
        );
        let l = Level::parse(&data, Game::Rtwp).unwrap();
        assert_eq!((l.version, l.map_id, l.music_track), (4, Some(42), Some(7)));
        assert!(l.houses.is_none());
    }

    #[test]
    fn v6_reads_four_custom_dirs() {
        let data = level(
            "Stinky & Loof Level File v6",
            |w| {
                w.string("a.lv5").int(1).string("Third");
                w.int(1).string("h").int(0).string("m");
                w.int(1).string("t").int(1).string("b");
            },
            20,
            true,
        );
        let l = Level::parse(&data, Game::Rtwp).unwrap();
        assert_eq!(l.version, 6);
        assert!(l.houses.as_ref().unwrap().enabled());
        assert!(!l.models.as_ref().unwrap().enabled());
        assert_eq!(l.backgrounds.unwrap().dir, "b");
        assert_eq!(l.signs[19], "sign 19");
        assert!(matches!(
            Level::parse(&data, Game::Wsw),
            Err(Error::Invalid(_))
        ));
    }

    #[test]
    fn rejects_trailing_bytes_and_oversized_maps() {
        let mut data = level(
            "Stinky & Loof Level File",
            |w| {
                w.string("a.lev").string("x");
            },
            5,
            false,
        );
        data.push(0);
        assert!(matches!(
            Level::parse(&data, Game::Tow),
            Err(Error::Invalid(_))
        ));

        let mut w = Writer(vec![]);
        w.string("v").string("a").string("x").int(0).int(0).int(0);
        w.int(102).int(1);
        assert!(matches!(
            Level::parse(&w.0, Game::Tow),
            Err(Error::Invalid(_))
        ));
    }
}
