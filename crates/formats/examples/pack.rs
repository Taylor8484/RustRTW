//! Inspect a game's pack.
//!
//!   cargo run -p wonderland-formats --example pack -- <game dir> list
//!   cargo run -p wonderland-formats --example pack -- <game dir> extract <name> <out file>
//!   cargo run -p wonderland-formats --example pack -- <game dir> extract-all <out dir>

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{env, fs};

use wonderland_formats::{GameAssets, Pack};

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
    let usage = "usage: pack <game dir> list | extract <name> <out file> | extract-all <out dir>";
    let [dir, command, rest @ ..] = args else {
        return Err(usage.into());
    };
    let pack = Pack::open(dir).map_err(|e| e.to_string())?;
    match (command.as_str(), rest) {
        ("list", []) => {
            for e in pack.index().entries() {
                println!("{:>10} {:>9}  {}", e.offset, e.len, e.name);
            }
            println!("{} entries", pack.index().len());
        }
        ("extract", [name, out]) => {
            let assets = GameAssets::open(dir).map_err(|e| e.to_string())?;
            let data = assets.read(name).map_err(|e| e.to_string())?;
            fs::write(out, &*data).map_err(|e| format!("{out}: {e}"))?;
        }
        ("extract-all", [out]) => {
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
        _ => return Err(usage.into()),
    }
    Ok(())
}

fn out_path(root: &Path, name: &str) -> PathBuf {
    name.split(['\\', '/'])
        .filter(|p| !p.is_empty() && *p != "..")
        .fold(root.to_path_buf(), |path, part| path.join(part))
}
