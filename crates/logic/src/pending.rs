//! The other object families as far as the ported code needs them, until
//! steps 2.2c–f port them. Each step moves its part out of this file.
//!
//! The monster and item lists stay empty for now, so the calls that act on
//! one of them cannot be reached yet. The others can: buttons and
//! teleporters are driven by tiles, and push cannons fire fireballs. They
//! do nothing until their family exists, so a level that uses them does
//! not play like the original yet.

use crate::World;

/// The `Monster*` arrays. Step 2.2c adds the rest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Monster {
    pub x: i32,
    pub y: i32,
    pub x2: i32,
    pub y2: i32,
    pub moving: bool,
    pub moving_timer: i32,
    pub speed: i32,
    pub dead: bool,
    pub dying: bool,
    /// `MonsterType`.
    pub kind: i32,
}

/// The `Item*` arrays. Step 2.2d adds the rest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Item {
    pub x: i32,
    pub y: i32,
    /// `ItemType`: 0 coin, 1 exit gate, 2 clock, 3 bonus coin.
    pub kind: i32,
}

impl World {
    /// 2.2c: `CanMonsterMove`.
    pub(crate) fn can_monster_move(&mut self, _i: usize, _dx: i32, _dy: i32) -> bool {
        unreachable!("2.2c: no monsters are created yet")
    }

    /// 2.2c: `MoveMonster`.
    pub(crate) fn move_monster(&mut self, _i: usize, _dx: i32, _dy: i32) {
        unreachable!("2.2c: no monsters are created yet")
    }

    /// 2.2c: `KillMonster`.
    pub(crate) fn kill_monster(&mut self, _i: usize) {
        unreachable!("2.2c: no monsters are created yet")
    }

    /// 2.2d: `PickUpItem`.
    pub(crate) fn pick_up_item(&mut self, _i: usize, _who: usize) {
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

    /// 2.2f: `CreateFireball`. Push cannons fire into nothing for now.
    pub(crate) fn create_fireball(&mut self, _x: i32, _y: i32, _dx: i32, _dy: i32) {}
}
