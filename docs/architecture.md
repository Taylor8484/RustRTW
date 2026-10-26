# Wonderland Rust Port — Architecture

## Goal

One executable that plays all three Wonderland Classic games. On launch the
player picks which game to play (The Original Wonderland, Return to Wonderland
Platinum, or Wonderland Secret Worlds); the same engine then runs that game's
content and rules.

The original Blitz3D sources (`TOW/`, `RTWP/`, `WSW/`) stay in the repo as the
reference implementation until the port reaches parity.

## The original code

The three games are forks of one engine, copied per game and edited
independently:

| Game | Lines | Notes |
|---|---|---|
| TOW — The Original Wonderland | ~15k | Earliest engine. Two hard-coded characters (Stinky, Loof) with dedicated functions; smaller object set. |
| RTWP — Return to Wonderland Platinum | ~22k | Most complete engine: generalised player system (up to 4), most object types, level packs, save states, custom content, level editor. |
| WSW — Wonderland Secret Worlds | ~16k | Close to RTWP; adds world map screen, story screens, ending sequence. |

About 105 functions exist in all three. RTWP's level loader already reads every
level format version (v1 used by TOW through v6), so RTWP is treated as the
superset and the base for the port. TOW becomes "two players, smaller object
set"; WSW's differences are mostly in the screens around the puzzles.

Functions with the same name may still behave differently per game. Those
differences are captured as per-game ruleset flags and verified with recorded
playthroughs of the original games (see Verification).

## Workspace layout

```
Cargo.toml                 workspace
crates/
  formats/   pack files, levels v1–v6, saves, options       (no engine deps)
  logic/     deterministic simulation, per-game Ruleset      (no engine deps)
  campaign/  level order & progression per game
  game/      Bevy app: launcher → game select → menus → play
  inspect/   wl-inspect CLI: dump packs (later levels, saves) for debugging
data/
  tow.ron, rtwp.ron, wsw.ron   per-game definitions (worlds, level lists,
                               names, story text, music, ruleset flags)
assets/      copied game data, one folder per game (gitignored, copyrighted)
```

Only `game` depends on Bevy. `formats` and `logic` are plain Rust so they can be
tested headless and outlive any framework upgrade or swap.

### formats

- `blitz` — Blitz3D binary primitives (`ReadInt`, `ReadFloat`, `ReadString`:
  little-endian, strings are a u32 length + bytes).
- `pack` — `res1.dat` index + `res2.dat` data, read in memory. The original
  extracted every packed file to a temp file on disk before loading it; the
  port does not.
- `assets` — resolves a game path the way the original does: pack first
  (case-insensitive), then a loose file in the game folder.
- `level` — level files v1–v6 as stored. Parsing takes the loading `Game`
  because TOW, RTWP and WSW read the header differently.
- `save` — player files (`players/*.pla`, `.bak`), one layout per game.
- `config` — `options.cfg`, TOW's `extra.cfg`, WSW's `story.dat`. These
  parse like Blitz: missing ints read as 0, since the shipped WSW
  `options.cfg` is shorter than its loader expects.
- `mesh` — `.3ds`, `.x` and `.md2`, produced as Blitz holds them after
  `LoadMesh`/`LoadMD2` (Blitz coordinates, winding and normals; see the
  module docs). The game crate converts to Bevy's coordinates.
- Textures, sound and music are standard formats, so `formats` has no
  parsers for them. The game crate decodes them with these crates, and
  `tests/real_media.rs` checks every shipped file:
  - Textures: 24-bit BMP (one 8-bit) and baseline JPEG, decoded with
    `image`, which is also Bevy's decoder. Blitz texture flags 1 (colour),
    2 (alpha), 4 (masked: black is transparent) and 6 are in use. The game
    crate must reproduce how Blitz builds the alpha for flags 2 and 4 from
    images that have no alpha channel. Check that against the Blitz source
    in step 3.
  - Sound: 16-bit PCM WAV, which Bevy's audio plays as is.
  - Music: Impulse Tracker modules (`.it`, stored as `res3.dat` and other
    loose `res*.dat` files) played with `xmrs`/`xmrsplayer` (MIT, pure Rust,
    IT replay modelled on Schism Tracker). The originals played them through
    FMOD, so playback may differ slightly by ear.
- Saves and configs are kept as stored and written back byte-identically.
  Load-time fix-ups (e.g. RTWP's world pack 0 → `i * 10`) belong to profile
  loading.

### logic

Fixed-tick simulation of the tile grid and all objects, mirroring the original
timing (a tile move takes `BaseSpeed = 24` ticks). Input is a stream of
commands, output is state — no rendering, no wall clock. Per-game behaviour
differences are parameters on a `Ruleset`, not separate code paths per game.

- `rng` — `BlitzRng`, Blitz3D's Park–Miller generator. Matching the
  originals means drawing from it in the same order, including draws that
  only feed particles and lighting (`skip`).
- `clock` — one tick is `TweenPeriod = 1000/85` = 11 ms. A frame runs at most
  20 ticks and the game crate interpolates with the remainder.
- `setup` — what the loaders do after reading a level: tile rewrites, sign
  positions, the electro map, and the `Create*` calls as an ordered `Spawn`
  list.
- `player` — the players: walking, pushing, ice, conveyors, trampolines,
  being hit, switching focus, keyboard control in both views, idle and
  blink animations. The entity position is simulated as well as the tile.
- `sign` — sign texts on screen and their effect on the lights.
- `block` — boxes and other pushables, box factories and transporters.
- `monster` — the monsters (beach balls, z-bots, bombers, saucers,
  chompers, ghosts, rainbow spirits, broken z-bots, the mother ship, shadow
  and fire stinkers) and the warp gates that spawn them.
- `item` — coins, the exit gate, clocks and bonus coins: picking them up,
  winning the level and the level-complete sign (the language changes
  the random draws). The profile's best score comes in as
  `World::record`; `Event::LevelWon` hands the score back to be saved.
- `gate` — `ToggleLaserGate`: the last coin opens the final gate, buttons
  the coloured ones, and a closing gate destroys what is in it.
- `switch` — buttons (conveyor stops, gate switches, timers) and
  teleporters.
- `hazard` — electro tiles, spikes, cannons and fireballs. Fireballs are
  kept as the original's array slots, stale entries included.
- `scenery` — fish and pillars, which draw random numbers when created.
- `md2` — Blitz's MD2 animation state, which the player code reads.
- `world` — `World::start` (`StartLevel`) and `World::step` (`UpdateGame`
  plus the frame's time-limit check). The camera and lights are simulated
  here, because the original moves them per tick. `Session` carries the
  generator and other cross-level state into the next level or a restart.
  Events (sounds, explosions) are returned per tick.

### campaign

How levels are sequenced and unlocked: TOW's world list, RTWP's worlds plus
level packs, WSW's world map and story screens.

### game

The Bevy front end. Detects which games are present under `assets/`, offers
them in a launcher, then loads the selected game's definition and assets.
Renders and animates the `logic` state and plays audio. Bevy's version is
pinned and upgraded deliberately.

- **Bevy 0.20.0**, pinned with `=`, default features off: only the 3D
  renderer, window, keyboard, UI text and WAV sound. Its minimum Rust
  (1.97.1) is the workspace's. Building `bevy_audio` on Linux needs the
  ALSA headers (`libasound2-dev`). Textures are decoded with `image`
  (BMP/JPEG), not Bevy's loaders.
- **Run:** `wonderland [--assets <dir>] [--level <name>] [--backend
  dx12|vulkan|gl]` starts an RTWP level (`intro` by default) from
  `<dir>/RTWP`, where `<dir>` defaults to `$WL_ASSETS`, else `assets`.
- **Renderer:** DX12 on Windows with Vulkan as the fallback, and Vulkan on
  Linux with OpenGL as the fallback. Both are passed to wgpu, which takes
  the first that works. wgpu's OpenGL on Windows needs ANGLE, so it is
  not offered there. `--backend` or `WGPU_BACKEND` overrides the choice.
- **Coordinates:** `logic` and the scene builders work in Blitz
  coordinates; `coords` negates z on the way to Bevy and reverses each
  triangle's winding, since the picture stays the same and Blitz's front
  faces are clockwise. Blitz pitch/yaw become a rotation, and
  `CameraZoom` becomes a vertical field of view from the horizontal one
  Blitz keeps fixed (zoom 1 is 90° across).
- **Lighting:** Blitz lights per vertex (Direct3D 7 fixed pipeline). Level
  faces are flat, so the scene uses unlit materials and sets each face
  group's colour to ambient + light × max(0, N·L), clamped, from
  `World::lights` every frame (`lighting`). The light is `Setup`'s single
  directional light at pitch 25, yaw -25. Tonemapping is off. Objects
  get the same formula per vertex: each object entity has its own mesh
  whose vertex colours are rebuilt when its turn, MD2 frame or the lights
  change (`objects`). That avoids a custom shader (Bevy 0.20 materials
  use WESL).
- **Objects:** `model` holds `Setup`'s model table (mesh file or Blitz
  primitive, baked `RotateMesh`/`PositionMesh`/`ScaleMesh`/`FitMesh`, the
  template's `ScaleEntity`/`RotateEntity`, colour, alpha, texture).
  `objects::draws` turns the `World` into a draw list following the
  `Create*`/`Control*` entity calls; spins and pulses the logic does not
  keep come from `level_timer`. A pool of entities per model takes the
  draws in order, so nothing is spawned in steady play.
- **Particles and sound:** `logic` reports where effects happen
  (`Event::Effect`, `Event::PlayerExploded`) and the renderer makes the
  particles with its own random numbers (`particles`); fireball trails and
  weather follow from the state. `audio` maps each `logic::Sound` to the
  file `LoadSounds` loads (custom voices first) with its pitch and
  volume. Music is not played yet.
- **Plugins:** `SimPlugin` runs ticks from `logic::Clock` and keeps the
  frame's events; `ScenePlugin` builds the level (`level_mesh`, `style`,
  `texture`), lights it and scrolls the water and conveyor textures;
  `CameraPlugin` follows `World::camera`; `ObjectsPlugin`,
  `ParticlesPlugin` and `AudioPlugin` draw and play the rest.
- **Checking without a display:** `WL_SCREENSHOT=out.png[,frames]` saves
  a frame and quits; with `WL_HEADLESS=1` there is no window and the
  camera draws into a 1024x768 image. WSLg could not present a window
  here (Wayland dropped the connection, GL had no present mode).

## Porting notes (Blitz3D semantics)

Behaviours that silently break a line-by-line translation:

- Assigning a float to an int **rounds to nearest**; it does not truncate.
- `And` / `Or` are bitwise and do not short-circuit; `True` is 1. `And`,
  `Or` and `Xor` share one precedence level and group left to right, so
  `a Or b And c` is `(a Or b) And c`. `Not` negates the whole chain after
  it (checked in `compiler/parser.cpp`).
- `For i=a To b` re-evaluates `b` on every pass, so loops that delete
  entries and step `i` back see the shrinking count.
- Locals start at 0 on every call, so a branch that leaves one unset
  passes 0 on.
- `Dim A(n)` has n+1 elements (0..=n). String functions are 1-based.
- Undeclared variables are implicitly created with value 0 — a typo in the
  original is a new variable, not an error.
- Reading past end of file returns zeros rather than failing.
- A nested `For` that reuses the outer loop's variable leaves it past
  the outer bound, so the outer loop runs once (`ControlWarpers`).
- Function arguments evaluate left to right, which fixes the order of
  random draws in calls like `CreateParticle(..., Rnd(), Rnd())`.

## Verification

1. Format parsers are tested against the real `assets/` files.
2. Record input sequences while playing each original game in Blitz3D, then
   replay them through `logic` and compare outcomes (level completed, positions,
   coins, timer).

### Recordings

- `RTWP/recorder.bb` logs a playthrough when the game runs with `-record`
  on its command line (in the Blitz3D IDE: Program > Program Command Line).
  Without it the game behaves as shipped: the recorder draws no random
  numbers and changes no state. Each session writes
  `replay <date> <time>.wlr` next to the game. Copy the files to
  `assets/recordings/`.
- The log is text, one record per line. At each `StartLevel` it holds the
  level file, `RndSeed()`, `DetailLevel`, the sound settings, `NoTimeLimit`,
  `ClassicDeathSound`, the menu style overrides and `Language`. Per tick
  it holds the keys held, `RndSeed()` before and after, and one line of
  state per object family. Per frame it holds the tick count and the `MilliSecs()` of the
  time-limit check. Floats are logged as their bits. `logic::replay`
  documents the records.
- `logic::replay` parses a log and replays each level start. It seeds
  `Session::rng`, sets `Options` from the log, feeds the keys and reports
  the first tick where each family differs. Families that are not ported
  yet are left out (`Family::PORTED`). The time limit counts down on the
  recorded frame clock, because the original checks it once per frame
  against the wall clock.
- If the seed before a tick differs from the seed after the previous one,
  the original drew random numbers outside `UpdateGame` (a pause menu, for
  example). The replay then reseeds and lists the tick in the report.
- `tests/recordings.rs` replays every `assets/recordings/*.wlr` and fails
  on any difference. Set `WL_REPLAY_REPORT=1` to print the report without
  failing.
