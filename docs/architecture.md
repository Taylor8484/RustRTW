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
- Next: level files (v1–v6), player saves (`.pla`), `options.cfg`.

### logic

Fixed-tick simulation of the tile grid and all objects, mirroring the original
timing (a tile move takes `BaseSpeed = 24` ticks). Input is a stream of
commands, output is state — no rendering, no wall clock. Per-game behaviour
differences are parameters on a `Ruleset`, not separate code paths per game.

### campaign

How levels are sequenced and unlocked: TOW's world list, RTWP's worlds plus
level packs, WSW's world map and story screens.

### game

The Bevy front end. Detects which games are present under `assets/`, offers
them in a launcher, then loads the selected game's definition and assets.
Renders and animates the `logic` state and plays audio. Bevy's version is
pinned and upgraded deliberately.

## Porting notes (Blitz3D semantics)

Behaviours that silently break a line-by-line translation:

- Assigning a float to an int **rounds to nearest**; it does not truncate.
- `And` / `Or` are bitwise and do not short-circuit; `True` is 1.
- `Dim A(n)` has n+1 elements (0..=n). String functions are 1-based.
- Undeclared variables are implicitly created with value 0 — a typo in the
  original is a new variable, not an error.
- Reading past end of file returns zeros rather than failing.

## Verification

1. Format parsers are tested against the real `assets/` files.
2. Record input sequences while playing each original game in Blitz3D, then
   replay them through `logic` and compare outcomes (level completed, positions,
   coins, timer).

## Roadmap

1. `formats`: packs ✅, levels, saves.
2. `logic` with RTWP rules.
3. `game`: one RTWP level playable end to end.
4. TOW and WSW as game definitions + ruleset flags; WSW map/story; launcher.
5. Later: level editor, translations, save import.
