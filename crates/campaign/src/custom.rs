//! RTWP's custom levels: the `CustomLevels` folder in the game folder, as
//! `StartMenu(14)` lists it for the custom level menu.
//!
//! A listing holds, in order: the entry that goes back up (`..`), the
//! folders sorted by name, then the level files. A level file is listed
//! when its name ends in `.LEV`, `.LV5` or `.LV6` and its first 20 bytes
//! add up to 1380, which every `Stinky & Loof Level File vN` magic does.
//! Names are listed in upper case (`Upper$(NextFile$(Dir))`), and each
//! level's title, map id and the player's score for it come from its
//! header and the profile.
//!
//! The original walks the folder in the order Windows gives, which on
//! NTFS is by name ignoring case; the port sorts by the upper-case name to
//! get the same order everywhere. Folders are found ignoring case, since
//! the names kept are upper case.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::profile::{ExtraRecord, Lenient};

/// The folder the levels are in, under the game folder.
pub const FOLDER: &str = "CustomLevels";

/// `CurrentLevel` while a custom level is played.
pub const LEVEL: i32 = -999;

/// `FileDirScore` of the entry that goes back up.
pub const BACK: i32 = -1;

/// `FileDirScore` of a folder.
pub const DIR: i32 = -2;

/// Level files a folder may hold: at 4000 the original stops with "TOO
/// MANY LEVEL FILES! SORT INTO DIRECTORIES!".
pub const MAX_FILES: usize = 4000;

/// One line of the custom level menu (`FileDir$`, `FileDir2$`,
/// `FileDirID`, `FileDirID2`, `FileDirScore`, `FileDirPerfect`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    /// The name in upper case, `..` for the way back.
    pub name: String,
    /// The level's title, `(Folder)` or `(Go Back)`.
    pub title: String,
    /// The map id in the level's header.
    pub id: i32,
    /// A sum over the title and stored file name, kept with the score.
    pub id2: i32,
    /// The player's score: 0 for a level not won, [`BACK`] or [`DIR`].
    pub score: i32,
    pub perfect: i32,
}

impl FileEntry {
    /// A level that Enter plays.
    pub fn is_level(&self) -> bool {
        self.score >= 0 && is_level_name(&self.name)
    }
}

/// The name ends in `.LEV`, `.LV5` or `.LV6`, in any case.
pub fn is_level_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    [".LEV", ".LV5", ".LV6"].iter().any(|e| upper.ends_with(e))
}

/// The first 20 bytes (zeros past the end) add up to 1380.
pub fn header_ok(data: &[u8]) -> bool {
    let sum: u32 = (0..20)
        .map(|i| u32::from(data.get(i).copied().unwrap_or(0)))
        .sum();
    sum == 1380
}

/// `FileDirID2`: each character code of the title, then of the stored
/// file name for as many characters as the listed name has (`Asc("")` is
/// -1 past its end), raised to the power `i Mod 3 + 1` for the `i`th
/// character, added up in Blitz floats.
pub fn id2(title: &str, stored_name: &str, listed_name: &str) -> i32 {
    // `Int + Float` is a float, rounded back into the int.
    let add = |sum: i32, code: i32, i: usize| -> i32 {
        let power = (code as f32).powi((i % 3 + 1) as i32);
        (sum as f32 + power).round() as i32
    };
    let mut sum = 0;
    for (i, c) in title.chars().enumerate() {
        sum = add(sum, c as i32 & 255, i + 1);
    }
    let stored: Vec<char> = stored_name.chars().collect();
    for i in 1..=listed_name.chars().count() {
        let code = stored.get(i - 1).map_or(-1, |&c| c as i32 & 255);
        sum = add(sum, code, i);
    }
    sum
}

/// The first line of every listing, which goes back up.
pub fn back() -> FileEntry {
    FileEntry {
        name: "..".to_owned(),
        title: "(Go Back)".to_owned(),
        id: 0,
        id2: 0,
        score: BACK,
        perfect: 0,
    }
}

/// The `CustomLevels` folder of a game folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomLevels {
    pub dir: PathBuf,
}

impl CustomLevels {
    pub fn new(game_dir: &Path) -> Self {
        Self {
            dir: game_dir.join(FOLDER),
        }
    }

    /// The game path of `name` in the folder `dirs` (upper-case names, as
    /// listed): `CustomLevels\<dirs>\<name>`.
    pub fn game_path(dirs: &[String], name: &str) -> String {
        let mut path = String::from(FOLDER);
        for d in dirs.iter().map(String::as_str).chain([name]) {
            path.push('\\');
            path.push_str(d);
        }
        path
    }

    /// `StartMenu(14)`'s listing of the folder `dirs`, with the scores
    /// from `extras`. `by_name` is `CustomSortByName`: off, the levels
    /// keep the folder's order, which here is the same. A folder that is
    /// not there lists only the way back.
    pub fn list(
        &self,
        dirs: &[String],
        by_name: bool,
        extras: &[ExtraRecord],
    ) -> io::Result<Vec<FileEntry>> {
        let names = match self.resolve(dirs) {
            Some(dir) => sorted_entries(&dir)?,
            None => Vec::new(),
        };
        let mut levels: Vec<FileEntry> = Vec::new();
        for (name, path) in names.iter().filter(|(_, p)| p.is_file()) {
            if !is_level_name(name) {
                continue;
            }
            let data = fs::read(path)?;
            if !header_ok(&data) {
                continue;
            }
            if levels.len() >= MAX_FILES {
                return Err(io::Error::other(
                    "TOO MANY LEVEL FILES! SORT INTO DIRECTORIES!",
                ));
            }
            let mut r = Lenient::new(&data);
            let _magic = r.string();
            let stored_name = r.string();
            let id = r.int();
            let title = r.string();
            let (score, perfect) = extras
                .iter()
                .find(|e| e.id == id)
                .map_or((0, 0), |e| (e.score, e.perfect));
            let at = levels
                .iter()
                .position(|l| by_name && name.as_bytes() <= l.name.as_bytes())
                .unwrap_or(levels.len());
            let entry = FileEntry {
                id2: id2(&title, &stored_name, name),
                name: name.clone(),
                title,
                id,
                score,
                perfect,
            };
            levels.insert(at, entry);
        }
        let mut list = vec![back()];
        // The folders, each put before the first one it sorts before.
        let mut folders: Vec<FileEntry> = Vec::new();
        for (name, _) in names.iter().filter(|(_, p)| p.is_dir()) {
            let at = folders
                .iter()
                .position(|f| name.as_bytes() <= f.name.as_bytes())
                .unwrap_or(folders.len());
            let folder = FileEntry {
                name: name.clone(),
                title: "(Folder)".to_owned(),
                id: 0,
                id2: 0,
                score: DIR,
                perfect: 0,
            };
            folders.insert(at, folder);
        }
        list.extend(folders);
        list.extend(levels);
        Ok(list)
    }

    /// The folder `dirs` below `CustomLevels`, each name found ignoring
    /// case.
    fn resolve(&self, dirs: &[String]) -> Option<PathBuf> {
        let mut dir = self.dir.clone();
        for d in dirs {
            dir = fs::read_dir(&dir)
                .ok()?
                .filter_map(Result::ok)
                .find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case(d))?
                .path();
        }
        dir.is_dir().then_some(dir)
    }
}

/// A folder's entries as upper-case names, in name order.
fn sorted_entries(dir: &Path) -> io::Result<Vec<(String, PathBuf)>> {
    let mut names: Vec<(String, PathBuf)> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_ascii_uppercase();
            (name, e.path())
        })
        .collect();
    names.sort();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level_bytes(magic: &str, file_name: &str, id: i32, title: &str) -> Vec<u8> {
        let mut b = Vec::new();
        let string = |b: &mut Vec<u8>, s: &str| {
            b.extend((s.len() as u32).to_le_bytes());
            b.extend(s.as_bytes());
        };
        string(&mut b, magic);
        string(&mut b, file_name);
        b.extend(id.to_le_bytes());
        string(&mut b, title);
        b
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("wonderland-custom-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(FOLDER)).unwrap();
        dir
    }

    #[test]
    fn level_magics_pass_the_header_check() {
        for v in 1..=6 {
            let magic = format!("Stinky & Loof Level File v{v}");
            assert!(header_ok(&level_bytes(&magic, "a.lev", 1, "A")));
        }
        assert!(!header_ok(b"not a level"));
        assert!(!header_ok(&[]));
    }

    #[test]
    fn id2_sums_powers_and_reads_past_the_stored_name() {
        // "AB": 65^2 + 66^3; then the stored name "a" for the two
        // characters of "XY": 97^2, then Asc("") = -1, cubed.
        let want = 65 * 65 + 66 * 66 * 66 + 97 * 97 - 1;
        assert_eq!(id2("AB", "a", "XY"), want);
        assert_eq!(id2("", "", ""), 0);
    }

    #[test]
    fn lists_back_folders_then_levels_with_scores() {
        let game = temp_dir("list");
        let root = game.join(FOLDER);
        let magic = "Stinky & Loof Level File v5";
        fs::write(root.join("zed.lv5"), level_bytes(magic, "zed", 7, "Zed")).unwrap();
        fs::write(
            root.join("Alpha.lev"),
            level_bytes(magic, "alpha", 3, "Alpha"),
        )
        .unwrap();
        fs::write(root.join("junk.lev"), b"junk").unwrap();
        fs::write(root.join("notes.txt"), b"x").unwrap();
        fs::create_dir(root.join("pack b")).unwrap();
        fs::create_dir(root.join("Pack A")).unwrap();
        fs::write(
            root.join("Pack A").join("inner.lev"),
            level_bytes(magic, "inner", 9, "Inner"),
        )
        .unwrap();

        let custom = CustomLevels::new(&game);
        let extras = [ExtraRecord {
            id: 7,
            id2: 0,
            score: 1234,
            perfect: 1,
        }];
        let list = custom.list(&[], true, &extras).unwrap();
        let names: Vec<_> = list.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["..", "PACK A", "PACK B", "ALPHA.LEV", "ZED.LV5"]);
        assert_eq!((list[0].score, list[1].score), (BACK, DIR));
        assert_eq!(list[1].title, "(Folder)");
        assert_eq!(
            (list[3].title.as_str(), list[3].id, list[3].score),
            ("Alpha", 3, 0)
        );
        assert_eq!((list[4].score, list[4].perfect), (1234, 1));
        assert!(list[4].is_level() && !list[1].is_level() && !list[0].is_level());

        let inner = custom.list(&["PACK A".to_owned()], true, &[]).unwrap();
        let names: Vec<_> = inner.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["..", "INNER.LEV"]);
        assert_eq!(
            CustomLevels::game_path(&["PACK A".to_owned()], "INNER.LEV"),
            "CustomLevels\\PACK A\\INNER.LEV"
        );

        let missing = custom.list(&["NONE".to_owned()], true, &[]).unwrap();
        assert_eq!(missing.len(), 1);
        fs::remove_dir_all(game).unwrap();
    }
}
