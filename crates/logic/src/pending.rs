//! The other object families as far as the player code needs them, until
//! steps 2.2b–f port them. Each step moves its part out of this file.
//!
//! The object lists stay empty for now, so the calls that act on a listed
//! object cannot be reached yet. The tile-driven calls (buttons, teleporters,
//! transporters) can be: they do nothing until their family exists, so a
//! level that uses them does not play like the original yet.

use crate::World;

/// A box, push cannon, sticky cube or link sphere (the `Box*` arrays).
/// Step 2.2b adds the rest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Block {
    pub x: i32,
    pub y: i32,
    pub x2: i32,
    pub y2: i32,
    pub moving: bool,
    pub moving_x: i32,
    pub moving_y: i32,
    /// `BoxType`.
    pub kind: i32,
}

/// The `Monster*` arrays. Step 2.2c adds the rest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Monster {
    pub x: i32,
    pub y: i32,
    pub x2: i32,
    pub y2: i32,
    pub moving: bool,
    /// `MonsterType`.
    pub kind: i32,
}

/// The `Item*` arrays. Step 2.2d adds the rest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Item {
    pub x: i32,
    pub y: i32,
}

impl World {
    /// 2.2b: `CanBoxMove`.
    pub(crate) fn can_box_move(&mut self, _i: usize, _dx: i32, _dy: i32) -> bool {
        unreachable!("2.2b: no boxes are created yet")
    }

    /// 2.2b: `MoveBox`.
    pub(crate) fn move_box(&mut self, _i: usize, _dx: i32, _dy: i32, _speed: i32) {
        unreachable!("2.2b: no boxes are created yet")
    }

    /// 2.2b: `KillBox`.
    pub(crate) fn kill_box(&mut self, _i: usize) {
        unreachable!("2.2b: no boxes are created yet")
    }

    /// 2.2b: `CanTransporterMove`. No transporters exist yet.
    pub(crate) fn can_transporter_move(&mut self, _x: i32, _y: i32, _dx: i32, _dy: i32) -> bool {
        false
    }

    /// 2.2b: `MoveTransporter`.
    pub(crate) fn move_transporter(&mut self, _x: i32, _y: i32, _dx: i32, _dy: i32) {
        unreachable!("2.2b: no transporters are created yet")
    }

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
}
