//! The small configuration files the games keep in their folder.
//!
//! * [`Options`] - `options.cfg`, read by `LoadOptions` and written by
//!   `SaveOptions` in `RTWP/menu.bb` (4441 / 4492), `TOW/menu.bb` (5005 /
//!   5071) and `WSW/menu.bb` (3134 / 3192). `RTWP/editor.bb:16` reads the same
//!   12-int layout as RTWP's menu.
//! * [`ExtraCfg`] - TOW's `extra.cfg` (`TOW/menu.bb` 5045 / 5103).
//! * [`StoryDat`] - WSW's `story.dat` (`WSW/setup.bb:710`, `WSW/menu.bb:3775`).
//!
//! All three are plain runs of `WriteInt` values with no header.
//!
//! # Short and long files
//!
//! Blitz's `ReadInt` returns 0 at end of file and the loaders never check, so
//! a file with fewer ints than the loader reads simply leaves the remaining
//! fields 0 (this is real: the shipped WSW `options.cfg` has 9 of the 12 ints
//! its loader reads, and TOW's loader only reads 9). The parsers here mirror
//! that: they never fail, missing ints are 0, and bytes beyond what the
//! loader reads are ignored. To still round-trip byte-for-byte, each type
//! remembers how many ints were actually in the file (`present`) and any
//! `trailing` bytes after them (unread extra ints, or a partial int), and
//! `to_bytes` emits exactly those. A value built in code (via `Default`) is
//! written in full, as the game's save function does.

use crate::Game;
use crate::Result;
use crate::blitz::{BlitzReader, BlitzWriter};

/// Reads up to `N` ints Blitz-style. Returns the values (0 when absent), how
/// many were really present, and the bytes left after those.
fn read_ints<const N: usize>(data: &[u8]) -> ([i32; N], usize, Vec<u8>) {
    let mut r = BlitzReader::new(data);
    let mut values = [0; N];
    let mut present = 0;
    for v in &mut values {
        match r.read_int() {
            Ok(x) => {
                *v = x;
                present += 1;
            }
            Err(_) => break,
        }
    }
    (values, present, data[present * 4..].to_vec())
}

fn write_ints(values: &[i32], present: usize, trailing: &[u8]) -> Vec<u8> {
    let mut w = BlitzWriter::new();
    for &v in &values[..present.min(values.len())] {
        w.write_int(v);
    }
    w.write_bytes(trailing);
    w.into_bytes()
}

/// Blitz `Int(x)`: round to nearest (the FPU rounds ties to even).
fn blitz_int(x: f32) -> i32 {
    x.round_ties_even() as i32
}

/// `options.cfg`: sound, graphics and gameplay settings.
///
/// Layout (little-endian `i32`s, in order): sound, music, detail, gfx mode,
/// colour mode, windowed, language, no time limit, lock worlds, then - RTWP
/// and WSW only - editor colour mode, editor windowed, teleport. TOW's loader
/// and saver stop after lock worlds; those three fields are always 0 there.
///
/// Values are as stored. Per the loaders, `LockWorlds` is read by all games.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// Sound volume as stored: `Int(level * 100)`. Loader divides by 100.
    /// See [`Options::sound_level`].
    pub sound_level: i32,
    /// Music volume as stored, same encoding as `sound_level`.
    pub music_level: i32,
    /// `DetailLevel`.
    pub detail_level: i32,
    /// `MyGfxMode`: index into the game's resolution list.
    pub gfx_mode: i32,
    /// `MyColMode`: colour depth selector (1, 2 or 3).
    pub col_mode: i32,
    /// `MyWindowed`: 0 fullscreen, 1 windowed.
    pub windowed: i32,
    /// `Language`: 0 is English.
    pub language: i32,
    /// `NoTimeLimit`: cheat/option flag.
    pub no_time_limit: i32,
    /// `LockWorlds`.
    pub lock_worlds: i32,
    /// `MyColModeEd`: the level editor's colour mode (`Graphics` depth is
    /// `8 + 8 * value`). Not stored by TOW.
    pub col_mode_ed: i32,
    /// `MyWindowedEd`: the level editor's windowed flag. Not stored by TOW.
    pub windowed_ed: i32,
    /// `OptionsTeleport`. Not stored by TOW.
    pub teleport: i32,
    /// How many of the ints were really in the file (see the module docs).
    pub present: usize,
    /// Bytes after the `present` ints (unread extra ints or a partial int).
    pub trailing: Vec<u8>,
}

impl Options {
    /// Number of ints the game's loader reads: 9 for TOW, 12 otherwise.
    pub fn int_count(game: Game) -> usize {
        match game {
            Game::Tow => 9,
            Game::Rtwp | Game::Wsw => 12,
        }
    }

    /// The values the loader uses when `options.cfg` does not exist
    /// (`RTWP/menu.bb:4444`, `TOW/menu.bb:5010`, `WSW/menu.bb:3141`).
    ///
    /// Volumes are `1.0` in the game, i.e. 100 stored. The menus leave the
    /// editor fields at their global default of 0 (the standalone editor,
    /// `editor.bb:18`, instead sets `col_mode_ed` to 1 when the file is
    /// missing). Defaults differ: detail is 5 in TOW and 3 elsewhere;
    /// teleport is 1 in WSW and 0 in RTWP.
    pub fn defaults(game: Game) -> Self {
        Self {
            sound_level: 100,
            music_level: 100,
            detail_level: if game == Game::Tow { 5 } else { 3 },
            gfx_mode: 1,
            col_mode: 1,
            windowed: 0,
            language: 0,
            no_time_limit: 0,
            lock_worlds: 0,
            col_mode_ed: 0,
            windowed_ed: 0,
            teleport: i32::from(game == Game::Wsw),
            present: Self::int_count(game),
            trailing: Vec::new(),
        }
    }

    /// Parses `options.cfg` as `game`'s loader would. Never fails: missing
    /// ints are 0 (see the module docs).
    pub fn parse(game: Game, data: &[u8]) -> Result<Self> {
        let (v, present, trailing) = match game {
            Game::Tow => {
                let (v, p, t) = read_ints::<9>(data);
                let mut full = [0; 12];
                full[..9].copy_from_slice(&v);
                (full, p, t)
            }
            Game::Rtwp | Game::Wsw => read_ints::<12>(data),
        };
        Ok(Self {
            sound_level: v[0],
            music_level: v[1],
            detail_level: v[2],
            gfx_mode: v[3],
            col_mode: v[4],
            windowed: v[5],
            language: v[6],
            no_time_limit: v[7],
            lock_worlds: v[8],
            col_mode_ed: v[9],
            windowed_ed: v[10],
            teleport: v[11],
            present,
            trailing,
        })
    }

    /// Serialises as stored: the `present` ints, then `trailing`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let v = [
            self.sound_level,
            self.music_level,
            self.detail_level,
            self.gfx_mode,
            self.col_mode,
            self.windowed,
            self.language,
            self.no_time_limit,
            self.lock_worlds,
            self.col_mode_ed,
            self.windowed_ed,
            self.teleport,
        ];
        write_ints(&v, self.present, &self.trailing)
    }

    /// Sound volume as the game uses it: `sound_level / 100.0`.
    pub fn sound_level(&self) -> f32 {
        self.sound_level as f32 / 100.0
    }

    /// Sets the stored sound volume like `SaveOptions`: `Int(level * 100)`,
    /// which rounds to nearest (ties to even).
    pub fn set_sound_level(&mut self, level: f32) {
        self.sound_level = blitz_int(level * 100.0);
    }

    /// Music volume as the game uses it: `music_level / 100.0`.
    pub fn music_level(&self) -> f32 {
        self.music_level as f32 / 100.0
    }

    /// Sets the stored music volume like `SaveOptions`: `Int(level * 100)`,
    /// which rounds to nearest (ties to even).
    pub fn set_music_level(&mut self, level: f32) {
        self.music_level = blitz_int(level * 100.0);
    }
}

/// TOW's `extra.cfg`: one int, `IsExtraDemoLevelActive`. When it is 1 the
/// loader points world levels 15 and 16 at `resdb1.dat` / `resdb2.dat`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExtraCfg {
    /// `IsExtraDemoLevelActive` (0 when the file is missing).
    pub extra_demo_level_active: i32,
    /// Ints present in the file (0 or 1); see the module docs.
    pub present: usize,
    /// Bytes after the `present` ints.
    pub trailing: Vec<u8>,
}

impl ExtraCfg {
    /// Value used when the file is missing (`TOW/menu.bb:5047`). A file
    /// created in code is written in full.
    pub fn defaults() -> Self {
        Self {
            extra_demo_level_active: 0,
            present: 1,
            trailing: Vec::new(),
        }
    }

    /// Parses `extra.cfg`. Never fails; an empty file reads as 0.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let (v, present, trailing) = read_ints::<1>(data);
        Ok(Self {
            extra_demo_level_active: v[0],
            present,
            trailing,
        })
    }

    /// Serialises as stored.
    pub fn to_bytes(&self) -> Vec<u8> {
        write_ints(
            &[self.extra_demo_level_active],
            self.present,
            &self.trailing,
        )
    }
}

/// WSW's `story.dat`: which story pictures the player has unlocked, nine
/// ints (`storymenupic(0..=8)`, each 0 or 1). The "view unlocked story
/// pictures" menu (`MenuLoop18`) shows pictures up to the highest set flag.
/// `StartStoryScreen` sets flag 0..=7 for story screens 6, 9, 10, 21, 24, 25,
/// 30 and 35 respectively and flag 8 for 44-46, then rewrites the file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StoryDat {
    /// `storymenupic(0..=8)` as stored.
    pub pictures: [i32; 9],
    /// Ints present in the file (0..=9); see the module docs.
    pub present: usize,
    /// Bytes after the `present` ints.
    pub trailing: Vec<u8>,
}

impl StoryDat {
    /// State when the file is missing (`WSW/setup.bb:711`: the loader
    /// returns and the array keeps its zeros), i.e. nothing unlocked. Written
    /// in full when saved.
    pub fn defaults() -> Self {
        Self {
            pictures: [0; 9],
            present: 9,
            trailing: Vec::new(),
        }
    }

    /// Parses `story.dat`. Never fails; missing ints are 0.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let (pictures, present, trailing) = read_ints::<9>(data);
        Ok(Self {
            pictures,
            present,
            trailing,
        })
    }

    /// Serialises as stored.
    pub fn to_bytes(&self) -> Vec<u8> {
        write_ints(&self.pictures, self.present, &self.trailing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ints(v: &[i32]) -> Vec<u8> {
        v.iter().flat_map(|x| x.to_le_bytes()).collect()
    }

    // Real assets/RTWP/options.cfg:
    // 3c 46 05 04 03 00 00 01 01 03 00 00
    #[test]
    fn rtwp_options_fields() {
        let bytes = ints(&[60, 70, 5, 4, 3, 0, 0, 1, 1, 3, 0, 0]);
        let o = Options::parse(Game::Rtwp, &bytes).unwrap();
        assert_eq!((o.sound_level, o.music_level), (60, 70));
        assert_eq!((o.detail_level, o.gfx_mode, o.col_mode), (5, 4, 3));
        assert_eq!((o.no_time_limit, o.lock_worlds), (1, 1));
        assert_eq!((o.col_mode_ed, o.windowed_ed, o.teleport), (3, 0, 0));
        assert_eq!(o.sound_level(), 0.6);
        assert_eq!(o.to_bytes(), bytes);
    }

    // Real assets/TOW/options.cfg: 100 100 3 3 3 0 0 1 1
    #[test]
    fn tow_options_has_nine_ints() {
        let bytes = ints(&[100, 100, 3, 3, 3, 0, 0, 1, 1]);
        let o = Options::parse(Game::Tow, &bytes).unwrap();
        assert_eq!((o.detail_level, o.gfx_mode, o.lock_worlds), (3, 3, 1));
        assert_eq!(o.teleport, 0);
        assert_eq!(o.present, 9);
        assert_eq!(o.to_bytes(), bytes);
    }

    // Real assets/WSW/options.cfg is 9 ints but the loader reads 12.
    #[test]
    fn wsw_short_file_reads_zero_and_round_trips() {
        let bytes = ints(&[100, 100, 3, 2, 3, 0, 0, 0, 0]);
        let o = Options::parse(Game::Wsw, &bytes).unwrap();
        assert_eq!(o.gfx_mode, 2);
        assert_eq!((o.col_mode_ed, o.windowed_ed, o.teleport), (0, 0, 0));
        assert_eq!(o.present, 9);
        assert_eq!(o.to_bytes(), bytes);
    }

    #[test]
    fn partial_int_and_extra_bytes_round_trip() {
        let mut bytes = ints(&[1, 2, 3]);
        bytes.extend([9, 9]);
        let o = Options::parse(Game::Rtwp, &bytes).unwrap();
        assert_eq!((o.present, o.detail_level, o.gfx_mode), (3, 3, 0));
        assert_eq!(o.to_bytes(), bytes);

        let mut long = ints(&[0; 9]);
        long.extend(ints(&[7, 8]));
        let o = Options::parse(Game::Tow, &long).unwrap();
        assert_eq!(o.trailing.len(), 8);
        assert_eq!(o.to_bytes(), long);
    }

    #[test]
    fn empty_file_is_all_zero() {
        let o = Options::parse(Game::Wsw, &[]).unwrap();
        assert_eq!(o.present, 0);
        assert_eq!(o.sound_level, 0);
        assert!(o.to_bytes().is_empty());
    }

    #[test]
    fn defaults_match_loaders() {
        let t = Options::defaults(Game::Tow);
        assert_eq!((t.sound_level, t.detail_level, t.teleport), (100, 5, 0));
        assert_eq!(t.to_bytes().len(), 36);
        let r = Options::defaults(Game::Rtwp);
        assert_eq!((r.detail_level, r.teleport), (3, 0));
        assert_eq!(r.to_bytes().len(), 48);
        let w = Options::defaults(Game::Wsw);
        assert_eq!((w.detail_level, w.teleport, w.gfx_mode), (3, 1, 1));
        assert_eq!(w.sound_level(), 1.0);
    }

    #[test]
    fn volume_setters_round_to_nearest() {
        let mut o = Options::defaults(Game::Rtwp);
        o.set_sound_level(0.666);
        assert_eq!(o.sound_level, 67);
        o.set_music_level(0.004);
        assert_eq!(o.music_level, 0);
    }

    #[test]
    fn extra_cfg() {
        let e = ExtraCfg::parse(&[0, 0, 0, 0]).unwrap();
        assert_eq!(e.extra_demo_level_active, 0);
        assert_eq!(e.to_bytes(), [0, 0, 0, 0]);
        let e = ExtraCfg::parse(&[1, 0, 0, 0]).unwrap();
        assert_eq!(e.extra_demo_level_active, 1);
        let e = ExtraCfg::parse(&[]).unwrap();
        assert_eq!((e.extra_demo_level_active, e.present), (0, 0));
        assert!(e.to_bytes().is_empty());
        assert_eq!(ExtraCfg::defaults().to_bytes(), [0; 4]);
    }

    // Real assets/WSW/story.dat: 1 1 1 1 1 0 0 0 0
    #[test]
    fn story_dat() {
        let bytes = ints(&[1, 1, 1, 1, 1, 0, 0, 0, 0]);
        let s = StoryDat::parse(&bytes).unwrap();
        assert_eq!(s.pictures, [1, 1, 1, 1, 1, 0, 0, 0, 0]);
        assert_eq!(s.to_bytes(), bytes);
        let short = StoryDat::parse(&ints(&[1, 1])).unwrap();
        assert_eq!(short.pictures, [1, 1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(short.to_bytes(), ints(&[1, 1]));
        assert_eq!(StoryDat::defaults().to_bytes().len(), 36);
    }
}
