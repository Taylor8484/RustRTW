//! `wonderland [--assets <dir>] [--level <name>] [--edit] [--windowed]
//! [--pillarbox] [--backend <name>]`
//!
//! Starts Return to Wonderland Platinum at its title screen, or with
//! `--level` straight in that level, going on through the pack's levels
//! as each is won, with no player and the music of the level's world. The game data is read from `<dir>/RTWP` (default:
//! `$WL_ASSETS`, else `assets`, else `assets` beside the executable). The window takes its size and full
//! screen setting from `options.cfg` as the original does; `--windowed`
//! keeps it in a window. Wide screens show more at the sides;
//! `--pillarbox` keeps the original's 4:3 picture with black bars. `--edit`
//! starts in the level editor, on `--level`'s level or a new one.

// No console window behind the game. `fail` covers what it would have shown.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

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
use wonderland_campaign::worlds::Worlds;
use wonderland_editor::Editor;
use wonderland_formats::Level;
use wonderland_game::GamePlugins;
use wonderland_game::camera::Pillarbox;
use wonderland_game::editor_scene::Editing;
use wonderland_game::flow::{Phase, Playlist, begin};
use wonderland_game::level::{GameData, default_assets_dir};
use wonderland_game::menu::GFX_MODES;
use wonderland_game::music::{MusicRequest, world_track};
use wonderland_game::shot::headless;
use wonderland_logic::Session;

struct Args {
    assets: PathBuf,
    level: Option<String>,
    edit: bool,
    windowed: bool,
    pillarbox: bool,
    backends: Option<Backends>,
}

const USAGE: &str = "usage: wonderland [--assets <dir>] [--level <name>] [--edit] [--windowed] \
                     [--pillarbox] [--backend dx12|vulkan|gl]";

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        assets: default_assets_dir(),
        level: None,
        edit: false,
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
            "--edit" => args.edit = true,
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

/// Reports why the game cannot start. A release on Windows has no console
/// to show it, so the message also goes to a file beside the executable.
fn fail(message: &str) -> ExitCode {
    eprintln!("{message}");
    if cfg!(all(windows, not(debug_assertions)))
        && let Ok(exe) = std::env::current_exe()
    {
        let _ = std::fs::write(exe.with_file_name("wonderland-error.txt"), message);
    }
    ExitCode::FAILURE
}

/// The level `name` finds, as its game reads it.
fn open_level(data: &GameData, name: &str) -> Result<Level, String> {
    let (content, path) = data
        .find_level(name)
        .ok_or_else(|| format!("no such level: {name}"))?;
    Ok(data.load_content_level(content, &path)?.level)
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => return fail(&e),
    };
    let data = match GameData::open(&args.assets) {
        Ok(d) => d,
        Err(e) => {
            return fail(&format!(
                "cannot open the game data: {e}\nASSETS.txt lists the files to copy."
            ));
        }
    };
    let missing = data.missing_files();
    if !missing.is_empty() {
        return fail(&format!(
            "{} is missing {} game files: {}\nASSETS.txt lists the files to copy.",
            args.assets.display(),
            missing.len(),
            missing.join(", ")
        ));
    }
    let edit = if args.edit {
        match args.level.as_deref().map(|n| open_level(&data, n)) {
            None => Some(Editor::new()),
            Some(Ok(level)) => Some(Editor::open(level)),
            Some(Err(e)) => return fail(&e),
        }
    } else {
        None
    };
    // `--level`: straight into play, skipping the menus.
    let start = match &args.level {
        None => None,
        Some(_) if args.edit => None,
        Some(n) => {
            let Some((content, name)) = data.find_level(n) else {
                return fail(&format!("no such level: {n}"));
            };
            match begin(&data, content, &name, Session::default(), 0, None) {
                Ok((level, sim)) => Some((level, sim, Playlist::from_pack(&data, content, &name))),
                Err(e) => return fail(&e),
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
        // `StartLevelMusic` for the world that lists the level.
        if let Some(l) = Worlds::game().index_of(playlist.content, &level.name) {
            app.insert_resource(MusicRequest(Some(Some(world_track(l / 10)))));
        }
        app.insert_resource(level)
            .insert_resource(sim)
            .insert_resource(playlist)
            .insert_state(Phase::Game);
    }
    if let Some(editor) = edit {
        app.insert_resource(Editing(editor))
            .insert_state(Phase::Editor);
    }
    if args.pillarbox {
        app.insert_resource(Pillarbox);
    }
    app.insert_resource(data).add_plugins(GamePlugins).run();
    ExitCode::SUCCESS
}
