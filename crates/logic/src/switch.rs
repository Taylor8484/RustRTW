//! Buttons and teleporters: `CreateButton` to `ControlTimerButtons`,
//! `PlayerStepOnTeleporter` and `BoxStepOnTeleporter` (`RTWP/gameobject.bb`).
//!
//! Button types: `kind / 3` is what the button works (0 the conveyors
//! 5x0–5x7, 1 the conveyors 5x8 up, n ≥ 2 the laser gates of colour
//! n - 1) and `kind % 3` how: 0 stays down, 1 pops up when nothing is left
//! on any button of its type, 2 runs a timer (`ButtonBaseTimer`, 15 tile
//! moves) that every button of its type restarts. A pressed button stops
//! its conveyors or opens its gates; releasing it does the opposite, and a
//! gate that closes destroys what stands in it.
//!
//! Teleporters (12xx tiles) come in pairs of the same tile; an object
//! arriving on one leaves the other in the first free direction of: on,
//! back, and the two sideways turns.

use crate::World;
use crate::world::{Event, Sound};

/// `ButtonBaseTimer`, in ticks: `BaseSpeed * 15`.
fn base_timer(w: &World) -> i32 {
    w.ruleset.base_speed * 15
}

/// One button (the `Button*` arrays).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Button {
    pub x: i32,
    pub y: i32,
    /// `ButtonType`; see the module docs.
    pub kind: i32,
    /// Ticks left on a timer button. The renderer turns it by
    /// `timer * 90 / ButtonBaseTimer` degrees.
    pub timer: i32,
    /// Pressed (`ButtonOffTexture`).
    pub active: bool,
}

impl World {
    /// `CreateButton`.
    pub(crate) fn create_button(&mut self, x: i32, y: i32, kind: i32) {
        self.buttons.push(Button {
            x,
            y,
            kind,
            timer: 0,
            active: false,
        });
    }

    /// `StepOnButton`: something arrived on the button tile `(x, y)`.
    pub(crate) fn step_on_button(&mut self, x: i32, y: i32) {
        for i in 0..self.buttons.len() {
            if (self.buttons[i].x, self.buttons[i].y) != (x, y) {
                continue;
            }
            if !self.buttons[i].active {
                self.activate_button(i);
                self.events.push(Event::Sound(Sound::Button));
            }
            if self.buttons[i].kind % 3 == 2 {
                self.buttons[i].timer = base_timer(self);
                self.events.push(Event::Sound(Sound::TimerButton));
                let kind = self.buttons[i].kind;
                for j in 0..self.buttons.len() {
                    if self.buttons[j].kind == kind && j != i {
                        // Even when already pressed: the gates or
                        // conveyors are set again.
                        self.activate_button(j);
                        self.buttons[j].timer = base_timer(self);
                    }
                }
            }
        }
    }

    /// `StepOffButton`: something left the button tile `(x, y)`.
    pub(crate) fn step_off_button(&mut self, x: i32, y: i32) {
        for i in 0..self.buttons.len() {
            let b = &self.buttons[i];
            if (b.x, b.y) == (x, y)
                && b.kind % 3 == 1
                && b.active
                && !self.check_button_type(b.kind)
            {
                self.deactivate_button(i);
            }
        }
    }

    /// `CheckButtons`: at level start, presses the buttons something
    /// stands on.
    pub(crate) fn check_buttons(&mut self) {
        for i in 0..self.buttons.len() {
            if self.check_button(i) {
                let (x, y) = (self.buttons[i].x, self.buttons[i].y);
                self.step_on_button(x, y);
            }
        }
    }

    /// `CheckButtonType`: something stands on a button of this type.
    fn check_button_type(&self, kind: i32) -> bool {
        (0..self.buttons.len()).any(|i| self.buttons[i].kind == kind && self.check_button(i))
    }

    /// `CheckButton`: a live player, box or monster is on button `i`'s
    /// tile. Only the current tile counts, not the one being moved to.
    fn check_button(&self, i: usize) -> bool {
        let at = (self.buttons[i].x, self.buttons[i].y);
        self.players[1..]
            .iter()
            .any(|p| p.exists && (p.x, p.y) == at && !p.dead)
            || self.blocks.iter().any(|b| (b.x, b.y) == at && !b.dead)
            || self.monsters.iter().any(|m| (m.x, m.y) == at && !m.dying)
    }

    /// `ActivateButton(i, 0)`: presses button `i` and marks the others of
    /// its type pressed.
    fn activate_button(&mut self, i: usize) {
        self.buttons[i].active = true;
        let kind = self.buttons[i].kind;
        match kind / 3 {
            0 => self.conveyor_active[0] = false,
            1 => self.conveyor_active[1] = false,
            n => self.toggle_laser_gate(n - 1, true),
        }
        for b in &mut self.buttons {
            if b.kind == kind {
                b.active = true;
            }
        }
    }

    /// `DeActivateButton(i, 0)`: releases every button of `i`'s type.
    fn deactivate_button(&mut self, i: usize) {
        self.buttons[i].active = false;
        let kind = self.buttons[i].kind;
        match kind / 3 {
            0 => self.conveyor_active[0] = true,
            1 => self.conveyor_active[1] = true,
            n => self.toggle_laser_gate(n - 1, false),
        }
        for b in &mut self.buttons {
            if b.kind == kind {
                b.active = false;
            }
        }
    }

    /// `ControlTimerButtons`.
    pub(crate) fn control_timer_buttons(&mut self) {
        for i in 0..self.buttons.len() {
            let b = &mut self.buttons[i];
            if b.kind % 3 == 2 && b.active {
                b.timer -= 1;
                if b.timer <= 0 {
                    b.active = false;
                    self.deactivate_button(i);
                }
            }
        }
    }

    /// The other teleporter of the pair at `(x, y)`: the last match in
    /// column order, as the original keeps searching after a find.
    fn teleporter_exit(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        let g = &self.level.grid;
        let tile = g.get(x, y);
        let mut found = None;
        for i in 0..g.width() as i32 {
            for j in 0..g.height() as i32 {
                if g.get(i, j) == tile && (i, j) != (x, y) {
                    found = Some((i, j));
                }
            }
        }
        found
    }

    /// The directions an object leaving a teleporter tries, in order.
    fn exits(dx: i32, dy: i32) -> [(i32, i32); 4] {
        [(dx, dy), (-dx, -dy), (dy, dx), (-dy, -dx)]
    }

    /// `PlayerStepOnTeleporter`: true if player `k` came out of the other
    /// teleporter, false if it stayed (with a "hmm" when boxed in).
    pub(crate) fn player_step_on_teleporter(
        &mut self,
        k: usize,
        x: i32,
        y: i32,
        dx: i32,
        dy: i32,
    ) -> bool {
        let Some((x2, y2)) = self.teleporter_exit(x, y) else {
            return false;
        };
        let place = |w: &mut World, tx: i32, ty: i32| {
            let p = &mut w.players[k];
            p.x = tx;
            p.y = ty;
            p.position = [tx as f32 + 0.5, 0.0, -(ty as f32 + 0.5)];
        };
        place(self, x2, y2);
        let mut moved = false;
        for (mx, my) in Self::exits(dx, dy) {
            if self.can_player_move(k, mx, my) {
                self.move_player(k, mx, my);
                moved = true;
                break;
            }
        }
        if !moved {
            place(self, x, y);
            self.events.push(Event::Sound(Sound::Hmmm));
            return false;
        }
        self.events.push(Event::Sound(Sound::Teleport));
        if self.players[k].focus {
            let to = self.players[k].position;
            self.camera_swoop = if self.level.view3d {
                [to[0] - x as f32 - 0.5, to[2] + y as f32 + 0.5]
            } else {
                let cam = self.camera.position;
                [to[0] - cam[0], (to[2] - 9.0) - cam[2]]
            };
        }
        true
    }

    /// `BoxStepOnTeleporter`: true if box `i` came out of the other
    /// teleporter. It leaves one tick ahead, so a player following can
    /// push it.
    pub(crate) fn box_step_on_teleporter(
        &mut self,
        i: usize,
        x: i32,
        y: i32,
        dx: i32,
        dy: i32,
    ) -> bool {
        let Some((x2, y2)) = self.teleporter_exit(x, y) else {
            return false;
        };
        let place = |w: &mut World, tx: i32, ty: i32| {
            let b = &mut w.blocks[i];
            b.x = tx;
            b.y = ty;
            b.position = [tx as f32 + 0.5, 0.4, -(ty as f32 + 0.5)];
        };
        place(self, x2, y2);
        self.blocks[i].moving = false;
        let mut moved = false;
        for (mx, my) in Self::exits(dx, dy) {
            if self.can_box_move(i, mx, my) {
                self.move_box(i, mx, my, 0);
                moved = true;
                break;
            }
        }
        if !moved {
            place(self, x, y);
            return false;
        }
        let b = &mut self.blocks[i];
        b.position[0] += b.moving_x as f32 / b.speed as f32;
        b.position[2] += -b.moving_y as f32 / b.speed as f32;
        b.moving_timer -= 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::setup::tests::level;
    use crate::world::{Event, Sound};
    use crate::{Input, Options, Ruleset, Session, World};

    fn world(width: usize, tiles: &[i32], objects: &[i32]) -> World {
        let l = level(5, width, tiles, objects);
        World::start(&l, Ruleset::rtwp(), Options::default(), Session::default()).unwrap()
    }

    const RIGHT: Input = Input {
        right: true,
        up: false,
        down: false,
        left: false,
        camera: false,
        switch: false,
    };

    fn walk(w: &mut World, input: Input) -> Vec<Event> {
        let mut events = vec![];
        w.step(input);
        events.extend_from_slice(w.events());
        while w.players[1].moving {
            w.step(Input::default());
            events.extend_from_slice(w.events());
        }
        events
    }

    #[test]
    fn round_button_opens_its_gate_while_held() {
        // Player, a round button for colour 1 (type 7: 7 / 3 = 2, 7 % 3 =
        // 1), floor, the colour-1 gate (1002).
        let mut w = world(4, &[100, 1107, 100, 1002], &[1, 0, 0, 0]);
        assert_eq!(w.buttons.len(), 1);
        let events = walk(&mut w, RIGHT);
        assert!(events.contains(&Event::Sound(Sound::Button)));
        assert!(w.buttons[0].active);
        assert_eq!(w.level.grid.get(3, 0), 1052);
        assert!(!w.laser_gate_active[1]);
        walk(&mut w, RIGHT);
        assert!(!w.buttons[0].active);
        assert_eq!(w.level.grid.get(3, 0), 1002);
    }

    #[test]
    fn a_closing_gate_kills_what_stands_in_it() {
        let mut w = world(4, &[100, 1107, 1002, 100], &[1, 0, 0, 0]);
        walk(&mut w, RIGHT);
        // Walk into the open gate, then off the button: it closes on us.
        walk(&mut w, RIGHT);
        assert_eq!(w.level.grid.get(2, 0), 1002);
        assert!(w.players[1].dying);
    }

    #[test]
    fn square_button_stays_and_stops_conveyors() {
        // Type 0: conveyors 5x0–5x7, stays down.
        let mut w = world(3, &[100, 1100, 100], &[1, 0, 0]);
        walk(&mut w, RIGHT);
        walk(&mut w, RIGHT);
        assert!(w.buttons[0].active);
        assert_eq!(w.conveyor_active, [false, true]);
    }

    #[test]
    fn timer_button_runs_out() {
        // Type 5: conveyors 5x8 up, timer.
        let mut w = world(3, &[100, 1105, 100], &[1, 0, 0]);
        let events = walk(&mut w, RIGHT);
        assert!(events.contains(&Event::Sound(Sound::TimerButton)));
        assert_eq!(w.conveyor_active, [true, false]);
        let mut ticks = 0;
        while w.buttons[0].active {
            w.step(Input::default());
            ticks += 1;
        }
        // Set on the tick the move ended; ControlTimerButtons ran before.
        assert_eq!(ticks, 24 * 15);
        assert_eq!(w.conveyor_active, [true, true]);
    }

    #[test]
    fn something_on_a_button_at_start_presses_it() {
        // A box (code 9) on a square colour-1 button (type 6).
        let w = world(3, &[100, 1106, 1002], &[1, 9, 0]);
        assert!(w.buttons[0].active);
        assert_eq!(w.level.grid.get(2, 0), 1052);
    }

    #[test]
    fn teleporter_sends_the_player_on() {
        // Player, teleporter, wall, teleporter, floor.
        let mut w = world(5, &[100, 1200, 0, 1200, 100], &[1, 0, 0, 0, 0]);
        let events = walk(&mut w, RIGHT);
        assert!(events.contains(&Event::Sound(Sound::Teleport)));
        assert_eq!((w.players[1].x, w.players[1].y), (4, 0));
    }

    #[test]
    fn boxed_in_teleporter_says_hmm() {
        let mut w = world(4, &[100, 1200, 0, 1200], &[1, 0, 0, 0]);
        let events = walk(&mut w, RIGHT);
        assert!(events.contains(&Event::Sound(Sound::Hmmm)));
        assert_eq!((w.players[1].x, w.players[1].y), (1, 0));
    }

    #[test]
    fn teleporter_sends_a_box_on_ahead() {
        // Player, box, teleporter, wall, teleporter, floor, floor.
        let mut w = world(
            7,
            &[100, 100, 1200, 0, 1200, 100, 100],
            &[1, 9, 0, 0, 0, 0, 0],
        );
        walk(&mut w, RIGHT);
        while w.blocks[0].moving {
            w.step(Input::default());
        }
        assert_eq!(w.blocks[0].x, 5);
    }
}
