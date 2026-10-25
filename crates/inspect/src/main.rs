//! Inspect Wonderland game data.
//!
//!   wl-inspect pack <game dir> list [filter]
//!   wl-inspect pack <game dir> stats
//!   wl-inspect pack <game dir> extract <name> <out file>
//!   wl-inspect pack <game dir> extract-all <out dir>
//!   wl-inspect level <game dir> <name>
//!   wl-inspect save <game dir> <file>
//!
//! `extract` resolves `<name>` the way the games do (pack first, then a loose
//! file in the game folder); the other pack commands only look at the pack.
//! `level` looks for `<name>` in the game's level folder, then the game
//! folder, and takes the game from the folder name (TOW, RTWP or WSW).
//! `save` dumps `options.cfg`, `extra.cfg`, `story.dat`, or a player file
//! (`players/<name>.pla` or `.bak`; a bare name means `players/<name>.pla`).
//! Run with `cargo run -p wonderland-inspect -- <args>`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{env, fs};

use wonderland_formats::save::Progress;
use wonderland_formats::{ExtraCfg, Game, GameAssets, Level, Options, Pack, PlayerFile, StoryDat};

const USAGE: &str = "\
usage: wl-inspect pack <game dir> list [filter]
       wl-inspect pack <game dir> stats
       wl-inspect pack <game dir> extract <name> <out file>
       wl-inspect pack <game dir> extract-all <out dir>
       wl-inspect level <game dir> <name>
       wl-inspect save <game dir> <file>";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("{msg}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    match args {
        [group, dir, rest @ ..] if group == "pack" => pack(dir, rest),
        [group, dir, name] if group == "level" => level(dir, name),
        [group, dir, file] if group == "save" => save(dir, file),
        _ => Err(USAGE.into()),
    }
}

fn pack(dir: &str, args: &[String]) -> Result<(), String> {
    let pack = Pack::open(dir).map_err(|e| e.to_string())?;
    match args {
        [command] if command == "list" => list(&pack, ""),
        [command, filter] if command == "list" => list(&pack, filter),
        [command] if command == "stats" => stats(&pack),
        [command, name, out] if command == "extract" => {
            let assets = GameAssets::open(dir).map_err(|e| e.to_string())?;
            let data = assets.read(name).map_err(|e| e.to_string())?;
            fs::write(out, &*data).map_err(|e| format!("{out}: {e}"))?;
        }
        [command, out] if command == "extract-all" => {
            for e in pack.index().entries() {
                let path = out_path(Path::new(out), &e.name);
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|err| format!("{}: {err}", parent.display()))?;
                }
                fs::write(&path, pack.entry_data(e))
                    .map_err(|err| format!("{}: {err}", path.display()))?;
            }
            println!("extracted {} entries to {out}", pack.index().len());
        }
        _ => return Err(USAGE.into()),
    }
    Ok(())
}

fn level(dir: &str, name: &str) -> Result<(), String> {
    let game = game_from_dir(dir)?;
    let assets = GameAssets::open(dir).map_err(|e| e.to_string())?;
    let folder = if game == Game::Wsw {
        "wswlevels"
    } else {
        "levels"
    };
    let data = [format!("{folder}\\{name}"), name.to_owned()]
        .iter()
        .find_map(|path| assets.read(path).ok())
        .ok_or_else(|| format!("{name}: not found in {folder}\\ or the game folder"))?;
    let l = Level::parse(&data, game).map_err(|e| format!("{name}: {e}"))?;

    println!("magic        {:?} (v{})", l.magic, l.version);
    println!("file name    {:?}", l.file_name);
    if let Some(id) = l.map_id {
        println!("map id       {id}");
    }
    println!("description  {:?}", l.description);
    for (label, dir) in [
        ("houses", &l.houses),
        ("models", &l.models),
        ("textures", &l.textures),
        ("backgrounds", &l.backgrounds),
    ] {
        if let Some(d) = dir {
            println!("{label:<12} {} {:?}", d.flag, d.dir);
        }
    }
    println!(
        "time {}  style {}  background {}",
        l.time, l.style, l.background
    );
    if let Some(track) = l.music_track {
        println!("music track  {track}");
    }
    println!("size         {}x{}", l.width, l.height);
    for (label, grid) in [("tiles", &l.tiles), ("objects", &l.objects)] {
        println!("{label}:");
        for row in grid.chunks(l.width.max(1)) {
            let cells: Vec<String> = row.iter().map(|v| format!("{v:>5}")).collect();
            println!("{}", cells.join(""));
        }
    }
    for (i, text) in l.signs.iter().enumerate() {
        if !text.is_empty() {
            println!("sign {i:<2} {text:?}");
        }
    }
    Ok(())
}

fn save(dir: &str, file: &str) -> Result<(), String> {
    let game = game_from_dir(dir)?;
    let lower = file.to_lowercase();
    let is_player = lower.ends_with(".pla") || lower.ends_with(".bak");
    let rel = if is_player || lower.contains('.') {
        file.to_owned()
    } else {
        format!("players/{file}.pla")
    };
    let path = Path::new(dir).join(&rel);
    let data = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let err = |e: wonderland_formats::Error| format!("{rel}: {e}");
    let name = lower.rsplit(['/', '\\']).next().unwrap_or(&lower);
    match name {
        "options.cfg" => {
            let o = Options::parse(game, &data).map_err(err)?;
            println!("{o:#?}");
        }
        "extra.cfg" => println!("{:#?}", ExtraCfg::parse(&data).map_err(err)?),
        "story.dat" => println!("{:#?}", StoryDat::parse(&data).map_err(err)?),
        _ if is_player || !lower.contains('.') => {
            player(&PlayerFile::parse(&data, game).map_err(err)?)
        }
        _ => return Err(format!("{rel}: not a save or config file")),
    }
    Ok(())
}

fn player(p: &PlayerFile) {
    println!("magic        {:?}", p.magic);
    println!("name         {:?}", p.name);
    if let Some(id) = p.id {
        println!("id           {id}");
    }
    println!("position     {}", p.position);
    let (levels, score) = p.computed_summary();
    println!(
        "completed    {} (computed {levels})\ntotal score  {} (computed {score})",
        p.levels_completed, p.total_score
    );
    let scores = |label: &str, values: &[i32]| {
        let played: Vec<String> = values
            .iter()
            .enumerate()
            .filter(|(_, v)| **v != 0)
            .map(|(i, v)| format!("{i}:{v}"))
            .collect();
        println!("{label:<12} {}", played.join(" "));
    };
    match &p.progress {
        Progress::Tow(s) => scores("scores", s),
        Progress::Rtwp(r) => {
            scores("scores", &r.scores);
            if let Some(perfect) = &r.perfect {
                scores("perfect", &perfect.levels);
            }
            for (i, e) in r.extra_levels.iter().enumerate() {
                let perfect = r.perfect.as_ref().map(|p| p.extra[i]);
                println!(
                    "extra {i:<6} id {} id2 {} score {} perfect {perfect:?}",
                    e.id, e.id2, e.score
                );
            }
            println!("world packs  {:?}", r.world_packs);
        }
        Progress::Wsw(w) => {
            for (i, e) in w.map.iter().enumerate() {
                if e.status != 0 || e.score != 0 || e.kind != 0 {
                    println!(
                        "map {i:<8} status {} score {} type {}",
                        e.status, e.score, e.kind
                    );
                }
            }
            println!(
                "gold stars {}  rainbow stars {}  peegue's pass {}  qookie's map {}  forest map {}",
                w.gold_stars, w.rainbow_stars, w.peegues_pass, w.qookies_map, w.forest_map
            );
            println!("story told   {:?}", w.story_told);
        }
    }
    if !p.trailing.is_empty() {
        println!("trailing     {} bytes", p.trailing.len());
    }
}

fn game_from_dir(dir: &str) -> Result<Game, String> {
    let name = Path::new(dir)
        .file_name()
        .map(|n| n.to_string_lossy().to_uppercase());
    match name.as_deref() {
        Some("TOW") => Ok(Game::Tow),
        Some("RTWP") => Ok(Game::Rtwp),
        Some("WSW") => Ok(Game::Wsw),
        _ => Err(format!("{dir}: folder must be named TOW, RTWP or WSW")),
    }
}

/// Lists entries whose name contains `filter` (case-insensitive).
fn list(pack: &Pack, filter: &str) {
    let filter = filter.to_lowercase();
    let mut shown = 0;
    for e in pack.index().entries() {
        if e.name.to_lowercase().contains(&filter) {
            println!("{:>10} {:>9}  {}", e.offset, e.len, e.name);
            shown += 1;
        }
    }
    println!("{shown} of {} entries", pack.index().len());
}

/// Entry count and total size per file extension.
fn stats(pack: &Pack) {
    let mut by_ext: BTreeMap<String, (usize, u64)> = BTreeMap::new();
    for e in pack.index().entries() {
        let name = e.name.rsplit(['\\', '/']).next().unwrap_or(&e.name);
        let ext = match name.rsplit_once('.') {
            Some((_, ext)) => ext.to_lowercase(),
            None => "(none)".into(),
        };
        let slot = by_ext.entry(ext).or_default();
        slot.0 += 1;
        slot.1 += u64::from(e.len);
    }
    for (ext, (count, bytes)) in &by_ext {
        println!("{ext:>8} {count:>6} {:>10} KB", bytes / 1024);
    }
    println!("{} entries", pack.index().len());
}

fn out_path(root: &Path, name: &str) -> PathBuf {
    name.split(['\\', '/'])
        .filter(|p| !p.is_empty() && *p != "..")
        .fold(root.to_path_buf(), |path, part| path.join(part))
}
