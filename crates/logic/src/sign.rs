//! Signs: `StartSign`, `StopSign`, `CheckSigns` and `ControlSigns` in
//! `RTWP/text.bb`.
//!
//! Tag 10 is the level name shown at the start; tags 11–30 are the level's
//! sign tiles 0–19, shown while the focused player stands on one. A level
//! sign dims the lights and sets `SignActive`, which holds off lightning and
//! psyche flashes. The renderer lays out the letters and fades tag 10 in
//! over ticks 0–200 and out over 600–800.

use crate::World;

/// A sign on screen (the letters `StartSign` creates under one tag).
#[derive(Debug, Clone, PartialEq)]
pub struct ShownSign {
    pub tag: i32,
    /// Lines are separated by `#`.
    pub text: String,
    /// The first line's row.
    pub y: f32,
    pub alpha: f32,
}

/// The level-complete tag.
pub const LEVEL_COMPLETE: i32 = 2;
/// The level-name tag.
pub const LEVEL_NAME: i32 = 10;
/// The first level-sign tag; sign tile `i` uses `FIRST_SIGN + i`.
pub const FIRST_SIGN: i32 = 11;

fn is_level_sign(tag: i32) -> bool {
    (FIRST_SIGN..FIRST_SIGN + 20).contains(&tag)
}

impl World {
    /// `StartSign`. An empty text shows nothing and leaves the lights alone.
    pub(crate) fn start_sign(&mut self, text: &str, y: f32, tag: i32, alpha: f32) {
        self.signs_shown.retain(|s| s.tag != tag);
        if text.is_empty() {
            return;
        }
        self.signs_shown.push(ShownSign {
            tag,
            text: text.to_owned(),
            y,
            alpha,
        });
        if is_level_sign(tag) {
            if let Some(k) = (1..self.players.len())
                .rev()
                .find(|&k| self.players[k].focus)
            {
                self.sign_active = k as i32;
            }
            self.lights.light_target = [0; 3];
            self.lights.light_speed = 10;
            self.lights.ambient_target = [100; 3];
            self.lights.ambient_speed = 5;
        }
    }

    /// `StopSign`. Stopping a level-sign tag restores the lights even when
    /// nothing was shown under it.
    pub(crate) fn stop_sign(&mut self, tag: i32) {
        self.signs_shown.retain(|s| s.tag != tag);
        if is_level_sign(tag) {
            self.level_light_targets();
            self.sign_active = 0;
            self.lights.light_speed = 10;
            self.lights.ambient_speed = 5;
        }
    }

    /// `StopSign` for every level-sign tag.
    pub(crate) fn stop_level_signs(&mut self) {
        for tag in FIRST_SIGN..FIRST_SIGN + 20 {
            self.stop_sign(tag);
        }
    }

    /// Shows the signs under player `k`, if `k` has the focus.
    pub(crate) fn start_signs_under(&mut self, k: usize) {
        let p = &self.players[k];
        if !p.focus {
            return;
        }
        let (x, y) = (p.x, p.y);
        for i in 0..self.level.signs.len() {
            let sign = &self.level.signs[i];
            if sign.x == x && sign.y == y {
                let text = sign.text.clone();
                self.start_sign(&text, 8.0, FIRST_SIGN + i as i32, 1.0);
            }
        }
    }

    /// `CheckSigns`.
    pub(crate) fn check_signs(&mut self) {
        for k in 1..self.players.len() {
            self.start_signs_under(k);
        }
    }

    /// `ControlSigns`: the level name goes at tick 800.
    pub(crate) fn control_signs(&mut self) {
        if self.level_timer == 800 {
            self.stop_sign(LEVEL_NAME);
        }
    }

    /// The signs on screen, in the order they were started.
    pub fn signs_shown(&self) -> &[ShownSign] {
        &self.signs_shown
    }
}
