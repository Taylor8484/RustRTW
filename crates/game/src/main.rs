//! `wonderland [--assets <dir>] [--level <name>] [--backend <name>]`
//!
//! Starts a Return to Wonderland Platinum level straight away. The game
//! data is read from `<dir>/RTWP` (default: `$WL_ASSETS`, else `assets`).

use std::path::PathBuf;
use std::process::ExitCode;

use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use wonderland_game::GamePlugins;
use wonderland_game::level::{GameData, default_assets_dir};
use wonderland_game::shot::headless;
use wonderland_game::sim::Sim;
use wonderland_logic::Session;

struct Args {
    assets: PathBuf,
    level: Option<String>,
    backends: Option<Backends>,
}

const USAGE: &str =
    "usage: wonderland [--assets <dir>] [--level <name>] [--backend dx12|vulkan|gl]";

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        assets: default_assets_dir(),
        level: None,
        backends: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        let mut value = || {
            it.next()
                .ok_or_else(|| format!("{a} needs a value\n{USAGE}"))
        };
        match a.as_str() {
            "--assets" => args.assets = PathBuf::from(value()?).join("RTWP"),
            "--level" => args.level = Some(value()?),
            "--backend" => {
                let v = value()?;
                args.backends = Some(Backends::from_comma_list(&v));
            }
            "-h" | "--help" => return Err(USAGE.to_owned()),
            _ => return Err(format!("unknown argument {a}\n{USAGE}")),
        }
    }
    Ok(args)
}

/// DX12 on Windows and Vulkan on Linux, each with a fallback wgpu tries
/// after it: Vulkan on Windows (wgpu's GL needs ANGLE there), OpenGL on
/// Linux. `--backend` or `WGPU_BACKEND` overrides.
fn backends(arg: Option<Backends>) -> Backends {
    arg.or_else(Backends::from_env).unwrap_or(if cfg!(windows) {
        Backends::DX12 | Backends::VULKAN
    } else {
        Backends::VULKAN | Backends::GL
    })
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let data = match GameData::open(&args.assets) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("cannot open the game data: {e}");
            return ExitCode::FAILURE;
        }
    };
    let name = match &args.level {
        Some(n) => data.find_level(n),
        None => data
            .find_level("intro")
            .or_else(|| data.level_names().into_iter().next()),
    };
    let Some(name) = name else {
        eprintln!(
            "no such level: {}",
            args.level.as_deref().unwrap_or("(none in the pack)")
        );
        return ExitCode::FAILURE;
    };
    let level = match data.load_level(&name) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let sim = match Sim::start(&data, &level, Session::default(), 0) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let render = RenderPlugin {
        render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
            backends: Some(backends(args.backends)),
            ..default()
        })),
        ..default()
    };
    let mut app = App::new();
    if headless() {
        app.add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .set(render)
                .disable::<WinitPlugin>(),
            ScheduleRunnerPlugin::run_loop(Duration::from_millis(1)),
        ));
    } else {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Wonderland".into(),
                        resolution: (1024, 768).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(render),
        );
    }
    app.insert_resource(data)
        .insert_resource(level)
        .insert_resource(sim)
        .add_plugins(GamePlugins)
        .run();
    ExitCode::SUCCESS
}
