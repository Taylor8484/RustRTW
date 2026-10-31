# RustRTW

A port of **Return to Wonderland Platinum** to Rust and the
[Bevy](https://bevyengine.org) engine, for Windows 10+ and Linux. Based on the
community's [OpenRTW](https://github.com/The-Wonderland-Team/OpenRTW) source
release.

It plays Return to Wonderland's levels and those of The Original Wonderland in
one game.

This is an unofficial community project. It is not checked by Midnight
Synergy, and you use it at your own risk.

**You need your own copy of the games.** This repository and its builds include
no levels, graphics, music or sound. Return to Wonderland and The Original
Wonderland are still on sale from
[Midnight Synergy](https://www.midnightsynergy.com).

## Status

Version 1 includes:

- The Tutorial, Classic and Deluxe level packs
- The Original Wonderland's worlds, its playground and its bonus levels
- Custom levels, with their houses and models
- Player profiles and saved progress, including your original RTW players
- The menus, endings and in-game menu

The game logic is written to match the original tick for tick. Levels from The
Original Wonderland run under Return to Wonderland's rules, with TOW's own
behaviour kept where it changes how a level plays. See
[docs/diff-tow.md](docs/diff-tow.md).

A new level editor is being developed. Until it is ready, levels can be made
with the original editor.

## Playing

Builds are posted on the [community forums](https://pcpuzzle.com/forum).

1. Unpack the build.
2. Copy your game files into `assets/RTWP` and, optionally, `assets/TOW`.
   `ASSETS.txt` in the build lists them.
3. Start `wonderland` (`wonderland.exe` on Windows).

The game names any missing files at startup. On Windows it writes them to
`wonderland-error.txt` beside the program.

| Key | Action |
|---|---|
| Arrow keys or keypad | Move |
| Enter | Switch between players |
| Shift + arrow keys | Look around |
| Esc | In-game menu |

| Option | Effect |
|---|---|
| `--windowed` | Stay in a window |
| `--pillarbox` | Keep the original's 4:3 picture on wide screens |
| `--backend dx12\|vulkan\|gl` | Choose the graphics API |
| `--assets <folder>` | Use another game data folder |
| `--level <name>` | Start straight in a level |

## Building from source

You need Rust 1.97.1 or later. On Linux you also need:

```sh
sudo apt-get install pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
```

Then copy your game files in and run the game. `tools/import-assets.sh` copies
them from a folder that holds your installed games:

```sh
tools/import-assets.sh /path/to/your/games
cargo run --release -p wonderland-game
```

The tests that need game data skip themselves when `assets` is missing:

```sh
cargo test --workspace
```

To build a release package in `dist/`:

```sh
tools/package-release.sh linux    # or: windows
```

## Project layout

| Path | Contents |
|---|---|
| `crates/formats` | Readers for the game's data files |
| `crates/logic` | The game simulation, with no engine code |
| `crates/campaign` | Player profiles, worlds and level order |
| `crates/game` | The Bevy game: graphics, sound and menus |
| `crates/inspect` | `wl-inspect`, a tool for looking inside game data |
| `data` | World and level tables |
| `docs` | [Architecture](docs/architecture.md) and the [TOW comparison](docs/diff-tow.md) |
| `tools` | Asset import and release packaging |
| `RTWP`, `TOW`, `WSW` | The original Blitz3D source, kept as the reference |

To work on the original Blitz3D code, see
[OpenRTW](https://github.com/The-Wonderland-Team/OpenRTW).

## Differences from the original

- Runs natively on modern Windows and Linux, with no Blitz3D or dgVoodoo.
- Wide screens show more of the level at the sides.
- The original's level editor is not included yet.
- The online menu entries still point to the original game's web pages.

## Licence and credits

Return to Wonderland, The Original Wonderland and their source code are
copyright Midnight Synergy. Before using this code, read
[license.txt](license.txt) and the
[Modding Rules](https://pcpuzzle.com/forum/viewtopic.php?t=27841). In short,
anything built on this code must be free, must include the licence, and must
not include the games' resources.

Original game by Patrick Maidorn and
[Midnight Synergy](https://www.midnightsynergy.com), with music by Jonne
"Purple Motion" Valtonen, and additional 3D modeling by Danny Hawrysio. Thanks
to the Wonderland community and
[The Wonderland Team](https://github.com/The-Wonderland-Team) for OpenRTW.
