//! Replays every recording in `assets/recordings` (gitignored; see
//! `RTWP/recorder.bb`) against the levels in `assets/RTWP`, comparing the
//! families the port covers. Skipped when either folder is missing.
//!
//! Set `WL_REPLAY_REPORT=1` to print the differences without failing, for
//! recordings of levels that use families still being ported.

use std::path::Path;

use wonderland_formats::{Game, GameAssets, Level};
use wonderland_logic::Ruleset;
use wonderland_logic::replay::{self, Recording, ReplayOptions};

#[test]
fn recordings_replay_like_the_original() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let (dir, game) = (root.join("recordings"), root.join("RTWP"));
    if !dir.is_dir() || !game.is_dir() {
        eprintln!("skipping: no recordings or no RTWP assets");
        return;
    }
    let assets = GameAssets::open(&game).unwrap();
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("wlr")))
        .collect();
    files.sort();

    let mut failed = Vec::new();
    for file in files {
        let name = file.file_name().unwrap().to_string_lossy().into_owned();
        let text = std::fs::read_to_string(&file).unwrap();
        let rec = Recording::parse(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
        for (i, run) in rec.runs.iter().enumerate() {
            let path = &run.start.path;
            let level = Level::parse(&assets.read(path).unwrap(), Game::Rtwp)
                .unwrap_or_else(|e| panic!("{name} run {i} ({path}): {e}"));
            let report = replay::replay(run, &level, Ruleset::rtwp(), &ReplayOptions::default())
                .unwrap_or_else(|e| panic!("{name} run {i} ({path}): {e}"));
            eprintln!(
                "{name} run {i} ({path}): {} ticks, {} with cheats, outside draws before {:?}",
                report.ticks, report.cheat_ticks, report.outside_draws
            );
            for d in &report.divergences {
                eprintln!("  {d}");
            }
            if !report.passed() {
                failed.push(format!("{name} run {i} ({path})"));
            }
        }
    }
    if std::env::var_os("WL_REPLAY_REPORT").is_none() {
        assert!(failed.is_empty(), "differs from the original: {failed:?}");
    }
}
