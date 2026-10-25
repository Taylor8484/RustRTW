//! Inspect Wonderland game data.
//!
//!   wl-inspect pack <game dir> list [filter]
//!   wl-inspect pack <game dir> stats
//!   wl-inspect pack <game dir> extract <name> <out file>
//!   wl-inspect pack <game dir> extract-all <out dir>
//!
//! `extract` resolves `<name>` the way the games do (pack first, then a loose
//! file in the game folder); the other pack commands only look at the pack.
//! Run with `cargo run -p wonderland-inspect -- <args>`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{env, fs};

use wonderland_formats::{GameAssets, Pack};

const USAGE: &str = "\
usage: wl-inspect pack <game dir> list [filter]
       wl-inspect pack <game dir> stats
       wl-inspect pack <game dir> extract <name> <out file>
       wl-inspect pack <game dir> extract-all <out dir>";

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
