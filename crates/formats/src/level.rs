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
//!
//! [`Level::upgraded`] turns any version into v6, the only one the editor
//! writes. It follows what `LoadLevelOldVersion` does with a file, not
//! `RTWP/editor.bb`'s own upgrade, whose tables change some objects: see
//! [`OBJECTS_V2`] and [`OBJECTS_V1`].

use crate::blitz::{BlitzReader, BlitzWriter};
use crate::{Error, Game, Result};

/// The magic string of a v6 file.
const MAGIC_V6: &str = "Stinky & Loof Level File v6";

/// v2–v4 object codes as v5 codes. This is `version5objects`, except that the
/// fish (41) goes to 48: the editor's 47 is the mother ship.
const OBJECTS_V2: [i32; 51] = [
    0, 1, 2, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27, 28, 29, 30,
    31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 48, 50, 51, 52, 66, 70, 84, 85, 86,
    87,
];

/// v1 object codes as v2 codes. This is `version2objects`, except that 46 and
/// 47 stay the mushroom and the tree: the editor swaps them.
const OBJECTS_V1: [i32; 51] = [
    0, 1, 2, 7, 8, 4, 3, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 5, 6,
    41, 47, 42, 43, 44, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 9, 10, 11, 12, 46, 45, 48, 49,
    50,
];

/// Object 88, which turns background 9 into 10 in v5 and later.
const HIDDEN_RAINBOW: i32 = 88;

/// Looks `code` up in an old object table. The old loaders ignore codes past
/// the end, as v5 ignores 0.
fn old_object(table: &[i32; 51], code: i32) -> i32 {
    usize::try_from(code)
        .ok()
        .and_then(|i| table.get(i))
        .copied()
        .unwrap_or(0)
}

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
    /// v6: custom textures (`customtextures\<dir>`), used by style 9. RTWP
    /// ignores the flag and goes by the style.
    pub textures: Option<CustomDir>,
    /// v6: custom backdrop (`custombackgrounds\<dir>`), used by background
    /// 11. RTWP ignores the flag and goes by the background.
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

    /// The file bytes, the reverse of [`Level::parse`]. The optional fields
    /// decide the layout, so a parsed level comes back byte for byte.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut w = BlitzWriter::new();
        w.write_string(&self.magic);
        w.write_string(&self.file_name);
        if let Some(id) = self.map_id {
            w.write_int(id);
        }
        w.write_string(&self.description);
        for dir in [
            &self.houses,
            &self.models,
            &self.textures,
            &self.backgrounds,
        ]
        .into_iter()
        .flatten()
        {
            w.write_int(dir.flag);
            w.write_string(&dir.dir);
        }
        w.write_int(self.time);
        w.write_int(self.style);
        w.write_int(self.background);
        w.write_int(self.width as i32);
        w.write_int(self.height as i32);
        for &code in self.tiles.iter().chain(&self.objects) {
            w.write_int(code);
        }
        for sign in &self.signs {
            w.write_string(sign);
        }
        if let Some(track) = self.music_track {
            w.write_int(track);
        }
        w.into_bytes()
    }

    /// This level as a v6 file that plays the same. A level before v5 changes
    /// as follows, and later versions only gain the empty v6 fields:
    ///
    /// - Button tiles from 1103 move up 3, past the timer buttons. 1197 and up
    ///   have no such code and stay.
    /// - Objects move to their v5 codes.
    /// - Background 10 becomes 9 and object 88, as v5 reads 10 as 11. A level
    ///   with no empty object cell has no room for it and keeps the 10.
    /// - Signs grow to 20, and a level before v4 gets music track 0. It used
    ///   to inherit the track and sign texts of the level played before it.
    /// - A TOW level gets map id 0. Saving a level sets a new id anyway.
    ///
    /// Old tiles in classes that only mean something from v5 (transporters,
    /// box factories and the like) are left as they are; the old editors could
    /// not place them.
    pub fn upgraded(&self) -> Self {
        let mut level = self.clone();
        let empty = || {
            Some(CustomDir {
                flag: 0,
                dir: String::new(),
            })
        };
        level.magic = MAGIC_V6.into();
        level.version = 6;
        level.map_id = level.map_id.or(Some(0));
        level.houses = level.houses.or_else(empty);
        level.models = level.models.or_else(empty);
        level.textures = level.textures.or_else(empty);
        level.backgrounds = level.backgrounds.or_else(empty);
        level.signs.resize(20, String::new());
        level.music_track = level.music_track.or(Some(0));
        if self.version >= 5 {
            return level;
        }

        for tile in &mut level.tiles {
            if (1103..1197).contains(tile) {
                *tile += 3;
            }
        }
        for object in &mut level.objects {
            if self.version == 1 {
                *object = old_object(&OBJECTS_V1, *object);
            }
            *object = old_object(&OBJECTS_V2, *object);
        }
        if level.background == 10
            && let Some(free) = level.objects.iter_mut().find(|o| **o == 0)
        {
            *free = HIDDEN_RAINBOW;
            level.background = 9;
        }
        level
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
    fn to_bytes_reverses_parse() {
        let v1 = level(
            "Stinky & Loof Level File",
            |w| {
                w.string("a.lev").string("First");
            },
            5,
            false,
        );
        let v6 = level(
            "Stinky & Loof Level File v6",
            |w| {
                w.string("a.lv5").int(1).string("Third");
                w.int(1).string("h").int(0).string("m");
                w.int(1).string("t").int(1).string("b");
            },
            20,
            true,
        );
        assert_eq!(Level::parse(&v1, Game::Tow).unwrap().to_bytes(), v1);
        assert_eq!(Level::parse(&v6, Game::Rtwp).unwrap().to_bytes(), v6);
    }

    fn old_level(version: u8) -> Level {
        Level {
            magic: format!("Stinky & Loof Level File v{version}"),
            version,
            file_name: "a.lev".into(),
            map_id: Some(42),
            description: "Old".into(),
            houses: None,
            models: None,
            textures: None,
            backgrounds: None,
            time: 300,
            style: 2,
            background: 4,
            width: 2,
            height: 1,
            tiles: vec![1103, 1102],
            objects: vec![41, 0],
            signs: vec!["sign".into(); 5],
            music_track: (version >= 4).then_some(7),
        }
    }

    #[test]
    fn upgrade_fills_in_the_v6_fields() {
        let l = old_level(3).upgraded();
        assert_eq!(l.magic, "Stinky & Loof Level File v6");
        assert_eq!((l.version, l.map_id, l.music_track), (6, Some(42), Some(0)));
        assert_eq!(l.backgrounds, l.houses);
        assert_eq!(l.houses.as_ref().unwrap().dir, "");
        assert!(!l.textures.as_ref().unwrap().enabled());
        assert_eq!(l.signs.len(), 20);
        assert_eq!((l.signs[4].as_str(), l.signs[5].as_str()), ("sign", ""));
        assert_eq!(old_level(4).upgraded().music_track, Some(7));
        assert_eq!(Level::parse(&l.to_bytes(), Game::Rtwp).unwrap(), l);
    }

    #[test]
    fn upgrade_recodes_tiles_and_objects() {
        let l = old_level(4).upgraded();
        assert_eq!(l.tiles, [1106, 1102]);
        assert_eq!(l.objects, [48, 0]);
        let mut v1 = old_level(1);
        v1.objects = vec![46, 47];
        assert_eq!(v1.upgraded().objects, [70, 66]);
        v1.objects = vec![51, -1];
        assert_eq!(v1.upgraded().objects, [0, 0]);
    }

    #[test]
    fn upgrade_keeps_the_hidden_rainbow_background() {
        let mut old = old_level(4);
        old.background = 10;
        let l = old.upgraded();
        assert_eq!((l.background, l.objects[1]), (9, 88));
        old.objects = vec![1, 2];
        assert_eq!(old.upgraded().background, 10);
    }

    #[test]
    fn upgrade_leaves_newer_codes_alone() {
        let mut v5 = old_level(5);
        v5.houses = Some(CustomDir {
            flag: 1,
            dir: "h".into(),
        });
        v5.models = v5.houses.clone();
        v5.signs = vec![String::new(); 20];
        v5.music_track = Some(7);
        let l = v5.upgraded();
        assert_eq!((l.tiles, l.objects), (v5.tiles.clone(), v5.objects.clone()));
        assert_eq!(l.houses, v5.houses);
        assert!(l.textures.is_some());

        let v6 = v5.upgraded();
        assert_eq!(v6.upgraded(), v6);
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
