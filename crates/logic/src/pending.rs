//! The other object families as far as the ported code needs them, until
//! steps 2.2d–f port them. Each step moves its part out of this file.
//!
//! Push cannons, saucers and fire stinkers fire fireballs. They go nowhere
//! until 2.2f, so a level that uses them does not play like the original
//! yet.

use crate::World;

impl World {
    /// 2.2f: `NofFireballs`.
    pub(crate) fn fireball_count(&self) -> usize {
        0
    }

    /// 2.2f: `CreateFireball`. Shots go nowhere for now.
    pub(crate) fn create_fireball(&mut self, _x: i32, _y: i32, _dx: i32, _dy: i32) {}

    /// 2.2f: the fireball part of a gate closing on `(x, y)` in
    /// `ToggleLaserGate`.
    pub(crate) fn kill_fireballs_in_gate(&mut self, _x: i32, _y: i32) {}
}
