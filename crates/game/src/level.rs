//! Finding and loading a level from a game's data folder.

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use wonderland_campaign::worlds::Content;
use wonderland_formats::config::Options as ConfigOptions;
use wonderland_formats::{Game, GameAssets, Level};
use wonderland_logic::{Options, Ruleset, Setup};

/// The game's data and settings for the session.
#[derive(Resource)]
pub struct GameData {
    pub assets: GameAssets,
    /// TOW's data folder beside RTWP's, for TOW's levels; `None` when
    /// there is none.
    pub tow: Option<GameAssets>,
    pub ruleset: Ruleset,
    pub options: Options,
    /// `OptionsTeleport`: 1 draws every teleporter with the generic texture.
    pub teleport_option: i32,
    /// `OptionsSoundLevel` and `OptionsMusicLevel`, 0–1.
    pub sound_volume: f32,
    pub music_volume: f32,
    /// `options.cfg` as loaded and as the options menu changes it.
    pub config: ConfigOptions,
}

impl GameData {
    /// Opens `dir` (e.g. `assets/RTWP`) and reads its `options.cfg` if
    /// there is one.
    pub fn open(dir: &Path) -> Result<Self, String> {
        let assets = GameAssets::open(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        // `LoadOptions`: the defaults when there is no file.
        let config = assets
            .read("options.cfg")
            .ok()
            .and_then(|d| ConfigOptions::parse(Game::Rtwp, &d).ok())
            .unwrap_or_else(|| ConfigOptions::defaults(Game::Rtwp));
        let tow = ["TOW", "tow"]
            .iter()
            .filter_map(|n| dir.parent().map(|p| p.join(n)))
            .find_map(|d| GameAssets::open(d).ok());
        let mut data = Self {
            assets,
            tow,
            ruleset: Ruleset::rtwp(),
            options: Options::default(),
            teleport_option: 0,
            sound_volume: 1.0,
            music_volume: 1.0,
            config,
        };
        data.apply_config();
        Ok(data)
    }

    /// Takes the settings from [`GameData::config`].
    pub fn apply_config(&mut self) {
        let c = &self.config;
        self.sound_volume = c.sound_level as f32 / 100.0;
        self.music_volume = c.music_level as f32 / 100.0;
        self.options.detail_level = c.detail_level.clamp(1, 5);
        self.options.no_time_limit = c.no_time_limit != 0;
        self.options.language = c.language;
        self.options.sound = c.sound_level > 0;
        self.teleport_option = c.teleport;
    }

    /// `SaveOptions`: writes all twelve values to `options.cfg` in the
    /// game folder.
    pub fn save_config(&mut self) -> Result<(), String> {
        self.config.present = ConfigOptions::int_count(Game::Rtwp);
        self.config.trailing.clear();
        let path = self.assets.dir().join("options.cfg");
        std::fs::write(&path, self.config.to_bytes())
            .map_err(|e| format!("{}: {e}", path.display()))
    }

    /// The level files in `content`'s pack, as the pack names them.
    pub fn level_names(&self, content: Content) -> Vec<String> {
        match content {
            Content::Rtwp => level_names(&self.assets),
            Content::Tow => self.tow.as_ref().map(level_names).unwrap_or_default(),
        }
    }

    /// Finds a level by file name, with or without its folder and
    /// extension (`intro`, `intro.lev`, `levels\intro.lev`): in RTWP's
    /// pack, else in TOW's, else as a file under the game folder
    /// (`CustomLevels\my.lv5`).
    pub fn find_level(&self, name: &str) -> Option<(Content, String)> {
        let want = stem(name);
        [Content::Rtwp, Content::Tow]
            .into_iter()
            .find_map(|c| {
                let found = self.level_names(c).into_iter().find(|n| stem(n) == want);
                found.map(|n| (c, n))
            })
            .or_else(|| {
                self.assets
                    .resolve(name)
                    .map(|_| (Content::Rtwp, name.to_owned()))
            })
    }

    /// Loads one of RTWP's levels.
    pub fn load_level(&self, name: &str) -> Result<LevelData, String> {
        self.load_content_level(Content::Rtwp, name)
    }

    /// Reads a level file from `content`'s folder. TOW's bonus levels
    /// came as a separate download, so a TOW level missing from
    /// `levels\` is also looked for in the folder itself.
    pub fn read_level(&self, content: Content, name: &str) -> Result<Vec<u8>, String> {
        let assets = match content {
            Content::Rtwp => &self.assets,
            Content::Tow => self.tow.as_ref().ok_or("no TOW data folder")?,
        };
        let read = |n: &str| assets.read(n).map(|d| d.into_owned());
        let file = name.rsplit('\\').next().unwrap_or(name);
        match read(name) {
            Err(_) if content == Content::Tow && file != name => read(file),
            r => r,
        }
        .map_err(|e| format!("{name}: {e}"))
    }

    /// Loads a level from `content`'s folder, parsed as that game reads
    /// it, then set up under `content`'s rules.
    pub fn load_content_level(&self, content: Content, name: &str) -> Result<LevelData, String> {
        let data = self.read_level(content, name)?;
        let level = Level::parse(&data, content.game()).map_err(|e| format!("{name}: {e}"))?;
        let ruleset = match content {
            Content::Rtwp => Ruleset::rtwp(),
            Content::Tow => Ruleset::tow(),
        };
        let setup =
            Setup::load(&level, &ruleset, self.options.load).map_err(|e| format!("{name}: {e}"))?;
        Ok(LevelData {
            name: name.to_owned(),
            level,
            setup,
            ruleset,
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
    pub ruleset: Ruleset,
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
