//! Checks against the real game data in `assets/<GAME>` (gitignored; populate
//! it with `tools/import-assets.sh`). Games whose folder is missing are
//! skipped.

use std::fs;
use std::path::{Path, PathBuf};

use wonderland_formats::{Game, GameAssets, Level, Pack};

const GAMES: [&str; 3] = ["TOW", "RTWP", "WSW"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn open_game(game: &str) -> Option<GameAssets> {
    let dir = repo_root().join("assets").join(game);
    if !dir.is_dir() {
        eprintln!("skipping {game}: {} not found", dir.display());
        return None;
    }
    Some(GameAssets::open(&dir).unwrap_or_else(|e| panic!("{game}: {e}")))
}

#[test]
fn packs_load_and_every_entry_is_readable() {
    for game in GAMES {
        let Some(assets) = open_game(game) else {
            continue;
        };
        let pack: &Pack = assets.pack().unwrap_or_else(|| panic!("{game}: no pack"));
        assert!(!pack.index().is_empty(), "{game}: empty pack");
        for entry in pack.index().entries() {
            let data = pack.entry_data(entry);
            assert_eq!(data.len(), entry.len as usize);
            if entry.name.to_lowercase().ends_with(".jpg") {
                assert_eq!(
                    &data[..2],
                    b"\xFF\xD8",
                    "{game}: {} is not a JPEG",
                    entry.name
                );
            }
        }
    }
}

/// Every level in each game's pack, plus TOW's loose bonus levels, parses with
/// no bytes left over.
#[test]
fn every_level_parses() {
    let expected = [
        (Game::Tow, "TOW", 87 + 5),
        (Game::Rtwp, "RTWP", 85 + 71),
        (Game::Wsw, "WSW", 110),
    ];
    for (game, name, count) in expected {
        let Some(assets) = open_game(name) else {
            continue;
        };
        let mut files: Vec<(String, Vec<u8>)> = assets
            .pack()
            .unwrap()
            .index()
            .entries()
            .iter()
            .filter(|e| is_level(&e.name))
            .map(|e| (e.name.clone(), assets.read(&e.name).unwrap().into_owned()))
            .collect();
        for entry in fs::read_dir(assets.dir()).unwrap() {
            let file_name = entry.unwrap().file_name().to_string_lossy().into_owned();
            if is_level(&file_name) {
                files.push((
                    file_name.clone(),
                    assets.read(&file_name).unwrap().into_owned(),
                ));
            }
        }
        assert_eq!(files.len(), count, "{name}: level count");
        for (file, data) in files {
            if let Err(e) = Level::parse(&data, game) {
                panic!("{name}: {file}: {e}");
            }
        }
    }
}

#[test]
fn level_fields_match_hand_checked_values() {
    if let Some(assets) = open_game("RTWP") {
        let l = Level::parse(&assets.read("levels\\522.lev").unwrap(), Game::Rtwp).unwrap();
        assert_eq!((l.version, l.map_id), (4, Some(111207919)));
        assert_eq!(l.description, "Five Squared, Squared");
        assert_eq!((l.time, l.style, l.background), (900, 2, 0));
        assert_eq!((l.width, l.height, l.music_track), (39, 39, Some(2)));
        assert_eq!((l.tile(0, 0), l.tile(2, 1), l.tile(2, 2)), (200, 300, 1300));
    }
    if let Some(assets) = open_game("TOW") {
        let l = Level::parse(&assets.read("bonus1.lev").unwrap(), Game::Tow).unwrap();
        assert_eq!((l.version, l.map_id, l.music_track), (1, None, None));
        assert_eq!((l.description.as_str(), l.time), ("Bonus 1", 300));
        assert_eq!((l.width, l.height, l.signs.len()), (15, 15, 5));
    }
}

fn is_level(name: &str) -> bool {
    let lower = name.to_lowercase();
    LEVEL_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

/// Every complete asset path written as a literal in a game's Blitz source
/// must resolve through that game's pack or loose files.
#[test]
fn every_asset_path_in_the_source_resolves() {
    for game in GAMES {
        let Some(assets) = open_game(game) else {
            continue;
        };
        let missing: Vec<String> = source_asset_paths(game)
            .into_iter()
            .filter(|path| !unreachable_in(game).contains(&path.as_str()))
            .filter(|path| !candidates(game, path).iter().any(|p| assets.exists(p)))
            .collect();
        assert!(
            missing.is_empty(),
            "{game}: {} paths not found: {missing:#?}",
            missing.len()
        );
    }
}

const ASSET_EXTENSIONS: [&str; 13] = [
    ".jpg", ".png", ".bmp", ".pcx", ".3ds", ".x", ".md2", ".b3d", ".wav", ".ogg", ".mp3", ".mid",
    ".dat",
];
const LEVEL_EXTENSIONS: [&str; 3] = [".lev", ".lv5", ".lv6"];

/// Files the games create at runtime rather than ship with, or that only
/// demo builds load (TOW's extra demo levels, used when `Version<10`).
const NOT_SHIPPED: [&str; 7] = [
    "screenshot.bmp",
    "story.dat",
    "savestate.wld",
    "resdb1.dat",
    "resdb2.dat",
    "levels\\resdb1.dat",
    "levels\\resdb2.dat",
];

/// Paths a game's source names but can never load.
fn unreachable_in(game: &str) -> &'static [&'static str] {
    match game {
        // RTWP's "PlayGround" world list (globals.bb) was copied from TOW;
        // RTWP declares PlaygroundLevel but never uses it.
        "RTWP" => &[
            "play1.lev",
            "play2.lev",
            "play3.lev",
            "play4.lev",
            "play5.lev",
            "bonus1.lev",
            "bonus2.lev",
            "bonus3.lev",
            "bonus4.lev",
            "bonus5.lev",
            "theend.lev",
        ],
        // Entry 0 of WSW's world map table is a placeholder; every loop
        // that loads map levels starts at 1.
        "WSW" => &["easy.lv5"],
        _ => &[],
    }
}

fn source_asset_paths(game: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(repo_root().join(game)).unwrap() {
        let path = entry.unwrap().path();
        let is_source = path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("bb"));
        let is_editor = path
            .file_stem()
            .is_some_and(|s| s.eq_ignore_ascii_case("editor"));
        if !is_source || is_editor {
            continue;
        }
        let text = String::from_utf8_lossy(&fs::read(&path).unwrap()).into_owned();
        for line in text.lines() {
            let code = line.split(';').next().unwrap_or("");
            for literal in code.split('"').skip(1).step_by(2) {
                let lower = literal.to_lowercase();
                let is_asset = ASSET_EXTENSIONS
                    .iter()
                    .chain(&LEVEL_EXTENSIONS)
                    .any(|ext| lower.len() > ext.len() && lower.ends_with(ext));
                // Literals starting with a separator are tails of concatenated
                // paths; "custom..." folders are optional user content.
                if is_asset
                    && !lower.starts_with('\\')
                    && !lower.starts_with("custom")
                    && !NOT_SHIPPED.contains(&lower.as_str())
                {
                    paths.push(lower);
                }
            }
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

/// Level lists and picture tables store bare file names; the loading code
/// adds the folder (`levels\`, WSW's `wswlevels\`, or `graphics\`).
fn candidates(game: &str, path: &str) -> Vec<String> {
    if path.contains('\\') {
        return vec![path.to_owned()];
    }
    let is_level = LEVEL_EXTENSIONS.iter().any(|ext| path.ends_with(ext));
    let folder = match (is_level, game) {
        (true, "WSW") => "wswlevels",
        (true, _) => "levels",
        (false, _) => "graphics",
    };
    vec![path.to_owned(), format!("{folder}\\{path}")]
}
