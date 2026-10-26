//! Lists which RTWP levels use which kinds of object, to pick levels for
//! checking the renderer: `cargo run -p wonderland-logic --example census`.

use std::collections::BTreeMap;

use wonderland_formats::{Game, GameAssets, Level};
use wonderland_logic::{Options, Ruleset, Session, Spawn, World};

fn main() {
    let dir = std::env::var("WL_ASSETS").unwrap_or_else(|_| "assets".into());
    let assets = GameAssets::open(format!("{dir}/RTWP")).expect("game data");
    let names: Vec<String> = assets
        .pack()
        .unwrap()
        .index()
        .entries()
        .iter()
        .map(|e| e.name.clone())
        .filter(|n| n.ends_with(".lev") || n.ends_with(".lv5"))
        .collect();
    let mut by_kind: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for name in names {
        let level = Level::parse(&assets.read(&name).unwrap(), Game::Rtwp).unwrap();
        let Ok(w) = World::start(
            &level,
            Ruleset::rtwp(),
            Options::default(),
            Session::default(),
        ) else {
            continue;
        };
        let mut kinds = Vec::new();
        for s in &w.level.spawns {
            kinds.push(match s {
                Spawn::LaserGate { col: 0, .. } => "final gate".to_string(),
                Spawn::LaserGate { .. } => "laser gate".into(),
                Spawn::Button { .. } => "button".into(),
                Spawn::Spike { .. } => "spike".into(),
                Spawn::Cannon { .. } => "cannon".into(),
                Spawn::Fish { .. } => "fish".into(),
                Spawn::Pillar { kind, .. } => format!("pillar {kind}"),
                Spawn::Monster { kind, .. } => format!("monster {kind}"),
                Spawn::Box { kind, .. } => format!("box {}", kind.min(&100)),
                _ => continue,
            });
        }
        if w.electros.iter().any(|e| e.exists) {
            kinds.push("electro".into());
        }
        kinds.sort();
        kinds.dedup();
        for k in kinds {
            by_kind.entry(k).or_default().push(name.clone());
        }
    }
    for (k, levels) in by_kind {
        let shown: Vec<&str> = levels.iter().take(4).map(String::as_str).collect();
        println!("{k:12} {:3}  {}", levels.len(), shown.join(", "));
    }
}
