//! Puts every level of the RTWP pack in a `CustomLevels` folder and lists
//! it as the custom level menu does: each passes the header check, with
//! the map id and title the level parser reads. Skipped when the pack is
//! missing.

use std::fs;
use std::path::Path;

use wonderland_campaign::custom::{self, CustomLevels};
use wonderland_formats::level::Level;
use wonderland_formats::{Game, GameAssets};

#[test]
fn pack_levels_list_as_custom_levels() {
    let game = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/RTWP");
    let Some(pack) = GameAssets::open(&game).ok().filter(|a| a.pack().is_some()) else {
        eprintln!("skipping: no RTWP pack in {}", game.display());
        return;
    };
    let dir = std::env::temp_dir().join(format!("wl-real-custom-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let root = dir.join(custom::FOLDER);
    fs::create_dir_all(&root).unwrap();
    let mut want = Vec::new();
    for e in pack.pack().unwrap().index().entries() {
        let name = e
            .name
            .rsplit(['\\', '/'])
            .next()
            .unwrap()
            .to_ascii_uppercase();
        if !custom::is_level_name(&name) {
            continue;
        }
        let data = pack.read(&e.name).unwrap();
        let level = Level::parse(&data, Game::Rtwp).unwrap();
        fs::write(root.join(&name), &data[..]).unwrap();
        want.push((name, level.map_id.unwrap(), level.description));
    }
    assert!(want.len() > 100, "{} levels", want.len());
    want.sort();

    let list = CustomLevels::new(&dir).list(&[], true, &[]).unwrap();
    let got: Vec<_> = (list.iter().skip(1))
        .map(|e| (e.name.clone(), e.id, e.title.clone()))
        .collect();
    assert_eq!(got, want);
    fs::remove_dir_all(&dir).unwrap();
}
