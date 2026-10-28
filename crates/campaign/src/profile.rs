//! RTWP's player profiles, as `menu.bb` keeps them: `CreateNewPlayer`,
//! `LoadPlayer`, `SavePlayer`, and the `players` folder that `StartMenu(2)`
//! lists and `MenuLoop6` deletes from.
//!
//! The file layout is `wonderland_formats::save` (RTWP). A [`Profile`] is
//! the `CurrentPlayer*` globals the game plays with, which are not quite the
//! file:
//!
//! - `LoadPlayer` reads like Blitz, with zeros past the end of the file and
//!   no check of the first string. A file `CreateNewPlayer` wrote stops after
//!   the world packs, so the perfect flags of levels 0 to 3 load as the
//!   stored world packs (0, 10, 20, 30) and the world packs as 0, which
//!   `LoadPlayer` turns back into 0, 10, 20, 30. Levels 1 to 3 then count as
//!   perfect as soon as they have a score; the port keeps that.
//! - `SavePlayer` recounts the completed levels and the total score, and
//!   stores the current world as its pack's position before writing.
//!
//! One difference from the original: `CreateNewPlayer` leaves the perfect
//! flags of the player loaded before in memory, so a new player could
//! inherit them; [`Profile::new`] starts with none.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use wonderland_formats::save::{
    ExtraLevel, LEVEL_COUNT, Perfect, PlayerFile, Progress, RtwpProgress,
};

/// The first string of an RTWP player file.
pub const MAGIC: &str = "Stinky & Loof Player File v2";

/// `CurrentPlayerName$` when none was typed.
pub const ANONYMOUS: &str = "ANONYMOUS";

/// Player files the select screen lists at most (`Dim PlayerName$(10)`
/// holds ten).
pub const MAX_PLAYERS: usize = 10;

/// Custom level records that fit `Dim CurrentPlayerExtraLevelID(30000)`.
const MAX_EXTRA: usize = 30_001;

/// A custom level the player has won (`CurrentPlayerExtraLevel*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtraRecord {
    pub id: i32,
    pub id2: i32,
    pub score: i32,
    pub perfect: i32,
}

/// The player being played: `CurrentPlayerName$` and the other
/// `CurrentPlayer*` globals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    /// As stored in the file; it names the file it is saved to.
    pub name: String,
    pub id: i32,
    /// `CurrentWorld`.
    pub world: i32,
    /// `CurrentPlayerLevelScore`: 0 for a level not won yet.
    pub scores: Vec<i32>,
    /// `CurrentPlayerLevelPerfect`: 1 once the level was won with every
    /// bonus coin (other values come from the short new-player file).
    pub perfect: Vec<i32>,
    pub extra: Vec<ExtraRecord>,
    /// `CurrentWorldPack`: the world last played in each group of ten.
    pub world_packs: [i32; 4],
}

/// Reads like Blitz's file streams: zeros past the end.
struct Lenient<'a> {
    data: &'a [u8],
    pos: usize,
}

impl Lenient<'_> {
    fn int(&mut self) -> i32 {
        let mut b = [0u8; 4];
        for (i, x) in b.iter_mut().enumerate() {
            *x = self.data.get(self.pos + i).copied().unwrap_or(0);
        }
        self.pos += 4;
        i32::from_le_bytes(b)
    }

    fn string(&mut self) -> String {
        let len = self.int().max(0) as usize;
        let start = self.pos.min(self.data.len());
        let end = self.pos.saturating_add(len).min(self.data.len());
        self.pos = self.pos.saturating_add(len);
        self.data[start..end]
            .iter()
            .map(|&b| char::from(b))
            .collect()
    }

    fn skip_ints(&mut self, n: usize) {
        self.pos += 4 * n;
    }
}

/// The player file's name for `name`: `Lower$(name)+".pla"`.
pub fn file_name(name: &str) -> String {
    format!("{}.pla", name.to_ascii_lowercase())
}

impl Profile {
    /// `CreateNewPlayer`'s player, without the file: world 0, no scores,
    /// the world packs at 0, 10, 20, 30. The id adds the name's character
    /// codes to `MilliSecs()` before an empty name becomes
    /// [`ANONYMOUS`].
    pub fn new(name: &str, millisecs: i32) -> Self {
        let id = name
            .chars()
            .fold(0i32, |id, c| id.wrapping_add(c as i32))
            .wrapping_add(millisecs);
        let name = if name.is_empty() { ANONYMOUS } else { name };
        Self {
            name: name.to_owned(),
            id,
            world: 0,
            scores: vec![0; LEVEL_COUNT],
            perfect: vec![0; LEVEL_COUNT],
            extra: Vec::new(),
            world_packs: [0, 10, 20, 30],
        }
    }

    /// The file `CreateNewPlayer` writes: the header, the scores, no
    /// custom levels and the world packs, but no perfect flags.
    pub fn new_file_bytes(&self) -> Vec<u8> {
        PlayerFile {
            magic: MAGIC.to_owned(),
            name: self.name.clone(),
            id: Some(self.id),
            position: self.world,
            levels_completed: 0,
            total_score: 0,
            progress: Progress::Rtwp(RtwpProgress {
                scores: self.scores.clone(),
                extra_levels: Vec::new(),
                perfect: None,
                world_packs: self.world_packs,
            }),
            trailing: Vec::new(),
        }
        .to_bytes()
    }

    /// `LoadPlayer` on a file's bytes. Fails only on a custom level count
    /// the original's arrays cannot hold.
    pub fn load(data: &[u8]) -> Result<Self, String> {
        let mut r = Lenient { data, pos: 0 };
        r.string(); // "just the player file message"
        let name = r.string();
        let id = r.int();
        let world = r.int();
        r.skip_ints(2); // levels completed, total score
        let scores = (0..LEVEL_COUNT).map(|_| r.int()).collect();
        let count = r.int().max(0) as usize;
        if count > MAX_EXTRA {
            return Err(format!("{count} custom levels"));
        }
        let mut extra: Vec<ExtraRecord> = (0..count)
            .map(|_| ExtraRecord {
                id: r.int(),
                id2: r.int(),
                score: r.int(),
                perfect: 0,
            })
            .collect();
        let perfect = (0..LEVEL_COUNT).map(|_| r.int()).collect();
        for e in &mut extra {
            e.perfect = r.int();
        }
        let mut world_packs = [0; 4];
        for (i, p) in world_packs.iter_mut().enumerate() {
            *p = r.int();
            if i > 0 && *p == 0 {
                *p = i as i32 * 10;
            }
        }
        Ok(Self {
            name,
            id,
            world,
            scores,
            perfect,
            extra,
            world_packs,
        })
    }

    /// `SavePlayer`: the file it writes. Stores the current world as its
    /// group's world pack first, as the original does.
    pub fn save_bytes(&mut self) -> Vec<u8> {
        let pack = match self.world {
            ..10 => Some(0),
            10..20 => Some(1),
            20..30 => Some(2),
            30..40 => Some(3),
            _ => None,
        };
        if let Some(p) = pack {
            self.world_packs[p] = self.world;
        }
        let mut file = PlayerFile {
            magic: MAGIC.to_owned(),
            name: self.name.clone(),
            id: Some(self.id),
            position: self.world,
            levels_completed: 0,
            total_score: 0,
            progress: Progress::Rtwp(RtwpProgress {
                scores: self.scores.clone(),
                extra_levels: self
                    .extra
                    .iter()
                    .map(|e| ExtraLevel {
                        id: e.id,
                        id2: e.id2,
                        score: e.score,
                    })
                    .collect(),
                perfect: Some(Perfect {
                    levels: self.perfect.clone(),
                    extra: self.extra.iter().map(|e| e.perfect).collect(),
                }),
                world_packs: self.world_packs,
            }),
            trailing: Vec::new(),
        };
        (file.levels_completed, file.total_score) = file.computed_summary();
        file.to_bytes()
    }

    /// `CurrentLevel=CurrentWorld*10`: where play starts after loading.
    pub fn first_level(&self) -> i32 {
        self.world.wrapping_mul(10)
    }

    /// The best score for main-game level `level`, if it has been won
    /// (what the level-complete sign compares with).
    pub fn record(&self, level: i32) -> Option<i32> {
        let s = *self.scores.get(usize::try_from(level).ok()?)?;
        (s > 0).then_some(s)
    }

    /// A main-game level won (`CurrentLevel<>-999` in `ControlCoins`):
    /// keeps the better score, and marks the level perfect when no bonus
    /// coin was left. The caller saves the profile then.
    pub fn record_win(&mut self, level: i32, score: i32, perfect: bool) {
        let Ok(i) = usize::try_from(level) else {
            return;
        };
        if let Some(best) = self.scores.get_mut(i)
            && score > *best
        {
            *best = score;
        }
        if let Some(p) = self.perfect.get_mut(i)
            && *p == 0
            && perfect
        {
            *p = 1;
        }
    }
}

/// A player on the select screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// `PlayerName$`: the file name without `.pla`, as on disk.
    pub file_stem: String,
    pub path: PathBuf,
    /// `PlayerLevels`, `PlayerScore`, `PlayerExtraLevels`: as stored.
    pub levels: i32,
    pub score: i32,
    pub extra_levels: i32,
}

/// The `players` folder in the game folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Players {
    pub dir: PathBuf,
}

impl Players {
    pub fn new(game_dir: &Path) -> Self {
        Self {
            dir: game_dir.join("players"),
        }
    }

    /// `StartMenu(2)`'s list: creates the folder if needed, takes the
    /// first ten `.pla` files in directory order (sorted ignoring case,
    /// as Windows lists them), and reads each one's header. A file that
    /// is not an RTWP v2 player file is renamed to `.old` and left out.
    ///
    /// The original, after renaming one, reads the same name again from
    /// the deleted file and drops the last player from the list; here the
    /// players after it simply move up.
    pub fn list(&self) -> io::Result<Vec<Entry>> {
        if !self.dir.is_dir() {
            fs::create_dir_all(&self.dir)?;
        }
        let mut files: Vec<(String, PathBuf)> = fs::read_dir(&self.dir)?
            .filter_map(|e| {
                let e = e.ok()?;
                let name = e.file_name().into_string().ok()?;
                Some((name, e.path()))
            })
            .collect();
        files.sort_by_key(|(n, _)| n.to_uppercase());
        let stems: Vec<(String, PathBuf)> = files
            .into_iter()
            .filter_map(|(n, p)| {
                let stem = n
                    .len()
                    .checked_sub(4)
                    .filter(|&i| n.is_char_boundary(i) && n[i..].eq_ignore_ascii_case(".pla"))
                    .map(|i| n[..i].to_owned())?;
                Some((stem, p))
            })
            .take(MAX_PLAYERS)
            .collect();

        let mut out = Vec::new();
        for (stem, path) in stems {
            let Ok(data) = fs::read(&path) else {
                continue;
            };
            let mut r = Lenient {
                data: &data,
                pos: 0,
            };
            if r.string() != MAGIC {
                // "uh oh! old player file"
                let old = self.dir.join(format!("{stem}.old"));
                fs::copy(&path, old)?;
                fs::remove_file(&path)?;
                continue;
            }
            r.string(); // name
            r.skip_ints(2); // id, world
            let levels = r.int();
            let score = r.int();
            r.skip_ints(LEVEL_COUNT);
            out.push(Entry {
                file_stem: stem,
                path,
                levels,
                score,
                extra_levels: r.int(),
            });
        }
        Ok(out)
    }

    /// `LoadPlayer` of a listed player.
    pub fn load(&self, entry: &Entry) -> Result<Profile, String> {
        let data = fs::read(&entry.path).map_err(|e| format!("{}: {e}", entry.path.display()))?;
        Profile::load(&data).map_err(|e| format!("{}: {e}", entry.path.display()))
    }

    /// `CreateNewPlayer`: keeps a `.bak` of a file of the same name, then
    /// writes the new player's file.
    pub fn create(&self, profile: &Profile) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let path = self.dir.join(file_name(&profile.name));
        if path.is_file() {
            fs::copy(&path, path.with_extension("bak"))?;
        }
        fs::write(path, profile.new_file_bytes())
    }

    /// `SavePlayer`.
    pub fn save(&self, profile: &mut Profile) -> io::Result<()> {
        let bytes = profile.save_bytes();
        fs::write(self.dir.join(file_name(&profile.name)), bytes)
    }

    /// `MenuLoop6`'s YES: copies the file to `.bak`, then deletes it.
    pub fn delete(&self, entry: &Entry) -> io::Result<()> {
        let bak = self.dir.join(format!("{}.bak", entry.file_stem));
        fs::copy(&entry.path, bak)?;
        fs::remove_file(&entry.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("wl-profile-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn new_player_id_and_anonymous() {
        let p = Profile::new("AB", 1000);
        assert_eq!(p.id, 65 + 66 + 1000);
        assert_eq!(p.name, "AB");
        let p = Profile::new("", 7);
        assert_eq!((p.name.as_str(), p.id), (ANONYMOUS, 7));
    }

    #[test]
    fn short_file_loads_with_the_world_packs_as_perfect_flags() {
        let p = Profile::new("ZED", 5);
        let bytes = p.new_file_bytes();
        let parsed = PlayerFile::parse(&bytes, wonderland_formats::Game::Rtwp).unwrap();
        assert!(matches!(&parsed.progress, Progress::Rtwp(r) if r.perfect.is_none()));
        let l = Profile::load(&bytes).unwrap();
        assert_eq!(l.name, "ZED");
        assert_eq!(l.id, 90 + 69 + 68 + 5);
        assert_eq!(&l.perfect[..5], &[0, 10, 20, 30, 0]);
        assert_eq!(l.world_packs, [0, 10, 20, 30]);
    }

    #[test]
    fn save_recounts_and_stores_the_world_pack() {
        let mut p = Profile::new("ZED", 5);
        p.world = 12;
        p.record_win(3, 500, true);
        p.record_win(3, 400, false);
        p.record_win(7, 250, false);
        p.extra.push(ExtraRecord {
            id: 1,
            id2: 2,
            score: 50,
            perfect: 1,
        });
        let bytes = p.save_bytes();
        assert_eq!(p.world_packs, [0, 12, 20, 30]);
        let f = PlayerFile::parse(&bytes, wonderland_formats::Game::Rtwp).unwrap();
        assert_eq!((f.levels_completed, f.total_score), (2, 800));
        assert_eq!(Profile::load(&bytes).unwrap(), p);
        assert_eq!(p.record(3), Some(500));
        assert_eq!(p.record(4), None);
        assert_eq!(p.perfect[3], 1);
        assert_eq!(p.perfect[7], 0);
    }

    #[test]
    fn players_folder_lists_creates_and_deletes() {
        let game = temp_dir("folder");
        let players = Players::new(&game);
        assert!(players.list().unwrap().is_empty());
        assert!(players.dir.is_dir());

        let mut bob = Profile::new("Bob", 1);
        players.create(&bob).unwrap();
        players.create(&Profile::new("ann", 2)).unwrap();
        fs::write(
            players.dir.join("old.pla"),
            b"\x19\0\0\0Stinky & Loof Player File",
        )
        .unwrap();
        fs::write(players.dir.join("notes.txt"), b"x").unwrap();

        bob.record_win(0, 120, false);
        players.save(&mut bob).unwrap();

        let list = players.list().unwrap();
        let stems: Vec<&str> = list.iter().map(|e| e.file_stem.as_str()).collect();
        assert_eq!(stems, ["ann", "bob"]);
        assert_eq!((list[1].levels, list[1].score), (1, 120));
        assert!(players.dir.join("old.old").is_file());
        assert!(!players.dir.join("old.pla").exists());

        assert_eq!(players.load(&list[1]).unwrap(), bob);

        // Creating over an existing player keeps a backup.
        players.create(&Profile::new("BOB", 3)).unwrap();
        assert!(players.dir.join("bob.bak").is_file());

        players.delete(&list[0]).unwrap();
        assert!(players.dir.join("ann.bak").is_file());
        assert_eq!(players.list().unwrap().len(), 1);
        fs::remove_dir_all(&game).unwrap();
    }

    #[test]
    fn lists_at_most_ten() {
        let game = temp_dir("ten");
        let players = Players::new(&game);
        for i in 0..12 {
            players
                .create(&Profile::new(&format!("P{i:02}"), i))
                .unwrap();
        }
        let list = players.list().unwrap();
        assert_eq!(list.len(), MAX_PLAYERS);
        assert_eq!(list[9].file_stem, "p09");
        fs::remove_dir_all(&game).unwrap();
    }
}
