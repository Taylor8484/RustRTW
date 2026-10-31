//! Level packs of the player's own: a folder in `CustomLevels`, which both
//! this game and the original already list. The levels play in name order, so
//! each file's name starts with its place in the pack (`03TOWER.LV6`) and
//! changing the order renames the files. Nothing else is stored, so a pack is
//! shared by sharing the folder.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::custom::{SOLUTION, is_level_name};

/// Two digits of place, and `SaveLevel`'s longest name.
pub const MAX_LEVELS: usize = 99;
const MAX_STEM: usize = 15 - 2;

/// The packs in the `CustomLevels` folder `dir`, by name.
pub fn packs(dir: &Path) -> io::Result<Vec<String>> {
    let mut names: Vec<String> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort_by_key(|n| n.to_ascii_uppercase());
    Ok(names)
}

/// The solution saved beside the level at `path`.
pub fn solution_path(path: &Path) -> PathBuf {
    path.with_extension(SOLUTION)
}

/// Moves a level and its solution.
fn move_level(from: &Path, to: &Path) -> io::Result<()> {
    fs::rename(from, to)?;
    let solution = solution_path(from);
    if solution.exists() {
        fs::rename(solution, solution_path(to))?;
    }
    Ok(())
}

/// `name` without its place in a pack.
pub fn unplaced(name: &str) -> &str {
    let bytes = name.as_bytes();
    let placed = bytes.len() > 2 && bytes[..2].iter().all(u8::is_ascii_digit);
    if placed { &name[2..] } else { name }
}

/// One folder of levels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pack {
    pub dir: PathBuf,
}

impl Pack {
    /// The pack `name` in the `CustomLevels` folder `dir`.
    pub fn new(dir: &Path, name: &str) -> Self {
        Self {
            dir: dir.join(name),
        }
    }

    /// The level files in play order.
    pub fn levels(&self) -> io::Result<Vec<String>> {
        let mut names: Vec<String> = fs::read_dir(&self.dir)?
            .filter_map(Result::ok)
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| is_level_name(n))
            .collect();
        names.sort_by_key(|n| n.to_ascii_uppercase());
        Ok(names)
    }

    /// Moves the level at `path` to the end of the pack, with its solution.
    /// Returns its new file name.
    pub fn add(&self, path: &Path) -> io::Result<String> {
        let levels = self.levels()?;
        if levels.len() >= MAX_LEVELS {
            return Err(io::Error::other(format!(
                "A pack holds {MAX_LEVELS} levels"
            )));
        }
        let name = path.file_name().and_then(|n| n.to_str());
        let name = name.ok_or_else(|| io::Error::other("not a level file"))?;
        let extension = name.rfind('.').map_or("", |dot| &name[dot..]);
        let stem = &name[..name.len() - extension.len()];
        let stem: String = unplaced(stem).chars().take(MAX_STEM).collect();
        let name = format!("{:02}{stem}{extension}", levels.len() + 1);
        move_level(path, &self.dir.join(&name))?;
        // Taking out an earlier level may have left a gap in the numbers.
        let mut order = levels;
        order.push(name);
        self.order(&order)?;
        Ok(self.levels()?.pop().unwrap_or_default())
    }

    /// Swaps the level at `index` with the one before it, and returns the
    /// new names.
    pub fn move_up(&self, index: usize) -> io::Result<Vec<String>> {
        let mut order = self.levels()?;
        if index == 0 || index >= order.len() {
            return Ok(order);
        }
        order.swap(index, index - 1);
        self.order(&order)?;
        self.levels()
    }

    /// Moves the level at `index` out to `to`'s folder, and numbers the rest
    /// again. Nothing moves if a level of that name is there.
    pub fn take_out(&self, index: usize, to: &Path) -> io::Result<()> {
        let mut order = self.levels()?;
        let name = order.remove(index);
        let target = to.join(unplaced(&name));
        if target.exists() {
            return Err(io::Error::other(format!(
                "{} is there already",
                unplaced(&name)
            )));
        }
        move_level(&self.dir.join(&name), &target)?;
        self.order(&order)
    }

    /// Names the level files `order` for their places in it: first all to
    /// names out of the way, as one level's new name may be another's old one.
    fn order(&self, order: &[String]) -> io::Result<()> {
        let holding = |place: usize| self.dir.join(format!("MOVING{place}.TMP"));
        let mut names = vec![];
        for (place, name) in order.iter().enumerate() {
            let wanted = format!("{:02}{}", place + 1, unplaced(name));
            if wanted != *name {
                move_level(&self.dir.join(name), &holding(place))?;
                names.push((place, wanted));
            }
        }
        for (place, wanted) in names {
            move_level(&holding(place), &self.dir.join(wanted))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(tag: &str) -> (PathBuf, Pack) {
        let dir = std::env::temp_dir().join(format!("wl-pack-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("MINE")).unwrap();
        let pack = Pack::new(&dir, "MINE");
        (dir, pack)
    }

    fn level(dir: &Path, name: &str, solved: bool) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, name).unwrap();
        if solved {
            fs::write(solution_path(&path), name).unwrap();
        }
        path
    }

    #[test]
    fn levels_join_at_the_end_with_their_solutions() {
        let (dir, pack) = folder("add");
        assert_eq!(packs(&dir).unwrap(), ["MINE"]);
        let first = level(&dir, "TOWER.LV6", true);
        assert_eq!(pack.add(&first).unwrap(), "01TOWER.LV6");
        assert!(!first.exists());
        assert_eq!(
            pack.add(&level(&dir, "CAVE.LV6", false)).unwrap(),
            "02CAVE.LV6"
        );
        assert_eq!(pack.levels().unwrap(), ["01TOWER.LV6", "02CAVE.LV6"]);
        assert_eq!(
            fs::read(pack.dir.join("01TOWER.WIN")).unwrap(),
            b"TOWER.LV6"
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_long_name_leaves_room_for_the_place() {
        let (dir, pack) = folder("long");
        let name = pack
            .add(&level(&dir, "ABCDEFGHIJKLMNO.LV6", false))
            .unwrap();
        assert_eq!(name, "01ABCDEFGHIJKLM.LV6");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn moving_up_swaps_places_and_the_contents_stay_with_the_names() {
        let (dir, pack) = folder("order");
        for name in ["A.LV6", "B.LV6", "C.LV6"] {
            pack.add(&level(&dir, name, true)).unwrap();
        }
        let order = pack.move_up(2).unwrap();
        assert_eq!(order, ["01A.LV6", "02C.LV6", "03B.LV6"]);
        assert_eq!(fs::read(pack.dir.join("02C.LV6")).unwrap(), b"C.LV6");
        assert_eq!(fs::read(pack.dir.join("02C.WIN")).unwrap(), b"C.LV6");
        assert_eq!(pack.move_up(0).unwrap(), order);
        assert_eq!(pack.move_up(9).unwrap(), order);
        let leftovers = fs::read_dir(&pack.dir).unwrap().flatten();
        assert!(!leftovers.map(|e| e.path()).any(|p| p.ends_with(".TMP")));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn taking_out_closes_the_gap_and_refuses_to_replace() {
        let (dir, pack) = folder("out");
        for name in ["A.LV6", "B.LV6", "C.LV6"] {
            pack.add(&level(&dir, name, true)).unwrap();
        }
        pack.take_out(0, &dir).unwrap();
        assert!(dir.join("A.LV6").exists() && dir.join("A.WIN").exists());
        assert_eq!(pack.levels().unwrap(), ["01B.LV6", "02C.LV6"]);

        fs::write(dir.join("B.LV6"), "mine").unwrap();
        assert!(pack.take_out(0, &dir).is_err());
        assert_eq!(fs::read(dir.join("B.LV6")).unwrap(), b"mine");
        assert_eq!(pack.levels().unwrap(), ["01B.LV6", "02C.LV6"]);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_full_pack_refuses_and_leaves_the_level() {
        let (dir, pack) = folder("full");
        for n in 1..=MAX_LEVELS {
            fs::write(pack.dir.join(format!("{n:02}X.LV6")), "").unwrap();
        }
        let last = level(&dir, "LAST.LV6", false);
        assert!(pack.add(&last).is_err());
        assert!(last.exists());
        fs::remove_dir_all(&dir).unwrap();
    }
}
