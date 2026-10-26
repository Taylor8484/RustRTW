//! Finding and loading a level from a game's data folder.

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use wonderland_formats::config::Options as ConfigOptions;
use wonderland_formats::{Game, GameAssets, Level};
use wonderland_logic::{Options, Ruleset, Setup};

/// The game's data and settings for the session.
#[derive(Resource)]
pub struct GameData {
    pub assets: GameAssets,
    pub ruleset: Ruleset,
    pub options: Options,
    /// `OptionsTeleport`: 1 draws every teleporter with the generic texture.
    pub teleport_option: i32,
    /// `OptionsSoundLevel` and `OptionsMusicLevel`, 0–1.
    pub sound_volume: f32,
    pub music_volume: f32,
}

impl GameData {
    /// Opens `dir` (e.g. `assets/RTWP`) and reads its `options.cfg` if
    /// there is one.
    pub fn open(dir: &Path) -> Result<Self, String> {
        let assets = GameAssets::open(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let config = assets
            .read("options.cfg")
            .ok()
            .and_then(|d| ConfigOptions::parse(Game::Rtwp, &d).ok());
        let mut options = Options::default();
        let mut teleport_option = 0;
        let (mut sound_volume, mut music_volume) = (1.0, 1.0);
        if let Some(c) = config {
            sound_volume = c.sound_level as f32 / 100.0;
            music_volume = c.music_level as f32 / 100.0;
            options.detail_level = c.detail_level.clamp(1, 5);
            options.no_time_limit = c.no_time_limit != 0;
            options.language = c.language;
            options.sound = c.sound_level > 0;
            teleport_option = c.teleport;
        }
        Ok(Self {
            assets,
            ruleset: Ruleset::rtwp(),
            options,
            teleport_option,
            sound_volume,
            music_volume,
        })
    }

    /// The level files in the pack, as the pack names them.
    pub fn level_names(&self) -> Vec<String> {
        level_names(&self.assets)
    }

    /// Finds a level by file name, with or without its folder and
    /// extension (`intro`, `intro.lev`, `levels\intro.lev`).
    pub fn find_level(&self, name: &str) -> Option<String> {
        let want = stem(name);
        self.level_names().into_iter().find(|n| stem(n) == want)
    }

    pub fn load_level(&self, name: &str) -> Result<LevelData, String> {
        let data = self.assets.read(name).map_err(|e| format!("{name}: {e}"))?;
        let level = Level::parse(&data, Game::Rtwp).map_err(|e| format!("{name}: {e}"))?;
        let setup = Setup::load(&level, &self.ruleset, self.options.load)
            .map_err(|e| format!("{name}: {e}"))?;
        Ok(LevelData {
            name: name.to_owned(),
            level,
            setup,
        })
    }
}

/// The level being played: the file and how the loaders left it, before
/// any object moved. The scene is built from `setup`.
#[derive(Resource, Clone)]
pub struct LevelData {
    pub name: String,
    pub level: Level,
    pub setup: Setup,
}

/// Every `.lev`/`.lv5` file in the pack.
pub fn level_names(assets: &GameAssets) -> Vec<String> {
    let Some(pack) = assets.pack() else {
        return Vec::new();
    };
    pack.index()
        .entries()
        .iter()
        .map(|e| e.name.clone())
        .filter(|n| {
            let n = n.to_lowercase();
            n.ends_with(".lev") || n.ends_with(".lv5")
        })
        .collect()
}

fn stem(name: &str) -> String {
    let file = name.rsplit(['\\', '/']).next().unwrap_or(name);
    let file = file.rsplit_once('.').map_or(file, |(s, _)| s);
    file.to_lowercase()
}

/// The RTWP data folder: `$WL_ASSETS/RTWP`, or `assets/RTWP` under the
/// current directory.
pub fn default_assets_dir() -> PathBuf {
    let root = std::env::var_os("WL_ASSETS").map_or_else(|| PathBuf::from("assets"), PathBuf::from);
    root.join("RTWP")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stems_ignore_folder_extension_and_case() {
        assert_eq!(stem(r"levels\Intro.LEV"), "intro");
        assert_eq!(stem("intro"), "intro");
        assert_eq!(stem("a/b.lv5"), "b");
    }
}
