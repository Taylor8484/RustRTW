//! The other object families as far as the ported code needs them, until
//! steps 2.2d–f port them. Each step moves its part out of this file.
//!
//! The item list stays empty for now, so the calls that act on an item
//! cannot be reached yet. The others can: buttons and teleporters are
//! driven by tiles, and push cannons, saucers and fire stinkers fire
//! fireballs. They do nothing until their family exists, so a level that
//! uses them does not play like the original yet.

use crate::World;

/// The `Item*` arrays. Step 2.2d adds the rest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Item {
    pub x: i32,
    pub y: i32,
    /// `ItemType`: 0 coin, 1 exit gate, 2 clock, 3 bonus coin.
    pub kind: i32,
}

impl World {
    /// 2.2d: `PickUpItem`.
    pub(crate) fn pick_up_item(&mut self, _i: usize, _who: usize) {
        unreachable!("2.2d: no items are created yet")
    }

    /// 2.2d: the rest of a rainbow spirit or shadow stinker taking coin
    /// `j` in `CanMonsterMove`: `NofCrystals - 1`, and at 0
    /// `PlayMySound(SFXLastCoin, 0)` and `ToggleLaserGate(0, True)`; then
    /// `KillItem(j)`. The coin stays until `CleanUpItems`, and nothing stops
    /// it being taken twice in a tick.
    pub(crate) fn take_coin(&mut self, _j: usize) {
        unreachable!("2.2d: no items are created yet")
    }

    /// 2.2d: the rest of a shadow stinker taking bonus coin `j`:
    /// `NofBonusCoins - 1` and `KillItem(j)`.
    pub(crate) fn take_bonus_coin(&mut self, _j: usize) {
        unreachable!("2.2d: no items are created yet")
    }

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
