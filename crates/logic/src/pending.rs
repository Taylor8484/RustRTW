//! The other object families as far as the ported code needs them, until
//! steps 2.2d–f port them. Each step moves its part out of this file.
//!
//! Buttons and teleporters are driven by tiles, and push cannons, saucers
//! and fire stinkers fire fireballs. These do nothing until their family
//! exists, so a level that uses them does not play like the original yet.

use crate::World;

impl World {
    /// 2.2e: `StepOnButton`. No buttons exist yet.
    pub(crate) fn step_on_button(&mut self, _x: i32, _y: i32) {}

    /// 2.2e: `StepOffButton`.
    pub(crate) fn step_off_button(&mut self, _x: i32, _y: i32) {}

    /// 2.2e: `PlayerStepOnTeleporter`; true if the player was sent on.
    pub(crate) fn player_step_on_teleporter(
        &mut self,
        _k: usize,
        _x: i32,
        _y: i32,
        _dx: i32,
        _dy: i32,
    ) -> bool {
        false
    }

    /// 2.2e: `BoxStepOnTeleporter`; true if the box was sent on.
    pub(crate) fn box_step_on_teleporter(
        &mut self,
        _i: usize,
        _x: i32,
        _y: i32,
        _dx: i32,
        _dy: i32,
    ) -> bool {
        false
    }

    /// 2.2f: `NofFireballs`.
    pub(crate) fn fireball_count(&self) -> usize {
        0
    }

    /// 2.2f: `CreateFireball`. Shots go nowhere for now.
    pub(crate) fn create_fireball(&mut self, _x: i32, _y: i32, _dx: i32, _dy: i32) {}
}
