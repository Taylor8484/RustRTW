//! Builds the scene geometry of every RTWP level and decodes every texture
//! it names. Uses `$WL_ASSETS/RTWP`, else `assets/RTWP` at the workspace
//! root; skipped when neither exists.

use std::collections::BTreeSet;
use std::path::PathBuf;

use wonderland_game::level::GameData;
use wonderland_game::level_mesh::{Tex, construct_background, construct_level};
use wonderland_game::style::{effective_background, effective_style, level_textures};
use wonderland_game::texture;

fn assets_dir() -> Option<PathBuf> {
    let dir = std::env::var_os("WL_ASSETS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets"))
        .join("RTWP");
    dir.is_dir().then_some(dir)
}

#[test]
fn every_rtwp_level_builds_its_scene() {
    let Some(dir) = assets_dir() else {
        eprintln!("skipping: no RTWP assets");
        return;
    };
    let data = GameData::open(&dir).unwrap();
    let names = data.level_names();
    assert_eq!(names.len(), 85 + 71);

    let mut paths = BTreeSet::new();
    let mut quads = 0;
    for name in &names {
        let level = data.load_level(name).unwrap();
        let s = &level.setup;
        let background = effective_background(s.background);
        let style = effective_style(s.style, background);
        let surfaces = construct_level(&s.grid, background);
        assert!(!surfaces.is_empty(), "{name}: no geometry");
        let floors = surfaces
            .iter()
            .filter(|x| matches!(x.texture, Tex::Floor(_)))
            .count();
        assert!(floors > 0, "{name}: no floor");
        quads += surfaces.iter().map(|x| x.quads.len()).sum::<usize>();
        construct_background(background, style, s.grid.width(), s.grid.height());

        for teleport in [0, 1] {
            let t = level_textures(s.style, s.background, s.weather.snow, teleport);
            let mut all = vec![t.floor_sign, t.wall_top, t.trampoline, t.water, t.lava];
            all.extend(t.floor);
            all.extend(t.wall_side);
            all.extend(t.teleport);
            all.extend(t.ice);
            all.extend(t.conveyor);
            all.extend(t.background.map(|b| b.0));
            paths.extend(all.into_iter().map(|r| (r.path, r.flags)));
        }
    }
    eprintln!(
        "{} levels, {quads} quads, {} textures",
        names.len(),
        paths.len()
    );
    for (path, flags) in &paths {
        let bytes = data
            .assets
            .read(path)
            .unwrap_or_else(|e| panic!("{path}: {e}"));
        texture::decode(&bytes, *flags).unwrap_or_else(|e| panic!("{path}: {e}"));
    }
}

/// Every track the menus and worlds play decodes and is not silent, and
/// every level the world table names is in the pack.
#[test]
fn the_music_plays_and_the_worlds_name_real_levels() {
    use bevy::audio::Decodable;
    use wonderland_game::music::{TITLE, Tune, world_levels, world_track};

    let Some(dir) = assets_dir() else {
        eprintln!("skipping: no RTWP assets");
        return;
    };
    let data = GameData::open(&dir).unwrap();
    let mut files: BTreeSet<&str> = (0..30).map(|w| world_track(w).file).collect();
    files.insert(TITLE.file);
    for file in files {
        let tune = Tune::load(&data, file).unwrap();
        let loud = tune
            .decoder()
            .take(44_100 * 4)
            .filter(|s| s.abs() > 1e-3)
            .count();
        assert!(loud > 1000, "{file}: silent");
    }
    // World 9 opens with `res10.dat`, a level kept under a resource name.
    for name in world_levels().filter(|n| *n != "res10") {
        assert!(data.find_level(name).is_some(), "{name} is not in the pack");
    }
}
