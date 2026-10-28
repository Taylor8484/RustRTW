//! `wonderland [--assets <dir>] [--level <name>] [--windowed]
//! [--pillarbox] [--backend <name>]`
//!
//! Starts Return to Wonderland Platinum at its title screen, or with
//! `--level` straight in that level, going on through the pack's levels
//! as each is won. The game data is read from `<dir>/RTWP` (default:
//! `$WL_ASSETS`, else `assets`). The window takes its size and full
//! screen setting from `options.cfg` as the original does; `--windowed`
//! keeps it in a window. Wide screens show more at the sides;
//! `--pillarbox` keeps the original's 4:3 picture with black bars.

use std::path::PathBuf;
use std::process::ExitCode;

use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::window::ExitCondition;
use bevy::window::WindowMode;
use bevy::winit::WinitPlugin;
use wonderland_game::GamePlugins;
use wonderland_game::camera::Pillarbox;
use wonderland_game::flow::{Phase, begin};
use wonderland_game::level::{GameData, default_assets_dir};
use wonderland_game::menu::GFX_MODES;
use wonderland_game::shot::headless;
use wonderland_logic::Session;

struct Args {
    assets: PathBuf,
    level: Option<String>,
    windowed: bool,
    pillarbox: bool,
    backends: Option<Backends>,
}

const USAGE: &str = "usage: wonderland [--assets <dir>] [--level <name>] [--windowed] [--pillarbox] \
                     [--backend dx12|vulkan|gl]";

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        assets: default_assets_dir(),
        level: None,
        windowed: false,
        pillarbox: false,
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
            "--windowed" => args.windowed = true,
            "--pillarbox" => args.pillarbox = true,
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
    // `--level`: straight into play, skipping the menus.
    let start = match &args.level {
        None => None,
        Some(n) => {
            let Some(name) = data.find_level(n) else {
                eprintln!("no such level: {n}");
                return ExitCode::FAILURE;
            };
            match begin(&data, &name, Session::default(), 0) {
                Ok(s) => Some(s),
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::FAILURE;
                }
            }
        }
    };

    // `Graphics3D` with `MyGfxMode` and `MyWindowed`; full screen keeps
    // the desktop's mode rather than switching.
    let (width, height) = usize::try_from(data.config.gfx_mode)
        .ok()
        .and_then(|i| GFX_MODES.get(i))
        .copied()
        .unwrap_or(GFX_MODES[0]);
    let mode = if data.config.windowed == 0 && !args.windowed {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
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
                        title: "Return To Wonderland".into(),
                        resolution: (width, height).into(),
                        mode,
                        ..default()
                    }),
                    ..default()
                })
                .set(render),
        );
    }
    if let Some((level, sim, playlist)) = start {
        app.insert_resource(level)
            .insert_resource(sim)
            .insert_resource(playlist)
            .insert_state(Phase::Game);
    }
    if args.pillarbox {
        app.insert_resource(Pillarbox);
    }
    app.insert_resource(data).add_plugins(GamePlugins).run();
    ExitCode::SUCCESS
}
