//! Path resolution across a game's pack and loose files.
//!
//! The original games load every asset through helpers (`MyLoadTexture`,
//! `MyReadFile`, ...) that try the pack first and fall back to a file of the
//! same path in the game folder. Paths are written Windows-style
//! (`graphics\rainbow.jpg`) and matched case-insensitively, so loose files are
//! looked up case-insensitively here too, which matters on Linux and macOS.

use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::io_error;
use crate::pack::{self, Pack};
use crate::{Error, Result};

/// One game's data folder (e.g. `assets/RTWP`).
#[derive(Debug, Clone)]
pub struct GameAssets {
    dir: PathBuf,
    pack: Option<Pack>,
}

impl GameAssets {
    /// Opens a game folder, loading its pack if `res1.dat` is present.
    pub fn open(dir: impl Into<PathBuf>) -> Result<Self> {
        let dir = dir.into();
        if !dir.is_dir() {
            return Err(Error::NotFound(dir.display().to_string()));
        }
        let pack = if find_case_insensitive(&dir, pack::INDEX_FILE).is_some() {
            Some(Pack::open(&dir)?)
        } else {
            None
        };
        Ok(Self { dir, pack })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn pack(&self) -> Option<&Pack> {
        self.pack.as_ref()
    }

    /// Reads a game file: from the pack if it has one, otherwise from disk.
    pub fn read(&self, path: &str) -> Result<Cow<'_, [u8]>> {
        if let Some(data) = self.pack.as_ref().and_then(|p| p.read(path)) {
            return Ok(Cow::Borrowed(data));
        }
        let disk_path = self
            .resolve(path)
            .ok_or_else(|| Error::NotFound(path.to_owned()))?;
        fs::read(&disk_path)
            .map(Cow::Owned)
            .map_err(io_error(disk_path))
    }

    /// True if `path` exists in the pack or as a loose file.
    pub fn exists(&self, path: &str) -> bool {
        self.pack
            .as_ref()
            .is_some_and(|p| p.index().get(path).is_some())
            || self.resolve(path).is_some()
    }

    /// Finds a loose file under the game folder, matching each path component
    /// case-insensitively.
    pub fn resolve(&self, path: &str) -> Option<PathBuf> {
        let mut current = self.dir.clone();
        for part in path.split(['\\', '/']).filter(|p| !p.is_empty()) {
            current = find_case_insensitive(&current, part)?;
        }
        current.is_file().then_some(current)
    }
}

fn find_case_insensitive(dir: &Path, name: &str) -> Option<PathBuf> {
    let exact = dir.join(name);
    if exact.exists() {
        return Some(exact);
    }
    fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(name)
        })
        .map(|entry| entry.path())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_game_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("wonderland-formats-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("Levels")).unwrap();
        dir
    }

    #[test]
    fn falls_back_to_loose_files_case_insensitively() {
        let dir = temp_game_dir("loose");
        fs::write(dir.join("Levels").join("Intro.LEV"), b"level").unwrap();
        let assets = GameAssets::open(&dir).unwrap();
        assert!(assets.pack().is_none());
        assert_eq!(&*assets.read("levels\\intro.lev").unwrap(), b"level");
        assert!(assets.exists("LEVELS/INTRO.lev"));
        assert!(matches!(
            assets.read("levels\\other.lev"),
            Err(Error::NotFound(_))
        ));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn prefers_the_pack_over_loose_files() {
        let dir = temp_game_dir("pack");
        let name = "levels\\intro.lev";
        let mut index = vec![];
        index.extend((name.len() as u32).to_le_bytes());
        index.extend(name.as_bytes());
        index.extend(0i32.to_le_bytes());
        index.extend(6i32.to_le_bytes());
        fs::write(dir.join("res1.dat"), index).unwrap();
        fs::write(dir.join("res2.dat"), b"packed").unwrap();
        fs::write(dir.join("Levels").join("intro.lev"), b"loose").unwrap();
        let assets = GameAssets::open(&dir).unwrap();
        assert_eq!(&*assets.read("levels/intro.lev").unwrap(), b"packed");
        fs::remove_dir_all(dir).unwrap();
    }
}
