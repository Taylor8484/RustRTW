//! Per-game parameters. The games share one engine, so where they differ the
//! simulation reads a field here instead of branching on the game.

/// The rules one game runs by. TOW levels run under `tow()`, which is RTWP's
/// rules except where the difference changes how a TOW level plays
/// (`docs/diff-tow.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ruleset {
    /// Ticks for a regular one-tile move (`Const BaseSpeed`).
    pub base_speed: i32,
    /// Milliseconds per tick: `TweenPeriod = 1000/85`, an int division.
    pub tick_ms: i32,
    /// The most ticks one frame may run to catch up (`20*TweenPeriod`).
    pub max_catch_up_ticks: i32,
    /// Entries per side of `Dim MapTile(n, n)`, i.e. n+1.
    pub map_size: usize,
    /// Player slots (`Dim PlayerX(4)` and friends; slot 0 is unused).
    pub players: usize,
    /// A cannon waits while this many fireballs are out. TOW has no such
    /// wait, so its value is `MaxFireballs`, which nothing reaches.
    pub cannon_fireball_limit: usize,
}

impl Ruleset {
    /// Return to Wonderland Platinum.
    pub fn rtwp() -> Self {
        Self {
            base_speed: 24,
            tick_ms: 1000 / 85,
            max_catch_up_ticks: 20,
            map_size: 102,
            players: 4,
            cannon_fireball_limit: 25,
        }
    }

    /// TOW's levels: RTWP's rules, with TOW's cannons.
    pub fn tow() -> Self {
        Self {
            cannon_fireball_limit: 200,
            ..Self::rtwp()
        }
    }
}

impl Default for Ruleset {
    fn default() -> Self {
        Self::rtwp()
    }
}
